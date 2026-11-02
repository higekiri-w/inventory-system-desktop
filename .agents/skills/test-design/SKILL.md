---
name: test-design
description: "Design effective tests before implementation. Use for behavior changes, and especially R3/R4 changes that affect schema, config, runtime behavior, output, policy, metrics, or data safety."
---

# Test Design Skill

## Purpose

Create the packet `## Test Plan` table (old packets: a Test Design Matrix) before implementation so tests are tied to contracts and failure modes.

## Required Reading

- `docs/project-profile.md`
- Plan Packet の `Design Readiness` と `Contract Ledger`（旧 packet は `Design Sources`・`Design Intent Trace`）と、そこが引く source design docs
- Plan Packet
- relevant specs / contracts
- relevant existing tests
- data safety rules

## Steps

1. Extract Risk Level.
2. Extract contracts under test from source design docs first, then the Plan Packet.
3. Extract design decision IDs and source doc sections that the tests should protect.
4. List failure modes.
5. Classify needed tests:
   - contract
   - negative/fail-fast
   - boundary
   - state/policy
   - compatibility/schema
   - main wiring/integration
   - data safety
   - regression
6. Fill the packet `## Test Plan` table (Contract / Failure Mode / Test Type / Test Name / Would fail if...).
7. Propose Red tests.
8. Add mutation-style adequacy questions.
9. List residual test gaps.

## Output

Use `docs/templates/test-design-matrix.md`.

## Rules

- Do not write implementation code unless explicitly asked.
- Do not invent behavior from the Plan Packet when source design docs are missing or ambiguous; return the work to Design Phase or record the design gap.
- Do not optimize for test count; optimize for test effectiveness.
- Prefer tests that cite spec IDs, design decision IDs, or source doc sections when the touched area has traceability.
- Every important test should state what broken implementation it catches.
- Accepted P1/P2 fixes are pinned per `docs/code_review.md` `## Same PR vs Follow-up` (a test that fails on the pre-fix code).
- Test Design Matrix の数値クレーム（duration / count / percentage / threshold）は測定コマンドと出力を併記するか、明示的に `未実測` とタグ付けする（D-062、AGENTS.md の数値主張規則と同一趣旨）。
