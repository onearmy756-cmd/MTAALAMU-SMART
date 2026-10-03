---
name: mtaalamu-channels
description: "Reach customers on WhatsApp, SMS, or email with consented messages."
version: 1.0.0
author: Zawadi Stephano (onearmy756-cmd), Hermes Agent
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [MTAALAMU, WhatsApp, SMS, Email, Communication]
    category: mtaalamu
    related_skills: [mtaalamu-triage]
---

# MTAALAMU Channels Skill

Send repair updates, HITL consent requests, and final reports to customers on
WhatsApp, SMS, or email. Every send is a real dispatch — the tools never fake
success; failures come back with the exact setup fix.

## When to Use

- "Mpe mteja taarifa kwa WhatsApp." / "Text the customer the ETA."
- Deliver a triage report or deploy completion notice.
- Ask a remote customer to approve a HITL action.

## Prerequisites

- WhatsApp: Hermes gateway running with the WhatsApp channel configured
  (`cli-config.yaml` → `channels:`; gateway default `:8088`).
- SMS: `FUNDI_SMS_URL` (HTTP provider) — store it in Infisical, never in files.
- Email: `FUNDI_SMTP_HOST/USER/PASS/PORT` (TLS) — also from Infisical.

## How to Run

1. Confirm the customer's channel preference and the exact number/address.
2. Draft the message in the customer's language; keep HITL consent requests
   explicit: state the action, the risk, and that a reply "ndiyo" approves it.
3. Call the matching tool:
   - `mtaalamu_send_whatsapp to=<number> msg=<text>`
   - `mtaalamu_send_sms to=<number> msg=<text>`
   - `mtaalamu_send_email to=<addr> subject=<s> msg=<body>`
4. Report the real result; on error, read the `hint` and fix the setup or offer
   another channel.

## Quick Reference

| Tool | Transport | Needs |
|---|---|---|
| `mtaalamu_send_whatsapp` | Hermes gateway :8088 | WhatsApp channel configured |
| `mtaalamu_send_sms` | FUNDI_SMS_URL | provider env in Infisical |
| `mtaalamu_send_email` | SMTP TLS | FUNDI_SMTP_* env in Infisical |

## Pitfalls

- Never send marketing or unrequested content.
- Consent requests must not count as consent — approval happens when the
  customer replies and you record it in the workflow.
- If a send fails, do not retry silently more than once; tell the customer.

## Verification

- The tool returns `ok: true` with the real provider/gateway status.
- When possible, the customer confirms receipt.
