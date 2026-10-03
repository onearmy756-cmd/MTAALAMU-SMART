//! mlinzi.rs — MLINZI WA MTEJA: kinga dhidi ya utapeli na wizi wa mtandao.
//!
//! 1. FORWARD GUARD — kuweka/kuzuia call forwarding za siri (GSM codes rasmi za carrier);
//!    kwenye simu iliyounganishwa adb, agent anaweza KUPIGA USSD yenyewe (mteja anaona jibu screen).
//! 2. SMS SHIELD — uchambuzi wa ujumbe (heuristics wazi, si uongo): alama za utapeli + hatua.
//! 3. LINK CHECK — ukaguzi wa URL (https, punycode, shorteners, brand-in-domain, TLD hatari)
//!    + connectivity halisi (TCP 443). HATUFUNGUI link — tunachambua nje tu.
//! 4. EMAIL PHISH — alama za barua za udanganyifu (na `email check` ya kweli kwa ports).
//! 5. ACADEMY — masomo 20 ya usalama (watu wengi hawajui) + TTS.
//!
//! Hakuna uongo: kila uchunguzi ni halisi au unaitwa wazi kuwa ni checklist/mwongozo.

use anyhow::{bail, Result};
use serde_json::json;
use std::io::Write;

// ---------- helpers ----------

fn confirm(q: &str) -> bool {
    print!("{q} (y/ndiyo = ndiyo): ");
    let _ = std::io::stdout().flush();
    let mut l = String::new();
    let _ = std::io::stdin().read_line(&mut l);
    matches!(l.trim().to_lowercase().as_str(), "y" | "yes" | "ndiyo")
}

fn adb_ussd(code: &str) -> Result<String> {
    let enc = code.replace('#', "%23").replace('*', "%2A").replace(',', "%2C");
    let out = std::process::Command::new("adb")
        .args(["shell", "am", "start", "-a", "android.intent.action.CALL", "-d", &format!("tel:{enc}")])
        .output()
        .map_err(|e| anyhow::anyhow!("adb haipatikani ({e}) — weka platform-tools kwenye PATH, simu: USB debugging ON"))?;
    let txt = format!("{}{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    if txt.to_lowercase().contains("error") {
        bail!("adb: {}", txt.trim());
    }
    Ok(format!("USSD {code} imepigwa simu — JIBU litatokea kwenye SKRINI ya simu (dialer ya carrier)."))
}

// ---------- 1. CALL FORWARD GUARD ----------

/// Orodha rasmi ya GSM call forwarding codes (hii ni standard ya carrier, si uchawi):
const FORWARD_CODES: &[(&str, &str)] = &[
    ("*#61#", "Angalia: forward isiyojibiwa (no-reply)"),
    ("*#62#", "Angalia: forward simu ikizimwa (unreachable) — WAIZI hii sana!"),
    ("*#21#", "Angalia: forward ya kudumu (all calls)"),
    ("*#004#", "Angalia: forward zote za conditioning"),
    ("##002#", "ZIMA forwarding ZOTE (kinga ya haraka)"),
    ("##61#", "Zima forward isiyojibiwa"),
    ("##62#", "Zima forward unreachable"),
    ("##21#", "Zima forward ya kudumu"),
];

pub fn forward_check() -> String {
    let mut out = String::from("\n🕵️ CALL FORWARD GUARD — angalia kama simu IMEBAHATISHIWA:\n\n");
    for (c, d) in FORWARD_CODES {
        out.push_str(&format!("  {c:<8} → {d}\n"));
    }
    out.push_str("\nJinsi ya kuzipiga: kifungue Dialer kisha bonyeza code + kitufe cha kuita (jibu kitatokea screen).\n");
    out.push_str("Kwa simu iliyounganishwa adb: fundi-mobile mlinzi forward-check --adb (agent anaipiga yenyewe).\n");
    out.push_str("\n⚠️ Alama za kuibwa: forward unreachable (##62#) ikiwa ON na wewe hujaiweka — wizi unawenda!\n");
    out
}

pub fn forward_check_adb() -> Result<String> {
    let mut out = String::from("\n🕵️ FORWARDED CHECK HALISI (agent anaipiga USSD kwa adb):\n");
    for (c, d) in [("*#62#", "unreachable (wizi)"), ("*#61#", "no-reply"), ("*#21#", "kudumu")] {
        out.push_str(&format!("\n→ {d} ({c})...\n  {}\n", adb_ussd(c)?));
    }
    Ok(out)
}

