//! pdf.rs — PDF export (bila dependencies za nje — writer ndogo ya kweli)
//!
//! Inazalisha PDF halisi (PDF 1.4) yenye:
//!   - Logo ya FUNDI (image XObject kutoka logo.rs) + jina la brand
//!   - Rangi za brand (cyan/teal), Helvetica/Helvetica-Bold
//!   - Hati: quote/invoice za B2B + ripoti ya Fundi Deploy (jobs/OS/agents)
//!
//! Sheria za mfumo: PDF ni "kalamu na karatasi" — haina malipo yoyote.

use crate::b2b::B2BDoc;

// ---------- helpers ----------

fn pdf_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('(', "\\(").replace(')', "\\)")
}

/// Herufi za WinAnsi tu — toa za nje (emoji n.k.) ili PDF isivurugike.
fn winansi(s: &str) -> String {
    // WinAnsi (CP1252) — herufi za Kiswahili zote (a-z + ) zinashibulika;
    // emoji na alama za nje zinabadilishwa '?' au '-'.
    s.chars()
        .map(|c| {
            let cp = c as u32;
            if (32..=255).contains(&cp) {
                match cp {
                    0x2026 => '\u{85}', // … → WinAnsi ellipsis
                    _ => c,
                }
            } else if (0x2000..0x2100).contains(&cp) {
                '-' // ndashes, bullets n.k.
            } else if cp >= 0x1F000 {
                ' ' // emoji
            } else if cp < 32 {
                ' '
            } else {
                '?'
            }
        })
        .collect()
}

fn money(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    let bytes = s.as_bytes();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(*b as char);
    }
    out
}

// ---------- PDF writer ndogo ----------

struct PdfBuilder {
    objs: Vec<String>,
}

impl PdfBuilder {
    fn new() -> Self {
        PdfBuilder { objs: Vec::new() }
    }
    fn obj(&mut self, body: &str) -> usize {
        self.objs.push(format!("{} 0 obj\n{body}\nendobj\n", self.objs.len() + 1));
        self.objs.len()
    }
    /// Kusanya PDF kamili: objects 1..n, kisha xref + trailer + Info.
    fn finish(self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"%PDF-1.4\n%\xc7\xec\x8f\xa2\n");
        let mut positions: Vec<usize> = Vec::with_capacity(self.objs.len());
        for o in &self.objs {
            positions.push(out.len());
            out.extend_from_slice(o.as_bytes());
        }
        let info_n = self.objs.len() + 1;
        let info_pos = out.len();
        out.extend_from_slice(
            format!("{info_n} 0 obj\n<< /Title (Hati ya FUNDI) /Producer (FUNDI MOBILE) >>\nendobj\n").as_bytes(),
        );
        let xref_pos = out.len();
        let mut xref = format!("xref\n0 {}\n", info_n + 1);
        xref.push_str("0000000000 65535 f \n");
        for p in &positions {
            xref.push_str(&format!("{p:010} 00000 n \n"));
        }
        xref.push_str(&format!("{info_pos:010} 00000 n \n"));
        out.extend_from_slice(xref.as_bytes());
        out.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R /Info {} 0 R >>\nstartxref\n{}\n%%EOF\n",
                info_n + 1,
                info_n,
                xref_pos
            )
            .as_bytes(),
        );
        out
    }
}

/// PDF kamili kutoka kwa "ukurasa content stream". Logo inachorwa kwa
/// vector (Form XObject kutoka logo::vector_logo_ops) — PDF halisi 100%.
fn build_pdf(draw: &str) -> Vec<u8> {
    let mut b = PdfBuilder::new();
    let cat = b.obj("<< /Type /Catalog /Pages 2 0 R >>");
    let pages = b.obj("<< /Type /Pages /Kids [3 0 R] /Count 1 >>");
    let page = b.obj(
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 4 0 R /F2 5 0 R >> /XObject << /LOGO 6 0 R >> >> /Contents 3 0 R >>",
    );
    let content = format!("<< /Length {} >>\nstream\n{}\nendstream", draw.len(), draw);
    let contents = b.obj(&content);
    let f1 = b.obj("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>");
    let f2 = b.obj("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>");
    let logo_ops = crate::logo::vector_logo_ops();
    let logo_stream = format!(
        "<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] /Resources << >> /Length {} >>\nstream\n{}\nendstream",
        logo_ops.len(),
        logo_ops
    );
    let logo = b.obj(&logo_stream);
    let _ = (cat, pages, page, contents, f1, f2, logo);
    b.finish()
}

