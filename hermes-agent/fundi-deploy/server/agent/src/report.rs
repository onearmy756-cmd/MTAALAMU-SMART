//! report.rs — Ripoti ya PDF ya Fundi Deploy (logo ya FUNDI, bila dependencies).
//!
//! Kanuni kama fundi-mobile/src/pdf.rs: PDF 1.4 halisi, logo vector,
//! Helvetica. Ripoti ina jobs zote (id, PC, OS, hali, %, ujumbe).
//! Hii ndiyo "kitu kama ulichoweka kwenye fundi mobile" — export ya PDF.

use serde_json::json;

fn pdf_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('(', "\\(").replace(')', "\\)")
}

fn winansi(s: &str) -> String {
    // WinAnsi (CP1252) — Kiswahili; emoji → space, dashes/bullets → '-'
    s.chars()
        .map(|c| {
            let cp = c as u32;
            if (32..=255).contains(&cp) {
                match cp {
                    0x2026 => '\u{85}',
                    _ => c,
                }
            } else if (0x2000..0x2100).contains(&cp) {
                '-'
            } else if cp >= 0x1F000 {
                ' '
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
    fn finish(self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");
        let mut positions: Vec<usize> = Vec::with_capacity(self.objs.len());
        for o in &self.objs {
            positions.push(out.len());
            out.extend_from_slice(o.as_bytes());
        }
        let info_n = self.objs.len() + 1;
        let info_pos = out.len();
        out.extend_from_slice(
            format!("{info_n} 0 obj\n<< /Title (Ripoti ya FUNDI DEPLOY) /Producer (FUNDI DEPLOY) >>\nendobj\n")
                .as_bytes(),
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

fn rect_fill(x: f64, y: f64, w: f64, h: f64, r: f64, g: f64, bl: f64) -> String {
    format!("{} {} {} rg {x} {y} {w} {h} re f\n", r, g, bl)
}

fn hline(x1: f64, y: f64, x2: f64, w: f64, r: f64, g: f64, bl: f64) -> String {
    format!("{} {} {} RG {w} w {x1} {y} m {x2} {y} l S\n", r, g, bl)
}

/// Logo ya FUNDI (vector, kama fundi-mobile/src/logo.rs::vector_logo_ops)
fn vector_logo_ops() -> String {
    let mut s = String::new();
    s.push_str("0 0.898 1 rg 50 4 m 50 96 l 4 50 l h f\n");
    s.push_str("0 0.51 0.56 rg 50 96 m 50 4 l 96 50 l h f\n");
    s.push_str("1 1 1 rg\n");
    s.push_str("28 30 11 52 re f\n");
    s.push_str("28 71 32 11 re f\n");
    s.push_str("28 48 25 11 re f\n");
    s
}

fn build_pdf(draw: &str) -> Vec<u8> {
    let mut b = PdfBuilder::new();
    b.obj("<< /Type /Catalog /Pages 2 0 R >>");
    b.obj("<< /Type /Pages /Kids [3 0 R] /Count 1 >>");
    b.obj(
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 4 0 R /F2 5 0 R >> /XObject << /LOGO 6 0 R >> >> /Contents 3 0 R >>",
    );
    let content = format!("<< /Length {} >>\nstream\n{}\nendstream", draw.len(), draw);
    b.obj(&content);
    b.obj("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>");
    b.obj("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>");
    let logo_ops = vector_logo_ops();
    b.obj(&format!(
        "<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] /Resources << >> /Length {} >>\nstream\n{}\nendstream",
        logo_ops.len(),
        logo_ops
    ));
    b.finish()
}

fn short(s: &str, n: usize) -> String {
    if s.chars().count() > n {
        format!("{}\u{2026}", s.chars().take(n).collect::<String>())
    } else {
        s.to_string()
    }
}

/// PDF kamili: kichwa (logo FUNDI) + muhtasari + jedwali la jobs.
pub fn jobs_report_pdf(
    summary: &serde_json::Value,
    jobs: &[serde_json::Value],
) -> Vec<u8> {
    let mut d = String::new();
    // kichwa: logo + brand
    d.push_str("q 44 0 0 44 40 762 cm /LOGO Do Q\n");
    d.push_str(&txt(96.0, 788.0, 20.0, true, 0.0, 0.51, 0.56, "FUNDI"));
    d.push_str(&txt(96.0, 772.0, 8.5, false, 0.33, 0.43, 0.47, "DEPLOY · LAN imaging + cloud"));
    d.push_str(&txt(400.0, 788.0, 16.0, true, 0.04, 0.13, 0.15, "RIPOTI YA DEPLOY"));
    let stamp = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    d.push_str(&txt(400.0, 772.0, 11.0, false, 0.33, 0.43, 0.47, &format!("FD-{stamp}")));
    d.push_str(&rect_fill(40.0, 756.0, 515.0, 2.5, 0.0, 0.51, 0.56));

    // muhtasari
    let getn = |v: &serde_json::Value, k: &str| v[k].as_i64().unwrap_or(0);
    d.push_str(&txt(40.0, 726.0, 10.0, true, 0.04, 0.13, 0.15, "Muhtasari:"));
    d.push_str(&txt(
        40.0,
        710.0,
        10.0,
        false,
        0.1,
        0.16,
        0.2,
        &format!(
            "jumla {} · zinasubiri idhini {} · zinaendesha {} · kamili {} · zimeshindwa {}",
            getn(summary, "total"),
            getn(summary, "awaiting_your_approval"),
            getn(summary, "running"),
            getn(summary, "done"),
            getn(summary, "failed")
        ),
    ));

    // jedwali la jobs
    let mut y = 686.0;
    d.push_str(&rect_fill(40.0, y - 6.0, 515.0, 22.0, 0.88, 0.97, 0.98));
    d.push_str(&txt(48.0, y, 9.5, true, 0.0, 0.29, 0.33, "Job"));
    d.push_str(&txt(175.0, y, 9.5, true, 0.0, 0.29, 0.33, "PC"));
    d.push_str(&txt(265.0, y, 9.5, true, 0.0, 0.29, 0.33, "OS"));
    d.push_str(&txt(325.0, y, 9.5, true, 0.0, 0.29, 0.33, "Hali"));
    d.push_str(&txt(395.0, y, 9.5, true, 0.0, 0.29, 0.33, "%"));
    d.push_str(&txt(425.0, y, 9.5, true, 0.0, 0.29, 0.33, "Ujumbe"));
    y -= 22.0;

    for (i, j) in jobs.iter().enumerate() {
        if y < 100.0 {
            break;
        }
        let id = j["id"].as_str().unwrap_or("?");
        let pc = j["device_name"].as_str().unwrap_or("?");
        let os = j["os_type"].as_str().unwrap_or("?");
        let st = j["status"].as_str().unwrap_or("?");
        let msg = j["message"].as_str().unwrap_or("");
        let prog = j["progress"].as_u64().unwrap_or(0);
        if i % 2 == 1 {
            d.push_str(&rect_fill(40.0, y - 6.0, 515.0, 20.0, 0.96, 0.99, 1.0));
        }
        d.push_str(&txt(48.0, y, 8.5, false, 0.1, 0.16, 0.2, &short(&id[..8.min(id.len())], 8)));
        d.push_str(&txt(175.0, y, 8.5, false, 0.1, 0.16, 0.2, &short(pc, 10)));
        d.push_str(&txt(265.0, y, 8.5, false, 0.1, 0.16, 0.2, &short(os, 6)));
        d.push_str(&txt(325.0, y, 8.5, false, 0.1, 0.16, 0.2, &short(st, 8)));
        d.push_str(&txt(395.0, y, 8.5, false, 0.1, 0.16, 0.2, &format!("{prog}%")));
        d.push_str(&txt(425.0, y, 8.5, false, 0.1, 0.16, 0.2, &short(msg, 36)));
        y -= 20.0;
    }
    if jobs.is_empty() {
        d.push_str(&txt(48.0, y, 9.5, false, 0.42, 0.47, 0.5, "Hakuna jobs bado."));
    }

    // footer
    d.push_str(&hline(40.0, 70.0, 555.0, 0.8, 0.78, 0.86, 0.87));
    d.push_str(&txt(
        40.0,
        56.0,
        8.0,
        false,
        0.42,
        0.47,
        0.5,
        "Ripoti hii imetengenezwa na FUNDI DEPLOY (Agent inafanya kazi; msimamizi anasimamia).",
    ));
    build_pdf(&d)
}

/// JSON ya health (inatumika kwenye /report/pdf meta)
pub fn report_meta() -> serde_json::Value {
    json!({ "kind": "deploy_report", "producer": "FUNDI DEPLOY", "logo": "fundi-vector" })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pdf_ina_kichwa_na_jobs() {
        let summary = json!({ "total": 2, "awaiting_your_approval": 1, "running": 0, "done": 1, "failed": 0 });
        let jobs = vec![
            json!({ "id": "aaaaaaaa-bbbb-cccc", "device_name": "PC-01", "os_type": "ubuntu", "status": "done", "progress": 100, "message": "Kamili" }),
            json!({ "id": "dddddddd-eeee-ffff", "device_name": "PC-02", "os_type": "auto", "status": "awaiting_approval", "progress": 10, "message": "Subiri idhini" }),
        ];
        let pdf = jobs_report_pdf(&summary, &jobs);
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(pdf.windows(5).any(|w| w == b"%%EOF"));
        let t = String::from_utf8_lossy(&pdf);
        assert!(t.contains("FUNDI"));
        assert!(t.contains("PC-01"));
        assert!(t.contains("PC-02"));
    }
}