pub fn forward_stop(yes: bool) -> Result<String> {
    if !yes && !confirm("ZIMA forwarding ZOTE kwa simu iliyounganishwa (##002#)? Hii inabadilisha settings za carrier") {
        return Ok("Imesitishwa — hakuna kilichobadilishwa.".into());
    }
    let r = adb_ussd("##002#")?;
    Ok(format!("\n🛡️ KINGA: {r}\n→ Thibitisha kwenye simu: piga *#21# na *#62# — zote zionekane 'Disabled'.\n→ Kama inarudisha 'Connection problem': piga simu carrier wako (100) wawazuie upande wao."))
}

// ---------- 2. SMS SHIELD ----------

fn sms_score(msg: &str) -> (u32, Vec<String>) {
    let m = msg.to_lowercase();
    let mut score = 0u32;
    let mut why = Vec::new();
    let mut hit = |cond: bool, s: u32, r: &str, why: &mut Vec<String>| {
        if cond {
            score += s;
            why.push(r.to_string());
        }
    };
    hit(m.contains("ushindi") || m.contains("winner") || m.contains("promo") || m.contains("bonus"), 2, "Ahadi ya ushindi/bonus (klasiki ya utapeli)", &mut why);
    hit((m.contains("bet") || m.contains("betting")) && (m.contains("malipo") || m.contains("paybill") || m.contains("tuma")), 3, "Betting inaomba malipo — hii si rasmi", &mut why);
    hit(m.contains("otp") || m.contains("pin yako") || m.contains("msimbo") && m.contains("tuma"), 4, "Inaomba OTP/PIN — HAKUNA mtoa huduma rasmi anayeomba OTP", &mut why);
    hit(m.contains("ajira") || m.contains("kazi") || m.contains("job") && (m.contains("fee") || m.contains("fomu") || m.contains("malipo")), 3, "Ajira inayotaka malipo ya awali — udanganyifu", &mut why);
    hit(m.contains("account") && (m.contains("blocked") || m.contains("imesimama") || m.contains("lock")) && m.contains("http"), 3, "Tishio la kufunga account + link — phishing", &mut why);
    hit(["bit.ly", "tinyurl", "t.co", "cutt.ly", "is.gd", "rb.gy", "shorturl"].iter().any(|s| m.contains(s)), 2, "Link ya kufupisha (shortener) — lengo lake limefichwa", &mut why);
    hit(m.contains("http") || m.contains("www.") || m.contains(".com") || m.contains(".xyz"), 1, "Ina link (thibitisha kwa `mlinzi link`)", &mut why);
    hit(m.contains("haraka") || m.contains("within") || m.contains("leo") || m.contains("24 hours"), 1, "Msukosuko wa muda (urgency) — mbinu ya kushangaza", &mut why);
    hit(m.contains("paybill") && !m.contains("vodacom") && !m.contains("airtel") && !m.contains("yas") && !m.contains("ttcl"), 1, "Paybill isiyojulikana — thibitisha na mtoa huduma", &mut why);
    hit(m.contains("zawadi") || m.contains("gift") || m.contains("simcard imepokea"), 1, "Zawadi isiyojulikana", &mut why);
    (score, why)
}

pub fn sms(msg: &str) -> String {
    let (score, why) = sms_score(msg);
    let verdict = if score >= 4 {
        "🚨 HATARI SANA — karibu uhakika ni UTAPELI. USIJIBU, USITOE kitu, USIBOFYE."
    } else if score >= 2 {
        "⚠️ SHUKA TAHADHARI — alama za utapeli zipo. Thibitisha kwa njia rasmi kabla ya kitu chochote."
    } else {
        "✅ Hakuna alama kali za utapeli kwenye uchambuzi huu — lakini ushauri wa kudumu unabaki."
    };
    let mut out = format!("\n📱 SMS SHIELD — uchambuzi (alama {score}):\n  {verdict}\n");
    if why.is_empty() {
        out.push_str("  Hakuna alama zilizogunduliwa.\n");
    }
    for w in &why {
        out.push_str(&format!("  ❗ {w}\n"));
    }
    out.push_str("\nSHERIA ZA DHAHABU:\n  1. OTP/PIN NI SIRI — hata 'mfanyakazi wa Vodacom' HATAKI kuijua.\n");
    out.push_str("  2. Thibitisha kwa nambari RASMI: piga 100 ya mtandao wako (Vodacom/Airtel/Yas) wewe mwenyewe.\n");
    out.push_str("  3. Usibofye link — fungua app RASMI badala yake.\n");
    out.push_str("  4. Ripoti: polisi 112 + mtandao wako (wanazuia namba za wapenzi wa mtandao).\n");
    let _ = crate::govagent::speak_sw(&if score >= 2 {
        "Tahadhari. Ujumbe huu una alama za utapeli. Usitoe OTP, usibofye link."
    } else {
        "Ujumbe haujaonyesha alama kali za utapeli. Lakini kuwa makini daima."
    });
    out
}

