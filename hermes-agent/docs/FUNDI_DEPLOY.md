# Fundi Deploy (ndani ya MTAALAMU SMART)

Mass OS deployment: **PXE + WOL + job API**.

Code: [`fundi-deploy/`](../fundi-deploy/README.md)

## Phases

| Phase | Deliverable |
|-------|-------------|
| **P1** (sasa) | docker-compose, dnsmasq, TFTP menu, nginx, Samba, Rust API (deploy/jobs/WOL) |
| **P2** | AI OS select (Ollama), backup hooks, multicast, image manager |
| **P3** | Cloud multi-tenant dashboard, VPN, billing reports |

## Integration

- **Agentic Vision** → hugundua hardware mbovu → ticket (si auto-wipe).
- **Fundi Deploy** → OS install kwa MAC list baada ya **idhini ya binadamu**.
- **Map** → wataalamu / job sites (si PXE).

## Lab only first

Usitumie kwenye production LAN bila VLAN ya majaribio + backup policy.
