//! Multicast deploy (P2) — picha moja kwa PCs nyingi kwa mtando mmoja.
//!
//! Server (agent) : UDP sender — inatuma image kwa group 239.255.0.1:5007
//! Client (PC)    : `fundi_recv.py` (PXE initramfs/live) — inapokea na kuandika disk
//! Ukadiriaji    : ACK counts kutoka kwa clients (`/api/mc/ack` kutoka recv script)
//!
//! Njia salama:
//!   - Manifest (jina, bytes, sha256) inatumwa kwanza (JSON line)
//!   - Baada ya kutuma: clients ripoti sha256 yao → server inalinganisha
//!   - HITL kazi kama kawaida — multicast ni stage moja ya pipeline

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::net::UdpSocket;
use std::path::Path;
use std::sync::Arc;

pub const MC_GROUP: &str = "239.255.0.1";
pub const MC_PORT: u16 = 5007;
const CHUNK: usize = 1400; // chini ya MTU kwa usalama
const PACKETS_PER_SEC: usize = 400;

#[derive(Serialize, Deserialize, Clone)]
pub struct McManifest {
    pub name: String,
    pub bytes: u64,
    pub sha256: String,
    pub total_chunks: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct McAck {
    pub mac: String,
    pub sha256: String,
    pub received_bytes: u64,
    pub ok: bool,
}

/// Tuma file kwa multicast group (blocking call ndani ya spawn)
pub fn send_file(path: &Path, session: &str) -> anyhow::Result<McManifest> {
    let data = std::fs::read(path).with_context(|| format!("soma {}", path.display()))?;
    let sha = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(&data);
        format!("{:x}", h.finalize())
    };
    let total_chunks = ((data.len() + CHUNK - 1) / CHUNK) as u64;

    let manifest = McManifest {
        name: path.file_name().map(|s| s.to_string_lossy().into()).unwrap_or_default(),
        bytes: data.len() as u64,
        sha256: sha.clone(),
        total_chunks,
    };

    let sock = UdpSocket::bind("0.0.0.0:0")?;
    sock.set_broadcast(true)?;
    let addr = format!("{MC_GROUP}:{MC_PORT}");

    // TTL ya multicast (scope ya LAN pekee)
    let _ = sock.set_multicast_loop_v4(true);

    // 1. Manifest (JSON + newline)
    let mline = format!("{}\n", serde_json::to_string(&manifest)?);
    sock.send_to(mline.as_bytes(), &addr)?;

    // 2. Data chunks
    let mut sent = 0u64;
    for (i, chunk) in data.chunks(CHUNK).enumerate() {
        // header: session|seq|payload
        let mut pkt = Vec::with_capacity(CHUNK + 32);
        pkt.extend_from_slice(session.as_bytes());
        pkt.push(b'|');
        pkt.extend_from_slice(&i.to_be_bytes());
        pkt.extend_from_slice(chunk);
        sock.send_to(&pkt, &addr)?;
        sent += 1;
        if sent % PACKETS_PER_SEC as u64 == 0 {
            std::thread::sleep(std::time::Duration::from_millis(1)); // pacing
        }
    }

    // 3. End marker
    let end = format!("END|{session}|{sha}\n");
    sock.send_to(end.as_bytes(), &addr)?;

    Ok(manifest)
}

/// Hifadhi ACK kutoka client (inaitwa na route /api/mc/ack)
#[derive(Default)]
pub struct AckBoard {
    pub acks: std::sync::Mutex<Vec<McAck>>,
}

impl AckBoard {
    pub fn record(&self, ack: McAck) {
        self.acks.lock().unwrap().push(ack);
    }
    pub fn count(&self) -> usize {
        self.acks.lock().unwrap().len()
    }
    pub fn verified(&self, sha: &str) -> usize {
        self.acks
            .lock()
            .unwrap()
            .iter()
            .filter(|a| a.ok && a.sha256 == sha)
            .count()
    }
    /// Jumla ya ACKs zilizothibitishwa (sha yoyote)
    pub fn ok_count(&self) -> usize {
        self.acks.lock().unwrap().iter().filter(|a| a.ok).count()
    }
}

pub type SharedAcks = Arc<AckBoard>;

/// Chunk pacing info kwa UI (packets sent)
pub fn stats_line(sent: usize, total: usize) -> String {
    format!("multicast: {sent}/{total} packets (group {MC_GROUP}:{MC_PORT})")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_roundtrip() {
        let m = McManifest {
            name: "os.iso".into(),
            bytes: 123,
            sha256: "abc".into(),
            total_chunks: 1,
        };
        let line = serde_json::to_string(&m).unwrap();
        let back: McManifest = serde_json::from_str(&line).unwrap();
        assert_eq!(back.name, "os.iso");
        assert_eq!(back.bytes, 123);
    }

    #[test]
    fn ack_board_inakusanya() {
        let b = AckBoard::default();
        b.record(McAck { mac: "aa".into(), sha256: "x".into(), received_bytes: 1, ok: true });
        b.record(McAck { mac: "bb".into(), sha256: "x".into(), received_bytes: 1, ok: false });
        assert_eq!(b.count(), 2);
        assert_eq!(b.verified("x"), 1);
    }
}