// ---------- 3. LINK CHECK (phishing) ----------

const SHORTENERS: &[&str] = &["bit.ly", "tinyurl.com", "t.co", "cutt.ly", "is.gd", "rb.gy", "shorturl.at", "ow.ly"];
const RISKY_TLD: &[&str] = &[".zip", ".xyz", ".top", ".tk", ".gq", ".cf", ".ml", ".click", ".loan"];
const BRANDS: &[&str] = &["mpesa", "vodacom", "airtel", "yas", "ttcl", "nmb", "crdb", "nbc", "nida", "tra", "heslb", "brela"];

pub fn link(url: &str) -> String {
    let u = url.trim().to_lowercase();
    let (scheme, rest) = match u.split_once("://") {
        Some((s, r)) => (s.to_string(), r.to_string()),
        None => ("(hakuna)".into(), u.clone()),
    };
    let host = rest.split('/').next().unwrap_or("").to_string();
    let host_no_www = host.trim_start_matches("www.").to_string();
    let mut flags: Vec<String> = Vec::new();
    if scheme != "https" {
        flags.push("Si HTTPS — data inasafiri wazi (HTTP)".into());
    }
    if !host.is_empty() && host.split('.').all(|p| p.chars().all(|c| c.is_ascii_digit())) && host.contains('.') {
        flags.push("Host ni IP (si jina la domain) — kawaida ya phishing".into());
    }
    if host.contains("xn--") {
        flags.push("Punycode (xn--) — inaweza kuiga herufi za domain rasmi".into());
    }
    if SHORTENERS.iter().any(|s| host_no_www == *s) {
        flags.push("Shortener — lengo halisi limefichwa".into());
    }
    if RISKY_TLD.iter().any(|t| host_no_www.ends_with(t)) {
        flags.push("TLD ya hatari (inatumika sana na wadanganyifu)".into());
    }
    if host_no_www.matches('-').count() >= 3 {
        flags.push("Mistari min3 (-) kwenye domain — mchanganyiko wa kuiga".into());
    }
    for b in BRANDS {
        if host_no_www.contains(b) && !host_no_www.ends_with(".co.tz") && !host_no_www.ends_with(".go.tz") && !host_no_www.ends_with(".tz") {
            flags.push(format!("Jina la '{b}' liko kwenye domain lakini SI domain rasmi ya .tz — uigaji"));
            break;
        }
    }
    if u.len() > 80 {
        flags.push("URL ndefu isiyo ya kawaida".into());
    }
    // Connectivity halisi (si "scan ya usalama" — ni kuwa ipo tu)
    let alive = std::net::TcpStream::connect_timeout(
        &format!("{host_no_www}:443").parse().unwrap_or_else(|_| std::net::SocketAddr::from(([0, 0, 0, 0], 0))),
        std::time::Duration::from_secs(3),
    )
    .is_ok();

    let verdict = if flags.len() >= 3 {
        "🚨 HATARI — USIFUNGUE. Alama nyingi za phishing."
    } else if flags.len() >= 1 {
        "⚠️ TAHADHARI — thibitisha kabla ya kufungua (fungua app rasmi badala yake)."
    } else {
        "✅ Hakuna alama za kiufundi za phishing kwenye URL hii — bado tumia akili: ikopezwa wapi? unaitazamaje?"
    };
    let mut out = format!(
        "\n🔗 LINK CHECK — {url}\n  Host: {host}\n  Inafikika (TCP 443): {alive}\n  {verdict}\n"
    );
    for f in &flags {
        out.push_str(&format!("  ❗ {f}\n"));
    }
    out.push_str("\nKANUNI: Benki/mgovu/hospitali HAWATUMI link ya kuomba password. Fungua APP rasmi au chapisha domain mwenyewe.\n");
    out
}

// ---------- 4. EMAIL PHISH ----------

