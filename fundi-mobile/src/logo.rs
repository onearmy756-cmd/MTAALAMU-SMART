//! logo.rs — LOGO rasmi ya FUNDI (PNG + SVG) — inline, hakuna crate za nje.
//!
//! - `funi_logo_png()`  — PNG 192×192 (mduara wa cog + herufi F, gradient cyan/teal)
//! - `LOGO_SVG`         — SVG ndogo kwa hati za HTML (quote/invoice/ripoti)
//!
//! PNG encoder ni ndogo kabisa: zlib "store blocks" (hakuna compression) +
//! CRC32 + Adler32 — inatosha kwa logo ndogo na haina dependencies.

const W: usize = 192;
const H: usize = 192;

/// RGBA pixel za logo (zinatumika na PNG encoder)
fn draw_logo_rgba() -> Vec<u8> {
    let mut px = vec![0u8; W * H * 4];
    let (cx, cy) = (W as f64 / 2.0, H as f64 / 2.0);
    for y in 0..H {
        for x in 0..W {
            let dx = x as f64 - cx + 0.5;
            let dy = y as f64 - cy + 0.5;
            let r = (dx * dx + dy * dy).sqrt();
            let ang = dy.atan2(dx);
            // mena 8 za cog: kila sekta ya 45°, mena kati yake (25%–75%)
            let seg_w = std::f64::consts::PI / 4.0;
            let t = (ang + std::f64::consts::PI) / seg_w;
            let frac = t - t.floor();
            let tooth = r > 88.0 && r <= 99.0 && (0.25..0.75).contains(&frac);
            let inside = r <= 88.0;
            let cyan = dx < 0.0; // nusu ya kushoto cyan, ya kulia teal
            let (cr, cg, cb) = if cyan { (0u8, 229, 255) } else { (0, 131, 143) };
            let mut a: u8 = 0;
            if inside || tooth {
                a = 255;
            } else if r <= 91.0 {
                a = 140; // edge antialias
            }
            // herufi F nyeupe (viwango vya 192-grid)
            let xf = x as f64;
            let yf = y as f64;
            let fbar = (76.0..=90.0).contains(&xf) && (60.0..=134.0).contains(&yf)
                || (76.0..=120.0).contains(&xf) && (60.0..=74.0).contains(&yf)
                || (76.0..=108.0).contains(&xf) && (92.0..=106.0).contains(&yf);
            let (mut rr, mut gg, mut bb) = (cr, cg, cb);
            if fbar && (inside || tooth) {
                rr = 255;
                gg = 255;
                bb = 255;
            }
            let i = (y * W + x) * 4;
            px[i] = rr;
            px[i + 1] = gg;
            px[i + 2] = bb;
            px[i + 3] = a;
        }
    }
    px
}

fn crc32(data: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for n in 0..256u32 {
        let mut c = n;
        for _ in 0..8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
        table[n as usize] = c;
    }
    let mut c = 0xFFFF_FFFFu32;
    for &b in data {
        c = table[((c ^ b as u32) & 0xFF) as usize] ^ (c >> 8);
    }
    c ^ 0xFFFF_FFFF
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &x in data {
        a = (a + x as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn push_chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let mut crc_in = Vec::with_capacity(4 + data.len());
    crc_in.extend_from_slice(kind);
    crc_in.extend_from_slice(data);
    out.extend_from_slice(&crc32(&crc_in).to_be_bytes());
}

/// Logo ya FUNDI kama PNG (RGBA 192×192) — tumia kwenye UI/print kama picha.
pub fn funi_logo_png() -> Vec<u8> {
    let px = draw_logo_rgba();
    // scanlines + filter byte 0
    let mut raw = Vec::with_capacity(H * (1 + W * 4));
    for y in 0..H {
        raw.push(0u8);
        raw.extend_from_slice(&px[y * W * 4..(y + 1) * W * 4]);
    }
    // zlib wrapper + store blocks (BTYPE=00)
    let mut z = vec![0x78u8, 0x01];
    let mut off = 0usize;
    while off < raw.len() {
        let n = std::cmp::min(65535, raw.len() - off);
        let last = off + n >= raw.len();
        z.push(if last { 1 } else { 0 });
        z.extend_from_slice(&(n as u16).to_le_bytes());
        z.extend_from_slice(&(!(n as u16)).to_le_bytes());
        z.extend_from_slice(&raw[off..off + n]);
        off += n;
    }
    z.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut out = vec![0x89u8, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&(W as u32).to_be_bytes());
    ihdr.extend_from_slice(&(H as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]); // 8-bit RGBA
    push_chunk(&mut out, b"IHDR", &ihdr);
    push_chunk(&mut out, b"IDAT", &z);
    push_chunk(&mut out, b"IEND", &[]);
    out
}

/// SVG ndogo ya logo — inabwatawa moja kwa moja kwenye hati za HTML.
pub const LOGO_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="44" height="44" viewBox="0 0 100 100" style="vertical-align:middle"><defs><linearGradient id="fundig" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#00e5ff"/><stop offset="1" stop-color="#00838f"/></linearGradient></defs><circle cx="50" cy="50" r="46" fill="url(#fundig)"/><g fill="#fff"><rect x="26" y="18" width="12" height="52" rx="2"/><rect x="26" y="18" width="32" height="11" rx="2"/><rect x="26" y="42" width="25" height="11" rx="2"/></g><text x="50" y="86" font-family="Arial, Helvetica, sans-serif" font-size="14" font-weight="bold" fill="#fff" text-anchor="middle">FUNDI</text></svg>"##;

/// Ops za PDF Form XObject (vector logo ndogo — inatumika na pdf.rs).
/// Coordinate space: 0..100 (BBox [0 0 100 100]); PDF y ni juu-juu.
/// Rejea pdf.rs: `q 44 0 0 44 40 762 cm /LOGO Do Q`
pub fn vector_logo_ops() -> String {
    let mut s = String::new();
    // mduara wa gradient (cyan → teal): nusu mbili
    s.push_str("0 0.898 1 rg 50 4 m 50 96 l 4 50 l h f\n");   // nusu ya kushoto cyan
    s.push_str("0 0.51 0.56 rg 50 96 m 50 4 l 96 50 l h f\n"); // nusu ya kulia teal
    // herufi F nyeupe (rects: x y w h — chini juu kwa PDF y)
    s.push_str("1 1 1 rg\n");
    s.push_str("28 30 11 52 re f\n");   // mabawa
    s.push_str("28 71 32 11 re f\n");   // juu
    s.push_str("28 48 25 11 re f\n");   // kati
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_ina_signature_na_size_sahihi() {
        let png = funi_logo_png();
        assert_eq!(&png[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
        // angalau ~1500 bytes (192*192*4 raw + zlib) — si tupu wala kubwa mno
        assert!(png.len() > 1000 && png.len() < 400_000);
    }

    #[test]
    fn svg_ina_fundi() {
        assert!(LOGO_SVG.contains("FUNDI"));
        assert!(LOGO_SVG.starts_with("<svg"));
    }

    #[test]
    fn vector_ops_za_pdf_zipo() {
        let ops = vector_logo_ops();
        assert!(ops.contains(" rg"));
        assert!(ops.contains("re f"));
    }
}