// ---------- page drawing helpers ----------

fn txt(x: f64, y: f64, size: f64, bold: bool, r: f64, g: f64, bl: f64, s: &str) -> String {
    format!(
        "BT /{} {} Tf {} {} {} rg {} {} Td ({}) Tj ET\n",
        if bold { "F2" } else { "F1" },
        size,
        r,
        g,
        bl,
        x,
        y,
        pdf_escape(&winansi(s))
    )
}

fn line(x1: f64, y1: f64, x2: f64, y2: f64, w: f64, r: f64, g: f64, bl: f64) -> String {
    format!("{} {} {} RG {w} w {x1} {y1} m {x2} {y2} l S\n", r, g, bl)
}

fn rect_fill(x: f64, y: f64, w: f64, h: f64, r: f64, g: f64, bl: f64) -> String {
    format!("{} {} {} rg {x} {y} {w} {h} re f\n", r, g, bl)
}

/// header ya kawaida: logo + brand + mstari wa chini (A4 595×842)
fn header(kind_sw: &str, doc_id: &str) -> String {
    let mut s = String::new();
    s.push_str("q 44 0 0 44 40 762 cm /LOGO Do Q\n");
    s.push_str(&txt(96.0, 788.0, 20.0, true, 0.0, 0.51, 0.56, "FUNDI"));
    s.push_str(&txt(96.0, 772.0, 8.5, false, 0.33, 0.43, 0.47, "MTAALAMU SMART - Mfumo wa mtaalamu"));
    s.push_str(&txt(400.0, 788.0, 16.0, true, 0.04, 0.13, 0.15, kind_sw));
    s.push_str(&txt(400.0, 772.0, 11.0, false, 0.33, 0.43, 0.47, doc_id));
    s.push_str(&rect_fill(40.0, 756.0, 515.0, 2.5, 0.0, 0.51, 0.56));
    s
}

fn footer() -> String {
    let mut s = String::new();
    s.push_str(&line(40.0, 70.0, 555.0, 70.0, 0.8, 0.78, 0.86, 0.87));
    s.push_str(&txt(
        40.0,
        56.0,
        8.0,
        false,
        0.42,
        0.47,
        0.5,
        "Malipo yanapokelewa na mtoa huduma (namba za malipo za fundi). Hati imetengenezwa na FUNDI MOBILE.",
    ));
    s
}

// ---------- B2B quote / invoice PDF ----------