pub fn email() -> String {
    String::from(
        "\n✉️ EMAIL PHISH — ALAMA ZA KUTAMBUA:\n  1. Anayetuma: jina linafanana rasmi LAKINI domain ni ya kigeni (mf: support@vodacom-secure.xyz)\n  2. Haraka ya lugha: 'account itafungwa ndani ya saa 24' — tishio la dharura\n  3. Link isiyo rasmi: elea juu ya link (bila kubofya) — angalia domain HALISI\n  4. Kiambatisho: .exe/.zip/.js AU 'invoice' usiyoiomba\n  5. Anaomba: password, OTP, card PIN, malipo ya awali — RASMI HAWAOMBI\n  6. Salamu jumla: 'Dear customer' badala ya jina lako\n  7. Spelling ya kigeni + muundo wa kampuni usio sahihi\n\nFANYA: fungua account kwa URL yako mwenyewe → angalia notifications rasmi.\nVIFAA: fundi-mobile email check <provider> (ports halisi) · shield scan (PC imeingizwa?)\nKama ulibofye na ukaweka password: BADILISHA Mara Moja + washa 2FA + `email password <provider>`.\n",
    )
}

// ---------- 5. ACADEMY (masomo ya usalama) ----------

const LESSONS: &[(&str, &str)] = &[
    ("Call forwarding za wizi", "Wizi wa kontena/MTANDAO: wajalifu wanawasha forwarding simu yako ikiwa imezimwa (##62#) — simu zako zinaenda kwao bila kujua. Angalia kila wiki: *#62# na *#21#. Zima zote: ##002#. Simu ikiwa 'inapokea Michango ya ajabu' — angalia mara moja."),
    ("OTP ni siri", "OTP (msimbo wa mara moja) ni FUNGUA la akaunti yako. Hakuna Vodacom/Airtel/benk/HESLB anayeomba OTP. Ukitoa — pesa inaondoka dakika hiyo. Hata 'mfanyakazi' akitaka: kata simu, piga 100 wewe mwenyewe."),
    ("SIM swap", "Kama simu inaonyesha 'No service' ghafla na mtu mwingine anapata SMS zako — SIM yako imenaswa (SIM swap). Piga simu 100 MARA MOJA, zuia line, badilisha SIM. Weka PIN ya SIM (Settings→Security)."),
    ("M-Pesa false deposit", "SMS ya 'umepokea TSH X' inaweza kuwa ya ULAGHAI (screenshot au SMS iliyotengenezwa). THIBITISHA kwenye app rasmi au *150*00# — SI kwa SMS. Usitoe bidhaa/deni kabla."),
    ("Utapeli wa ajira", "Kazi halisi HAILIPIwi na mwombaji. 'Tuma fee ya fomu/kaunti/making' = udanganyifu. Thibitisha kampuni kwenye tovuti rasmi + BARUA rasmi zenye domain ya kampuni."),
    ("Password na password manager", "Password tofauti kwa kila akaunti; ndefu (maneno 4+). Tumia password manager (Bitwarden bure). Password moja kwa kila kitu = mlango mmoja unaofungua nyumba zote."),
    ("2FA (uthibitisho wa hatua mbili)", "Washa 2FA kwenye Gmail, WhatsApp, Facebook, benki. Pendelea authenticator app (Google Authenticator) badala ya SMS. Hifadhi backup codes KARIAKI."),
    ("WiFi za umma", "WiFi ya hotelu/cafeteria isiyo na password: usitumie banking, usiingie password. Kama ni lazima: tumia data ya simu au VPN. Wizi wa sessions hutokea buliani."),
    ("Phishing emails", "Barua inayotaka 'uthibitishe account' kwa link = phishing. Elea juu ya link kuona domain halisi. Fungua app rasmi badala ya link. Tazama `mlinzi email`."),
    ("SMS utapeli", "Ushindi ulioshindikana, betting inayotaka malipo, 'anka yako imekufa' — zote ni mbinu. Tazama SMS SHIELD: fundi-mobile mlinzi sms '<ujumbe>'."),
    ("Facebook/WhatsApp account", "Hacker anaituma code ya WhatsApp au anadai 'nitumie code niliyokosea'. USITUMIE code. Washa 2FA ya WhatsApp (Settings→Account). Piga simu kuthibitisha mswada wa haraka."),
    ("PC virusi na crack", "Programu za crack/games bure kutoka tovuti zisizo rasmi = malware + ransomware. Nunua leseni (tazama `license genuine`) au tumia mbadala huria (LibreOffice, GIMP). Antivirus: Defender inatosha — iwe ON."),
    ("Ransomware", "Faili zote zimefungwa na inataka fidia? USILIPE. Zima network, taja fundi, angalia backup safi. Kinga: backup 3-2-1 (nakala 3, media 2, moja nje ya ofisi) + usifungue attachments."),
    ("Backup ya kudumu", "Watu wanaibiwa data MARA moja tu. Google Drive/OneDrive bure; external disk kwa PC. Jaribu kurejesha backup MARA MOJA — backup isiyotestwa si backup."),
    ("Utapeli wa kifaa kilichopotea", "Simu imepotea: zuia line 100, kisha Find My Device (android.com/find) au iCloud — lock + remote wipe. PC: BitLocker ON + password ya BIOS."),
    ("ATM na POS", "Angalia keypad ina kifaa cha ziada? Ficha PIN kwa mkono. Kama ATM inashika card — siondoke: simu 112 mara moja. Tumia ATM ndani ya benki."),
    ("Utapeli wa mapenzi/rafiki mtandaoni", "Rafiki mpya mtandaoni anayetaka pesa, cards, crypto = udanganyifu (hata kama 'anapenda'). HAKUNA upendo unaoomba iTunes cards."),
    ("Pyramid/Vima (ponzi)", "'Weka 10k, pokea 100k kwa wiki' = ponzi. Wanaolipa mwanzo wanalipwa na wanaoingia baadaye — mwisho: wote wanapotea. Kampuni halisi ina leseni ya BOT/CMS."),
    ("Kuingia salama (login)", "Kwenye NIDA/TRA/HESLB/benk: fungua tovuti RASMI (chapisha mwenyewe), angalia https + domain, usiingie kutoka link ya SMS/email. Kama una shaka: tafuta namba rasmi uwapigie. Agent ya FUNDI inakuongoza: `gov intake`."),
    ("Mtoto na mtandaoni", "Washa Family Link (Google) au Screen Time (Apple). Zipiga mijadala: usitoe jina/shule picha kwa wageni. Apps za kibepari: zikague Google Play tu."),
];

