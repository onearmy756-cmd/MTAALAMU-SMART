//! pdf_extract.rs — Kutoa maandishi HALISI kutoka PDF + utafutaji parallel wa PDF nyingi.
//!
//! Inaunga mkono:
//!  - PDF 1.x zenye streams za FlateDecode (zlib) na LZWDecode
//!  - Text operators: Tj, TJ, ' na \" (ndani ya BT/ET)
//!  - Escape sequences za PDF strings (\n \r \t \( \) \\ \ddd octal)
//!  - Utafutaji wa PDF nyingi KWA WAKATI MMOJA (std::thread — kasi × N cores)
//!
//! Hakuna dependency ya nje kwa PDF yenyewe: flate2 tu kwa zlib.

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

// ------------------------------------------------------------ zlib helpers

pub fn zlib_decompress(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    ZlibDecoder::new(data)
        .read_to_end(&mut out)
        .map_err(|e| format!("zlib: {}", e))?;
    Ok(out)
}

pub fn zlib_compress(data: &[u8]) -> Vec<u8> {
    let mut enc = ZlibEncoder::new(Vec::new(), Compression::default());
    let _ = enc.write_all(data);
    enc.finish().unwrap_or_default()
}

// ---------------------------------------------------------------- LZWDecode

/// LZW decoder ya PDF (early change 1 — kama Acrobat default).
fn lzw_decode(data: &[u8]) -> Result<Vec<u8>, String> {
    let mut dict: Vec<Vec<u8>> = (0..256).map(|i| vec![i as u8]).collect();
    dict.push(vec![]); // 256 = clear
    dict.push(vec![]); // 257 = EOD
    let mut out = Vec::new();
    let mut bitpos = 0usize;
    let mut code_len = 9usize;
    let mut prev: Option<Vec<u8>> = None;
    let total_bits = data.len() * 8;

    let read_code = |bitpos: &mut usize, code_len: usize| -> Option<u64> {
        if *bitpos + code_len > total_bits {
            return None;
        }
        let mut code: u64 = 0;
        for _ in 0..code_len {
            let byte = data[*bitpos / 8];
            let bit = (byte >> (7 - (*bitpos % 8))) & 1;
            code = (code << 1) | bit as u64;
            *bitpos += 1;
        }
        Some(code)
    };

    while let Some(code) = read_code(&mut bitpos, code_len) {
        match code {
            256 => {
                code_len = 9;
                dict.truncate(258);
                prev = None;
                continue;
            }
            257 => break,
            c => {
                let entry = if (c as usize) < dict.len() {
                    dict[c as usize].clone()
                } else if let Some(p) = &prev {
                    let mut e = p.clone();
                    if let Some(first) = p.first() {
                        e.push(*first);
                    }
                    e
                } else {
                    return Err("LZW: code batili".into());
                };
                out.extend_from_slice(&entry);
                if let Some(p) = &prev {
                    let mut ne = p.clone();
                    if let Some(first) = entry.first() {
                        ne.push(*first);
                    }
                    dict.push(ne);
                    if dict.len() + 1 >= (1 << code_len) && code_len < 12 {
                        code_len += 1;
                    }
                }
                prev = Some(entry);
            }
        }
    }
    Ok(out)
}

// ------------------------------------------------------------ PDF text pull

#[derive(Debug, Clone)]
pub struct PdfText {
    pub pages: usize,
    pub text: String,
    pub streams_decoded: usize,
    pub warnings: Vec<String>,
}