pub fn b2b_doc_pdf(doc: &B2BDoc) -> Vec<u8> {
    let title = if doc.kind == "quote" { "QUOTE (OLE BEI)" } else { "INVOICE" };
    let mut d = String::new();
    d.push_str(&header(title, &doc.id));

    // taarifa
    let date = chrono::DateTime::from_timestamp(doc.created_ts as i64, 0)
        .map(|x| x.format("%Y-%m-%d").to_string())
        .unwrap_or_default();
    d.push_str(&txt(40.0, 726.0, 10.0, true, 0.04, 0.13, 0.15, "Fundi / Mtoa huduma:"));
    d.push_str(&txt(150.0, 726.0, 10.0, false, 0.1, 0.16, 0.2, &doc.fundi));
    d.push_str(&txt(40.0, 710.0, 10.0, true, 0.04, 0.13, 0.15, "Mteja (kampuni):"));
    d.push_str(&txt(150.0, 710.0, 10.0, false, 0.1, 0.16, 0.2, &doc.company));
    if !doc.company_contact.is_empty() {
        d.push_str(&txt(40.0, 694.0, 10.0, true, 0.04, 0.13, 0.15, "Mawasiliano:"));
        d.push_str(&txt(150.0, 694.0, 10.0, false, 0.1, 0.16, 0.2, &doc.company_contact));
    }
    if let Some(site) = &doc.site {
        d.push_str(&txt(40.0, 678.0, 10.0, true, 0.04, 0.13, 0.15, "Eneo/tawi:"));
        d.push_str(&txt(150.0, 678.0, 10.0, false, 0.1, 0.16, 0.2, site));
    }
    d.push_str(&txt(40.0, 662.0, 10.0, true, 0.04, 0.13, 0.15, "Tarehe:"));
    d.push_str(&txt(150.0, 662.0, 10.0, false, 0.1, 0.16, 0.2, &date));
    if let Some(rq) = &doc.ref_quote {
        d.push_str(&txt(40.0, 646.0, 10.0, true, 0.04, 0.13, 0.15, "Rejea:"));
        d.push_str(&txt(150.0, 646.0, 10.0, false, 0.1, 0.16, 0.2, rq));
    }

    // jedwali: vichwa
    let mut y = 620.0;
    d.push_str(&rect_fill(40.0, y - 6.0, 515.0, 22.0, 0.88, 0.97, 0.98));
    d.push_str(&txt(48.0, y, 10.0, true, 0.0, 0.29, 0.33, "Kazi / Vifaa"));
    d.push_str(&txt(330.0, y, 10.0, true, 0.0, 0.29, 0.33, "Idadi"));
    d.push_str(&txt(410.0, y, 10.0, true, 0.0, 0.29, 0.33, "Bei (TZS)"));
    d.push_str(&txt(490.0, y, 10.0, true, 0.0, 0.29, 0.33, "Jumla"));
    y -= 22.0;

    let n = doc.items.len();
    for (i, it) in doc.items.iter().enumerate() {
        if i % 2 == 1 {
            d.push_str(&rect_fill(40.0, y - 6.0, 515.0, 20.0, 0.96, 0.99, 1.0));
        }
        let desc = if it.desc.chars().count() > 48 {
            format!("{}\u{2026}", it.desc.chars().take(48).collect::<String>())
        } else {
            it.desc.clone()
        };
        d.push_str(&txt(48.0, y, 9.5, false, 0.1, 0.16, 0.2, &desc));
        d.push_str(&txt(340.0, y, 9.5, false, 0.1, 0.16, 0.2, &it.qty.to_string()));
        d.push_str(&txt(410.0, y, 9.5, false, 0.1, 0.16, 0.2, &money(it.price)));
        d.push_str(&txt(490.0, y, 9.5, false, 0.1, 0.16, 0.2, &money(it.price * it.qty as u64)));
        y -= 20.0;
    }
    d.push_str(&line(40.0, y + 14.0, 555.0, y + 14.0, 0.8, 0.78, 0.86, 0.87));

    // jumla
    y -= 6.0;
    d.push_str(&rect_fill(330.0, y - 8.0, 225.0, 30.0, 0.0, 0.51, 0.56));
    d.push_str(&txt(342.0, y + 2.0, 13.0, true, 1.0, 1.0, 1.0, &format!("JUMLA: {} {}", doc.currency, money(doc.total))));

    // hali + note
    y -= 34.0;
    d.push_str(&txt(40.0, y, 10.0, true, 0.04, 0.13, 0.15, &format!("Hali: {}", doc.status.to_uppercase())));
    if let Some(note) = &doc.note_sw {
        y -= 16.0;
        let note = if note.chars().count() > 90 {
            format!("{}\u{2026}", note.chars().take(90).collect::<String>())
        } else {
            note.clone()
        };
        d.push_str(&txt(40.0, y, 9.0, false, 0.33, 0.43, 0.47, &format!("Angalizo: {note}")));
    }

    let _ = n; // items zote zinaonekana kwenye ukurasa mmoja (fupi)
    d.push_str(&footer());
    build_pdf(&d)
}

// ---------- Fundi Deploy report PDF ----------

