---
name: mtaalamu-triage
description: "Diagnose and fix a customer's computer or software problem end-to-end."
version: 1.0.0
author: Zawadi Stephano (onearmy756-cmd), Hermes Agent
license: MIT
platforms: [linux, macos, windows]
metadata:
  hermes:
    tags: [MTAALAMU, Troubleshooting, Diagnostics, Fundi, Kiswahili]
    category: mtaalamu
    related_skills: [mtaalamu-maintenance, mtaalamu-deploy]
---

# MTAALAMU Triage Skill

Take a customer's problem statement and drive it to a fix using the MTAALAMU
engine (Rust) for all computation. You (the LLM) never calculate — you collect
symptoms in the customer's language (Kiswahili or English), call the engine
tools, and narrate every step until the problem is solved.

## When to Use

- "Kompyuta yangu inasumbua…" / "My computer is slow / crashing / won't boot."
- A customer describes any hardware or software fault.
- Periodic health check on a customer machine.

## Prerequisites

- MTAALAMU plugin enabled (tools `mtaalamu_solve`, `mtaalamu_agentic`,
  `mtaalamu_scan`, `mtaalamu_remediate`).
- Engine built: `cd engine-rust && cargo build --release` (the plugin locates
  `engine-rust/target/release/mtaalamu[.exe]` automatically).

## How to Run

1. Restate the customer's problem in one sentence and confirm it with them.
2. Call `mtaalamu_scan` (deep=true for slow/crash cases) — read the real probe.
3. Call `mtaalamu_solve` with the customer's words (`msg`) — the engine returns
   grounded diagnosis + plan from rules and its knowledge base.
4. Explain the plan to the customer **in their language**, step by step.
5. Any action that changes the system (remediate, deploy, config edits) is
   HITL-gated: ask the customer for consent first, then re-call with
   `approved=true` and `role` in {admin, specialist}.
6. After each fix step, verify with `mtaalamu_scan` and tell the customer what
   changed. Loop until symptoms are gone.
7. Close with a Kiswahili summary of what was wrong, what was done, and how to
   prevent recurrence (offer `mtaalamu_maintenance`).

## Quick Reference

| Tool | Use for | HITL |
|---|---|---|
| `mtaalamu_scan` | see real system state | no |
| `mtaalamu_solve` | diagnose + plan | no (read-only) |
| `mtaalamu_agentic` | full autonomous run | yes (`approve=true`) |
| `mtaalamu_remediate` | apply a fix action | yes (`approved=true`) |

## Pitfalls

- Never invent readings — if the engine returns an error, tell the customer
  the truth and fix the setup (e.g. engine not built).
- Never run `approved=true` because the customer "sounds" agreeable — restate
  the exact action and get an explicit yes.
- User-role callers cannot self-approve HITL actions.

## Verification

- Re-run `mtaalamu_scan` after remediation and compare against the first probe.
- The customer confirms the original symptom is gone in their own words.
