//! Wake-on-LAN — magic packet via wakeonlan binary au raw UDP

use anyhow::Context;
use std::net::UdpSocket;

pub async fn wake(mac: &str) -> anyhow::Result<()> {
    let out = tokio::process::Command::new("wakeonlan")
        .arg(mac)
        .output()
        .await;
    if let Ok(o) = out {
        if o.status.success() {
            return Ok(());
        }
    }
    send_magic_packet(mac)
}

fn send_magic_packet(mac: &str) -> anyhow::Result<()> {
    let bytes = parse_mac(mac)?;
    let mut packet = vec![0xFFu8; 6];
    for _ in 0..16 {
        packet.extend_from_slice(&bytes);
    }
    let sock = UdpSocket::bind("0.0.0.0:0").context("bind")?;
    sock.set_broadcast(true)?;
    sock.send_to(&packet, "255.255.255.255:9")?;
    Ok(())
}

fn parse_mac(mac: &str) -> anyhow::Result<[u8; 6]> {
    let parts: Vec<&str> = mac.split(|c| c == ':' || c == '-').collect();
    if parts.len() != 6 {
        anyhow::bail!("MAC batili: {mac}");
    }
    let mut out = [0u8; 6];
    for (i, p) in parts.iter().enumerate() {
        out[i] = u8::from_str_radix(p, 16)?;
    }
    Ok(out)
}