/// Toa maandishi kutoka faili ya PDF (byte-level parser nyepesi ila HALISI).
pub fn pdf_to_text(bytes: &[u8]) -> Result<PdfText, String> {
    if bytes.starts_with(b"%PDF-") == false {
        return Err("Si PDF (header ya %PDF- haipo)".into());
    }
    let mut pages = 0usize;
    let mut streams_decoded = 0usize;
    let mut warnings = Vec::new();
    let mut full_text = String::new();

    // Tafuta kila "stream ... endstream"
    let mut i = 0usize;
    while let Some(rel) = find(&bytes[i..], b"stream") {
        let s = i + rel;
        // epuka "endstream" — hakikisha "stream" ni keyword kamili
        if s > 0 && (bytes[s - 1] == b'd' || bytes[s - 1] == b'E') {
            i = s + 6;
            continue;
        }
        // dictionary iliyotangulia
        let dstart = i.max(s.saturating_sub(4096));
        let dict = String::from_utf8_lossy(&bytes[dstart..s]).to_string();

        // anza baada ya EOL
        let mut data_start = s + 6;
        if data_start < bytes.len() && bytes[data_start] == b'\r' {
            data_start += 1;
        }
        if data_start < bytes.len() && bytes[data_start] == b'\n' {
            data_start += 1;
        }
        let Some(rel_end) = find(&bytes[data_start..], b"endstream") else {
            break;
        };
        let data_end = data_start + rel_end;
        let raw = &bytes[data_start..data_end];
        i = data_end + 9;

        let decoded: Option<Vec<u8>> = if dict.contains("/FlateDecode") {
            zlib_decompress(raw).ok().or_else(|| {
                // baadhi ya files hazina zlib header — jaribu raw deflate
                let mut d = flate2::read::DeflateDecoder::new(raw);
                let mut out = Vec::new();
                d.read_to_end(&mut out).ok()?;
                Some(out)
            })
        } else if dict.contains("/LZWDecode") {
            lzw_decode(raw).ok()
        } else if dict.contains("/Filter") {
            None // filter isiyotumwa (DCT/JPX n.k.) — picha, hakuna maandishi
        } else {
            Some(raw.to_vec()) // bila filter
        };

        match decoded {
            Some(content) => {
                streams_decoded += 1;
                let chunk = extract_text_ops(&content);
                let pages_here = chunk.matches('\u{000C}').count(); // form feeds
                pages += pages_here.max(1);
                full_text.push_str(&chunk);
                full_text.push('\n');
            }
            None => {
                warnings.push("stream moja haikudecode (filter isiyoungwa mkono)".into());
            }
        }
    }

    if full_text.trim().is_empty() {
        return Err("Hakuna maandishi yanayotolewa (PDF inaweza kuwa picha/scanned)".into());
    }
    Ok(PdfText {
        pages: pages.max(1),
        text: full_text,
        streams_decoded,
        warnings,
    })
}

/// Toa maandishi kutoka content stream ya PDF (operators Tj TJ ' ").
fn extract_text_ops(content: &[u8]) -> String {
    let mut out = String::new();
    let mut i = 0usize;
    let n = content.len();
    while i < n {
        match content[i] {
            b'(' => {
                // string literal — soma na escapes
                let (s, next) = read_pdf_string(content, i);
                out.push_str(&s);
                i = next;
            }
            b'[' => {
                // TJ array — soma strings zote ndani
                let mut j = i + 1;
                let mut depth = 1;
                while j < n && depth > 0 {
                    match content[j] {
                        b'[' => depth += 1,
                        b']' => depth -= 1,
                        b'(' => {
                            let (s, next) = read_pdf_string(content, j);
                            out.push_str(&s);
                            j = next;
                            continue;
                        }
                        _ => {}
                    }
                    j += 1;
                }
                i = j;
            }
            b'T' if i + 1 < n && (content[i + 1] == b'j') => {
                out.push(' '); // Tj — nafasi kati ya strings
                i += 2;
            }
            b'T' if i + 1 < n && content[i + 1] == b'*' => {
                out.push('\n'); // T* — mstari mpya
                i += 2;
            }
            b'\'' | b'"' => {
                out.push('\n');
                i += 1;
            }
            b'>' if i + 1 < n && content[i + 1] == b'>' => {
                i += 2;
            }
            _ => {
                // hex string <...> rudisha kama bytes (latin1)
                if content[i] == b'<' {
                    let mut j = i + 1;
                    let mut hex = String::new();
                    while j < n && content[j] != b'>' {
                        if content[j].is_ascii_hexdigit() {
                            hex.push(content[j] as char);
                        }
                        j += 1;
                    }
                    if hex.len() % 2 == 1 {
                        hex.push('0');
                    }
                    for k in (0..hex.len()).step_by(2) {
                        let v = u8::from_str_radix(&hex[k..k + 2], 16).unwrap_or(32);
                        out.push(v as char);
                    }
                    out.push(' ');
                    i = j + 1;
                } else {
                    i += 1;
                }
            }
        }
    }
    // safisha nafasi nyingi
    let mut clean = String::new();
    let mut prev_space = false;
    for c in out.chars() {
        if c == ' ' || c == '\t' {
            if !prev_space {
                clean.push(' ');
            }
            prev_space = true;
        } else {
            clean.push(c);
            prev_space = c == '\n';
        }
    }
    clean
}

