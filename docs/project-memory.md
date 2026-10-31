# Project Memory

## Purpose

Durable memory for the inventory management project.
Use this to survive context loss, handoffs, and long-running work.
Keep it factual and stable.

## Stable Facts

- Product: inventory management system for a single physical handicraft store
- Users:
  - developer: family member with software background
  - operator: non-IT store owner using Excel and CASIO ECR+ daily
- Method: USDM-driven requirements and design process
- Stack: `Tauri 2.x + React + TypeScript + SQLite`
- Architecture layers: `UI / CMD / BIZ / IO / MNT`
- Public requirements source: `docs/spec/requirements.md` + `docs/spec/requirements-coverage.md` + linked design docs. Owner-retained source material stays outside the repository.

## Business Boundaries

- Single-store local desktop application
- Low concurrency, mostly one operator
- File-based POS integration, not API-based
- CSV workflows are first-class product behavior
- POS integration should preserve a replaceable adapter boundary: register-specific formats/procedures stay in the adapter, while app-internal sales, report, inventory, import lifecycle, and operator concepts stay in the core model.

## POS Facts

- Register family: `CASIO SR-S4000`
- Known CSV families: `Z001`, `Z002`, `Z004`, `Z005`
- Field-check 2026-06-30: current daily report operation uses CASIO PC tool / SD-card `Z001`, `Z002`, and `Z005` as the main report inputs. `Z004` is a PLU/product track and must not be treated as the only current sales import source without a follow-up SALES redesign.
- Field-check 2026-07-06: `Z004` is a confirmed CV17 PLU-by-product sales report, not a future file candidate or PLU master. It has `memory No. / code / name / quantity / amount`, dumps all 5000 slots, and the controlled one-product sale appeared as the only non-zero quantity row in that sample. CV17 presents Z001/Z002/Z004/Z005 in the same report screen family and can write them together; app-core semantics remain split between official aggregates (`Z001/Z002/Z005`) and product-level sales/inventory (`Z004`).
- Confirmed `Z004` characteristics for the existing parser / field-layout enablement:
  - encoding: `CP932`
  - line separators: the parser normalizes `NEL` / CRLF / LF / CR defensively, but the carried-back Z files and SD files are CRLF only and no `NEL` line terminator has been observed (2026-10 redesign fact list; the earlier "may include `NEL`" was not backed by a sample)
  - returns can appear as negative values
- Installed field PC tool observed in 2026-06 field-check is `カシオレジスターツール for SR-S500/SR-C550/SR-S4000/SR-S200`, ProductName `CV-17`, FileVersion/ProductVersion `1.1.1.0`.
- Local field-check reference materials are kept outside the repo at `/home/kosei/downloads/inventory-field-check` (`\\wsl.localhost\Ubuntu-22.04\home\kosei\downloads\inventory-field-check` from Windows). Use `summaries/` and `approved-readable/ECRCV17.pdf` for CV17/register-tool facts when needed, but do not commit real CSV/XLSX/PDF outputs, screenshots, register backups, JANs, product names, prices, or store-specific sales/cost data.
- CV17 1.1.1 daily report profiles observed in 2026-07 L3: files that persist in the PC-side `EcrDatas` directory after SD import are layout A, while register-tool explicit export output is layout B. On 2026-10-06 the owner replaced the 2026-08-01 standard procedure (`SD -> CV17 import -> select files from EcrDatas`): daily sales data (Z and EJ) is read by the app directly from the register SD without opening CV17, and CV17 is used only to write PLU data to the register (D-111, the decided design). Selecting files by hand (including the PC-side `EcrDatas` copies) is a fallback, and explicit CV17 export is not a normal path. These layout A/B observations are for the daily reports (Z001/Z002/Z005), and the REQ-401 daily-report parser compatibility should accept both layouts without exposing a layout/source choice in the normal UI. For Z004 (PLU item sales) the owner policy is also to accept both, but the current Z004 parser accepts the legacy shape and layout A only; layout B is not yet supported and stops safely (`docs/function-design/23-io-z004-parser.md`, backlog). The remembered-folder behavior applies only to the hand-selection fallback. Layout A is CP932/CRLF with 7-row preamble, 1 header row, then 4-column data rows (`record code`, `label`, `quantity/count`, `amount`). Layout B is a concatenated export shape with leading meta fields, a header, then 4-column repeated rows. `Z001` / `Z002` dates may be `YYYY/M/D`; `Z005` and some export outputs may use `YYYY-MM-DD`.
- The current Excel report files are overwrite targets, not source data or a daily archive. The store owner answered that the printed pages serve as both the submission to the tax accountant and the store's copy (2026-09-29; this corrects the earlier "not external/tax submissions"). Each day the Z001/Z002/Z005 content is pasted almost as-is into the same Excel file group, which is overwritten before printing; prior daily states do not remain in Excel. The printed pages filed in a binder are therefore the store's only current per-day history. Whether the app's daily/monthly history fully replaces that operator function remains a go-live acceptance check.
- Initial deployment baseline: 初導入の事実・owner確認日は下記「Store Premises Facts（現場の前提）」を正とする。本番開始時は同節へ実際の開始日と変わった前提を追記する。この初導入の前提は、開発/試験データを削除してよいという許可ではない。
- Owner rollout intent confirmed 2026-08-01 (gradual move to PLU) was replaced on 2026-10-16 (D-117, owner decision TD-274): the decided design has no phased-onboarding tools (no bulk PLU-target toggle, no migration-status column or filter). The current build still has the bulk PLU-target button on the product list; it is removed by the PLU export redesign runtime (backlog). Which products are sold by scanning is set per product (`plu_target`, including the product CSV `PLU対象` column), and all target products are written from the first export. Department-key sales for products without a JAN continue alongside PLU sales. Z004 layout A/B enablement and end-to-end validation of the existing inventory pipeline still need go-live planning.
- CV17 1.1.1 scanning PLU import/apply profile observed in 2026-07 field gate: use tab-delimited CP932/CRLF `.txt`; 11 columns `メモリNo.` / `ｽｷｬﾆﾝｸﾞｺｰﾄﾞ` / `名称` / `単価` / `課税方式` / `単品売り` / `負単価` / `品番PLU` / `ゼロ単価` / `入力桁制限` / `部門リンク`; PLU total memory is 5000 shared by normal PLU and scanning PLU; scanning PLU starts at 217, the next number after the factory allocation of 216 normal PLU slots (the store has registered only 2 test entries in normal PLU, so the start is not the count of registrations); the start number can be changed in CV17 settings but the store keeps 217 (owner answer; D-117 (7)); UI-08 does not add an operator setting and keeps the code-side profile `217..=5000`; `入力桁制限` is `無し`; `単品売り` must be `いいえ` (2026-08-19 Windows native L3: `はい` makes SR-S4000 auto-settle as exact cash and print a receipt on scan-call alone, confirmed by switching T1 to `いいえ` on CV17); scanning code must be a valid 13-digit JAN/EAN-13 code and must not fall back to `product_code`; the practical external gate is `CV17 TXT import -> PC tool SD settings write -> SR-S4000 設定読み -> barcode/register behavior confirmation`.
- PLU export is app-to-register only; the app cannot confirm reflection at export time. After the next settlement, the Z004 read from the SD shows the codes and names of all 5,000 slots, so code/name reflection appears in the daily occupancy check (decided design, D-117; until the PLU export redesign runtime the current build checks occupancy from a Z004 file the operator selects), but price, tax and department link are not in Z004 (D-117 (7), correcting the D-011 premise). UI-08 design splits PLU file generation from app-side exported confirmation so `plu_dirty` remains set until the operator explicitly marks the saved PLU file as exported after CV17 import, SD-card write, SR-S4000 read, and representative register call succeed.
- 2026-07-03 field gate: the store-laptop/register-side flow succeeded for the confirmed CV17 `.txt` shape. Treat CV17 import success alone as insufficient. For PR #122, the owner accepted structural equivalence to that confirmed shape as the external gate because the approved-readable file has the same CV17 1.1.1 11-column profile as the app formatter; the latest app-generated `.txt` CV17 / SD-card / SR-S4000 / representative barcode recheck was a follow-up after UI-08 (not a PR #122 blocker) and was done on 2026-07-06 (`docs/plu-export-and-real-csv-verification.md`).