/// Ripoti ya deploy (jobs za LAN) — PDF yenye logo ya FUNDI.
pub fn deploy_report_pdf(
    hostname: &str,
    generated: &str,
    jobs: &[(String, String, String, String, u32, String)], // (id, pc, os, status, progress, message)
) -> Vec<u8> {
    let mut d = String::new();
    d.push_str(&header("RIPOTI YA DEPLOY", &format!("FD-{}", chrono::Local::now().format("%Y%m%d-%H%M"))));
    d.push_str(&txt(40.0, 726.0, 10.0, true, 0.04, 0.13, 0.15, "Server/HOST:"));
    d.push_str(&txt(140.0, 726.0, 10.0, false, 0.1, 0.16, 0.2, hostname));
    d.push_str(&txt(40.0, 710.0, 10.0, true, 0.04, 0.13, 0.15, "Imetengenezwa:"));
    d.push_str(&txt(140.0, 710.0, 10.0, false, 0.1, 0.16, 0.2, generated));

    let mut y = 686.0;
    d.push_str(&rect_fill(40.0, y - 6.0, 515.0, 22.0, 0.88, 0.97, 0.98));
    d.push_str(&txt(48.0, y, 9.5, true, 0.0, 0.29, 0.33, "Job"));
    d.push_str(&txt(170.0, y, 9.5, true, 0.0, 0.29, 0.33, "PC"));
    d.push_str(&txt(260.0, y, 9.5, true, 0.0, 0.29, 0.33, "OS"));
    d.push_str(&txt(330.0, y, 9.5, true, 0.0, 0.29, 0.33, "Hali"));
    d.push_str(&txt(400.0, y, 9.5, true, 0.0, 0.29, 0.33, "%"));
    d.push_str(&txt(430.0, y, 9.5, true, 0.0, 0.29, 0.33, "Ujumbe"));
    y -= 22.0;

    let mut shown = 0;
    for (id, pc, os, status, prog, msg) in jobs {
        if y < 120.0 {
            break; // ukurasa mmoja — ripoti fupi
        }
        if shown % 2 == 1 {
            d.push_str(&rect_fill(40.0, y - 6.0, 515.0, 20.0, 0.96, 0.99, 1.0));
        }
        let short = |s: &str, n: usize| {
            if s.chars().count() > n {
                format!("{}\u{2026}", s.chars().take(n).collect::<String>())
            } else {
                s.to_string()
            }
        };
        d.push_str(&txt(48.0, y, 8.5, false, 0.1, 0.16, 0.2, &short(id, 14)));
        d.push_str(&txt(170.0, y, 8.5, false, 0.1, 0.16, 0.2, &short(pc, 10)));
        d.push_str(&txt(260.0, y, 8.5, false, 0.1, 0.16, 0.2, &short(os, 8)));
        d.push_str(&txt(330.0, y, 8.5, false, 0.1, 0.16, 0.2, &short(status, 12)));
        d.push_str(&txt(400.0, y, 8.5, false, 0.1, 0.16, 0.2, &format!("{prog}%")));
        d.push_str(&txt(430.0, y, 8.5, false, 0.1, 0.16, 0.2, &short(msg, 34)));
        y -= 20.0;
        shown += 1;
    }
    if shown == 0 {
        d.push_str(&txt(48.0, y, 9.5, false, 0.42, 0.47, 0.5, "Hakuna jobs bado."));
    }
    d.push_str(&footer());
    build_pdf(&d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::b2b::B2BItem;

    #[test]
    fn pdf_ya_quote_ina_header_sahihi() {
        let doc = B2BDoc {
            id: "QT-TEST-0001".into(),
            kind: "quote".into(),
            fundi: "Juma Fundi".into(),
            company: "Kampuni Ltd".into(),
            company_contact: "0712345678".into(),
            site: Some("Kariakoo".into()),
            items: vec![B2BItem { desc: "Screen replacement".into(), qty: 2, price: 60_000 }],
            total: 120_000,
            currency: "TZS".into(),
            status: "sent".into(),
            ref_quote: None,
            note_sw: Some("Bei inaisha baada ya siku 14".into()),
            created_ts: 1_760_000_000,
        };
        let pdf = b2b_doc_pdf(&doc);
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(pdf.len() > 400, "PDF si tupu");
        assert!(pdf.windows(5).any(|w| w == b"%%EOF"));
        let txt = String::from_utf8_lossy(&pdf);
        assert!(txt.contains("FUNDI"), "PDF ina brand FUNDI");
        assert!(txt.contains("Kampuni Ltd"), "PDF ina jina la mteja");
        assert!(txt.contains("120,000"), "PDF ina jumla (TZS 120,000)");
    }

    #[test]
    fn deploy_pdf_ina_jobs() {
        let jobs = vec![(
            "JOB-1".to_string(),
            "PC-OFFICE".to_string(),
            "ubuntu".to_string(),
            "done".to_string(),
            100u32,
            "Kamili".to_string(),
        )];
        let pdf = deploy_report_pdf("server-lab", "2026-09-30 10:00", &jobs);
        let txt = String::from_utf8_lossy(&pdf);
        assert!(txt.contains("RIPOTI YA DEPLOY"));
        assert!(txt.contains("PC-OFFICE"));
    }
}
