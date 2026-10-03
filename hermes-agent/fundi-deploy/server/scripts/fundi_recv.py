#!/usr/bin/env python3
"""Fundi multicast receiver — inaendeshwa kwenye PC ya lengo (initramfs/live).

Mtiririko:
  1. Soma manifest (JSON line) kutoka group 239.255.0.1:5007
  2. Poka chunks (session|seq|payload) hadi END marker
  3. Thibitisha sha256
  4. Andika kwenye disk (OF, default /dev/sda) au file (TEST_OF kwa majaribio)
  5. Tuma ACK kwa server API (FUNDI_API)

Mazingira:
  FUNDI_API     http://server:8080  (kwa ACK)
  OF            /dev/sda (default) au file kwa test
  MAC           mac ya client (kwa ACK)
"""
import hashlib
import json
import os
import socket
import sys
import urllib.request

GROUP = os.environ.get("FUNDI_MC_GROUP", "239.255.0.1")
PORT = int(os.environ.get("FUNDI_MC_PORT", "5007"))
OF = os.environ.get("OF", "/dev/sda")
API = os.environ.get("FUNDI_API", "http://127.0.0.1:8080")
MAC = os.environ.get("MAC", "00:00:00:00:00:00")
BUFSZ = 65536


def send_ack(sha: str, received: int, ok: bool):
    try:
        req = urllib.request.Request(
            f"{API}/api/mc/ack",
            data=json.dumps({
                "mac": MAC, "sha256": sha,
                "received_bytes": received, "ok": ok,
            }).encode(),
            headers={"Content-Type": "application/json"},
        )
        urllib.request.urlopen(req, timeout=5)
    except Exception as e:  # ACK ni best-effort
        print(f"ACK fail: {e}", file=sys.stderr)


def main():
    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    try:
        sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEPORT, 1)
    except (AttributeError, OSError):
        pass
    sock.bind(("", PORT))
    mreq = socket.inet_aton(GROUP) + socket.inet_aton("0.0.0.0")
    sock.setsockopt(socket.IPPROTO_IP, socket.IP_ADD_MEMBERSHIP, mreq)
    print(f"[fundi_recv] listeners on {GROUP}:{PORT} → {OF}")

    # 1. Manifest
    manifest = None
    while manifest is None:
        data, _ = sock.recvfrom(BUFSZ)
        try:
            manifest = json.loads(data.decode().strip())
        except Exception:
            continue
    name = manifest["name"]
    expect_sha = manifest["sha256"]
    expect_bytes = manifest["bytes"]
    print(f"[fundi_recv] image={name} bytes={expect_bytes}")

    # 2. Chunks
    chunks = {}
    session = None
    while True:
        data, _ = sock.recvfrom(BUFSZ)
        if data.startswith(b"END|"):
            parts = data.decode().strip().split("|")
            session = parts[1] if len(parts) > 1 else session
            break
        try:
            head, seq_bytes = data.split(b"|", 2)[0], data.split(b"|", 2)[1]
        except ValueError:
            continue
        if session is None:
            session = head.decode(errors="replace")
        if head.decode(errors="replace") != session:
            continue
        seq = int.from_bytes(seq_bytes[:4], "big")
        payload = data[len(head) + 1 + 4:]
        chunks[seq] = payload

    blob = b"".join(chunks[i] for i in sorted(chunks))
    sha = hashlib.sha256(blob).hexdigest()
    ok = sha == expect_sha and len(blob) == expect_bytes
    print(f"[fundi_recv] received={len(blob)} sha_ok={ok}")

    if not ok:
        send_ack(expect_sha, len(blob), False)
        sys.exit(1)

    # 3. Andika (disk au file)
    if OF.startswith("/dev/") and not os.environ.get("FUNDI_MC_TEST"):
        with open(OF, "wb") as f:
            f.write(blob)
            f.flush()
            os.fsync(f.fileno())
        print(f"[fundi_recv] disk {OF} imeandikwa")
    else:
        out = os.environ.get("FUNDI_MC_TEST_OF", "/tmp/fundi_mc_out.bin")
        with open(out, "wb") as f:
            f.write(blob)
        print(f"[fundi_recv] test output: {out}")

    send_ack(expect_sha, len(blob), True)


if __name__ == "__main__":
    main()
