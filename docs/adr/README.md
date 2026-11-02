# ADR Index

New durable architecture decisions should use [../templates/adr.md](../templates/adr.md) and live in this directory.

## Existing Decision Records

- [棚卸しの数と売上の前後を精算の並びで決め、在庫数は記録から求め直す（accepted）](2026-10-16-stocktake-count-placement.md): 計数の置き場（日付と精算の並び）、在庫数の求め直し、決まらない記録だけの要再確認、Z004 の精算回数による識別と欠けの知らせ、完了と評価額。D-118 の design 出力（実装の状態は ADR の Status の節）。
- [棚卸しと後着売上の時点証拠（superseded）](2026-09-18-stocktake-time-evidence.md): 計数窓、資料受領、判定不能からの再実測、取消と移行。㉗のdesign出力。2026-10-16 に上の ADR（D-118）へ置き換えた。
- [旧棚卸しとZ004業務commit・取消の一時停止（superseded）](2026-09-23-legacy-stocktake-z004-write-stop.md): 5 つの BIZ 入口の停止、既存 kind の停止 error、旧本体の保持、画面の案内と無効化、再現 fixture。㉘ runtime ①。2026-10-23 に D-118 の runtime の b1 で解除し、上の ADR へ置き換えた。

The project already has ADR-like records under `docs/research/`. They remain valid and are linked here instead of moved in this workflow retrofit.

| Decision | Existing record |
|---|---|
| Router selection | [../research/2026-04-20-router-adr.md](../research/2026-04-20-router-adr.md) |
| Invoke type generation | [../research/2026-04-20-invoke-type-adr.md](../research/2026-04-20-invoke-type-adr.md) |
| Invoke wrapper | [../research/2026-04-20-invoke-wrapper-adr.md](../research/2026-04-20-invoke-wrapper-adr.md) |
| Query cache strategy | [../research/2026-04-20-query-cache-adr.md](../research/2026-04-20-query-cache-adr.md) |

## Rules

- Create a new ADR when a decision changes architecture, workflow gates, data safety, UI framework direction, command wire shape, persistence strategy, or external integration policy.
- Do not move existing research ADRs as part of unrelated implementation work.
- If a research ADR is promoted later, keep redirects or links so old plan evidence remains readable.
- Reference ADRs from Plan Packets and source docs when a change depends on the decision.
