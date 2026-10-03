---
name: mtaalamu-deploy
description: "Deploy operating systems to lab machines via Fundi Deploy, HITL-gated."
version: 1.0.0
author: Zawadi Stephano (onearmy756-cmd), Hermes Agent
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [MTAALAMU, Fundi Deploy, OS, Provisioning]
    category: mtaalamu
    related_skills: [mtaalamu-triage, mtaalamu-maintenance]
---

# Fundi Deploy Skill

Provision computers end-to-end with Fundi Deploy through the MTAALAMU engine:
discover hosts on the network, pick the OS, get the customer's consent, start
the deployment, and follow the job to completion.

## When to Use

- "Weka Windows/Ubuntu kwenye kompyuta za lab."
- "Deploy the new image to the training room machines."
- Customer asks for a fresh OS setup on one or many machines.

## Prerequisites

- MTAALAMU plugin enabled (`mtaalamu_deploy`).
- Fundi Deploy server reachable (`FUNDI_DEPLOY_URL`, default 127.0.0.1:8080) —
  otherwise the engine returns the real connection error; relay it honestly.

## How to Run

1. `mtaalamu_deploy sub=discover` — list real hosts (MAC, name) found on the LAN.
2. Confirm with the customer which machines and which OS/need profile
   (`os=auto|windows|ubuntu`, `need=office|design|dev`).
3. Show the plan, then ask for explicit consent (HITL). Users cannot approve;
   the customer (admin/specialist) must say yes.
4. `mtaalamu_deploy sub=start macs=<aa:bb:...,...> approved=true role=<role>` —
   the engine creates the job with HITL approval recorded.
5. Poll `mtaalamu_deploy sub=jobs` and narrate progress per machine.
6. `sub=approve`/`sub=cancel` with the job `id` if the customer changes their mind.

## Quick Reference

| sub | Effect | HITL |
|---|---|---|
| `discover`/`hosts` | LAN scan (read-only) | no |
| `summary`/`jobs`/`images`/`cloud` | status (read-only) | no |
| `start` | begin deployment | **yes** (`approved=true`) |
| `approve` / `cancel` | job control | yes |

## Pitfalls

- `start` without `approved=true` is refused by design — do not try to bypass.
- MACs must come from `discover` output, never from memory.
- If the deploy server is down, say so; do not guess job states.

## Verification

- `sub=jobs` shows the job `completed` for every requested MAC.
- Customer boots a machine and confirms the expected OS.