pub fn academy(n: Option<usize>) -> String {
    let Some(idx) = n else {
        let mut out = String::from("\n🎓 SHULE YA USALAMA — masomo 20 (watu wengi hawajui — hii ndiyo wanaibiwa navyo):\n\n");
        for (i, (t, _)) in LESSONS.iter().enumerate() {
            out.push_str(&format!("  {:>2}. {}\n", i + 1, t));
        }
        out.push_str("\nSoma moja: fundi-mobile mlinzi academy <namba> (inasoma kwa SAUTI pia)\n");
        return out;
    };
    let Some((t, body)) = LESSONS.get(idx.saturating_sub(1)) else {
        return format!("Masomo ni 1-{} (tazama: fundi-mobile mlinzi academy)", LESSONS.len());
    };
    let _ = crate::govagent::speak_sw(&format!("Somo la usalama namba {idx}. {t}. {body}"));
    format!("\n🎓 SOMO {idx}: {t}\n\n{body}\n")
}

// ---------- 6. STATUS (kinga halisi ya PC) ----------

pub fn status() -> Result<String> {
    let mut out = String::from("\n🛡️ MLINZI — HALI YA USALAMA:\n");
    if cfg!(target_os = "windows") {
        let ps = "Get-MpComputerStatus | Select-Object AMRunning,AntivirusEnabled,RealTimeProtectionEnabled,AntispywareEnabled | Format-List | Out-String";
        match std::process::Command::new("powershell").args(["-NoProfile", "-Command", ps]).output() {
            Ok(o) => {
                let t = String::from_utf8_lossy(&o.stdout);
                for line in t.lines() {
                    let line = line.trim();
                    if line.contains(':') {
                        out.push_str(&format!("  Defender · {line}\n"));
                    }
                }
            }
            Err(_) => out.push_str("  Defender: haipatikani (si Windows 10/11?) — sakinisha antivirus rasmi\n"),
        }
    } else {
        out.push_str("  PC: Linux — hakuna Defender; hakikisha firewall (ufw/gufw) + updates za kila siku\n");
    }
    out.push_str("\n✅ CHECKLIST YA MTEJA (uliza mwenyewe):\n  [ ] Simu: ##62# imepinga (forward hakuna)?\n  [ ] SIM PIN + 2FA zipo?\n  [ ] Password manager + password tofauti?\n  [ ] Backup inafanyika na IMETESTWA?\n  [ ] PC: Defender/updates ON, hakuna cracks?\n  [ ] Familia imeshawahi kusomeshwa academy?\n");
    Ok(out)
}

/// JSON kwa UI (masomo + codes)
pub fn summary_json() -> serde_json::Value {
    json!({
        "forward_codes": FORWARD_CODES.iter().map(|(c, d)| json!({"code": c, "desc": d})).collect::<Vec<_>>(),
        "lessons": LESSONS.iter().enumerate().map(|(i, (t, _))| json!({"n": i + 1, "title": t})).collect::<Vec<_>>(),
    })
}