## Settled Design Facts

- DB design is materially complete for the implemented backend. The daily report tables (`daily_report_imports`, `daily_report_*_lines`) for the REQ-401 redesign were implemented in migration v4 (`src-tauri/src/db/schema_v4.rs`). The current table list is in `docs/DB_DESIGN.md` (counts are not copied here).
- Import rollback uses logical invalidation, not physical delete
- CSV import pipeline is `Parse -> Validate -> Preview -> Commit`
- Parse failures are logged to `operation_logs`, not persisted in `csv_imports`
- Multiple `jan_code` matches resolve by `ORDER BY product_code ASC`
- `pos_stock_sync` is an explicit business flag
- `plu_dirty` and `plu_exported_at` track app-side PLU sync state; `plu_exported_at` is not proof of PC-tool acceptance or register reflection
- D-072 / D-117 / REQ-907: free-slot authority is the Z004 full-slot snapshot, read automatically when the daily closing reads the register SD; allocation is sticky by JAN with a persistent memory No.; the app never clears a slot or reuses one for another JAN (a discontinued product's slot stays); both Diff and Full are importable (decided design, D-117. The current build still writes Clear rows (`PLU_CLEAR_ROW_ENABLED = true`) and checks occupancy from a Z004 file the operator selects; the PLU export redesign runtime switches it). For PLU, CV17 is opened only when PLU data changes (new products, price changes), writing the accumulated unreflected products at once; until the D-111 (1)(b) lane (moving imported files to `XZ_BKUP`), the daily CV17 import also continues under the D-111 operating constraint.
- PLU export real-device confirmation and Z004 field-layout enablement belong to one go-live verification flow, while `Z001`/`Z002`/`Z005` remain the official daily-report aggregate track; checklist source: `docs/plu-export-and-real-csv-verification.md`
- POS adapter boundary is a settled architecture constraint (`D-023`): CASIO `Z001`/`Z002`/`Z004`/`Z005`, CV17, SD-card, and PC tool details are adapter facts unless a source design doc explicitly promotes a concept to the app core.
- REQ-401 daily report redesign is a settled design direction (`D-025`): `Z001`/`Z002`/`Z005` create app-internal daily report aggregates, while `Z004` remains item-level product-sales/inventory track after PLU verification. Daily reports must not be expanded into fake `sale_records` or `inventory_movements`.
- `stocktake_items.valuation_cost_price_centi` freezes stocktake valuation
- Stocktake may proceed while CSV import remains allowed (design fact, kept by D-118. The stop ADR `docs/adr/2026-09-23-legacy-stocktake-z004-write-stop.md` was lifted and superseded by the D-118 runtime b1 on 2026-10-23: counts are saved to `stock_counts`, the stock is recomputed from records, and completion does not move stock)

## Key Domain Rules

- Products can use JAN-based codes or department-prefix custom codes
- Length products (fabric etc.) are entered and shown in `m` (D-113); `roll` and `cm` were removed from the unit list at the rebuild starting point of the development and test DB (vB, [22 §16](function-design/22-mnt-migration.md), D-119/D-128). Cost is held in 1/100 yen (`*_centi`, tax-exclusive, D-128)
- Some products must not auto-decrement stock from POS sales import
- Register-processed returns are reflected through CSV import
- Manual return/exchange exists for non-register cases
- Price-revision notices for handicraft goods arrive via one main wholesaler, one notice per maker, as printed/PDF lists carrying barcode / product name / maker code / old & new price
- Non-handicraft categories (hair accessories, aprons, bags, etc.) learn price changes only from receiving slips at reorder, not from wholesaler notices
- Markup ratio (掛率) varies per product and is a derived value (current cost ÷ current selling price), not persisted in a dedicated column
- Cost is set by the wholesaler/maker, not chosen by the store, except occasional seller-negotiated discounts (約定) on bulk orders
- Selling price is the tax-inclusive reference list price (`selling_price` is tax-inclusive); some categories have no reference list price and the store sets price from cost instead
- `suppliers` means maker/brand (the price-revision notice originator), not the wholesaler order channel
- Stock-take valuation population = products registered in the product master only
- A small number of very old items per year (about 1-5) are kept outside the product master (no product code), sold via department key with no item-level sales tracking, and re-registered with a new code/price if reordered
- The operator PC screen is not customer-visible during normal operation
- Price revisions are applied promptly without waiting for any notice-listed effective date; large lists (up to ~400 lines, including non-stocked items) can take several days to work through

## Current Work and History

完了は git log、lane の現在地は helper の status、導入の前の関門・ブロッカー・製品の未決判断は [Plans.md](Plans.md)、未了候補は [Backlog](backlog.md)（D-145）。この文書には進行状態を複製しない。

旧Progress SnapshotとOpen Itemsは [移送前のmemory](archive/harness-context/2026-09-14-project-memory.md) に保存した。未決記録の本文は [過去memoryの未決記録](backlog.md#過去memoryの未決記録)（閉じた記録は [archive の同じ節](archive/2026-10-19-backlog-closed.md#過去memoryの未決記録)）に残し、関係する作業で成立・解消状況を確認する。

重要な判断と理由は [decision-log](decision-log.md) と該当design docを参照する。

## Store Premises Facts（現場の前提、owner確認 2026-09-19）

店の実態を前提にする設計・レビュー・移行の判断をする前に、まずここを引く。載っていない事実が判断に要るときは owner に 1 問だけ尋ね、答えを確認日つきで同じ commit へ足す（backlog の該当 entry を参照）。本番開始後に変わる前提（レジで売る商品の範囲、印刷要否など）は、変わった時点で開始日とともに更新する。

起票の背景: 2026-09-19、Plan Review round 3 で「実店舗に本番データが在る」という誤った前提のまま移行設計を議論し、owner へ escalation する事態が発生した。本節はその再発防止として、散らばった記録から拾った事実と owner の 2026-09-19 回答を 1 枚に統合したもの。種類 1・3〜7（下記）は owner の直接確認欄が未記入のため、出典を現物確認できたものだけを「記録済み（出典つき）」として収載している。種類 2「無いもの」と 15 問の owner 回答は 2026-09-19 に直接確認済み。

### 在るもの

- Register is `CASIO SR-S4000` — 2026-06-30 field-check — `docs/project-memory.md`（本文書 POS Facts）
- Store has `カシオレジスターツール for SR-S500/SR-C550/SR-S4000/SR-S200`（CV-17 1.1.1.0）installed on the PC — 2026-06 field-check — `docs/project-memory.md`
- SD card → CV17 import is the existing data-recovery route from register to PC — 2026-08-01 owner判断 — `docs/plu-export-and-real-csv-verification.md`。2026-10-06 から、アプリ導入後の毎日の売上データ（Z・EJ）はアプリが SD を直接読み、CV17 は PLU の書込みだけに使う — owner判断2026-10-06 — `docs/decision-log.md`（D-111）。ただし取込み済みを `XZ_BKUP` へ移すアプリの操作ができるまでは CV17 の取込みも今の運用のまま続ける（`### 決めた運用` の D-111 の運用の制約の行）
- Store also uses CASIO ECR+（スマホアプリ）daily, but it will not be the long-term primary POS integration because it has a planned service end — 2026-06-30 field-check — `docs/plu-export-and-real-csv-verification.md`（decision-log D-022 でも同じ懸念を記録）
  - 店主は 2026-03 のヒアリングで、ECR+（スマホ）が無いと精算できないと答えている — 店主回答（2026-03 ヒアリング）。その後、スマホを切ってレジ本体だけで精算した日が 2026-10 に 5 回あり、どれも精算は済んだ（スマホへの送信は失敗し、データを SD へ控えて次の精算で送る旨が印字された。取扱説明書も SD へ控えられれば精算できるとする）— owner 経由の確認 2026-10-07〜14
- Daily report files (Z001/Z002/Z005) are pasted into an Excel file group almost as-is and overwritten/printed each day; printed pages are filed in a binder — 2026-06-30 field-check — `docs/project-memory.md`
- Developer is a family member of the store owner with a software background — （記録済み） — `docs/project-memory.md`
- Price-revision notices for handicraft goods all pass through one main wholesaler（メーカー兼業）; non-handicraft categories get no such notice and learn price changes only from reorder slips — 2026-08-22 owner原文 — `docs/evidence/issue-90/hearing-2026-08-21-22.sanitized.md`
- About 80 supplier relationships total: ~5 direct wholesalers/makers, ~43 handicraft makers via wholesaler, ~31 others; the set shrinks over time as suppliers close or merge — 2026-08-22 owner原文 — `docs/evidence/issue-90/hearing-2026-08-21-22.sanitized.md`
- Slip retention duty was 5 years, is now 7 years; about 4 years ago a tax accountant's cost inquiry required checking every retained slip individually — 2026-08-22 owner原文 — `docs/evidence/issue-90/hearing-2026-08-21-22.sanitized.md`
- Register numeric keypad has a decimal point, so quantity × unit price with one decimal digit (e.g. 1.3 × 70円/m) can be entered — 2026-09-15 owner伝聞 — `docs/evidence/hearing-2026-09-14-stock-units.sanitized.md`（追記）
  - PLU に登録した商品を小数の数量で売ると、Z004 の個数は小数 1 桁で出て、EJ と精算レシートにも小数で残る — 店主回答（フォーム）2026-10-07
- Excluded long-dormant items（単品コードなし）are sold via department key + amount, tracked only in the owner's memory — 2026-08-22 owner原文 — `docs/evidence/issue-90/hearing-2026-08-21-22.sanitized.md`
- Of 216 normal PLU slots, only 2 have real store-entered data; the remaining 214 are factory-default `PLU####` placeholders — see also「レジ・レジスターツール」節の訂正事実 — 2026-08-17 実機機械抽出 — `docs/plu-export-and-real-csv-verification.md`
- 10 scanning PLU slots hold an 8-digit code + `EEEEEE` padding; the owner rewrote them for a test and left them because PLU is unused (correcting the earlier "non-handicraft goods" description; the app treats them as `external` either way) — 2026-08-17 実機機械抽出 / owner回答 2026-10-07 — `docs/plu-export-and-real-csv-verification.md`
- Fabric/ribbon/string receiving slips record quantity in `m`. Shown a list of 12 unit words prepared by the asking side (個・枚・本・袋・箱・巻・組・セット・玉・丁・m・cm), the store owner answered that none of them is unused and added no other word; this is not a record that the store counts items in each of the 12 (the words the store owner named unprompted on 2026-08-15 were 個・玉・枚・本・丁, and the 2026-09-12 version of the stock Excel had 0 rows in 巻 and cm; D-119) — 2026-09-14 店主回答（LINE の返信を owner が転記。旧記載の「owner原文」を訂正）— `docs/evidence/hearing-2026-09-14-stock-units.sanitized.md`
- Some items are received boxed and sold individually after unboxing (e.g. one sewing-notion item) — 2026-09-14 owner原文 — `docs/evidence/hearing-2026-09-14-stock-units.sanitized.md`
  - 箱・袋で仕入れてばらす商品は、伝票が 1 袋いくらで、在庫はばらした 1 個あたりで数える（粒で数えるものも、詰め直した小袋もある）。1 個の原価は 1 袋の値段 ÷ 入り数を、小数第 3 位で四捨五入して小数第 2 位まで持つ。切り売りのひも類も同様の商品 — 店主回答（owner 経由の質問票）2026-09-29
- Purchase orders are written on one sheet per wholesaler, grouped by wholesaler — 2026-09-05 owner原文 — `docs/evidence/hearing-2026-09-05-stock-inquiry.sanitized.md`
- Owner already knows almost all discontinued items and defunct suppliers from memory — 2026-09-05 owner原文 — `docs/evidence/hearing-2026-09-05-stock-inquiry.sanitized.md`
- Counting words differ by category: 毛糸=玉, 布=枚, ファスナー=本, はさみ=丁, スナップ=枚, 刺繍糸=本, otherwise mostly 個 — 2026-08-15 owner原文 — `docs/evidence/issue-76/form-response-2026-08-15.sanitized.md`
  - 在庫計数 Excel では毛糸の単位が「個」になっているが、owner の入力の誤りで、数え方は「玉」。Excel は意味が通るのでそのままでよい — owner 2026-10-07。Excel から「個」で初期投入した毛糸は、登録の後に個数の単位どうしの変更で「玉」へ直せる（D-119）
- The store never sources the same product from two different suppliers — 2026-08-15 owner原文 — `docs/evidence/issue-76/form-response-2026-08-15.sanitized.md`
  - 訂正: 発注書を分ける「取引先」は注文先（問屋・仕入先）。同じ商品を 2 つ以上の注文先から仕入れるのは緊急時だけ（お客様に約束した日に、通常の仕入れ先の納品が間に合わないとき）。上の行の「never」は「通常は 1 つの注文先だけ」の意味 — 店主回答（owner 経由の質問票）2026-09-29
  - この「取引先」は問屋・仕入先を指し、D-075 (6) が `suppliers` に持たせたメーカー/ブランドとは別の軸。見直しは `docs/backlog.md` の取引先の項目で扱い、決定はまだ変えていない
- Maker package renewals can change the maker's product code — 2026-08-15 owner原文 — `docs/evidence/issue-76/form-response-2026-08-15.sanitized.md`
- Color numbers are commonly used to find/order color-variant products — 2026-08-15 owner原文 — `docs/evidence/issue-76/form-response-2026-08-15.sanitized.md`
- Price-revision notices always carry barcode / product name / maker code / old & new price; one maker's notices lack catalog page references and renewals can confuse identification — 2026-08-22 owner原文 — `docs/evidence/issue-90/hearing-2026-08-21-22.sanitized.md`
- 「約定」: the owner can negotiate a lower cost directly with a sales rep on bulk orders, below the standard reference cost — 2026-08-22 owner原文 — `docs/evidence/issue-90/hearing-2026-08-21-22.sanitized.md`
- Some hair-accessory items have no reference list price; the store sets cost itself for those — 2026-08-22 owner原文 — `docs/evidence/issue-90/hearing-2026-08-21-22.sanitized.md`
- 店にネットワーク（Wi-Fi・LAN）がある — owner回答2026-09-19
- 店での運用を想定した市販の USB HID バーコードリーダーを owner が用意し（owner 2026-08-09）、店の PC で Excel シートの JAN 入力に使っている — owner回答2026-09-23
- 店の PC は Windows 11 Pro 64-bit、画面は 1366×768・表示倍率 100%、BIZ UDPゴシックが入っている — owner確認 2026-09-29
- プリンタは Canon TR8630（既定のプリンタ、標準 TCP/IP ポート）。Wi-Fi か有線 LAN かの区別と、Edge からの A4 1 枚の印刷は次の訪店で確かめる — owner確認 2026-09-29

### 無いもの（owner回答 2026-09-19、種類2 全13件）

- No production DB / production import history exists yet in the real store; only dev/demo/test DBs exist — owner回答2026-09-19
- Everything outside the register is currently paper operation; this app is close to a from-scratch (greenfield) introduction — owner回答2026-09-19。owner が用意した Excel シートが店舗に渡っており、データの初期投入にも使える想定だが、Excel シートの運用定着は未確認
  - Excel シートの定着について、owner は運用できる見込みは薄く、店主をどう導けばよいかも分からないと述べた — owner回答2026-09-19
- No prior electronic stock record exists; the Excel sheet the owner just handed the store is still being filled in — owner回答2026-09-19
  - 日々の在庫の増減の電子記録は無いが、前年の棚卸しリスト（Word 等）は店の PC に残っている。今の棚卸しは、owner が新しく作って渡した Excel シートで行っている — owner回答2026-09-23
- No electronic purchasing/ordering record; paper slips/lists are the base, though some vendor documents arrive as PDF/email — owner回答2026-09-19
- No sales-record system other than the register; daily reports are register → Excel transcription → print only — owner回答2026-09-19
- Single store, not multiple locations — owner回答2026-09-19
- No multi-user/employee-account concept; effectively single-operator, low concurrency — owner回答2026-09-19
- No off-PC or network backup destination is in use now — owner回答2026-09-19。owner は、PC の外の保存先はあったほうがよいとは思うとしたうえで、個人店の規模で Google Drive を使うのはセキュリティ面に不安が残り、AWS 等で小さいものがあればという留保を付けた — owner回答2026-09-19。外部に保存するなら置き場所は店 — owner回答2026-09-23。暗号化したクラウド保存は今は見送り、v1.0 に含めない方向（費用の懸念）— owner判断2026-09-23。クラウド保存の扱いは backup の設計 lane で判断する — Coordinator の扱い 2026-09-23
- No accounting/bookkeeping software; the Excel sheet above is the closest thing, and its operational adoption is unconfirmed — owner回答2026-09-19
- No barcode label printer in the store; rejected earlier as impractical for a non-IT elderly-adjacent operator and because some items have no JAN — owner回答2026-09-19
- No wheeled/mobile work table in the store — owner回答2026-09-19
- `suppliers` table intentionally holds maker/brand only, not the wholesaler order channel — owner回答2026-09-19
- Only a single PC is in use; more than one PC would only happen if the PC itself is replaced — owner回答2026-09-19

### いまの手作業

- Daily: SD card recovery → CV17 import to PC → paste Z001/Z002/Z005 into Excel → overwrite → print → file in a binder — 2026-06-30 field-check — `docs/project-memory.md`
- 締めで印刷している Excel は 1 シート（売上日報）で、A4 縦・1 ページに縮小して収める。上に年月日、その下に 4 つの枠（売上明細 Z001・取引キー Z002・部門 Z005・PLU Z004）を横に並べ、各枠の列は CSV と同じ（レコード / キャラクター / 個数(件数) / 金額）。行数は Z001 が 28、Z002 が 38、Z005 が 21。PLU の枠は見出しだけで中身は空。1 日 1 枚 — owner確認 2026-09-29
- 締めで印刷する紙は A4 縦。見返すのは日付と印刷のずれの確認だけ。導入後も紙で要るもの = 日報・月の売上・棚卸しの一覧・入荷や廃棄の記録。申告の前に紙で作る集計の用紙の項目 = ページ番号、メーカー、JAN コード（メーカー型番と兼用）、売価、仕入れ価格、数量、金額 — 店主回答（owner 経由の質問票）2026-09-29
- 毎日の締めで EJ も取り込める（取込みの頻度は今は月 1 回ほど）— 店主回答（owner 経由の質問票）2026-09-29
- PLU をレジに書いた後の値下げ品の売り方: バーコードのある商品は読んでから値引きキー、バーコードの無い商品は部門キー（今は PLU を使っておらず、全部の商品を部門キーで売る。PLU-24、2026-07-23。2026-09-17 の確かめも同じ） — 店主回答（owner 経由の質問票）2026-09-29
- Daily register closing happens same-day, with the store kept open; during closing the SD card is imported through CV17, the reports are pasted into Excel and printed, and only then is the store closed — owner回答2026-09-22
- Sales made between the day's register closing and the store's actual close are therefore folded into the next day's closing — Coordinator の推論（上の行の owner回答2026-09-22 からの帰結で、owner の発言ではない）
- PC work such as product edits can also happen after the day's register closing — owner回答2026-09-22
- Product lookup: check name on the shelf first; if absent, search the name on PC to find the maker, check the catalog, or call the supplier; product code is the final tie-breaker — 2026-08-15 owner原文 — `docs/evidence/issue-76/form-response-2026-08-15.sanitized.md`
- Year-end stocktake: use a paper list in shelf order; blank entries mean not-yet-counted; newly added items are hand-written onto the paper list — 2026-08-15 owner原文 — `docs/evidence/issue-76/form-response-2026-08-15.sanitized.md`
  - この行は前回の年末棚卸し（Excel シートを渡す前）の運用。2026-09-23 時点は、owner が渡した Excel シートへ、机に置いたノート PC と棚を往復して入力している（棚の前で PC を持って入力するのではない）— owner回答2026-09-23。訪店の観察では PC を持って移動しながら入力する場面もあり、回答と食い違う（時期・場面の違いかは確かめていない）。D-118 の計数の設計は入力の仕方に依らない（数えたら保存、前の日の分は日付を選ぶ）。次の年末の棚卸しも Excel で行うかは未確認
  - 新規品の書き足しは、途中に手書きしたり最後にまとめたりで統一されていない（紙のリストに行を挿入できない前提での試行錯誤）。商品の場所はすべて把握している — 店主回答・owner 伝聞 2026-08-20
- During stocktake, items received but not yet billed are excluded from the count; discarded items are subtracted from the count — 2026-08-15 owner原文 — `docs/evidence/issue-76/form-response-2026-08-15.sanitized.md`
  - 未会計の取り置きは数に入れる（取り置きは会計の前が多い）。取り寄せは先払いで、先払いの取り寄せ品がお渡し待ちの間に棚卸しになっても数えない。先払いの取り寄せは、受注の時点で JAN が分からないことがあるため、レジで部門キーで金額を打つ — 店主回答（owner 経由の質問票）2026-09-29
  - 棚卸しの途中の入荷のうち、問屋の締め・請求・支払いの周期で請求と支払いが翌年になるものは、12 月末の在庫に数えない。見分けは問屋の締め日 — 店主回答（owner 経由の質問票）2026-09-29
- Price revision: hand-correct the paper prior-year list's cost/price, re-tag the shelf price, then correct the PC copy of the prior-year list afterward — 2026-08-15 owner原文 — `docs/evidence/issue-76/form-response-2026-08-15.sanitized.md`
- Price-revision notices: lists up to ~400 lines are worked through over several days, prioritizing high-volume items, without waiting for any listed effective date — 2026-08-21/22 owner原文 — `docs/evidence/issue-90/hearing-2026-08-21-22.sanitized.md`
- A heavy stocktake year took about 1 week to transcribe onto the paper list, then 2-3 more days to transcribe into the PC — 2026-08-22 owner原文 — `docs/evidence/issue-90/hearing-2026-08-21-22.sanitized.md`
  - この転記期間の記録は、アプリを使えていなかった頃の原始的な運用（Word や手書き）のもので、前提としては正確とは言いにくい — owner 2026-09-23（疑問形の留保つき）
- Old stock with no item code that sells is tracked from memory only and rung up on a department key — 2026-08-22 owner原文 — `docs/evidence/issue-90/hearing-2026-08-21-22.sanitized.md`
- Rounding: maker markup ratios are usually truncated; the store's own cut-sale fractions are rounded up — 2026-08-21 owner原文 — `docs/evidence/issue-90/hearing-2026-08-21-22.sanitized.md`
- 棚卸しの評価額の丸め（上の行の価格を決めるときの端数処理とは別）: 商品別の金額を小数第 3 位で四捨五入して小数第 2 位まで持ち、全商品の金額を合計した最終合計で四捨五入する — owner 伝聞 2026-09-24（店主本人の答えを owner が確認）
- 棚卸しは税理士の指示で年 1 回、大晦日時点で行い、報告するのは仕入原価の総額だけ。差異は棚卸しロスとして許容する — 店主回答（2026-03）。owner は年 1 回、多くても 2 回と述べた — owner 2026-08-27
- 年末の棚卸しの時期は毎年変わらない — owner回答2026-09-19
- 棚卸しで数える時間帯は決まっていない — owner 伝聞 2026-09-16
- 10 月からの棚卸しの準備の中身は、値上げ品の原価・売価の確認と訂正、新商品のリストへの追加、原価不明品の問い合わせ — owner 伝聞 2026-07-07
- 棚卸しの途中でノート PC の蓋を閉じ（家へ持ち帰る場合を含む）、続きを後で入力することはたまにやる。アプリの完成後も続く見込み（今の頻度は Excel の棚卸しシートを埋める作業も込み）— owner回答2026-09-29
- 値上げの PDF は画面では作業しにくいので、印刷して書き込み、後で PC の棚卸しリストの売価・原価をまとめて直す（紙と PC の二重管理）— owner 伝聞 2026-08-21
- 期限切れ・破損品は記録しておらず、棚卸しのときに在庫数を変えてロスとして計上してきた — 店主回答（2026-03）
- 発注時はメーカー品番で照会し、分からなければ JAN の一部、それも無ければメーカー名＋商品名で仕入先へ照会する。品番と JAN の対応が載る資料は商品によって異なる（カタログの一覧・一覧でない記載・品番だけ・カタログが無く現物確認だけ）— 店主回答（2026-03）
- 在庫が少ないと感じる数は商品で違う（毛糸・スナップ・編み針・刺繍糸・はさみ類で目安が別、ミシン糸は定番色とそれ以外で分ける）— 店主回答（2026-08-15 フォーム）— `docs/evidence/issue-76/form-response-2026-08-15.sanitized.md`

### 営業中の作業と中断（owner回答 2026-09-19、旧記録の訂正）

旧記録「営業中はレジで販売のみ、システム操作なし」（`docs/SCREEN_DESIGN.md` の1日の動線図にあった記述）は誤りだった。owner が同日の会話で明確に訂正し、該当記述は本 commit で是正済み。

- 営業中でも PC 作業がある
- 入荷時の商品登録は営業中に発生し得る
- 商品編集・新規登録・廃番のいずれも営業中に起こり得る、接客で中断されながら
- 上記はいずれも owner回答2026-09-19 が基準。設計・レビューで「営業中はシステム操作が発生しない」という前提を置かない。

### 決めた運用（このアプリの使い方について owner が決めたこと）

- （2026-10-16 に置き換え）2026-08-01 の「PLU へ段階的に移す」は D-117（owner 決定 TD-274）で置き換えた。段階導入の道具は持たず、レジで売る商品は商品ごとの欄で決める（採用済みの方針。今の build には商品一覧の一括の PLU 対象化の button が残り、撤去は PLU の書出しの見直しの runtime） — owner判断2026-10-16 — `docs/decision-log.md`（D-117）
- 毎日の売上データ（Z001/Z002/Z004/Z005 と EJ）の取込みは、CV17 を開かずにアプリがレジの SD から直接読む。CV17 は PLU をレジへ書き込むときだけ使う（日報〈Z001/Z002/Z005〉は売上データ取込み画面の「SD から読む」で読み、予備は利用者が選ぶ file。Z004・EJ を SD から取り込むのは後続の lane）。2026-08-01 の「SD → CV17 取込み → EcrDatas から選ぶ」の標準手順を置き換えた。SD 直読みの最初の段階（IO-09 と SD 直読みの runtime の lane）では SD の file は動かさず（`XZ_BKUP` へ移さない。アプリが移す操作は後続の lane、owner判断2026-10-07）、取り込んだ原本の写しをアプリが PC 側に残す。SD の場所はアプリが自動で探し、見つからなければ利用者が選ぶ。アプリが気を利かせて、CV17 での作業を利用者から隠す方向 — owner判断2026-10-06 — `docs/decision-log.md`（D-111）
- 取込み済みの Z・EJ を `XZ_BKUP` へ移すアプリの操作（後続の lane）ができるまで、店は CV17 の取込みを今の運用のまま続ける（日々の売上の Z は毎日、EJ は毎月末日）— owner判断2026-10-07 — `docs/decision-log.md`（D-111 の運用の制約）
- v1.0 gates on including the Z004 automatic-inventory-sync path; the app will not ship v1.0 unfinished in that respect — 2026-08-16 owner裁定 — `docs/decision-log.md`（D-070）
- Long-dormant no-code excluded items are not modeled as a system feature; they stay outside the product master and are re-registered at new price/cost if they come back — 2026-08-22 owner同意 — `docs/decision-log.md`（D-076）
- The ~80 suppliers are not bulk pre-loaded; they get linked incrementally when chosen in flows like bulk price revision — 2026-08-21/22 owner同意 — `docs/decision-log.md`（D-075）
- Weight-based units（g・kg）are not used in this app — 2026-09-13 owner原文 — [共通規則](function-design/10-common-rules.md) SPEC-UNIT-D1 の単位の一覧の段落、`docs/archive/2026-10-19-backlog-closed.md`（単位の拡張 項）
- For the question "from how many cm at the shortest do you sell", the store owner chose the option "10 cm 単位"; whether 10 cm is the minimum or the increment is not determined by the answer (in 2026-03 the store owner said length items are also sold in 10 cm units). Remaining stock, purchasing, and price tags are all expressed in `m`（値札は 1 m あたり）. The app accepts lengths to 1 cm and does not enforce a 10 cm increment (D-119) — 2026-09-14 店主回答（LINE の返信を owner が転記。旧記載の最小と刻みの混同と「owner回答」を訂正）— `docs/evidence/hearing-2026-09-14-stock-units.sanitized.md`
- Scanning-PLU の「単品売り」setting must always be `いいえ`; setting it to `はい` was confirmed on real hardware to make the register auto-settle and print a receipt on scan-call alone — 2026-08-19 Windows native L3実機確認 — `docs/project-memory.md`
- 人は間違える前提で、間違えたときに検知・停止・訂正できる仕組み（起きても補える・実行前に止める・そもそも操作できない）で守り、設計で防げるものを運用に任せない — owner 判断 2026-09-18〜2026-09-23
- 毎日の精算の作業に EJ（電子ジャーナル）の取込みを足すことを owner が了承した（SD からファイルを取るだけなので負担は小さい、という owner の判断で、店主本人には未確認）。アプリがどう支援するかは EJ の設計 lane で決める — owner判断2026-09-23

### 利用者

- Developer is a family member of the store owner, with a software background — （記録済み） — `docs/project-memory.md`
- Operator is the store owner (non-IT); they are the one who operates the PC, and design should not omit explanations for them. Owner also operates it on occasion; nobody else does — 2026-09-19 owner回答
- Design must assume a non-IT, possibly elderly, low-vision-adjacent user; readability and distinguishability are treated as functional requirements — （記録済み） — `docs/SCREEN_DESIGN.md`
- One real user has glaucoma; this shaped the accessibility baseline (forced-colors, target size, contrast) — PR #95 Windows native L3 owner所見 — `docs/archive/plans/2026-08-23-ui-list-backbone-d.md`
  - 実利用者は赤と黄色の区別がつかない。文字が小さすぎる可能性もあり、老眼もある — owner 伝聞 2026-06-06
- The PC screen is not customer-visible during normal operation (operator's back faces the screen); it is shown to customers only for maker-site catalogs — 2026-08-22 owner原文 — `docs/evidence/issue-90/hearing-2026-08-21-22.sanitized.md`
- 店主は、システムの在庫が実在庫と正確に合っていることを望み、ずれるならそのずれを棚卸しでロス（廃棄・万引き等）として出る範囲内にしてほしいと答えている — 店主回答（2026-03）
- CSV 取込み画面の「エラー N 件」が展開できる操作部だと owner 自身も最初は気づかなかった。owner は、IT に不慣れな店主はもっと気づきにくいと判断した — owner判断2026-08-03
- owner は店と家の 2 拠点を行き来し、家で作業するときはノート PC を家へ持って行く（PC は 1 台のまま）— owner回答2026-09-23

### レジ・レジスターツール

- Register model: `CASIO SR-S4000`; main daily inputs are Z001/Z002/Z005, with Z004 as the item-level PLU track — 2026-06-30/2026-07-06 field-check — `docs/project-memory.md`
- Normal PLU occupies slots 1-216 (barcode-less, dial-in); scanning PLU is 217+ — 2026-07 field gate — `docs/project-memory.md`
- Of the 216 normal PLU slots, only 2 hold real store data; the rest are unused factory defaults — 2026-08-17 実機機械抽出 — 2026-09-19 owner回答で、この2件はテスト登録だったと確認済み — `docs/plu-export-and-real-csv-verification.md`
- 既存のスキャニング PLU は死蔵で、運用上は存在しないものとして扱ってよい — owner 伝聞 2026-07-06。件数は「約 929」でなく、2026-08-20 の画面で 933、その後もレジ側で名前のある枠が増えている（2026-10-12〜10-14 の Z004）。アプリは（D-117 の runtime の後は毎日、今の build は利用者が選ぶ Z004 で）Z004 の照合で実数を読むので、件数を設計の前提にしない（D-117 (7)）— 件数の旧記載は `docs/archive/harness-context/2026-09-14-PROJECT_HANDOFF.md`
- 2026-08-20 の店 PC での確認で、スキャニング PLU 領域は既存登録 933 件・空き 3,851・アプリ管理 0 — owner（店 PC での L3 報告）2026-08-20。933 件は、死蔵の約 929 件に owner がテストで登録した検証用スキャニング PLU 4 件（owner 2026-08-15）を足した数と合う（推測）— `docs/archive/plans/2026-08-18-plu-slot-core-implementation.md`
- レジの売上/EJ 保存設定は有効で、SD の Z004 と同じフォルダに EJ（電子ジャーナル）のファイルがある。中身は締めレシートのような普通のジャーナルのテキスト — owner確認 2026-09-17。実物は採取済みで、repo 外の現場資料にある — owner回答2026-09-23
- EJ は日報とは別の CV17「電子ジャーナルを閲覧する」操作で取り込み、取り込むと SD 上では `XZ_BKUP` へ移り、PC 側 `EcrDatas` にも残る。SD 上に 2022-07 以降の約 4 年分がある — 2026-08-15 訪店の実機確認。店で EJ を PC へ取り込むのは月 1 回程度で、日次ではない（店主本人の話と EJ ファイルの日付で確認）— owner回答2026-09-23
- レジの PLU 名称欄は 16 バイトまでで、この制約は動かせない — owner確認 2026-09-17
- CV17 は日報をコピーして貼り付ける元としてだけ使われ、十分に活用されていない。店の CV17 は 1.1.1 で、公式の最新は 2.0.1。CV17 の更新は owner が行う — owner 2026-08-15
- SD カードは 16GB・FAT32 で使用率は約 4%、PC 側の CV17 履歴は約 416MB — 2026-08-15 訪店記録
- SD の配置: 機種の root は `CASIO\SR500_550_4000\`。精算前の Z は `XZ\yyyy\mm\Znnn_dda.CSV`、EJ は `XZ\EJyymmdd.TXT`（SD 上は大文字の `EJ`）。CV17 で取り込むと `XZ_BKUP` へ 4 桁の連番つきの名前で移る。同じ日・同じ系列の Z が 2 本以上の日があり、接尾字は空白・A・B。旧い `XZ\yyyy\mm\` に名前の規則に合わない file が残る。系列は Z001/Z002/Z004/Z005 だけで Z006/Z009/Z011 は無かった。`XZ_BKUP` の file は PC の `EcrDatas` の file と同じ bytes で、CP932・BOM 無し・CRLF。店の PC の reader で SD は取外し可能な drive として見え、読取り専用の採取の前後で SD の差は 0 — 2026-10-06 店 PC での SD の metadata 採取（名前・形・hash だけ）。取込み前の `XZ` の原本が取込み後と同じ bytes であること（CV17 の SD の取込みは原本を移して複製するだけで中身を書き換えない）は未確認の前提（実機の前後比較で確かめる）— `docs/function-design/29-io-daily-report-parser.md` §29.7.2
  - SD カードは SanDisk Ultra 16GB（表示速度 80MB/s）— owner確認 2026-09-29。CASIO の動作確認一覧との照合は未実施（`docs/plu-export-and-real-csv-verification.md` PLU-04）
- PLU export is app-to-register only; the app cannot auto-confirm register-side reflection — （記録済み） — `docs/project-memory.md`
- CV17 shows Z001/Z002/Z004/Z005 in one report-screen family and can write them together — 2026-07-06 field-check — `docs/project-memory.md`
- Files left in PC-side `EcrDatas` after SD import are "layout A"; CV17's explicit export is "layout B" (observed on daily reports; the Z004 parser accepts layout A only, layout B is not yet supported) — 2026-07 L3 — `docs/project-memory.md`
- Z004 returns appear as negative quantity/amount — 2026-08-15 issue #76 実機バッチ — `docs/plu-export-and-real-csv-verification.md`
- Non-JAN custom-code items are currently all sold via department key; there is no item-level register sale or automatic stock decrement for them — 2026-07-23 owner確認 — `docs/plu-export-and-real-csv-verification.md`
- CASIO ECR+（スマホアプリ）has a planned service end and is not the long-term primary integration — 2026-09-19 owner が同じ認識を示し、既存記録との対応を確認済み — `docs/plu-export-and-real-csv-verification.md`（"サービス終了予定があるため長期の primary integration にはしない"）/ `docs/decision-log.md`（D-022 "despite service-end risk"）
  - CASIO の公式ページ（<https://web.casio.jp/ecr/ble/ecr.html>、2026-09-24 確認）では、ECR+ は新規申込受付を 2026年1月4日に終了し、サービス提供は 2028年12月末に終了予定。ECR+ の終了後にレシートの控えをアプリで見られる必要があるかという問いに、owner はまず分からないと答えたうえで、ECR+ が終わるならレジの入替えと同時でないと困り、続かないなら代わりを自分たちで作るしかないと見ている — owner回答2026-09-23
  - 一方、店主は精算に ECR+ を要すると答えていたが、2026-10 にレジ本体だけで精算できることを確かめた（`### 在るもの` の ECR+ の行）。ECR+ の終了後に送信の失敗の印字や SD の控えがどう積もるかは未確認で、`docs/backlog.md`（ECR+ のサービス終了の項）に残る論点として置く
- 精算の系列（持ち帰りの EJ 1,504 files〈2022-07-28〜2026-09-29〉と 2026-07〜09 の Z 帳票の手元集計）— 持ち帰りデータの手元集計 2026-09-29、数え直し 2026-09-29（Issue #105）
  - 2026-07〜09 の 103 組すべてで、Z001 / Z002 / Z005 の精算回数は同じ値を共有し、Z004 の精算回数はそれより +1。EJ の本文に `日計明細 Z` の題と終わりの印字がある完了した精算 1,618 件（見出しのモード欄は精算 1,593・空 20・点検 3・PGM 2）は、2022-08-18 以降、Z004 に一意に対応する（ヘッダの 4 桁 = machine_no。この期間の持ち帰りデータでは、EJ タイトル末尾 4 桁 + 1 が Z004 の settlement_no と同じ値だった。2 つは別の番号で、ＰＬＵだけの精算が入るたびに差が変わるので、これは 2022-08-18 以降の観測であって、対応づけの規則には使わない。対応づけは時点証拠 ADR の SPEC-STK-TIME-D5a。D-118〈2026-10-16〉で EJ を在庫に使わないことになり、この対応づけは在庫の判定には使わない）。記録の種類は見出しのモード欄ではなく本文の題の行で決める
  - 使い始めの 2022-07-28〜08-17 は差が +1 にならない。Z004 のファイルは 2022-08-13 から始まり、この期間は単独の `PLU Z` 精算 3 件もそれぞれ Z004 を 1 つ使う。日付ごとの件数では、EJ の精算（`日計明細 Z` + `PLU Z`）と Z004 は、Z004 の無い 2022-07-28 と 08-05 を除くすべての日で一致する。Z004 の日付が EJ の印字日より遅れる例は無い。2022 年に中断した精算と再実行で末尾番号が重なる組が 5 組あるため、精算の区切りには終わりの印字がある完了した精算だけを使う。EJ の取引番号は全期間 8,624 件が +1 で連続
  - 逆向き: 本文で数えると、EJ に精算の記録が無い Z004 は 0 件（Z004 1,619 = `日計明細 Z` 1,616 + `PLU Z` 3）。見出しのモード欄で精算を数えると、モード欄が精算でない 25 件を取りこぼし、26 件が欠けて見える。精算の区切りはモード欄で判定しない
  - EJ ファイル名の日付は、そのファイルの最後の精算の日。精算の後の販売と、精算の無かった日の販売は、次に精算した日のファイルに入る。暦日でファイルを選ぶと取りこぼすため、欠けの判定はファイルの有無ではなく取引番号の連続性で行う
- 取消・訂正・値引きなどの EJ の形（持ち帰りの EJ の手元集計）— 持ち帰りデータの手元集計 2026-09-29（Issue #105）
  - 取消（`取引中止`、取引全体を無効にする記録で商品の行を含む）50 件、訂正 59 件、％値引き 57 件、マイナスキー 7 件（最後は 2023-03）、訂正ラベルの無いマイナスの部門行 7 件（2022-08）。全種類の実例があり、試し打ちは要らない
  - 今の parser が知らないモード: `点検` 181、`OFF` 34、`練習` 33（売上に入らないトレーニングの記録と見られる）、`PGM3` 7、`PGM1` 2
- 部門キーで小数 1 桁の数量を売った日（2026-09-29）は、EJ の数量・点数の行、Z001 の `総売` の個数/件数、Z005 のその部門の個数が小数になる（Z002 と Z004 には出ない）。2026-09-29 時点の `daily_report_parser.rs` は Z001 の個数/件数と Z005 の個数を整数でしか読まず、その日の日報の取込み全体が失敗した（その後、Z001 の総売の個数と Z005 の個数を 100 倍の整数〈小数 2 桁まで〉で読むようになった。2026-10-15 の現物の code）。`ej_parser.rs` の `parse_quantity` は整数だけ（今の code も同じ）。PLU の小数の Z004 は 2026-09-30 に試し、店主の回答（上の 2026-10-07 の行）と持ち帰りデータで確かめた — 持ち帰りデータの手元集計 2026-09-29（Issue #105）
- clear した PLU 枠の行は、名前が空・コード全桁 0 のまま、数量と金額を持って次の Z004 に残る（2026-09-28 の検証で 1 行）。2026-09-29 時点の `z004_parser.rs` はコード全桁 0 の行を数量・金額を見ずに空スロットとして読み飛ばしていた（その後 D-103 で、コード全桁 0 で個数か金額が 0 でない行を `InvalidJan` の行エラーにし、0/0 の枠だけを読み飛ばすようになった。2026-10-15 の現物の code）。D-117 で、アプリは clear 行を書かないと決めた（今の build は Clear 行を書く〈`PLU_CLEAR_ROW_ENABLED = true`〉。撤去は PLU の書出しの見直しの runtime）— 持ち帰りデータの手元集計 2026-09-29（Issue #105）

### 未確認

- Whether the daily/monthly report screens can fully replace the current Excel-print-binder record has not been accepted; this is planned as a go-live side-by-side check on one real day, not yet done — `docs/backlog.md`（日報画面のExcel印刷・バインダー代替受入判定 項） / `docs/function-design/56-ui-daily-sales.md`
- CSV export and print behavior have not been end-to-end verified in real use; owner's own characterization is "まともにテストしたことがない" / "印刷は中身を作っていない" — `docs/backlog.md`（CSV出力・印刷の実挙動確認 項）
- Whether the store will want a network (UNC/NAS) backup destination is still open; current handling is low priority pending an explicit need — `docs/backlog.md`（バックアップ保存先のUNC/ネットワークパス対応 項）
- 導入後、値上げのときに紙の前年リストを手で直す作業をやめるかは未確認。owner は、このアプリを渡すことがその作業の代わりになるはずと考えている — owner回答2026-09-23
- 布・ファスナー以外に、複数の商品が同じバーコードを共有するものがあるかは、店主が確認中 — 店主回答（owner 経由の質問票）2026-09-29

owner が既出の記憶があると答えた 2 件は、記録を探索した結果、以下のとおり見つかった（未確認へは回さない）。

- ECR+のサービス終了予定 → 見つかった。`docs/plu-export-and-real-csv-verification.md`「ECR+ は店舗で利用中だが、サービス終了予定があるため長期の primary integration にはしない」/ `docs/decision-log.md` D-022 の Alternatives considered「make ECR+ the primary integration despite service-end risk」
- 部門キー（現行レジ最大20程度）の構成は聞き取りで決めた → 見つかった。`docs/db-design/master-tables.md`「初期データ（全21部門、C-1/C-3 2026-03-29 確定）」は同じ 2026-03-29 の「廃番特価の対応方針（利用者ヒアリングで確定）」と同日の店舗ヒアリングに基づく