/// Soma PDF string literal kuanzia '(' — inarudisha (maandishi, position baada ya ')').
fn read_pdf_string(content: &[u8], start: usize) -> (String, usize) {
    let mut out = String::new();
    let mut i = start + 1;
    let mut depth = 1usize;
    while i < content.len() {
        match content[i] {
            b'\\' => {
                if i + 1 >= content.len() {
                    break;
                }
                let c = content[i + 1];
                match c {
                    b'n' => {
                        out.push('\n');
                        i += 2;
                    }
                    b'r' => {
                        out.push('\n');
                        i += 2;
                    }
                    b't' => {
                        out.push('\t');
                        i += 2;
                    }
                    b'b' | b'f' => {
                        i += 2;
                    }
                    b'(' => {
                        out.push('(');
                        i += 2;
                    }
                    b')' => {
                        out.push(')');
                        i += 2;
                    }
                    b'\\' => {
                        out.push('\\');
                        i += 2;
                    }
                    b'0'..=b'7' => {
                        // octal \ddd (hadi tarakimu 3)
                        let mut val = 0u32;
                        let mut k = i + 1;
                        let mut count = 0;
                        while k < content.len() && count < 3 && (b'0'..=b'7').contains(&content[k]) {
                            val = val * 8 + (content[k] - b'0') as u32;
                            k += 1;
                            count += 1;
                        }
                        out.push((val.min(255)) as u8 as char);
                        i = k;
                    }
                    b'\n' => {
                        i += 2; // line continuation
                    }
                    _ => {
                        out.push(c as char);
                        i += 2;
                    }
                }
            }
            b'(' => {
                depth += 1;
                out.push('(');
                i += 1;
            }
            b')' => {
                depth -= 1;
                if depth == 0 {
                    i += 1;
                    break;
                }
                out.push(')');
                i += 1;
            }
            c => {
                out.push(c as char);
                i += 1;
            }
        }
    }
    (out, i)
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

// ------------------------------------------------------- Parallel PDF search

#[derive(Debug, Clone, Serialize)]
pub struct PdfHit {
    pub file: String,
    pub page_hint: usize,
    pub snippet: String,
    pub score: i64,
}

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PdfSearchResult {
    pub query: String,
    pub searched: usize,
    pub matched: usize,
    pub duration_ms: u128,
    pub hits: Vec<PdfHit>,
}

fn score_text(text: &str, terms: &[String]) -> (i64, Option<(usize, usize)>) {
    let lower = text.to_lowercase();
    let mut score = 0i64;
    let mut first_pos: Option<(usize, usize)> = None;
    for t in terms {
        let mut from = 0usize;
        let mut count = 0i64;
        while let Some(p) = lower[from..].find(t.as_str()) {
            count += 1;
            let abs = from + p;
            if first_pos.is_none() {
                first_pos = Some((abs, abs + t.len()));
            }
            from = abs + t.len().max(1);
            if from >= lower.len() {
                break;
            }
        }
        score += count * 10;
    }
    (score, first_pos)
}

fn snippet_around(text: &str, pos: usize, len: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    let pos = pos.min(chars.len());
    let s = pos.saturating_sub(60);
    let e = (s + len + 120).min(chars.len());
    format!(
        "...{}...",
        chars[s..e].iter().collect::<String>().replace('\n', " ")
    )
}

/// Tafuta PDF nyingi KWA WAKATI MMOJA (thread kwa kila PDF, worker pool nyepesi).
pub fn search_pdfs(dir: &Path, query: &str, limit: usize, max_files: usize) -> Result<PdfSearchResult, String> {
    let start = std::time::Instant::now();
    if !dir.exists() {
        return Err(format!("Folda haipo: {}", dir.display()));
    }
    let terms: Vec<String> = query
        .to_lowercase()
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
    if terms.is_empty() {
        return Err("Tafuta nini? (query tupu)".into());
    }

    let mut pdfs: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("read_dir: {}", e))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .map(|e| e.eq_ignore_ascii_case("pdf"))
                .unwrap_or(false)
        })
        .collect();
    pdfs.sort();
    pdfs.truncate(max_files);

    let (tx, rx) = std::sync::mpsc::channel::<PdfHit>();
    let terms_arc = std::sync::Arc::new(terms);
    let mut handles = Vec::new();

    // thread kwa kila PDF (hadi 16 kwa wakati mmoja)
    for (chunk_idx, chunk) in pdfs.chunks((pdfs.len() / 16).max(1)).enumerate() {
        let _ = chunk_idx;
        let terms = terms_arc.clone();
        let tx = tx.clone();
        let chunk: Vec<PathBuf> = chunk.to_vec();
        handles.push(std::thread::spawn(move || {
            for path in chunk {
                let Ok(bytes) = fs::read(&path) else { continue };
                let Ok(doc) = pdf_to_text(&bytes) else { continue };
                let (score, pos) = score_text(&doc.text, &terms);
                if score > 0 {
                    let snippet = pos
                        .map(|(s, l)| snippet_around(&doc.text, s, l))
                        .unwrap_or_else(|| doc.text.chars().take(140).collect());
                    let _ = tx.send(PdfHit {
                        file: path.file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default(),
                        page_hint: doc.pages,
                        snippet,
                        score,
                    });
                }
            }
        }));
    }
    drop(tx);

    let mut hits: Vec<PdfHit> = rx.into_iter().collect();
    for h in handles {
        let _ = h.join();
    }
    hits.sort_by(|a, b| b.score.cmp(&a.score));
    let searched = pdfs.len();
    let matched = hits.len();
    hits.truncate(limit);

    Ok(PdfSearchResult {
        query: query.to_string(),
        searched,
        matched,
        duration_ms: start.elapsed().as_millis(),
        hits,
    })
}

