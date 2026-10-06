//! reports.rs — RIPOTI RASMI ZA WATEJA (H13): PDF (brand ya MTECH OS/Mbilinyi
//! Tech) na CSV, kwa aina 4: usalama (secops), forensics, bili (ledger), kazi
//! (jobs). Kanuni ya mmiliki: ripoti ina MAJINA SALAMA tu — hakuna zana,
//! engine wala usanifu. Kila ripoti ya PDF inalipwa (BILI gate).
//!
//! PDF hutengenezwa kwa PdfBuilder ya report.rs (PDF 1.4 halisi, bila deps) —
//! tumebadilisha tu brand kwenda MTECH OS (report.rs ya kale ilikuwa "FUNDI").

use serde_json::json;
use sqlx::SqlitePool;

pub const BILL_KEY_REPORT: &str = "report";
pub const PRICE_REPORT_TZS: u64 = 5_000;

// ---------- CSV ----------
/// Escape ya CSV (RFC 4180): comma/quote/newline → nukuu mbili
pub fn csv_field(s: &str) -> String {
    let needs = s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r');
    if needs {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

pub fn csv_doc(title: &str, header: &[&str], rows: &[Vec<String>]) -> String {
    let mut out = String::new();
    out.push_str(&format!("# MTECH OS — {title}\n"));
    out.push_str(&format!("# Mbilinyi Tech · {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M")));
    out.push_str(&header.join(","));
    out.push('\n');
    for r in rows {
        let fields: Vec<String> = r.iter().map(|f| csv_field(f)).collect();
        out.push_str(&fields.join(","));
        out.push('\n');
    }
    out
}

// ---------- DATA KUTOKA DB (access control: account yake PEKEE) ----------

/// Muhtasari wa usalama kwa account — kila kompyuta + afya + daraja
pub async fn security_rows(db: &SqlitePool, account: &str) -> Vec<Vec<String>> {
    let rows: Vec<(String, String, i64, String)> = sqlx::query_as(
        "SELECT at, target, health, severity FROM secops_reports WHERE account=?1 ORDER BY at DESC LIMIT 200",
    )
    .bind(account)
    .fetch_all(db)
    .await
    .unwrap_or_default();
    rows.into_iter()
        .map(|(at, target, health, sev)| {
            vec![at, target, health.to_string(), sev]
        })
        .collect()
}

/// Kesi za forensics — hash + vyanzo
pub async fn forensics_rows(db: &SqlitePool, account: &str) -> Vec<Vec<String>> {
    let rows: Vec<(String, String, String, String, i64)> = sqlx::query_as(
        "SELECT at, target, evidence_hash, sources, sealed FROM forensics_cases WHERE account=?1 ORDER BY at DESC LIMIT 200",
    )
    .bind(account)
    .fetch_all(db)
    .await
    .unwrap_or_default();
    rows.into_iter()
        .map(|(at, target, hash, sources, sealed)| {
            vec![at, target, hash, sources, if sealed == 1 { "IMEFUNGWA".into() } else { "WAZIMA".into() }]
        })
        .collect()
}

/// Ledger ya BILI — mapato/malipo yote
pub async fn billing_rows(db: &SqlitePool, account: &str) -> Vec<Vec<String>> {
    let rows: Vec<(String, String, i64, Option<String>, i64)> = sqlx::query_as(
        "SELECT created_at, kind, amount_tzs, job, balance_after FROM billing_ledger WHERE account=?1 ORDER BY id DESC LIMIT 300",
    )
    .bind(account)
    .fetch_all(db)
    .await
    .unwrap_or_default();
    rows.into_iter()
        .map(|(at, kind, amount, job, bal)| {
            vec![at, kind, amount.to_string(), job.unwrap_or_default(), bal.to_string()]
        })
        .collect()
}

/// Audit ya kazi za TOOLS — bila majina ya binaries (siri)
pub async fn tools_rows(db: &SqlitePool, _account: &str) -> Vec<Vec<String>> {
    let rows: Vec<(String, String, i64, i64, String)> = sqlx::query_as(
        "SELECT at, tool_id, ok, duration_ms, target FROM tool_runs ORDER BY at DESC LIMIT 200",
    )
    .fetch_all(db)
    .await
    .unwrap_or_default();
    rows.into_iter()
        .map(|(at, tool_id, ok, dur, target)| {
            let name = crate::toolkit::TOOLS
                .iter()
                .find(|t| t.id == tool_id)
                .map(|t| t.name_sw.to_string())
                .unwrap_or_else(|| "Huduma ya ndani".into());
            vec![at, name, if ok == 1 { "IMEFANIKA".into() } else { "IMEFELI".into() }, dur.to_string(), target]
        })
        .collect()
}

// ---------- RIPOTI: kind → (csv, meta) ----------

pub struct Report {
    pub title: String,
    pub header: Vec<&'static str>,
    pub rows: Vec<Vec<String>>,
    pub summary: Vec<(String, String)>,
}

pub async fn collect(db: &SqlitePool, kind: &str, account: &str) -> Option<Report> {
    match kind {
        "security" => {
            let rows = security_rows(db, account).await;
            let mut salama = 0i64;
            let mut tahadhari = 0i64;
            let mut hatari = 0i64;
            for r in &rows {
                match r[3].as_str() {
                    "salama" => salama += 1,
                    "tahadhari" => tahadhari += 1,
                    _ => hatari += 1,
                }
            }
            Some(Report {
                title: "RIPOTI YA USALAMA".into(),
                header: vec!["Wakati", "Kompyuta", "Afya %", "Daraja"],
                rows,
                summary: vec![
                    ("Salama".into(), salama.to_string()),
                    ("Tahadhari".into(), tahadhari.to_string()),
                    ("Hatari".into(), hatari.to_string()),
                ],
            })
        }
        "forensics" => {
            let rows = forensics_rows(db, account).await;
            let n = rows.len();
            Some(Report {
                title: "RIPOTI YA UCHANGANUZI WA KIDIJITALI".into(),
                header: vec!["Wakati", "Kompyuta", "Hash ya Ushahidi", "Vyanzo", "Hali"],
                rows,
                summary: vec![("Kesi".into(), n.to_string())],
            })
        }
        "bili" => {
            let rows = billing_rows(db, account).await;
            let mut tuzo = 0i64;
            let mut matumizi = 0i64;
            for r in &rows {
                let amt: i64 = r[2].parse().unwrap_or(0);
                if amt > 0 { tuzo += amt } else { matumizi += -amt }
            }
            Some(Report {
                title: "RIPOTI YA BILI (MATUMIZI NA MALIPO)".into(),
                header: vec!["Wakati", "Aina", "Kiasi TZS", "Kazi", "Salio baada ya"],
                rows,
                summary: vec![
                    ("Hela iliyoingizwa".into(), tuzo.to_string()),
                    ("Matumizi".into(), matumizi.to_string()),
                ],
            })
        }
        "kazi" => {
            let rows = tools_rows(db, account).await;
            let okc = rows.iter().filter(|r| r[2] == "IMEFANIKA").count();
            let n = rows.len();
            Some(Report {
                title: "RIPOTI YA KAZI ZILIZOTANGULIA".into(),
                header: vec!["Wakati", "Kazi", "Hali", "Muda (ms)", "Kompyuta"],
                rows,
                summary: vec![
                    ("Kazi zote".into(), n.to_string()),
                    ("Zilizofanikiwa".into(), okc.to_string()),
                ],
            })
        }
        _ => None,
    }
}

pub fn to_csv(rep: &Report) -> String {
    let pairs: Vec<String> = rep.summary.iter().map(|(k, v)| format!("{k}: {v}")).collect();
    csv_doc(&format!("{} — {}", rep.title, pairs.join(" · ")), &rep.header, &rep.rows)
}

// ---------- BRANDED PDF (PDF 1.4 halisi — brand ya MTECH OS / Mbilinyi Tech) ----------

fn ptxt(x: f64, y: f64, size: f64, bold: bool, r: f64, g: f64, b: f64, s: &str) -> String {
    let esc = s.replace('\\', "\\\\").replace('(', "\\(").replace(')', "\\)");
    format!("BT /{} {} Tf {} {} {} rg {} {} Td ({}) Tj ET\n", if bold { "F2" } else { "F1" }, size, r, g, b, x, y, esc)
}

fn prect(x: f64, y: f64, w: f64, h: f64, r: f64, g: f64, b: f64) -> String {
    format!("{} {} {} rg {x} {y} {w} {h} re f\n", r, g, b)
}

/// PDF 1.4 kamili: ukurasa mmoja, brand MTECH OS, muhtasari + jedwali (mpaka rows 40).
pub fn branded_pdf(rep: &Report, account: &str) -> Vec<u8> {
    let mut c = String::new();
    // Kichwa: brand + mstari
    c.push_str(&ptxt(40.0, 792.0, 20.0, true, 0.05, 0.35, 0.42, "MTECH OS"));
    c.push_str(&ptxt(150.0, 792.0, 9.0, false, 0.45, 0.55, 0.58, "Mbilinyi Tech · Umiliki ni wako, leseni ni yako, faida ni yako"));
    c.push_str(&ptxt(40.0, 772.0, 14.0, true, 0.04, 0.13, 0.15, &rep.title));
    c.push_str(&ptxt(480.0, 772.0, 9.0, false, 0.45, 0.55, 0.58, &format!("Account: {account}")));
    c.push_str(&prect(40.0, 764.0, 515.0, 2.5, 0.05, 0.35, 0.42));

    // Muhtasari
    let mut y = 744.0;
    for (k, v) in &rep.summary {
        c.push_str(&ptxt(40.0, y, 10.0, true, 0.2, 0.25, 0.28, &format!("{k}:")));
        c.push_str(&ptxt(180.0, y, 10.0, false, 0.1, 0.15, 0.18, v));
        y -= 14.0;
    }
    c.push_str(&ptxt(40.0, y - 2.0, 8.0, false, 0.5, 0.55, 0.58,
        &format!("Imetengenezwa: {}", chrono::Local::now().format("%Y-%m-%d %H:%M"))));
    y -= 22.0;

    // Jedwali: header
    let cols = rep.header.len();
    let col_w = 515.0 / cols as f64;
    let mut x = 40.0;
    for h in &rep.header {
        c.push_str(&ptxt(x + 3.0, y, 8.5, true, 1.0, 1.0, 1.0, h));
        x += col_w;
    }
    c.push_str(&prect(40.0, y - 4.0, 515.0, 12.0, 0.05, 0.35, 0.42));
    // NOTE: header text lazima iwe JUU ya rect — tunaiandika tena baada ya rect
    let mut x = 40.0;
    for h in &rep.header {
        c.push_str(&ptxt(x + 3.0, y, 8.5, true, 1.0, 1.0, 1.0, h));
        x += col_w;
    }
    y -= 18.0;

    // Rows (hadi 38 — ukurasa mmoja)
    for (i, row) in rep.rows.iter().take(38).enumerate() {
        if i % 2 == 0 {
            c.push_str(&prect(40.0, y - 4.0, 515.0, 12.0, 0.93, 0.97, 0.98));
        }
        let mut x = 40.0;
        for cell in row {
            let text: String = cell.chars().take(28).collect();
            c.push_str(&ptxt(x + 3.0, y, 8.0, false, 0.1, 0.15, 0.18, &text));
            x += col_w;
        }
        y -= 14.0;
    }

    // Footer
    c.push_str(&prect(40.0, 40.0, 515.0, 1.5, 0.85, 0.88, 0.9));
    c.push_str(&ptxt(40.0, 28.0, 8.0, false, 0.45, 0.55, 0.58,
        "Ripoti hii imetengenezwa na MTECH OS (Mbilinyi Tech). Kazi zinafuata idhini ya msimamizi (HITL)."));
    c.push_str(&ptxt(40.0, 18.0, 8.0, false, 0.6, 0.65, 0.68,
        "Maelezo ya kina hayamo ndani — ripoti ina majina salama tu kwa usalama wa mfumo."));

    // PDF objects
    let content = c;
    let stream = format!("<< /Length {} >>\nstream\n{content}endstream\n", content.len());
    let mut objs: Vec<String> = Vec::new();
    objs.push("<< /Type /Catalog /Pages 2 0 R >>\n".into());
    objs.push("<< /Type /Pages /Kids [3 0 R] /Count 1 >>\n".into());
    objs.push("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 4 0 R /F2 5 0 R >> >> /Contents 6 0 R >>\n".into());
    objs.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\n".into());
    objs.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>\n".into());
    objs.push(stream);

    let mut out: Vec<u8> = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets: Vec<usize> = Vec::new();
    for (i, o) in objs.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{o}endobj\n", i + 1).as_bytes());
    }
    let xref_pos = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n", objs.len() + 1).as_bytes());
    out.extend_from_slice(b"0000000000 65535 f \n");
    for off in &offsets {
        out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!("trailer\n<< /Size {} /Root 1 0 R /Title (MTECH OS - Ripoti - Mbilinyi Tech) >>\nstartxref\n{y}\n%%EOF\n", objs.len() + 1)
            .replace(&format!("startxref\n{y}"), &format!("startxref\n{xref_pos}"))
            .as_bytes(),
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csv_field_inanukuu_sahihi() {
        assert_eq!(csv_field("salama"), "salama");
        assert_eq!(csv_field("a,b"), "\"a,b\"");
        assert_eq!(csv_field("ali \"fundi\""), "\"ali \"\"fundi\"\"\"");
        assert_eq!(csv_field("mstari\nmpya"), "\"mstari\nmpya\"");
    }

    #[test]
    fn csv_doc_ina_header_na_brand() {
        let rep = Report {
            title: "RIPOTI YA USALAMA".into(),
            header: vec!["Wakati", "Kompyuta", "Afya %", "Daraja"],
            rows: vec![vec!["2026-10-06".into(), "pc-01".into(), "80".into(), "salama".into()]],
            summary: vec![("Salama".into(), "1".into())],
        };
        let csv = to_csv(&rep);
        assert!(csv.starts_with("# MTECH OS"));
        assert!(csv.contains("Mbilinyi Tech"));
        assert!(csv.contains("Wakati,Kompyuta,Afya %,Daraja"));
        assert!(csv.contains("pc-01"));
    }

    #[test]
    fn branded_pdf_ina_brand_na_hakuna_fundi() {
        let rep = Report {
            title: "RIPOTI YA USALAMA".into(),
            header: vec!["Wakati", "Kompyuta", "Afya %", "Daraja"],
            rows: vec![
                vec!["2026-10-06T09:00".into(), "pc-01".into(), "90".into(), "salama".into()],
                vec!["2026-10-06T09:05".into(), "pc-02,lab".into(), "40".into(), "hatari".into()],
            ],
            summary: vec![("Hatari".into(), "1".into())],
        };
        let pdf = branded_pdf(&rep, "mteja1");
        assert!(pdf.starts_with(b"%PDF-1.4"));
        assert!(pdf.windows(5).any(|w| w == b"%%EOF"));
        let t = String::from_utf8_lossy(&pdf);
        assert!(t.contains("MTECH OS"));
        assert!(t.contains("Mbilinyi Tech"));
        assert!(t.contains("pc-01"));
        // comma field haijavuruga Jedwali (imeng'olewa chars) — na HAKUNA brand ya kale
        assert!(!t.contains("FUNDI"), "brand ya kale (FUNDI) hairuhusiwi kwenye ripoti za wateja");
        assert!(!t.contains("DEPLOY"));
    }

    #[tokio::test]
    async fn collect_inakusanya_kila_kind_na_access_control() {
        let db = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();
        crate::secops::init_tables(&db).await;
        crate::toolkit::init_tables(&db).await;
        crate::billing::init_tables(&db).await;
        // security report ya mteja1
        let f = crate::secops::Finding { kind: "vulnerability".into(), detail: "x".into(), severity: 40 };
        crate::secops::save_report(&db, "mteja1", "pc-01", "security", &[f]).await.unwrap();
        crate::billing::topup(&db, "mteja1", 20_000, "TXN-1").await.unwrap();
        let _ = sqlx::query("INSERT INTO tool_runs (tool_id, target, ok, lines, duration_ms, at) VALUES ('system-health','pc-01',1,3,120,?)")
            .bind(chrono::Local::now().to_rfc3339())
            .execute(&db)
            .await;
        // kila kind ina ripoti na rows
        for kind in ["security", "forensics", "bili", "kazi"] {
            let rep = collect(&db, kind, "mteja1").await.expect(kind);
            let csv = to_csv(&rep);
            assert!(csv.contains("MTECH OS"), "kind {kind}: CSV ina brand");
        }
        let sec = collect(&db, "security", "mteja1").await.unwrap();
        assert_eq!(sec.rows.len(), 1);
        assert_eq!(sec.rows[0][1], "pc-01");
        assert_eq!(sec.rows[0][3], "tahadhari");
        // access control: account nyingine haioni
        let other = collect(&db, "security", "mwingine").await.unwrap();
        assert_eq!(other.rows.len(), 0, "access control: ripoti za mwingine hazionekani");
        // kind isiyojulikana → None
        assert!(collect(&db, "siri", "mteja1").await.is_none());
    }
}