// ------------------------------------------------------------------- Tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_string_escape() {
        let content = b"(Hello \\(world\\) 100\\%) Tj";
        let (s, _) = read_pdf_string(content, 0);
        assert_eq!(s, "Hello (world) 100%");
    }

    #[test]
    fn test_extract_text_ops() {
        let content = b"BT /F1 12 Tf (Umeme wa) Tj T* (Tanzania) Tj ET";
        let t = extract_text_ops(content);
        assert!(t.contains("Umeme wa"));
        assert!(t.contains("Tanzania"));
    }

    #[test]
    fn test_extract_tj_array() {
        let content = b"[(Ka) 120 (bulanga)] TJ";
        let t = extract_text_ops(content);
        assert!(t.contains("Ka"), "got: {}", t);
        assert!(t.contains("bulanga"), "got: {}", t);
    }

    #[test]
    fn test_zlib_roundtrip() {
        let data = b"MTAALAMU SMART PDF engine halisi".repeat(50);
        let compressed = zlib_compress(&data);
        let back = zlib_decompress(&compressed).unwrap();
        assert_eq!(back, data);
    }

    #[test]
    fn test_score_text() {
        let (s, p) = score_text("Umeme wa Dar es Salaam umekatika", &["umeme".into(), "dar".into()]);
        assert!(s >= 20);
        assert!(p.is_some());
    }
}
