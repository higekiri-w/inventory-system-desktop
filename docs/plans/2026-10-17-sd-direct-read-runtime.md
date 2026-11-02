# Plan Packet: SD 直読みの runtime（毎日の売上データをレジの SD から読む、D-111 の実装）

wave に属さない単独の lane（2026-10-17 起票。owner の「動かせるだけ動かす」〈repo 外の回答台帳 TD-304〉で Coordinator が同時に起票した 6 本の 1 本）。設計は merge 済み（PR #150、[decision-log](../decision-log.md) D-111。設計 lane の packet と Matrix は `docs/archive/plans/2026-10-06-sd-direct-read.md`・`docs/archive/plans/test-matrices/2026-10-06-sd-direct-read.md`）で、本 lane はその既決の契約を実装する。branch は `agent/sd-direct-read-runtime`（main `f5d7aaf4` から起票し、2026-10-23 に main `eed07d37` を取り込んで本 packet を今の code と migration の版に当て直した）。2026-10-23 の並走は `agent/stocktake-count-runtime-b`（PR #188、棚卸しの runtime の b1、migration vZ）と `agent/backup-offsite-runtime`（PC の外の控えの runtime、D-114）で、owner の決定（repo 外の回答台帳 TD-384）で本 lane は並走の候補。本 lane は新しい決定の番号を使わない（下の Design Readiness）。

**規模と手法（1 lane のまま。実装の順を S1〜S6 に固定する）**: 2026-10-17 に本 worktree（HEAD `f5d7aaf4`）で数え、2026-10-23 に main `eed07d37` を取り込んだ head で同じ command を数え直して同じ件数だった（原価 vB・棚卸し (a) は下の構築の箇所を増やさず、行の位置だけを動かした）。形の変わる型を作る・読む既存の Rust は 19 file（`rg -l 'AppState \{|NewDailyReportImport \{|DailyReportInputFile \{|DailyReportSourceFileRequest \{|CachedDailyReportPreview \{|commit_daily_report_import\(|DailyReportParseResult \{|enum BizError|impl From<BizError> for CmdError|collect_commands!|daily_report_imports \(' src-tauri/src src-tauri/tests | sort | wc -l` → `20`。うち `db/schema_v4.rs` は CREATE 文の hit で変えない）に、登録の 3 file（`io/mod.rs`・`db/mod.rs`・`tests/design_compliance_test.rs`）と新設 3〜4 file（`io/register_sd.rs`・`io/pos_source_copy.rs`・`db/schema_v12.rs`〈番号は 22 §17。#188 b1 の vZ が v11 を取った後に取り直した〉、scan を分けるなら `biz/daily_report_import_service/scan.rs`）と、版の期待を持つ既存の test の 5 file（下の S2。`migration.rs`・`lib.rs` は上の 19 file に入り、新しく足すのは `schema_v2.rs`・`schema_v9.rs`・`schema_v10.rs` の 3 file。2026-10-23 の Plan Review round 2 の是正で `schema_v10.rs` と `lib.rs` の起動の test を足した）。frontend は `src/features/daily-report-import/` の 8 file（`find src/features/daily-report-import -type f | sort | wc -l` → `8`。2026-10-23 の Plan Review round 1 の是正で数え直した。前に書いた `rg -l 'daily-report-import|DailyReportSourceFileRequest' src --glob '!src/lib/bindings.ts'` の `7` は、この directory の外の file〈import の path の文字列を含む `csv-import`・`query-keys.ts` 等〉を数えていて、directory の 8 file を数えていなかった）と新設の test 2 file、生成物の `src/lib/bindings.ts`。wire の型は新しい command 2 つと DTO 4 つ、既存の DTO への省略可の field 1 つ（`DailyReportSourceFileRequest.source_path`）。DB は NULL を許す列 1 つ（migration 1 つ）。画面は既存の日報取込みタブ 1 つ（新しい route・画面は無い）。横断の約束（範囲・0・失敗の時点と順序・種類の混在）がかかる経路は、`settlement_no`（IO-07 → cache → DB、wire に出さない）と、写しの失敗の順序（TX の前）と、SD から来た失敗の戻り先（UI の `sd_list`）の 3 つで、どれも 1 つの正本の節が持つので、別の横断の表は作らない（下の「新しい契約の文と境界の 4 観点」で一巡した）。
- 分けない理由: backend（IO-09・IO-10・BIZ-08・CMD-12・migration）だけを先に入れる切れ目は、画面から呼ばれない command と、写しの規則の効く手で選ぶ経路（UI-07-D16 の `source_path`）の片側だけの main を作り、L3 が 2 回になる（owner は 2026-10-20 から手が使いにくい。介入の上限 9、TD-296）。1 lane で順に積み、L3 を 1 回にする。
- 実装の順: S1 IO（IO-07-D5・IO-09・IO-10）→ S2 DB（migration・`sales_repo`）→ S3 BIZ（scan・D4・D5・D6・手順 1a・手順 9）→ S4 CMD と登録・bindings → S5 frontend → S6 設計正本の「未実装」の注記を現行の記述へ直す。

## Workflow State

- Phase: implementing
- Risk: R3
- Plan Commit: 7883545fe67cb8f4d4de8c2de109c5e691c62d05
- Amendments: ed721d013c023390e3c79146bbfd7896357fd708
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（subagent_type: writer）
- Plan Reviewer: fresh Opus 5.5 + Codex（model は発注時に決める。Codex が起動できない日は座組表の Codex 停止中の列）
- Final Reviewer: Fable 5.1（Claude 側、R3）+ Codex（GPT-6.1 Sol 既定）。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Review Minimum: 1
- Human Gate: ready,merge,manual
- Branch: agent/sd-direct-read-runtime

遷移の記録:

1. kickoff → spec-check（2026-10-17、起草役）: Coordinator の発注 367（D-111 の既決の契約の実装。D-117・D-118 の runtime は入れない）を Scope にし、規模を数えて 1 lane のままと決め、Risk を R3 と記録した（下の Risk）。
2. spec-check → design（2026-10-17、起草役）: 対象の正本は 29 §29.2（IO-07-D5）・§29.7・§29.8、37 §37.3・§37.4・§37.9、45 §45.2〜§45.8、55 UI-07-D12・D15・D16、24 §14.14・§14.18a、pos-tables §12b、22。設計 lane の fresh Final broad の P3 4 件（`docs/backlog.md` の「SD 直読みの runtime」）と、起票で見つけた正本の穴（migration の手順が 22 に無い、IO-07-D5 の `i64` の範囲、UI-07-D16 の選択の一覧の細部と文言、37 §37.9 手順 10・45 §45.6a 手順 2a〈D-117〉の実装の lane）を、本 packet と同じ commit で正本に入れた（下の Design Readiness）。
3. design → plan-draft（2026-10-17、起草役）: 上の更新は正本にあり、D-111 の決定の向きを変えない。未解決の設計の問いは無い。残る P1・P4（Contract Probe）は設計の問いではなく、L3 の前に確かめる未確認の前提（下の Contract Probe）。plan-gate へは Coordinator が進める。
4. plan-draft → plan-gate（2026-10-23、Coordinator、本 commit）: main（`eed07d37`）への同期と、Scope・AC を今の code と migration の版へ当て直した（D-114 の 71 MNT-01-D10・§71.11.3 を前提として 29 §29.7.4 と IO-09-D3 を直した）。Contract Probe P1 (a)・P4 は合格（下の Contract Probe）、P1 (b) は未確認のまま。owner の判断事項は無い。Plan Review round 1 に出す。Codex は利用上限で 2026-10-24 まで使えないので、round 1 は fresh Opus 5.5 で始め、Codex が戻った時点で同じ round の Codex 分を足す（`docs/AGENT_OPERATING_MANUAL.md` §3.3）。
5. plan-gate のまま是正（round 1、2026-10-23、Coordinator、本 commit）: round 1（Claude 側 fresh Opus 5.5 だけ。Codex は利用上限で 2026-10-24 まで使えず、`docs/AGENT_OPERATING_MANUAL.md` §3.3）の reject を起草役が `881700ca` で直した（下の Review Response）。設計の方向は変えないので plan-gate に留め、round 2（fresh Opus 5.5 + Codex）で再 review する（`docs/DEV_WORKFLOW.md` Workflow State「a plan-gate rejection corrected in place stays at plan-gate」）。
6. plan-gate のまま是正（round 2、2026-10-23、Coordinator、本 commit）: round 2（Claude 側 fresh Opus 5.5 と Codex）の reject を起草役が `b852071f`（Claude 側）と `af0f95b0`（Codex 側）で直した（下の Review Response）。設計の方向は変えないので plan-gate に留め、round 3（天井、fresh Opus 5.5 + Codex）で再 review する。
7. plan-gate のまま是正（round 3〈天井〉、2026-10-23、Coordinator、本 commit）: round 3 の Claude 側 fresh Opus 5.5 の reject を起草役が `1c154230` で直した（下の Review Response）。round 4 は回さず、Coordinator の disposition（4 件の一括是正）の後に closure（fresh Opus 5.5 + Codex。Codex は round 3 で未実施なので、是正の後の版の全体を見る）に出す。P1/P2 = 0 なら plan-approved へ進める。
8. plan-gate のまま是正（closure、2026-10-23、Coordinator、本 commit）: closure の Codex 側の reject（P2 1）を起草役が `8fef2a03` で直した（下の Review Response）。path の包含の規則で、`CLAUDE.md` が安全境界（path traversal）の review を Claude の subagent に出さないと定めるので、是正の後の確かめは Codex の closure と Coordinator の inline の review で行う（inline は fresh context でないので Final Review の本数に数えない）。
9. plan-gate → plan-approved（2026-10-23、Coordinator、本 commit）: closure は Claude 側 fresh Opus 5.5 = approve（`499da80d`）、Codex = closure 2 回目 approve（`7883545f`、P1/P2/P3 0）、Coordinator の inline の review（`8fef2a03` の link 先の包含）で、P1/P2 = 0（下の Review Response）。Plan Commit は Codex の closure が監査した `7883545f`。Contract Probe P1 (b) は L3 の前の必須（AC10）で、plan-approved の条件ではない。implementing へは、並走の PR #188（migration vZ、同じ file の重なり）と実装の Writer の枠を見て Coordinator が進める。
10. plan-approved → implementing（2026-10-24、Coordinator、本 commit）: main（`843c4938`、PR #192・#195・#191 と closeout の後）を単段で取り込み（衝突なし。PR #191 で `identifier` が `com.inventory-system.desktop` に変わったが、本 lane の docs・code に前の名の hit は無い）、AC5・AC6・AC9 の rg の baseline を逐語で再測して上の値と一致した（AC5・AC6 は hit 0、AC9 は 14 件・10 件・1 件）。cargo と npm の test の baseline は実装の Writer が着手時に取る。実装の Writer に Scope の S1 から発注する。
11. implementing のまま Gated Amendment（2026-10-25、Coordinator、本 commit）: #188 b1 の vZ が先に merge して v11 を取ったので、本 lane の migration を v12 に取り直した（code と 22 §17 は Writer の `621f86e5`・`9a6fa61b`）。packet が file 名 `schema_v11.rs` を現在形で持ち偽になったので、`docs/DEV_WORKFLOW.md` の連番 registry の規則（正本の merge の後に採番し直したら gated amendment と同じ packet の中の full sweep を記録する）に従い、`ed721d01` で packet の現在形の箇所（規模と手法・S2 の migration・Matrix の migration の行・Contract Ledger の版の番号の行）を v12 に直し、Scope に無かった `docs/inventory_system_erd.html`（版の範囲と `settlement_no`）を足した（上の「Gated Amendment 1 で変えた正本」）。日付のある記録（当て直し・Review の是正の節、Implementation Results の 2026-10-24 の行）は直さない。sweep の式は `rg -n 'schema_v11|\bv11\b|版 11'` で、残る当たりは #188 b1 の vZ を指す行と日付のある記録だけ。登録は本 commit で `ed721d01` を `Amendments` に足す。review は Final Review broad に含める（前例: PR #188 の amendment 2 本目）。

## Owner Effort Budget

- 介入回数上限: 9（repo 外の回答台帳 TD-296 の 5a。既定 6 から上げた）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 9 | 0（2026-10-17 起票時。plan-approved は Coordinator が進める〈TD-296 の 4a〉ので数えない） | 5（Contract Probe P1 の (b) の入力〈取込みの後の file の写し〉を owner に頼む 1、L3 の 1 round 1、L3-4 を導入の時へ送る残余の受容 1、Ready 1、merge 1） | 4（L3 の 2 round 目 1、P1 が不一致のときの設計の見直しの判断 1、Open Questions の行の直し方の文を添えるかの判断 1、その他 1） | 9 = 0 + 5 + 4 |

既定値・数え方は `docs/DEV_WORKFLOW.md` `Owner Effort Budget`（decision point の単位。同じ依頼で得ても、判断の数だけ数える）。P4 は 2026-10-23 に合格したので、P4 の判断の予備は外した（Plan Review round 2 の是正）。承認依頼は `この change での介入 N 回目 / 予算 9 回` と「承認すると、売上データ取込み画面でレジの SD を読み、精算を選んで日報を取り込めるようになる」の 1 文。owner は 2026-10-20 から手が使いにくいので（TD-296 の 1a）、問いは記号か短文で答えられる形にする。L3 は D-124 のとおり、本 PR が merge の train の先頭に来て最新の `origin/main` を取り込んだ head で頼む。他の PR と 1 回の依頼に束ねるのは `docs/DEV_WORKFLOW.md` `## Wave Operation` の条件（先に merge する全ての PR と path を共有しない）を満たすときだけで、#188 とは `lib.rs`・`db/migration.rs`・`src/lib/bindings.ts` 等を共有する見込み（下の節の所有表）なので、#188 の後に束ねずに頼む見込み。

## Risk

Risk: R3

Reason:
`docs/DEV_WORKFLOW.md` Risk Tiers の R3 の行のうち「DB」（`daily_report_imports.settlement_no` の migration）、「POS CSV」（SD の Z の読取りの経路と同じ精算の二重取込みの拒否）、「Tauri command DTO」（新しい 2 command と DTO 4 つ、`DailyReportSourceFileRequest.source_path`）、「operator workflow」（日報取込みの標準の手順を SD から読む形にする）に当たる。R4 ではない: migration は NULL を許す列を足すだけで既存の値を変えず（22 §17）、SD へは書かない（IO-09-D3。許可の列の API だけで、source の検査 test が固定する）、写しはアプリの data の folder に上書きせずに書き（IO-10）、実店舗のデータを repo に入れない（test は合成の tree）。backup・restore・破壊的な操作を変えない。
required gate の green / red は変わらない（製品の code と test と docs だけで、gate の定義・helper・classifier・CI・workflow の正本に触らない）。Scope の予定 path は `scripts/ci/classify-changes.sh` の workflow の一覧（AGENTS・CLAUDE・DEV_WORKFLOW・MANUAL・code_review・project-profile・agent-guidance・templates・`.agents`・`.claude/{rules,commands,skills}`・PR template）に当たらないので Final Review Minimum 1。operator に見える状態の lifecycle（scan → 一覧 → preview → 写し → 結果）を変えるので、Contract Audit の 2 本目は推奨（`docs/DEV_WORKFLOW.md` Contract Audit の Double audit。採るかは Coordinator）。Windows の実機の確認があるので Human Gate に `manual`。

## Goal

Goal Invariant:

### 最小完了条件

- 店の PC で、売上データ取込み画面の日報取込みタブの「SD から読む」を押すと、取外し可能な drive の `CASIO\SR500_550_4000` を探して `XZ`・`XZ_BKUP` の Z001 / Z002 / Z005 を読み、精算ごとの一覧（取り込めます / 取込み済み / 同じ精算を取込み済み / ファイルがそろっていません / 読めません）と SD を戻す案内を出す。「取り込めます」の行から既存のプレビュー・取込みへ進め、取り込むと読んだ 3 本の写しが `pos-sources/casio-sr-s4000/sd/` に残り、`daily_report_imports.settlement_no` に精算回数が入る（IO-09・IO-10・IO-07-D5・BIZ-08-D3〜D6・CMD-12-D1・UI-07-D12・D15・D16、22 §17）。
- 予備の「ファイルを選んで取り込む」で 1 つずつ選び足し・外せ、選んだ file がレジの SD の上なら SD の入力として扱われる（写し・精算回数の検査が効く）。
- 同じ精算を別の bytes で 2 回取り込めない（BIZ-08-D4）。SD の経路で精算回数の読めない束は取り込めない（BIZ-08-D6）。

### 失敗定義

- SD に書く・file を動かす経路がある、または root の外を読む（root の下の link・junction が外を指すときを含む。IO-09-D3・D5）。
- 同じ精算が追加確認だけで 2 回保存される、または同じ日の正常な 2 回目の精算が止まる。
- 写しの無い SD の取込みができる、または写しの失敗の後に同じ preview で再試行できない。
- 既存の手で選ぶ経路（PC 上の file）の取込み・同日追加の確認・取消が変わる。ただし、同じ対象日・同じ精算回数で別の bytes の束を BIZ-08-D4 で拒むのは D-111 の意図した変更で、ここに当たらない（精算回数の違う同じ日の束の追加確認と、精算回数の読めない束〈`None`〉の扱いは変わらない。既存の test の fixture の扱いは S3）。
- CMD が IO を直接呼ぶ、規則が CMD・UI にある（`src-tauri/tests/architecture_test.rs`）。

### 非目的

- 取込み済みの Z・EJ を `XZ_BKUP` へ移す操作（D-111 (1) の (b)、`docs/backlog.md` の別項目）。CV17 の取込みは (b) まで店が今の運用で続ける（D-111 の運用の制約）。
- PLU の占有の照合（37 §37.9 手順 10、45 §45.6a 手順 2a、D-117）と、Z004・EJ の取込み・計数の置き場（D-118）。Z004・日報・EJ の一括取込みと EJ の閲覧（D-121 の design lane）。
- 写しの保持期間・削除（D-111 の未決 A。既定は消さない）。backup・PC の外の控えには写しを入れない（未決 B、D-114 で決まった。71 の冒頭）。窓より前の未取込みの検知。
- 画面のデザインの刷新（TD-296 の 11c。UI-07-D16 の置き場所はひとまず）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

設計を含む変更なので毎日の取込みの操作列を置く。**この lane を完了できる**（merge）と、**通常運用を達成できる**（店で毎日 SD から取り込む）は別で、後者は本番の DB を店に入れた後（`docs/project-memory.md` の「無いもの」）。(b) の lane まで店は CV17 の取込みも続ける（D-111 の運用の制約）。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| その日の精算の後 | 精算する（ECR+ かレジ） | レジが SD の `XZ\yyyy\mm\` に Z001/Z002/Z004/Z005、`XZ` 直下の EJ に記録を書く | SD に精算の file がある | 2026-10-06 版での SD の採取（29 §29.7.2 SD-02・SD-03） |
| SD がレジにある | SD を抜いて PC に差し、CV17 の取込みをいつもどおり済ませる（(b) の lane まで） | Z・EJ が `XZ_BKUP` へ移る | 取込みが終わった | D-111 の運用の制約。アプリは `XZ` と `XZ_BKUP` の両方を読むので、CV17 の前でも後でも取り込める。前後で同じ精算を二重に取り込まないのは、移った file が同じ bytes であること（P1 の (b)、L3 の前に確かめる） |
| 日報取込みタブ | 「SD から読む」 | 「読み終わりました。いつもの取込みが済んでいれば、SD はレジに戻してください（次の精算に要ります）。」と精算の一覧（UI-07-D15） | 「取り込めます」の行がある（無ければ (b)・(c) の文） | 自動の発見は L3-1 |
| 一覧が出た | SD をレジへ戻す | 次の精算ができる | 読んだ内容はアプリが 30 分持つ（CMD-12-D1） | — |
| 「取り込めます」の行 | 「確認する」 | 既存のプレビュー（同日追加の確認を含む） | プレビューが出る | 取込み前の原本の精算回数が読めること（P1 の (a)。2026-10-23 に 2026-10-18 版での SD の写しで合格） |
| プレビュー | 「取り込む」 | 日報が保存され、3 本の写しが `pos-sources/` に残る。「日次売上を見る」へ進める | 結果が出る | 写しの失敗は preview に戻って同じ token で再試行（55 の日報取込みの利用者フロー手順 2） |
| 結果（SD から来た取込み） | 「SD の一覧へ戻る」 | 同じ一覧に戻り、いま取り込んだ行が「取込み済み」になる。SD を読み直さない（SD はレジに戻したままでよい） | 残りの「取り込めます」の行の「確認する」へ（45 §45.6b 手順 4、55 UI-07-D15 (6)） | 読んだ内容の期限は 30 分（CMD-12-D1）。過ぎたら「確認する」で固定の文が出て、一覧の上の「SD から読む」で読み直す |
| 同じ日に 2 回目の精算をした | 同じ操作 | 1 回目は「取込み済み」、2 回目は「取り込めます」 | — | 同じ日の複数の Z（SD-08、2026-10-06 版での観測） |
| SD が読めない・過去の分 | 「ファイルを選んで取り込む」で 1 つずつ選ぶ | 3 つそろうと「確認する」。SD の上の file なら SD の入力として扱う | 3 つそろう | UI-07-D16 |

Plan Review は、この列が「正常な条件で目的を達成できるか」と「危険な結果を出さないか」を別々に答える。

## owner の判断事項

本 lane で新しく owner に諮る製品の判断は無い。D-111 の判断 1〜5 は 2026-10-06 に、P2・P3 の扱いは 2026-10-07 に決まっている（D-111）。owner に頼む・諮るのは Contract Probe P1 の (b) の入力の写し（L3 の前、店の事実）・L3（manual）・L3-4 を導入の時へ送る残余の受容・Ready・merge と、Open Questions の候補 1 つ（Owner Effort Budget）。P1 が不一致なら設計の見直し（BIZ-08-D4・D6）を owner に諮る（予備）。

## Scope

### S1 IO（`io::daily_report_parser`・`io::register_sd`・`io::pos_source_copy`）

- IO-07-D5: `DailyReportParseResult.settlement_no: Option<i64>`（29 §29.2）。作る所は `src-tauri/src/io/daily_report_parser.rs` の `parse_daily_report_bundle` の `DailyReportParseResult` の構築の 1 か所。今の `settlement_number` は `u64` を返し、D3 の一致の判定はその値で行う（変えない）。そろった値を `i64::try_from` し、収まらなければ `None`（29 IO-07-D5 に 2026-10-17 に足した文）。読む所: `biz/daily_report_import_service/parse.rs`（cache へ）、同 file の test の helper。
- IO-09（新設 `src-tauri/src/io/register_sd.rs`、`io/mod.rs` に `pub mod register_sd;`）: 29 §29.7.3 の型と §29.7.4〜§29.7.6 の 6 関数（`find_register_sd_roots`・`resolve_register_sd_root`・`locate_in_register_sd_roots`・`check_selected_file_present`・`list_register_sd_entries`・`read_register_sd_file`）。Windows の drive の列挙は既存の `windows-sys`（`src-tauri/Cargo.toml` の `[target.'cfg(windows)'.dependencies]`、`=0.61.2`）で、新しい crate を足さない。使う項目と feature（29 §29.7.4 手順 1。2026-10-23 に cargo の cache の windows-sys 0.61.2 の source で置き場を確かめた）: `GetLogicalDrives`・`GetDriveTypeW` は既にある `Win32_Storage_FileSystem`、定数 `DRIVE_REMOVABLE` は `Win32::System::WindowsProgramming` にあるので feature `Win32_System_WindowsProgramming` を足す、`SetThreadErrorMode`・`SEM_FAILCRITICALERRORS`・型 `THREAD_ERROR_MODE` は feature `Win32_System_Diagnostics_Debug` を足す（PC の外の控えの runtime も同じ feature を足すので、先に merge した lane が足し、後の lane は足さない）。`CASIO\SR500_550_4000` の folder 名は 1 つの定数（2 つの要素の配列 `REGISTER_SD_FOLDER: [&str; 2]`。形は 29 §29.7.4 手順 1、Plan Review round 3。#190 の packet の所有表の `io/mod.rs` の行も「`CASIO`・`SR500_550_4000`」の 2 つの要素で、矛盾しない）にし、71 MNT-01-D10（`mnt::backup`・`mnt::offsite`）と共有する（D-114）。置き場は merge の順に依らず `src-tauri/src/io/mod.rs` の `pub(crate) const`（29 §29.7.4 手順 1。PC の外の控えの runtime (a) の packet〈PR #190〉の所有表の `io/mod.rs` の行と同じ所。本 lane が先なら本 lane が置き、(a) が先なら (a) が置いた定数を `io::register_sd` が使う）。`io::register_sd` の module の中に置かない（置くと MNT が register_sd の module に依存し、(a) が先のときと置き場が変わる）。許可の列の外の API を使わない（IO-09-D3。許可の列に型 `THREAD_ERROR_MODE` を 2026-10-23 の round 1 の是正で足した。`BOOL`・`PCWSTR` は名指しせず推論で書く。`crate::`・`super::` の参照は `super::` の folder 名の定数だけ〈round 2〉）。root の下かは 29 IO-09-D5 のとおり、root と見る path（入る folder・返す file・開く file）を `std::fs::canonicalize` で実体の path に解いてから要素ごとに比べ、確かめた解いた path をそのまま `read_dir`・`metadata`・`File::open` に渡す（Plan Review の closure の Codex 側の是正。`canonicalize` を許可の列に足した。外なら `InvalidPath` で、37 §37.9 手順 3 の固定の文）。thread の error mode は module の中の 1 つの guard（作ると `SEM_FAILCRITICALERRORS` にし、drop で元の値へ戻す。Windows 以外は何もしない）で持ち、file system を見る 5 関数（`find_register_sd_roots`・`resolve_register_sd_root`・`check_selected_file_present`・`list_register_sd_entries`・`read_register_sd_file`）の最初で作る（29 §29.7.4 手順 1、round 2。純関数の `locate_in_register_sd_roots` は file system を見ないので作らない）。`cfg(windows)` の本体は hosted CI（ubuntu）でも WSL の `cargo check --release` でも compile されない（下の Residual Risk）。`RegisterSdError::Io.relative_path` の意味は 29 §29.7.3 の注記（2026-10-17）。§29.7.6 手順 2 の「`max_bytes + 1` byte まで読む」は、`std::io::Read` を受ける module の中の非公開の関数（`pub` にしない。29 に signature の無い `pub fn` を作らない、下の設計照合）に分け、`read_register_sd_file` は開いた `File` をそれに渡す。test は読んだ byte の数を数える reader を同じ関数に渡す（Matrix の `register_sd::read_stops_at_limit`。Plan Review round 2 の Codex 側の是正）。
- IO-10（新設 `src-tauri/src/io/pos_source_copy.rs`、`io/mod.rs`）: 29 §29.8.2 の `save_pos_source_copy` と §29.8.3 の手順（path を決める → `create_dir_all` → 一時 file → rename、上書きしない、同じ bytes は書かない、違う bytes は `~` + SHA-256 の先頭 12 桁）。SHA-256 は既存の `sha2`。
- 設計照合: `src-tauri/tests/design_compliance_test.rs` の `build_doc_to_modules_map()` の `29-io-daily-report-parser.md` の行に `io::register_sd` と `io::pos_source_copy` を足す。新しい `pub fn` は 29 にシグネチャのあるものに限る（無い `pub fn` は設計照合の警告になる）。S3 の scan を `biz/daily_report_import_service/scan.rs` に分けるなら、同じ map の `37-biz-daily-report-import-service.md` の行に `biz::daily_report_import_service::scan` を足す（map に無い module の `pub fn` は「未文書化 pub 関数」として test が落ちる。Plan Review round 3）。

### S2 DB（migration・`sales_repo`）

- migration（22 §17 MNT-03-D15）: 新設 `src-tauri/src/db/schema_v12.rs`（番号は merge の時点の次。#188 b1 の vZ が先に merge して v11〈`schema_v11.rs`〉を取ったので、本 lane が v12 に取り直した〈Gated Amendment 1〉）、`db/mod.rs` の `mod schema_v11;` の後（`mod` の並びは名前の順）に `mod schema_v12;`、`db/migration.rs` の `migrations()` の最後（v11 の後）に `MigrationKind::Sql` で登録。vS は `MigrationKind::Sql` なので、`Custom` の migration の source の helper の数を数える `db/migration.rs` の test には足さない。
- 版の期待を持つ既存の test（Plan Review round 2 の是正で洗い直した）: 全部の migration を当ててから版を期待する test は、vS を登録すると red になる。直し方を 2 つに分け、版が増えても test を直さずに済む形にする（22 §17）。
  - 最新の版を期待する test（`init_database`・`setup_latest`・`migration::migrate`・起動の `prepare_database_with` の後の `MAX(version)`、`schema_versions` の行数）は、版の数値・版の定数でなく `app_max_version()`（`src-tauri/src/db/migration.rs`、`migrations()` の最大の版。先例は `db/schema_time_evidence.rs` の `test_time_evidence_schema_req205_t4_normal_migrate_has_no_new_schema` と `db/migration.rs` の `test_migrate_req903_d11_same_version_is_noop`）と比べる。行数は版が 1 から連番なので `app_max_version()` と同じ値。期待の意味（新しい DB は最新の版まで当たる、2 回目は版を増やさない）は変えない: `db/migration.rs` の `test_migration_req903_is_idempotent`（1 回目と 2 回目の `MAX(version)`、`schema_versions` の行数の 3 か所）・`test_migration_req401_v4_creates_daily_report_tables_and_indexes`・`test_migration_req907_v5_seeds_all_plu_slots`・`test_migration_req907_upgrades_v4_database_and_seeds_all_plu_slots`・`test_migration_req903_v8_fresh_database_and_empty_tables`・`test_migration_req903_v8_commit_and_fk_restore_failures`、`db/schema_v2.rs` の `test_v2_req903_applied_on_fresh_db`・`test_v2_req903_idempotent_rerun`（3 か所。round 2 で見つけた）、`db/schema_v9.rs` の `test_migration_req903_vl_fresh_database_has_keys_counts_and_copy_columns`、`db/schema_v10.rs` の `test_migration_req903_vb_fresh_database_renames_cost_columns`（`setup_latest` の後。原価の 6 列の期待は vS が触らないので変えない）、`lib.rs` の `test_startup_req903_rebuild_moves_app_data_and_legacy_db_then_empty_vb`・`test_startup_req903_rebuild_with_committed_manifest_writes_no_restore_log`（`(crate::db::VB_VERSION, 0)` の版の側。商品 0 件の期待は変えない）。
  - vB が対象の test（vB の後の全 object の SQL か、vB の失敗を外した後の成功を見る）は、`migration::migrate(&conn)` を `migrate_up_to(&conn, VB_VERSION)` に替え、`(VB_VERSION, VB_VERSION)` の期待は変えない（22 §16 の「版の上限を取る test 用の migrate」と同じ。`schema_objects` を後の migration の列に結びつけない）: `db/schema_v10.rs` の `test_migration_req903_vb_rebuild_keeps_columns_indexes_and_fk`（vS まで当てると `daily_report_imports` の CREATE 文が変わり `schema_objects` の比較で落ちる）・`test_migration_req903_vb_verify_or_commit_failure_rolls_back`（末尾の「失敗を外せば同じ DB で当たる」）。
  - 変えない: `migrate_up_to` で版を指定する test（`VB_VERSION - 1`・`(8, 8)`・`(9, 9)` 等）、すでに `app_max_version()` と比べる test、`db/schema_v10.rs` の `test_migration_req903_vb_rerun_is_noop`（vB の行が 1 つかを数える。最新の版に依らない）、`db/migration.rs` の `apply_sql_migration` に版の引数 `10` を渡す test（任意の版の番号で、最新の版の期待でない）と `migrations()` の v10 の登録、`mnt/backup.rs` の `version >= 1`。
  - 洗い直しの command と出力（2026-10-23、本 worktree の HEAD `9f02ddd7`。code は main `eed07d37` と同じ。数値の `10` の期待と、版の定数〈`VB_VERSION` 等、2 文字目が大文字の `V?_VERSION`〉で最新を期待する組の両方を拾う。`VB_VERSION - 1` は拾わない）:

    ```
    $ rg -n 'assert_eq!\((v1|v2|v|version|max_version), 10\b|count, 10[,)]|\(10, 10\)|\((crate::db::)?V[A-Z]_VERSION, ((crate::db::)?V[A-Z]_VERSION|0)\)' src-tauri/src | sort -t: -k1,1 -k2,2n
    src-tauri/src/db/migration.rs:757:        assert_eq!(v1, 10);
    src-tauri/src/db/migration.rs:767:        assert_eq!(v2, 10, "バージョンが変わってはいけない");
    src-tauri/src/db/migration.rs:774:            count, 10,
    src-tauri/src/db/migration.rs:846:        assert_eq!(version, 10);
    src-tauri/src/db/migration.rs:910:        assert_eq!((max_version, version_count), (10, 10));
    src-tauri/src/db/migration.rs:926:        assert_eq!(max_version, 10);
    src-tauri/src/db/migration.rs:1548:        assert_eq!(max_version_and_count(&conn), (10, 10));
    src-tauri/src/db/migration.rs:1687:            assert_eq!(max_version_and_count(&conn), (10, 10));
    src-tauri/src/db/schema_v10.rs:390:        assert_eq!(version(&conn), (VB_VERSION, VB_VERSION));
    src-tauri/src/db/schema_v10.rs:441:        assert_eq!(version(&conn), (VB_VERSION, VB_VERSION));
    src-tauri/src/db/schema_v10.rs:569:        assert_eq!(version(&conn), (VB_VERSION, VB_VERSION));
    src-tauri/src/db/schema_v2.rs:378:        assert_eq!(max_version, 10);
    src-tauri/src/db/schema_v2.rs:586:        assert_eq!(v, 10);
    src-tauri/src/db/schema_v2.rs:596:        assert_eq!(v, 10, "バージョンは変わらない");
    src-tauri/src/db/schema_v2.rs:601:        assert_eq!(count, 10, "v1+v2+v3+v4+v5+v6+v7+v8+v9+v10の10レコードのみ");
    src-tauri/src/db/schema_v9.rs:471:        assert_eq!(version(&conn), (10, 10));
    src-tauri/src/lib.rs:1215:        assert_eq!(version_and_products(&conn), (crate::db::VB_VERSION, 0));
    src-tauri/src/lib.rs:1249:        assert_eq!(version_and_products(&conn), (crate::db::VB_VERSION, 0));
    ```

    18 行・14 本の test（`app_max_version()` にする 12 本と `migrate_up_to(&conn, VB_VERSION)` にする 2 本）。拾い漏れの確かめとして、`MAX(version)`・`schema_versions` を読む test のある file（`rg -l 'MAX\(version\)|schema_versions' src-tauri/src src-tauri/tests`）の各 hit を読み、上の「変えない」に分けた。着手時（と下の取り直し）に同じ command を逐語で回し、表と違えば本節を直してから進める。
  - 取り直し（22 §17、#188 b1 の vZ と先に merge した側が若い番号）: 後から merge する側が直すのは file 名・`mod` の行・`migrations()` の登録で、上の形にした test は直さない。#188 b1 の packet（Plan Commit `aba17e2e`、Matrix の T1）は同じ test の一部（`lib.rs` の 2 本、`migration.rs` の `:910`・`:1548`・`:1687`、`schema_v9.rs` の `:471`、`schema_v10.rs` の `:390`）を vZ の版の定数（例 `VZ_VERSION`）へ替え、`schema_v10.rs` の 2 本を `migrate_up_to(&conn, VB_VERSION)` に替える計画。b1 が先に merge したら、本 lane は取り込んだ後に上の command を回し、vZ の版の定数で最新の版を期待する test と、b1 が数値（`11` 等）にした最新の版の期待を、すべて `app_max_version()` に替える（command の `\b10\b` の所はその時点の最新の版の数値に読み替える）。vZ の振舞いを見る test（vZ の版まで当てる `migrate_up_to`）は変えない。
- `NewDailyReportImport.settlement_no: Option<i64>`（24 §14.14）: 定義は `db/sales_repo.rs` の `NewDailyReportImport`、INSERT は同 file の `insert_daily_report_import`。構築 8 か所（`rg -n 'NewDailyReportImport \{' src-tauri/src` から定義と `-> NewDailyReportImport {` の signature の行を除く。2026-10-23）: 本番 `biz/daily_report_import_service/commit.rs` の `execute_commit`、test は `db/sales_repo.rs` の helper `new_daily_report_import` と `test_get_completed_daily_report_aggregate_req501_propagates_parent_and_line_nulls` の 2 か所、`biz/daily_report_import_service/tests.rs` の `test_daily_report_req401_corrupt_same_date_source_files_metadata_fails_safe`、`biz/sales_service.rs` の helper `seed_daily_report`、`cmd/sales_cmd.rs` の `test_get_daily_sales_cmd_passes_official_report_req501`、`cmd/daily_report_import_cmd.rs` の `test_daily_report_cmd_req401_snapshot_mismatch_deletes_preview_token`（どれにも `settlement_no` を足す。既存の test は `None` で、意味を変えない）。
- 新しい repository 関数 `find_same_settlement_daily_report_import`（24 §14.18a、id だけを返す）。`DailyReportImport`（list の DTO、`db/sales_repo.rs`）と既存の SELECT・`row_to_daily_report_import` は変えない（24 §14.14 の後の注記）。
- 変えない: `db/migration.rs` の `test_migration_req401_v4_daily_report_constraints` の 3 つの INSERT と helper `seed_v6_daily_report_lines` の INSERT（列を名指しするので新しい列は NULL になる）。`db/schema_v4.rs` の CREATE 文。

### S3 BIZ（`biz::daily_report_import_service`、`biz::mod`）

- `scan_register_sd_daily_reports`（37 §37.9 の手順 1〜9。手順 10 と snapshot の `plu_register_z004s` は作らない〈D-117、PLU の書出しの見直しの runtime の lane〉）、型 `DailyReportSdSelection`・`DailyReportSdScanResult`・`DailyReportSdScan`・`DailyReportSdCandidate`・`DailyReportSdCandidateStatus`・`DailyReportSdScanSnapshot`（wire の 3 つに `specta::Type`）。
- `DailyReportInputFile` に `sd_relative_path: Option<String>`・`source_path: Option<PathBuf>`（37 §37.2）: 定義 `biz/daily_report_import_service/mod.rs` の `DailyReportInputFile`、本番の構築 `cmd/daily_report_import_cmd.rs` の `parse_and_validate_daily_report`、test の構築 `biz/daily_report_import_service/tests.rs` の helper `source_file`（helper `z001_with_lines`〜`z005` と `with_settlement` はこれを呼ぶ）、`biz/sales_service.rs` の `test_get_monthly_sales_req502_official_quantity_in_units` の中の helper `file` と `test_get_daily_sales_req501_summary_imports_from_fixture_bundles` の構築。
- `CachedDailyReportPreview` に `settlement_no`・`sd_source_files`（37 §37.2）、`SdSourceFileForCopy`: 定義 `biz/daily_report_import_service/mod.rs` の `CachedDailyReportPreview`、構築 `parse.rs` の `parse_and_validate_daily_report`、test の構築 `cmd/daily_report_import_cmd.rs` の helper `cached`（helper `cached_with_status` はそれを呼ぶ）。`sd_source_files` は §37.3 手順 9 の文（2026-10-17）どおり詰める。
- §37.3 手順 1a（`parse_and_validate_daily_report_with_sd_lookup`、root を取る closure）、手順 4 の BIZ-08-D6、手順 8 の BIZ-08-D4。
- §37.4: `commit_daily_report_import` に `app_data_dir: &Path`（定義 `commit.rs` の `commit_daily_report_import`。呼出し: `cmd/daily_report_import_cmd.rs` の `commit_daily_report_import`、`biz/sales_service.rs` の上の 2 つの test、`tests.rs` の 29 か所〈`rg -c 'commit_daily_report_import\(' src-tauri/src/biz/daily_report_import_service/tests.rs` → `29`〉。test は一時 directory を渡す）、手順 2a（D6）・2b（D5）・4a（D4）・6（`settlement_no` と `source_files_json` の `sd_relative_path`・`copy_path`。`commit.rs` の `execute_commit` の `source_files_json` の wire 型の serialize を内部の serialize 用の型に替え、既存の 4 key は変えない）。読む側 `parse.rs` の `source_filenames` は無い field を許す（変えない）。
- `BizError::SourceCopyFailed(std::io::ErrorKind)`: `src-tauri/src/biz/mod.rs` の `BizError` と `impl fmt::Display for BizError`。網羅の match は `Display` と `cmd/mod.rs` の `impl From<BizError> for CmdError` の 2 か所（`rg -ln 'BizError::IdempotencyConflict\(.*\) =>' src-tauri/src` → `src-tauri/src/biz/mod.rs`・`src-tauri/src/cmd/mod.rs`）。
- 既存の BIZ の test の fixture（Plan Review round 1、2026-10-23）: `biz/daily_report_import_service/tests.rs` の合成の束は Z001・Z002・Z005 とも精算回数 `0001`（helper `daily_report_preamble` と `z005_with_department`）で、BIZ-08-D4（§37.3 手順 8・§37.4 手順 4a）は手で選ぶ経路でも `settlement_no` が `Some` なら効く。同じ日に別の bytes の束を取り込む次の 8 本は、そのままだと 2 本目以降が D4 の `ImportError` になる: `test_daily_report_req401_additional_confirmation_for_same_date_different_bundle`・`test_daily_report_req401_commit_same_date_is_insert_only`・`test_daily_report_req401_commit_additional_unconfirmed_validation_failed`・`test_daily_report_req401_stale_additional_preview_same_bundle_conflicts`・`test_daily_report_req401_snapshot_change_requires_repreview_without_side_effects`・`test_daily_report_req401_snapshot_replace_requires_repreview_without_side_effects`・`test_daily_report_req401_rollback_keeps_same_date_sibling`・`test_daily_report_rollback_req401_logs_exact_selected_id_and_survives_log_failure`（`rg -n 'z001_with_lines\(' src-tauri/src/biz/daily_report_import_service/tests.rs` の hit のうち、同じ test で `valid_files()` の束を先に commit するもの。2026-10-23）。各 test の中で bytes の違う束ごとに別の精算回数を与える（1 本目は `0001` のまま、2 本目 `0002`、3 本目 `0003`。同じ bytes の束〈`stale_additional_preview_same_bundle_conflicts` の 2 つの preview〉は同じ番号）。Z001・Z002・Z005 の 3 本とも既存の helper `with_settlement` で替える（1 本だけ替えると `settlement_mismatch`）。assert は変えない（D4 が手順 5 の snapshot の再検査や同日追加の確認より先に止めるのを避けるためで、各 test が確かめる振舞いはそのまま）。`biz/sales_service.rs` の 2 つの test（別の日、または fixture の `260324_0001`・`_0002` で精算回数が別）と CMD の test（直接作った cache で `None`）は当たらない。
- L3 の合成の SD の tree の生成（Plan Review round 1）: scan の test の tree を作る helper を使い、出力先を環境変数 `INVENTORY_L3_SD_TREE_OUT` で受ける `#[ignore = "…"]` の test 1 本（`scan::write_l3_register_sd_tree`。既存の例は `io/ej_parser.rs` の `INVENTORY_EJ_PROBE_DIR` の `#[ignore]` の test）を置く。出力: `CASIO\SR500_550_4000\XZ\yyyy\mm\` に L3-1 の 1 組（3 本そろい、精算回数あり）、L3-3 の Z002 の無い組と Z005 だけ BOM 付きの組。組の日付（folder の年・月、名前の日、中身の対象日）は固定の値にせず、生成した日（PC の今日、`chrono::Local::now().date_naive()`）の前日と前々日にする（Plan Review round 2）。scan の窓は取込みの無い開発の DB では L3 の日 − 30 日から（37 §37.9 手順 2）で、固定の日付の組は L3 の日によって窓の外になり、一覧が 0 件の「新しい精算はありません」になるため。生成から 28 日以内に L3 をする（過ぎたら生成し直す）。実行は `cd src-tauri && INVENTORY_L3_SD_TREE_OUT=<出力先> cargo test --lib -- --ignored write_l3_register_sd_tree`。環境変数が無いときは何も書かずに panic の文で出力先を求める。中身は合成で、名前の形だけを 29 §29.7.2 に合わせる（Matrix の Data Safety Checks）。

### S4 CMD・登録・bindings

- `AppState.register_sd_scan_cache`（45 §45.2）: 定義 `cmd/mod.rs` の `AppState`。literal の構築 15 か所すべてに field を足す（`rg -n 'AppState \{' src-tauri/src` から定義の行と `-> AppState {` の signature の行を除く。2026-10-23 も 15 か所）: 本番 `lib.rs` の `run` の `app.manage(AppState {`、test は `cmd/product_cmd.rs` 2・`cmd/settings_cmd.rs` 1（`app_state_for_test`）・`cmd/plu_export_cmd.rs` 1・`cmd/stocktake_cmd.rs` 1（`app_state_for_test`）・`cmd/csv_import_cmd.rs` 4・`cmd/integrity_cmd.rs` 1（`app_state_for_test`）・`cmd/daily_report_import_cmd.rs` 1・`cmd/sales_cmd.rs` 3（`app_state_for_test` と 2 つの `let state = AppState {`）。
- `scan_register_sd`（§45.6a の手順 1・2・3・4。手順 2a は作らない）と `parse_and_validate_daily_report_from_sd`（§45.6b）、`RegisterSdScanResponse`。`DailyReportSourceFileRequest.source_path`（`#[serde(default)]`、§45.3。定義 `cmd/daily_report_import_cmd.rs` の `DailyReportSourceFileRequest`、test の helper `request`）。commit に `app: tauri::AppHandle` と手順 3a、test 用の `commit_daily_report_import_with_state` に `app_data_dir`。`From<BizError> for CmdError` に `SourceCopyFailed` → `CmdError::internal`（§45.7 の固定の文）。
- 登録: `lib.rs` の `export_bindings_to` の `collect_commands!` と `run` の `tauri::generate_handler!` の両方に 2 command（45 §45.8。`scripts/check-command-drift.sh` の 4 集合の一致）。`cd src-tauri && cargo run --bin generate_bindings` で `src/lib/bindings.ts` を再生成（手で編集しない）。

### S5 frontend（`src/features/daily-report-import/`）

本節の行番号は 2026-10-17 の観測で、2026-10-23 の main でも同じ（`git diff --stat f5d7aaf4 eed07d37 -- src/features/daily-report-import` は `DailyReportImportPage.test.tsx` の comment の 1 行の置き換えだけ）。

- `types.ts`・`reducer.ts`（UI-07-D15 (4)(5)。2026-10-23 の Plan Review round 1 で 55 UI-07-D15 (5) に足した文）: `DailyReportErrorRecoverTo` に `"sd_list"`（今は `types.ts:4` の `"idle" | "preview"`）。状態に `{ status: "scanning" }` と `{ status: "sd_list"; scan: DailyReportSdScan; scanToken: string }`。`parsing`・`preview`・`importing`・`result` に省略可の `sdOrigin?: { scan; scanToken; candidateKey }`（SD から来たときだけ持つ。`candidateKey` は「確認する」を押した行の `candidate_key`。手で選んだ経路は持たない。`result` と `candidateKey` は Plan Review round 2 の Codex 側の是正で足した）。action を足す: scan の開始・成功（scan と scanToken）・失敗（`idle` へ。文は hook の 1 スロット〈UI-07-D15 の「SD から読む」の直下〉が持ち、`error` の状態にしない）、`sd_list` の「確認する」（`parsing` へ `sdOrigin` つき）、一覧へ戻る（`sdOrigin` のある `preview` の「ファイルを選び直す」と、`sdOrigin` のある `result` の「SD の一覧へ戻る」の 2 か所から。`sd_list` へ）。名前は Writer が決める。reducer の遷移を次のとおり変える（写しの失敗の分岐〈hook の `decideRecoverTo` と recoverTo `preview`〉は変えない）:
  - `parse_succeeded`・`confirm_import`・`import_succeeded` は `sdOrigin` を持ち越す。`select_files`（手で選ぶ）は `sdOrigin` を持たない `parsing` を作る。
  - 一覧へ戻る action（UI-07-D15 (4)・(6)）: `preview` からは `sdOrigin` の scan のまま `sd_list` を作る。`result` からは `sdOrigin` の scan の候補のうち `candidate_key` が `candidateKey` の 1 行の `status` を `Imported` に替えた scan で `sd_list` を作る（scanToken は同じ。ほかの行は scan の時のまま。CMD の snapshot は替えない、45 §45.6b 手順 4 の「snapshot は消さない」）。`sdOrigin` の無い状態では何もしない。`rollback_succeeded` は今どおり `idle`（取り消した行を一覧に戻す扱いは持たない。読み直せば scan が「取り込めます」と出す）。
  - `parse_failed`: `parsing` が `sdOrigin` を持てば recoverTo `sd_list`・previousState は `sdOrigin` から作った `sd_list`（error の kind・message に依らない）、持たなければ今どおり recoverTo `idle`。recoverTo を hook でなく reducer が状態から決める（action の形は変えない）。
  - `import_failed`（今の `reducer.ts` の `case "importing"`）: recoverTo `preview` で preview を作り直すとき `sdOrigin` を引き継ぐ。recoverTo `idle`（`import_error`、preview の期限切れ等）は今どおり。
  - `dismiss_error`（今の `case "error"`。recoverTo `preview` のときだけ previousState へ戻し、ほかは `idle`）: recoverTo `sd_list` で previousState が `sd_list` なら previousState を返す分岐を足す。
  - 作る所・読む所（`rg -n 'state\.status|status: "|dailyReportImportReducer\(|DailyReportImportState|DailyReportErrorRecoverTo' src/features/daily-report-import`、2026-10-23）: 作る所は `reducer.ts` の全分岐と hook の `INITIAL_STATE`（`:14`）。読む所は hook の `useBlocker`（`:77`・`:78`、`importing` だけ block。`scanning` は block しない）・`confirmImport`（`:222`）・`rollback`（`:235`）・`decideRecoverTo`（`:35`、型だけ）、Page の `switch (state.status)`（`DailyReportImportPage.tsx:29`、`scanning`・`sd_list` の分岐を足す。`result` の分岐〈`:52`〉は `sdOrigin` があれば `DailyReportResultStep` に一覧へ戻る操作を渡す）と `:34`。test の fixture は `reducer.test.ts`（`:13`・`:35`・`:58`・`:70`・`:78` の state の literal）と `DailyReportImportPage.test.tsx` の `setFlow`（`:95`）に渡す state の literal（`:121`・`:134`・`:155`・`:232`・`:257`・`:276`・`:294`・`:347`・`:368`・`:390`・`:433`・`:468`）。test の呼び方は `reducer.test.ts` の `dailyReportImportReducer(` の 6 か所（`:20`・`:43`・`:48`・`:64`・`:72`・`:85`。`:48` が `dismiss_error`）で、hook の test（`useDailyReportImportFlow.test.tsx`）は hook の API を通すので reducer の action を直接呼ばない。足すものは省略可の field と新しい状態・action なので、既存の literal と呼び方は書き換えない（型の網羅で Page の `switch` に分岐が要る）。
- `hooks/useDailyReportImportFlow.ts`: `scanRegisterSd`・`parseAndValidateDailyReportFromSd` の mutation。SD から来た parse の失敗の recoverTo `sd_list` は上の reducer が決める（hook は kind・message で分岐しない）。`decideRecoverTo`（`:35`）は変えない（写しの失敗は `internal` で preview に戻る）。予備の経路は UI-07-D16 の選択の一覧（55 の (a)〜(f)、2026-10-17）で、dialog の path を各要素の `source_path` に入れる（今は `:207` で `extractFilename` だけを送る）。`selectFiles`（`:173`）は外す。「場所を選ぶ」は `@tauri-apps/plugin-dialog` の `open({ directory: true })`（capability は既存の `dialog:allow-open`、変えない）。
- `DailyReportImportPage.tsx`: 「SD から読む」・「場所を選ぶ」・一覧（状態の label と icon、上の文の (a)〜(c)、読まなかった file の件数）・SD を戻す案内・「ファイルを選んで取り込む」の補助のリンクと選択の一覧・置き換えの確認の dialog（`src/components/ui/alert-dialog.tsx`）。`DailyReportResultStep`（今は「日次売上を見る」と「取り消す」）に、SD から来た結果だけ「SD の一覧へ戻る」の button を足し、上の一覧へ戻る action を送る（55 UI-07-D15 (6)。手で選んだ結果には出さない）。preview の「ファイルを選び直す」（今は `onChooseFiles`）は、`sdOrigin` があれば一覧へ戻す action（UI-07-D15 (4)）。`sdOrigin` の無い preview（手で選んだ経路）では file の dialog を開き、1 つ以上選べば `reset` で `idle` に戻して、選んだ file を空の新しい一覧に UI-07-D16 (b)〜(d)・(f) の規則で足す（キャンセルなら preview のまま。55 UI-07-D16 (a) に 2026-10-23 の Plan Review round 2 で足した文）。前の preview の 3 つは一覧に戻さない（UI-07-D16 (a) の一覧は `idle` の間だけ hook が持つ）。reducer の `preview` の `select_files` の分岐（今は preview から直接 `parsing` へ進む）は外し、`select_files` は `idle` から（D16 (e) の「確認する」）だけにする。
- 既存 test の書換え（assert は弱めない）: `hooks/useDailyReportImportFlow.test.tsx` の `selectFiles` を使う 6 か所（`:212`・`:412`・`:459`・`:491`・`:558`・`:615`）を `chooseFiles` と dialog の mock に、3 つ一度の件数の検査の test（`:232`〜`:387` の `chooseFiles` の test）を 1 つずつの選択に、期待の payload（`:395`〜`:397`）に `source_path` を足す。`DailyReportImportPage.flow.test.tsx` の 1 本（`REQ-401: shows inline selection error for 2 files and clears it after 3 files parse`。2 本の選択の件数の error と「日報ファイルを選択」の button を前提にする）は、D16 の 1 つずつの選択に書き換える（Plan Review round 2）: 1 回目の dialog で Z001・Z002 の 2 本を選ぶと一覧に 2 行が足され、「確認する」が無効で `parseAndValidateDailyReport` が呼ばれない。2 回目の dialog で Z005 を選ぶと「確認する」が有効になり、押すと 3 要素とも `source_path` 付きで呼ばれて preview（「取込み内容」）が出る。件数の error の文と toast の assert（D16 で文が無くなる）は「2 本の時点で『確認する』が無効で parse が呼ばれない」の assert に移す（`readFile` は D16 (f) の検査で選んだ時に呼ばれるので、`readFile` が呼ばれない assert も parse が呼ばれない assert に移す）。確かめる振舞い（3 本そろうまで parse しない、そろえば 3 本の path で parse する）は弱めない。`DailyReportImportPage.test.tsx` の hook の mock の `selectFiles`（`:15`・`:98`・`:247`）を外す。`reducer.test.ts` に遷移を足す（Matrix の UI-07-D15 の reducer の行。SD の preview → 写しの失敗 → 戻る → 一覧へ戻る、SD から来た parse の失敗 → 戻る → 一覧、手で選んだ parse の失敗は今どおり `idle`、SD から来た取込みの成功 → 一覧へ戻る〈取り込んだ行だけ `Imported`〉）。新設: `DailyReportImportPage.sd.test.tsx`・`DailyReportImportPage.files.test.tsx`（Matrix）。
- 文言の衝突: 新しい label を `rg -F '<語>' src --glob '*.test.*'` で当てた（2026-10-17）。「取込み済み」は `DailyReportImportPage.test.tsx:413` の preview の badge の matcher と同じ語だが、一覧（`sd_list`）と preview は同時に描かれない状態なので衝突しない（SD の一覧の test は `within` で行を絞る）。「確認する」「外す」は他の画面の test にだけある。「SD の一覧へ戻る」は 0 件で、近い「一覧へ戻る」は商品の画面の test（`ProductFormPage.test.tsx`、`getByRole` の name の完全一致）にだけある（2026-10-23、Plan Review round 2 の Codex 側の是正）。ほかの新しい語（「SD から読む」「場所を選ぶ」「取り込めます」「同じ精算を取込み済み」「ファイルがそろっていません」「読めません」「新しい精算はありません」「読み終わりました」「置き換える」「読まなかったファイル」「ファイルを選んで取り込む」）は 0 件。

### S6 設計正本の「未実装」の注記を現行の記述へ直す（実装と同じ PR）

- 29 §29.7.1 の「`io::register_sd`、未実装」と登録の義務の文・§29.8.1 の「`io::pos_source_copy`、未実装」と登録の文、45 §45.3 の「今の構築…に field を足す」と §45.4 の「今の実装（…）は `State<AppState>` だけを受ける」、55 §55.0 の reducer の状態の文の「今の `DailyReportErrorRecoverTo` は…」と UI-07-D16 の「今の `useDailyReportImportFlow.ts` は path を…捨てて…」・「今の日報取込みタブは dialog だけを使う〈…`onChooseFiles`〉」・「今の hook の `selectFiles`…は外す」（後の 2 文は Plan Review round 3 で足した。実装の後の形〈dialog だけを使う、`selectFiles` は無い〉に直す）、§55.1 の `DailyReportImportPage.tsx`（「UI-07-D15、未実装」）・`types.ts`・`useDailyReportImportFlow.ts`（「D-111 の runtime で」）の行、37 §37.4 手順 6 の「今の … `execute_commit` の `source_files_json` は…」、45 §45.8 の「D-111 の runtime では」、pos-tables §12b の `settlement_no` の行の「〈SD 直読みの runtime の lane〉」、22 §17 の版の番号（merge の時点の番号に）。
- 設計正本の外の「runtime は未実装」の注記（Plan Review round 1 で足した。2026-10-23 の `rg -n 'Draft PR #172|runtime は未実装' docs …`〈AC9 の 1 つ目〉の 14 行）: `docs/ARCHITECTURE.md` の POS Adapter Boundary の「Register SD source」の行・IO-09 と IO-10 の部品の行・データの流れの IO-09 と IO-10 の 2 行、`docs/architecture/io-task-specs.md` の IO-09 と IO-10 の「理由」、`docs/architecture/biz-task-specs.md` の BIZ-08 の入力の行、`docs/SCREEN_DESIGN.md` の 1 日の流れの売上データ取込みと日報取込みの行、`docs/DB_DESIGN.md` の `daily_report_imports` の行、`docs/project-memory.md` の D-111 の 3 行（CV17 の layout の観測の行・SD card → CV17 の行・毎日の売上データの行）。「今の build は利用者が選ぶ file から」を外し、SD から読むのを今の形として書く（予備の手で選ぶ経路は残す）。
- 受入台本（Plan Review round 3）: `docs/ACCEPTANCE_SCENARIO.md` の Step 1 の手順 2（「日報ファイルを選択」で 3 file を同時に選ぶ）を、UI-07-D16 の「ファイルを選んで取り込む」で同じ 3 file を選び（1 回の dialog で 3 つ選んでよい、D16 (a)）、一覧に 3 行がそろったら「確認する」を押す、に書き換える（S5 で button が無くなり、台本の規則〈FAIL なら後続の step へ進まない〉で止まるため）。手順 3 以降と期待値は変えない（fixture は PC の上の file で、写しも精算回数の検査も効かない）。
- 対象外: `docs/backlog.md`・`docs/Plans.md` の「Draft の PR #172」の文（Post-Merge Closeout が本 lane の項目を移すときに直す。Plans は本 lane の Non-scope、D-097）、`docs/decision-log.md`（append-only の本文）、D-117・D-114・D-121 の lane の「今の build は」「runtime の lane で」の文（それぞれの lane が持つ）、`docs/project-memory.md` の D-117 の行。
- 37 §37.9 手順 10・45 §45.6a 手順 2a の「SD 直読みの runtime の lane は…」の文は残す（PLU の runtime の lane が実装の後に外す）。
- `docs/spec/requirements.md` の REQ-401 の部品の列（今は `UI-07, IO-07, BIZ-08, CMD-12, BIZ-03`）に `IO-09`・`IO-10` を足すなら `cargo run --bin generate_traceability` で `docs/function-design/90-traceability.md` を再生成する（足すかは Writer。足さなくても test の REQ-401 は今どおり当たる）。

### 本 commit（plan-first）で変えた正本

| file | 変更 |
|---|---|
| `docs/function-design/37-biz-daily-report-import-service.md` | §37.3 手順 9 に `sd_source_files` を詰める手順（P3 (1)）、BIZ-08-D5 の保証の範囲に TX の中で拒んだ束の写し（P3 (2)）、§37.4 手順 2b の行番号の参照を外した、§37.9 手順 10 の実装の lane、更新履歴 |
| `docs/function-design/29-io-daily-report-parser.md` | §29.7.3 の `RegisterSdError::Io` の `relative_path` の意味（P3 (3)）、IO-07-D5 の `i64` の範囲と `0000` |
| `docs/function-design/45-cmd-daily-report-import.md` | §45.6a 手順 2a の実装の lane、更新履歴 |
| `docs/function-design/55-ui-csv-import.md` | UI-07-D16 の選択の一覧の細部と文言、`selectFiles` を外す、更新履歴 |
| `docs/function-design/22-mnt-migration.md` | §17（MNT-03-D15、`settlement_no` の migration、番号の決め方、DB の作り直しとの関係） |
| `docs/db-design/pos-tables.md` | §12b の `settlement_no` の行から 22 §17 へ、更新履歴 |
| `docs/backlog.md` | 「SD 直読みの runtime」の P3 4 件を直した旨と、P1・P4 の採取の見込み |

### 2026-10-23 の当て直し（main `eed07d37` への同期）で変えた正本

main に入った D-114（#155、PC の外の控え）・D-121・D-123・D-125 が本 lane の節の前提を変えた所だけを直した。

| file | 変更 | 理由 |
|---|---|---|
| `docs/function-design/22-mnt-migration.md` | §17 の版の番号の文（main の最大 v10、先に merge する migration が無ければ v11、#188 b1 の vZ と先に merge した側が若い番号）、DB の作り直しとの関係の文（§16 の「DB の作り直しと順」、vB・v9・vZ の止まる検査は `daily_report_imports` を見ない）、登録の順（merge の時点の最後の版の後） | v9（§19）・v10（§16 の vB）が main に入り、D-123 が「作り直しを 1 回にまとめる順」を置き換えた |
| `docs/function-design/29-io-daily-report-parser.md` | §29.7.4 手順 1 に、drive を見る間の thread の error mode（`SEM_FAILCRITICALERRORS`）と、folder 名の定数を 71 MNT-01-D10 と共有すること。IO-09-D3 の許可の列に `SetThreadErrorMode`・`SEM_FAILCRITICALERRORS` | D-114 の設計（71 §71.11.3）が、空の読取り機の口を見ると Windows が error の dialog を出しうる前提を見つけて設計で外した。SD をレジに差したままの日は PC の読取り機の口が空なので、IO-09-D1 の drive の列挙にも同じ前提が当たる。71 MNT-01-D10 は folder 名の定数を IO-09 と共有すると決めた |
| `docs/function-design/37-biz-daily-report-import-service.md` | §37.8 の写しの行を、D-114 で決まった D-111 の B（backup と PC の外の控えに写しを入れない）に合わせた | D-111 の Superseded in part（D-114） |
| `docs/backlog.md` | 「SD 直読みの runtime」の D-111 の B（D-114 で決まった）と、Contract Probe の P1 (a)・P4 の結果 | 同上と下の Contract Probe |

直さないと判断したもの: D-117（PLU の占有の照合。本 lane の Non-scope のまま）、D-118（Z004 の精算回数の新契約。日報の側は変えないと D-118 の Compatibility に書いてある）、D-119・D-128（原価と単位。日報の束と `daily_report_imports` に触れない）、D-121 の BIZ-08-D8（同日追加の確認を精算回数で外すのは runtime (A) で、本 lane の Ordinary Operation の「同日追加の確認を含む」は (A) の merge まで正しい）、#155 の 43・53（PC の外の控えの command と画面。IO-09 の契約を引かない）。

P3 (4) は archive の packet の記述の誤りで、IO-10 の task spec は `docs/architecture/io-task-specs.md` の IO-10 に既にあった。本 packet の Contract Ledger は IO-07-D5 の位置を 29 §29.2 の末尾と書く。

### Plan Review round 1 の是正（2026-10-23）で変えた正本

round 1 の是正（Review Response 参照）。設計の向き（D-111）は変えない。

| file | 変更 | 指摘 |
|---|---|---|
| `docs/function-design/55-ui-csv-import.md` | UI-07-D15 (5) に、SD から来たこと（scan と scan_token）を `parsing`・`preview`・`importing` が持ち続けること、`import_failed` が preview を作り直すときに引き継ぐこと、`parse_failed` の recoverTo `sd_list` を reducer が状態で決めること、「戻る」（`dismiss_error`）が recoverTo `sd_list` で一覧へ戻ること。更新履歴 | #1 |
| `docs/function-design/29-io-daily-report-parser.md` | §29.7.4 手順 1 の folder 名の定数の置き場（`crate::io` の `pub(crate) const`、merge の順に依らない）と `DRIVE_REMOVABLE` の feature（`Win32_System_WindowsProgramming`）。手順 4 の path の判定を文字列の規則にした（`\` と `/` の両方で区切る、絶対 path の形）。IO-09-D3 の許可の列に型 `THREAD_ERROR_MODE`、source の検査の red に `as` の改名のすべての形と書く module への参照 | #5・#6・#8・#11 |
| `docs/backlog.md` | 「日報取込み標準手順の残設計」の取消の導線に、BIZ-08-D4 の文がその導線を前提にすること（AI review 由来） | #9 |

### Plan Review round 2 の是正（2026-10-23）で変えた正本

round 2 の是正（Review Response 参照）。設計の向き（D-111）は変えない。

| file | 変更 | 指摘 |
|---|---|---|
| `docs/function-design/22-mnt-migration.md` | §17 の取り直しの範囲: 後から merge する側が直すのは file 名・`mod` の行・登録だけにし、最新の版を期待する test は `app_max_version()`、vB が対象の test は `migrate_up_to(&conn, VB_VERSION)` の形にして版が増えても直さない（直す一覧は本 packet の S2）。先に merge した lane が版の定数で最新を期待する test を残していれば、後の lane が `app_max_version()` に替える | #1 |
| `docs/function-design/29-io-daily-report-parser.md` | §29.7.4 手順 1 の thread の error mode を、`find_register_sd_roots` の中だけから、file system を見る 5 関数のどれもが最初に作る 1 つの guard に広げた（SD を抜いた後の path を見る `check_selected_file_present`・`list_register_sd_entries`・`read_register_sd_file` も空の口を見る）。IO-09-D3 の source の検査が red にする参照を、書く module の名前の列から「`crate::`・`super::` の参照は `super::` の folder 名の定数だけ」に替えた | #6・#7 |
| `docs/function-design/55-ui-csv-import.md` | UI-07-D16 (a) に、手で選んだ経路の preview の「ファイルを選び直す」の行き先（dialog を開き、選べば `idle` の空の新しい一覧に足す。キャンセルなら preview のまま）と、reducer の `preview` の `select_files` の分岐を外すこと。更新履歴 | #3 |
| `docs/function-design/55-ui-csv-import.md` | UI-07-D15 (6) を足した: SD から来た取込みの結果に「SD の一覧へ戻る」を出し、同じ scan と scan_token の一覧へ戻す。取り込んだ行だけを「取込み済み」に替えて出す。(5) の SD から来たことを持つ状態に `result` を足した。更新履歴 | Codex 側の #2 |

### Plan Review round 3 の是正（2026-10-23）で変えた正本

round 3 の是正（Review Response 参照）。設計の向き（D-111）は変えない。L3-2・L3-3 の到達経路（#1）、S6 と AC9 の受入台本と UI-07-D16 の「今の」の文（#2）、S1 と Registration Obligations の `scan.rs` の登録（#4）は本 packet の中だけで直した。

| file | 変更 | 指摘 |
|---|---|---|
| `docs/function-design/29-io-daily-report-parser.md` | §29.7.4 手順 1 の folder 名の定数の形を `pub(crate) const REGISTER_SD_FOLDER: [&str; 2] = ["CASIO", "SR500_550_4000"];` に決めた（`\` を含む 1 つの文字列にしない。path は要素ごとに `join` し、手順 3 の判定と test の tree も要素で行う） | #3 |

### Plan Review の closure の是正（2026-10-23）で変えた正本

closure の Codex 側の是正（Review Response 参照）。設計の向き（D-111）は変えない。Matrix に `register_sd::list_and_read_reject_links_outside_root` を足し、`scan::root_and_read_failures_use_fixed_texts` に `InvalidPath` の文を足した。

| file | 変更 | 指摘 |
|---|---|---|
| `docs/function-design/29-io-daily-report-parser.md` | IO-09-D5 を足した: 列挙・読取りは root と見る path を `std::fs::canonicalize` で実体の path に解いてから root の下かを要素ごとに比べ、外なら `InvalidPath` で入らず・開かない。確かめた解いた path をそのまま開く。root そのものが link ならその先を root にし、root の下を指す link は通し、確かめてから開くまでの差し替えは想定しない（理由を書いた）。§29.7.3 の `InvalidPath` の注記、§29.7.5 手順 6、§29.7.6 手順 1・2、IO-09-D3 の許可の列（`canonicalize`）、§29.7.7 の `InvalidPath` の行（BIZ の扱い）、§29.7.8 の採らなかった案 3 行 | closure の Codex 側の #1 |
| `docs/function-design/37-biz-daily-report-import-service.md` | §37.9 手順 3 に `InvalidPath` の固定の文（`Io` の「差し直して」の文と分けた）。更新履歴 | 同上 |

### Gated Amendment 1（2026-10-25、#188 b1 の merge の後の版の取り直し）で変えた正本

| file | 変更 | 理由 |
|---|---|---|
| `docs/inventory_system_erd.html` | 対象の版を「v1〜v12」に、`daily_report_imports` に `settlement_no`（NULL を許す）を足す | 本 lane の vS（v12）が列を足すのに、図が v11 までで列が無かった（Scope に無かった file。本 amendment で足す） |

### 節の所有表（並走の lane と重なる file）

2026-10-23 に当て直し、同じ日の Plan Review round 1〜3 の是正で数え直した。Scope の予定 path は 59 本（新設を含む。round 1 の前の 51 本に、S6 に足した設計正本の外の 6 本〈`docs/ARCHITECTURE.md`・`docs/architecture/io-task-specs.md`・`docs/architecture/biz-task-specs.md`・`docs/SCREEN_DESIGN.md`・`docs/DB_DESIGN.md`・`docs/project-memory.md`〉を足し、round 2 で版の期待の test の `src-tauri/src/db/schema_v10.rs` を、round 3 で S6 の受入台本 `docs/ACCEPTANCE_SCENARIO.md` を足した。内訳: 冒頭の型の `rg -l` の 19〈`schema_v4.rs` を除く〉、登録 3、新設 4〈`scan.rs` を含む〉、版の期待の test の 3〈`schema_v9.rs`・`schema_v2.rs`・`schema_v10.rs`。`migration.rs`・`lib.rs` は 19 に入る〉、`Cargo.toml` 1、frontend 8 と新設の test 2、`bindings.ts` 1、設計正本と backlog 7、`requirements.md`・`90-traceability.md` 2、packet と Matrix 2、S6 の 6、受入台本 1。この列〈冒頭の `rg -l` の出力から `schema_v4.rs` を除いたものと、`find src/features/daily-report-import -type f` の出力と、ほかの path の名前の列〉を `sort -u` した file の `wc -l` → `59`、round 3 の是正の head で実行。closure の是正で触る 29・37・packet・Matrix はすでに列にあり、数は変わらない）。並走の branch の差分（`git diff --name-only --no-renames origin/main...origin/<branch>`）と `comm -12` で突き合わせた結果（2026-10-23）は、`origin/agent/stocktake-count-runtime-b`（#188 b1、head `40ee8de4`、plan-gate で docs だけ）と `docs/db-design/pos-tables.md`・`docs/function-design/22-mnt-migration.md` の 2 本、`origin/agent/backup-offsite-runtime`（PC の外の控えの runtime (a)、PR #190、head `a899a1f1`、plan-gate で docs だけ）と `docs/backlog.md` の 1 本（項目が別）。round 3 で足した `docs/ACCEPTANCE_SCENARIO.md` は、どちらの branch の差分（#190 は head `ee0e45bd`）にも #188 b1 の packet にも無い。code と S6 の 6 本の重なりは #188 の packet の予定 path（S6 の 6 本はどれも #188 の予定 path にある）と、#190 の packet の所有表（`io/mod.rs` の folder 名の定数、`windows-sys` の feature、command の登録）から見込んだ（下の表）。

| file | 本 lane が所有する節 | 他の lane |
|---|---|---|
| `docs/function-design/37-biz-daily-report-import-service.md` | §37.2 の `DailyReportInputFile`・`CachedDailyReportPreview`・`SdSourceFileForCopy`、§37.3 手順 1a・4（BIZ-08-D6）・8（BIZ-08-D4）・9、§37.4 手順 2a・2b・4a・6、§37.7 の D-111 の行、§37.8 の D-111 の行、§37.9 の手順 1〜10 と BIZ-08-D3〜D6・preview への接続 | §37.10・§37.11（D-121、main の設計。runtime (A) は本 lane の merge の後に §37.9 の snapshot と応答に field を足す）と §37.9 の「Z004 と EJ への引継ぎ」の段落は触らない。§37.3 手順 6・7 の部門・総売・支払の意味は D-120 の設計（main） |
| `docs/function-design/29-io-daily-report-parser.md` | §29.2 の `settlement_no` と IO-07-D5、§29.7（IO-09）、§29.8（IO-10） | §29.7.4 手順 3 の SD の写しの置き場の文は D-114（main、71 MNT-01-D10）が足した文のまま。PC の外の控えの runtime と、folder 名の定数（置き場は `src-tauri/src/io/mod.rs`、§29.7.4 手順 1）と feature `Win32_System_Diagnostics_Debug` を共有する（feature は先に merge した lane が足す） |
| `docs/function-design/45-cmd-daily-report-import.md` | §45.2・§45.3 の `source_path`・§45.4・§45.6a・§45.6b・§45.7 の `SourceCopyFailed`・§45.8 | §45.6c（CMD-12-D2、D-121 の一括取込み）は runtime (A) |
| `docs/function-design/55-ui-csv-import.md` | §55.0 の日報の行と「日報取込みの利用者フロー」、UI-07-D12・D15・D16、§55.1 の日報の 3 行、更新履歴の本 lane の行 | UI-07-D17（D-121）は runtime (A) |
| `docs/db-design/pos-tables.md` | §12b の `settlement_no` の行と冪等性の行、更新履歴の本 lane の行 | #188 は冒頭の「計数の置き場と在庫の求め直しの契約」（Z004 の取込みの精算回数・機械の番号の列と在庫の movement の鍵。Z004 の側）。§12c〜§12e は D-120、「SD の一括取込みと EJ の控えの契約」は D-121 |
| `docs/function-design/22-mnt-migration.md` | §17 | #188 は §20（vZ）を足す。番号は先に merge した側が若い番号を取る（§17・§20 の両方に同じ文） |
| `docs/backlog.md` | 「SD 直読みの runtime」の項目と、「日報取込み標準手順の残設計」の取消の導線の文（round 1 の #9） | 各 lane が自分の項目だけ。#190 は PC の外の控えの項目 |
| `docs/ARCHITECTURE.md`・`docs/architecture/io-task-specs.md`・`docs/architecture/biz-task-specs.md`・`docs/SCREEN_DESIGN.md`・`docs/DB_DESIGN.md`・`docs/project-memory.md`（S6） | D-111 の「runtime は未実装〈Draft PR #172〉」「今の build は利用者が選ぶ file から」の注記の文だけ（S6 の 14 行） | #188 も予定 path に持つ（棚卸しの数の置き場の節）。文が別なので merge の順で両方を残す |
| `src/lib/bindings.ts`（生成物） | 本 lane の 2 command と DTO | #188・PC の外の控えの runtime も再生成する。衝突は手で解かず、取り込んだ後に再生成する（D-098） |
| `src-tauri/src/lib.rs` の `collect_commands!`・`generate_handler!` | 本 lane の 2 command | #188（command の変更）・PC の外の控えの runtime（6 command）。merge の順で両方を残す |
| `src-tauri/src/db/migration.rs`・`db/mod.rs`・版の期待の test（`db/schema_v2.rs`・`db/schema_v9.rs`・`db/schema_v10.rs`・`lib.rs` の `test_startup_req903_` の 2 本） | vS の登録と、版の期待を `app_max_version()`・`migrate_up_to(&conn, VB_VERSION)` の形にすること（S2） | #188 の vZ（b1 の Matrix の T1 は同じ test の一部を vZ の版の定数へ替える計画）。後から merge する側が file 名・登録を取り直す。b1 が先なら、本 lane が vZ の版の定数で最新の版を期待する test を `app_max_version()` に替える（22 §17、S2 の取り直し） |
| `src-tauri/Cargo.toml` の `windows-sys` の feature | `Win32_System_Diagnostics_Debug`・`Win32_System_WindowsProgramming`（`DRIVE_REMOVABLE`） | PC の外の控えの runtime も `Win32_System_Diagnostics_Debug` を足す（先に merge した lane が足す） |
| `src-tauri/src/io/mod.rs` | `pub mod register_sd`・`pub mod pos_source_copy` と、レジの SD の folder 名の `pub(crate) const`（本 lane が先に merge するとき） | #190 の packet の所有表の `io/mod.rs` の行と同じ置き場（#190 が先なら #190 が置き、本 lane は使う） |
| `src-tauri/src/cmd/mod.rs` ほか `AppState` の 15 か所、`biz/mod.rs`・`db/sales_repo.rs`・`cmd/csv_import_cmd.rs`・`cmd/stocktake_cmd.rs` | `register_sd_scan_cache` の field、`BizError::SourceCopyFailed`、`settlement_no` の INSERT と照合の関数 | #188 の packet もこれらの file を触る予定。違う symbol なので merge の順で両方を残す |

## Non-scope

- 37 §37.9 手順 10・`plu_register_z004s`・45 §45.6a 手順 2a・BIZ-04 `refresh_plu_register_snapshot_from_sd`（D-117、PLU の書出しの見直しの runtime）。
- Z004 の SD からの取込み（BIZ-03、D-118 の 32 の新契約）、EJ の取込み・閲覧、一括取込み（D-121）。
- `XZ_BKUP` へ移す操作（(b) の lane）。`io::register_sd` に書込みの API を足さない。
- 写しの保持・削除（未決 A）、`docs/function-design/71-mnt-backup.md` の変更（backup は DB の 1 file のまま。写しを入れないのは D-114 の未決 B）。
- `docs/Plans.md`（D-097）、`docs/decision-log.md`（新しい決定は無い）。

## Acceptance Criteria

baseline は 2026-10-23 の head（main `eed07d37` を取り込んだ merge と当て直しの docs）で、各 command を逐語で実行し直した（2026-10-17 の HEAD `f5d7aaf4` の値から変わったのは AC1 の filtered の数と AC9 の位置だけ）。AC9 は同じ日の Plan Review round 1 の是正で範囲を広げ、baseline を取り直した。round 3 の是正で AC9 に (3) を足し、その baseline は round 3 の是正の head で逐語で実行した。

- AC1（IO の test）: `cd src-tauri && cargo test --lib -- io::register_sd io::pos_source_copy` が exit 0 で、Matrix の `register_sd::*`・`pos_source_copy::*` の行の test がすべて PASS。baseline: `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 1315 filtered out; finished in 0.00s`（module が無い）。
- AC2（SD に書かない）: Matrix の `register_sd::module_uses_only_allowed_fs_api` と `register_sd::scan_leaves_tree_unchanged` が PASS し、Matrix に挙げた mutant（`File::create`・`fs::copy`・`OpenOptions::new()`・`use std::io::Write;`・`set_modified` のどれかを module に足す）で red になることを Final Review で実注入して確かめる。
- AC3（BIZ・DB・CMD の契約）: `cd src-tauri && cargo test` が exit 0 で、Matrix の `daily_report_parser::settlement_no_*`・`scan::*`・`daily_report_import_service::*`（D4・D5・D6・手順 1a・手順 9）・`sales_repo::*settlement*`・`migration::adds_daily_report_settlement_no_nullable`・`daily_report_import_cmd::*` の行の test が PASS。
- AC4（層）: `cd src-tauri && cargo test --test architecture_test` が exit 0（baseline: `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s`）。`cmd` が `io::register_sd` を use しない。
- AC5（command と bindings）: `rg -n 'scanRegisterSd|parseAndValidateDailyReportFromSd' src/lib/bindings.ts` が 2 つの command を返す（baseline: hit 0、exit 1）。`cd src-tauri && cargo run --bin generate_bindings` の後に `git diff --exit-code -- src/lib/bindings.ts` が exit 0（生成物と一致）。
- AC6（登録と設計照合）: `rg -n 'io::register_sd|io::pos_source_copy' src-tauri/tests/design_compliance_test.rs` が 2 つの module を返す（baseline: hit 0、exit 1）。`cd src-tauri && cargo test --test design_compliance_test` が exit 0（baseline: `test design_code_compliance_phase_a ... ok`）。`cd src-tauri && cargo run --bin generate_traceability -- --check` が `ERROR 0 件`（baseline: `traceability check: OK（ERROR 0 件 / WARN 0 件）`）。
- AC7（frontend）: `npx vitest run src/features/daily-report-import` が全 PASS で、`DailyReportImportPage.sd.test.tsx`・`DailyReportImportPage.files.test.tsx` を含む（baseline: `Test Files  4 passed (4)`、`Tests  34 passed (34)`）。`npm run typecheck && npm run lint && npm run format:check && npm run build` が exit 0。
- AC8（docs）: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan` が ERROR 0（command registry の drift を含む。WARN は報告）。
- AC9（未実装の注記の sweep、S6）: 次の 3 つの command が hit 0（exit 1）。(1) `rg -n 'Draft PR #172|runtime は未実装' docs -g '!docs/plans/**' -g '!docs/archive/**' -g '!docs/Plans.md' -g '!docs/backlog.md' -g '!docs/decision-log.md'`（baseline: 14 件 = `docs/ARCHITECTURE.md` 5・`docs/architecture/io-task-specs.md` 2・`docs/architecture/biz-task-specs.md` 1・`docs/SCREEN_DESIGN.md` 2・`docs/DB_DESIGN.md` 1・`docs/project-memory.md` 3）。(2) `rg -n 'io::register_sd`、未実装|io::pos_source_copy`、未実装|UI-07-D15、未実装|D-111 の runtime で|runtime の lane で' docs/function-design/29-io-daily-report-parser.md docs/function-design/37-biz-daily-report-import-service.md docs/function-design/45-cmd-daily-report-import.md docs/function-design/55-ui-csv-import.md`（baseline: 10 件 = 29 §29.7.1・§29.8.1、37 §37.4 手順 6、45 §45.4・§45.8、55 §55.0 の利用者フロー手順 2・UI-07-D16・§55.1 の Page・`types.ts`・hook の行。37 §37.9 手順 10・45 §45.6a 手順 2a の「SD 直読みの runtime の lane は」は当たらず、残す）。(3) `rg -n '日報ファイルを選択' docs -g '!docs/plans/**' -g '!docs/archive/**'`（Plan Review round 3。baseline: 1 件 = `docs/ACCEPTANCE_SCENARIO.md:60` の Step 1 の手順 2。55 UI-07-D16 (f) の「日報ファイルの選択または読み取りに失敗しました」は当たらない）。
- AC10（L3 の前）: Human Gate の L3 の前に Writer が `cd src-tauri && cargo check --release` を通し（Linux の WSL では `cfg(windows)` の本体を compile しないので、FFI の型と feature はこれで確かめられない。下の Test Plan と Residual Risk）、Contract Probe の P1・P4 の結果を本 packet に書く（P1 が不一致なら L3 に進まず design へ戻す）。P1 の (a) と P4 は 2026-10-23 に書いた（どちらも合格）。P1 の (b) も L3 の前に書き、合格でなければ L3 を頼まない（下の Contract Probe。D-111 の Guarantee range と 29 §29.7.2 の SD-23 の「L3 の前の前後比較」）。
- AC11（hosted）: Ready の後の run の `Merge gate` が success（`docs/ci.md`）。

## Design Readiness

- 引用する設計正本（節まで）: 29 §29.2（IO-07-D5、`DailyReportParseResult`）・§29.7.1〜§29.7.8（IO-09-D1〜D4）・§29.8（IO-10）／37 §37.1・§37.2・§37.3 手順 1a・4・8・9・§37.4 手順 2a・2b・4a・6・§37.7・§37.9（BIZ-08-D3〜D6）／45 §45.2・§45.3・§45.4 手順 3a・7・§45.6a・§45.6b（CMD-12-D1）・§45.7・§45.8／55 §55.0 の日報取込みの利用者フロー・UI-07-D12・D15・D16・§55.1／24 §14.14・§14.18a／pos-tables §12b／22 §17（MNT-03-D15）・§16 の「DB の作り直しと順」と「作り直しの手順」／71 MNT-01-D10（folder 名の定数の共有）・§71.11.3 の「error の dialog を出さない」／`docs/architecture/io-task-specs.md` IO-09・IO-10／`docs/decision-log.md` D-111・D-114。
- 必要な設計成果物: function-design（IO-09・IO-10・BIZ-08・CMD-12・UI-07） = existing sufficient、ただし P3 の 3 件・IO-07-D5 の範囲・UI-07-D16 の細部・手順 10 と 2a の実装の lane、Plan Review round 1 の UI-07-D15 (5) の状態の持ち越しと 29 §29.7.4 手順 1・4・IO-09-D3、round 2 の 29 §29.7.4 手順 1・IO-09-D3・UI-07-D16 (a)・UI-07-D15 (6)、round 3 の 29 §29.7.4 手順 1（folder 名の定数の形）、closure の 29 IO-09-D5（と §29.7.3・§29.7.5 手順 6・§29.7.6 手順 1・2・IO-09-D3 の許可の列・§29.7.7・§29.7.8）・37 §37.9 手順 3 は updated in this PR／DB（migration） = updated in this PR（22 §17）／SCREEN_DESIGN = existing sufficient（PR #150 で同期済み）／decision-log = 足さない（下）。
- plan にしかない durable な判断の昇格先: 無い。足した文は D-111 の下の正本の細部（22 §17・29・37・45・55）に置き、cross-cutting な新しい決定を作らないので D-124 は使わない。
- 前提・制約と延期した design gap: P1（SD-23、取込み前の原本の形・精算回数・bytes。2026-10-06 版での採取では `XZ_BKUP` の分だけを観測）と P4 は L3 の前に確かめる前提で、P1 の (a) と P4 は 2026-10-23 に合格、P1 の (b)（bytes の比較）は未確認で、L3 の前に確かめる（Contract Probe）。窓・(b) の lane・D-117 の照合は延期で（写しの保持期間・削除の未決 A は消さないと決めた〈D-111、repo 外の回答台帳 TD-296。容量が目に見えて増えたら D-111 の Revisit で見直す〉。未決 B は D-114 で決まった）、どれも本 lane の取込みを止めない（SD を読むだけで、CV17 の取込みが続く間 `XZ` の file の数は今と変わらない）。
- 絶対保証の自己点検: 「SD に書かない」の例外は Windows の FAT の最終アクセス日と `System Volume Information`（IO-09-D3 に明記）。「二重に数えない」の例外は手で選んだ `None` の束（layout B）と NULL の既存行だけ（D-111 の Guarantee range）。「写しの無い SD の取込みは無い」は §37.4 手順 2b が TX の前に書き、失敗で止めるので成り立つ。逆向き（写しがあるのに取り込んでいない）はありうると BIZ-08-D5 に書いた（2026-10-17）。
- 判定: ready。上の正本で実装の何を・なぜ・何を棄てたかが答えられ、未解決の設計の問いは無い。

## Registration / Generation Obligations

| 変更対象 | 義務（本 lane） |
|---|---|
| Tauri command | `scan_register_sd`・`parse_and_validate_daily_report_from_sd` に `#[tauri::command]` + `#[specta::specta]`、`lib.rs` の `collect_commands!`（`export_bindings_to`）と `generate_handler!`（`run`）の両方、`cargo run --bin generate_bindings` |
| function-design doc 新設 | 該当なし（新しい doc を作らない）。既存の `29-io-daily-report-parser.md` の map の行に `io::register_sd`・`io::pos_source_copy` を足す（`design_compliance_test.rs` の `build_doc_to_modules_map()`）。scan を `biz/daily_report_import_service/scan.rs` に分けるなら、`37-biz-daily-report-import-service.md` の行に `biz::daily_report_import_service::scan` を足す（S1 の設計照合、round 3） |
| source / workflow doc 新設・改名・削除 | 該当なし |
| REQ / coverage | REQ-401 の部品の列に IO-09・IO-10 を足すなら `generate_traceability` で 90 を再生成（S6） |
| route | 該当なし（新しい route は無い） |
| operator 画面新設 | 該当なし（既存の日報取込みタブを変える。到達導線は変わらない） |
| migration | `db/schema_v12.rs`（番号は 22 §17）・`db/mod.rs`・`migration.rs` の `migrations()`、版の期待を持つ既存の test を `app_max_version()`・`migrate_up_to(&conn, VB_VERSION)` の形に（S2）、22 §17 の版の番号 |

## Impact Review Lenses

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | Which concepts belong to replaceable external adapters, and which concepts are stable app-core contracts? | Architecture, function design, decision-log, Plan Packet | SD の配置・名前・形は IO-09。core は精算ごとの束・取込み済み・同じ精算の拒否（D-111 の Compatibility のまま） | — |
| Fact check / design decision split | Which claims are observed facts from hardware/tool/files, and which are app decisions that need source-doc promotion? | Investigation doc, source design docs, decision-log | 事実は 29 §29.7.2（2026-10-06 版での SD の採取）。SD-23・SD-24 の原本の分は未確認の前提（TD-278、P1） | Contract Probe |
| Lifecycle / retry | What happens before, during, after, and after failure for import/export, duplicate input, rollback, retry, cancellation, and re-run? | Function design, DB design, UI design, Test Matrix | 読取りの途中の失敗は全体の失敗、scan は 30 分、写しの失敗は preview に戻る、TX の中で拒んだ束の写しは残りうる（`copy_path` が正本） | Matrix の State Lifecycle |
| Operator workflow | What does the operator do in the real sequence across app, external tool, media, print/export, backup, and recovery? | Screen/UI design, function design, Plan Packet manual checks | Ordinary Operation の表。(b) まで CV17 の取込みも続く | L3 |
| Replacement path | If the external system changes, which files/modules/docs are replaced and which app-core contracts remain stable? | Architecture, function design, decision-log | `io::register_sd`・§37.9 の組分けを取り替える | — |
| Data safety / evidence | How can the claim be supported by anonymized shape/count/hash/procedure evidence without committing real store data? | Plan Packet Data Safety, investigation doc, review evidence | test は合成の tree。P1・P4 は hash と名前の一覧だけ | Data Safety |
| Reporting / accounting semantics | Are totals, summaries, item records, returns, corrections, and inventory movements modeled separately enough to avoid false business meaning? | DB design, function design, report design, Test Matrix | 二重計上を D4・D6 で止める。日報は在庫を動かさない（D-025、変えない） | — |
| Manual verification | Which assertions cannot be proven by automated tests and require Windows native L3, external tool import, or real-device confirmation? | Plan Packet, Test Matrix, PR body | 取外し可能な drive の自動の発見、実 SD の写しを読む、写しが置き場にできる、画面の目視 | 下の Test Plan の L3 |
| 環境・再現性 | 新設の環境依存（toolchain / CI runner / OS 差異等）を repo-pinned config で強制するか、明示的に defer するか | repo-pinned config, Plan Packet | drive の列挙は Windows だけ（既存の `windows-sys` の pin。feature `Win32_System_Diagnostics_Debug`〈`SetThreadErrorMode`。PC の外の控えの runtime と共有〉と `Win32_System_WindowsProgramming`〈`DRIVE_REMOVABLE`〉を足す）。Windows 以外は空の列で、test は `resolve_register_sd_root`。`cfg(windows)` の本体は hosted CI（ubuntu）と WSL の check で compile されず、初めて compile されるのは L3 の build | Test Plan の FFI の読み、Residual Risk |

## Boundary / Wire Contract

- producer: BIZ-08 §37.9（`DailyReportSdScan`）、CMD-12 §45.6a・§45.6b
- consumer: UI-07（`scanRegisterSd`・`parseAndValidateDailyReportFromSd`、予備の `parseAndValidateDailyReport` の `source_path`）
- wire type: `RegisterSdScanResponse { scan, scan_token }`、`DailyReportSdScan { volume_label, scanned_from, candidates, skipped_unknown_count }`、`DailyReportSdCandidate { candidate_key, path_date, report_date?, source_filenames, status }`、`DailyReportSdCandidateStatus`（5 値の string union）、`DailyReportSourceFileRequest.source_path?`。既存の `DailyReportPreviewResponse` を返す
- internal type: `DailyReportSdScanSnapshot`（AppState）、`settlement_no: Option<i64>`（IO-07 → cache → DB。wire に出さない）、IO-09 の型（BIZ の中だけ）、`source_files_json` の内部の serialize 用の型
- precision/range: `settlement_no` は `i64`（収まらない値は `None`、29 IO-07-D5）。`skipped_unknown_count` は `i64` で、SD の file の数なので JavaScript の安全な整数の範囲を出ない。日付は YYYY-MM-DD
- round-trip path: scan → candidate_key → snapshot の 3 本 → 既存の preview → commit（DB に `settlement_no`、写しに `copy_path`）
- invalid input: 空の `selected_path` は validation、期限切れの scan_token は import_error（固定の文）、snapshot に無い candidate_key は validation、UUID でない token は validation
- compatibility: 既存の command と DTO の wire は `source_path`（省略可、`#[serde(default)]`）のほかは変えない。`daily_report_imports` の既存行は NULL

## Test Plan

Test Design Matrix: [2026-10-17-sd-direct-read-runtime](test-matrices/2026-10-17-sd-direct-read-runtime.md)。

- targeted tests: AC1〜AC7 の command。実装中は `bash scripts/local-ci.sh changed`。
- negative tests: Matrix の Negative Paths。
- compatibility checks: 既存の手で選ぶ経路・同日追加の確認・取消の既存 test（`biz/daily_report_import_service/tests.rs`、`DailyReportImportPage*.test.tsx`）を回帰で回す。`tests.rs` の 8 本は S3 のとおり束ごとに精算回数を分ける（assert は変えない。同じ精算回数で別の bytes の束を D4 で拒むのは D-111 の意図した変更で、その振舞いは Matrix の D4 の行が確かめる）。
- data safety checks: 実 SD・`EcrDatas` の file の中身、実の名前の一覧、hash の値を repo に入れない。
- main wiring/integration checks: `scan_register_sd` → BIZ-08 §37.9 → IO-09、`parse_and_validate_daily_report_from_sd` → §37.3、commit → IO-10、`lib.rs` の 2 つの登録。
- Human Gate に L3 があるので、Writer は L3 の前に `cargo check --release` を通す（AC10）。
- `cfg(windows)` の FFI（Plan Review round 1）: `find_register_sd_roots` の Windows の本体（`GetLogicalDrives`・`GetDriveTypeW`・`DRIVE_REMOVABLE`・`SetThreadErrorMode`・`SEM_FAILCRITICALERRORS`・`THREAD_ERROR_MODE`、元の error mode へ戻す処理）は hosted CI（全 job が ubuntu）でも、WSL の `cargo check --release` でも compile されない（Windows の target はこの環境に入れない。`rustup target add` をしない）。代わりに (1) Final Review で、各項目の module の path・引数と戻りの型・feature を windows-sys 0.61.2 の source と照らして読み（2026-10-23 の確認: `GetLogicalDrives() -> u32`・`GetDriveTypeW(PCWSTR) -> u32` は `Win32::Storage::FileSystem`、`DRIVE_REMOVABLE: u32` は `Win32::System::WindowsProgramming`、`SetThreadErrorMode(THREAD_ERROR_MODE, *mut THREAD_ERROR_MODE) -> BOOL`・`SEM_FAILCRITICALERRORS: THREAD_ERROR_MODE`・`THREAD_ERROR_MODE = u32` は `Win32::System::Diagnostics::Debug`）、途中で返る経路を含めて元の値へ戻すことと、file system を見る 5 関数がどれも最初に error mode の guard を作ることを確かめる（round 2）。(2) L3 の build を owner に頼む前に、Coordinator が同じ照合を merge の train の head の source で行う（owner の build で初めて compile の誤りが出て、介入を 1 回使うのを避ける）。

**L3（Windows の実機、owner の目視と PASS / FAIL。D-124 のとおり merge の train の先頭で最新の main を取り込んだ head で頼む。束ねる条件は上の Owner Effort Budget）**:

| # | 確かめること（観測できる合格の基準） | 到達経路 | 入力物 | 依存する既知の backlog・前提 |
|---|---|---|---|---|
| L3-1 | 取外し可能な drive の SD（か USB メモリ）を差して「SD から読む」を押すと、自動で見つかり、SD を戻す案内と一覧が出る。PC に SD の読取り機があるなら、その口を空のまま（USB メモリだけ差して）押しても Windows の error の dialog が出ない（29 §29.7.4 手順 1） | 売上データ取込み → 日報取込みタブ →「SD から読む」 | 合成の SD の tree（`CASIO\SR500_550_4000\XZ\yyyy\mm\` に合成の Z001/Z002/Z005 の 1 組。S3 の `scan::write_l3_register_sd_tree`〈`#[ignore]`、出力先 `INVENTORY_L3_SD_TREE_OUT`〉で Coordinator が作り、Ready の依頼と同時に渡す。組の日付は生成した日の前日・前々日〈S3〉で、生成から 28 日以内に L3 をする）を、USB メモリの直下に `CASIO` が来るように入れた USB メモリ。この USB メモリは PC の外の控えの媒体と backup の保存先に使わない（直下の `CASIO\SR500_550_4000` は 71 MNT-01-D10 がレジの SD の目印として書かない・消さない。PC の外の控えの runtime〈PR #190・#194〉の L3 の媒体と別の USB メモリにする） | 無い |
| L3-2 | SD が自動で見つからないときに「SD から読む」の直下に「場所を選ぶ」が出て（UI-07-D15 (1)）、PC の folder の中の SD の写しの `CASIO` か `SR500_550_4000` を選ぶと一覧が出て、「取り込めます」の行から取り込め、アプリのデータの folder（`identifier` の名の folder、`docs/msi-distribution-runbook.md` の「データの folder」）の下の `pos-sources\casio-sr-s4000\sd\` に、選んだ行の置き場のまま（`XZ\…` か `XZ_BKUP\…`。SD の root からの相対 path、29 §29.8.1）3 本ができる（どちらの置き場でも合格）。もう一度「SD から読む」→「場所を選ぶ」で同じ folder を読むと、取り込んだ行が「取込み済み」 | L3-1 の後に L3-1 の USB メモリを抜き（取外し可能な drive の直下に `CASIO\SR500_550_4000` が無い状態にする。自動で見つかると「場所を選ぶ」は出ない。Plan Review round 3）→ 同じタブの「SD から読む」→「場所を選ぶ」→ PC の folder の中（drive の直下でない、29 §29.7.4 手順 3）に置いた写しの `CASIO` か `SR500_550_4000` | 2026-10-18 版での SD の写し（owner の手元、`.local/reports/visit-2026-10-19/sd-copy.md` の出力。repo に入れない）。開発の DB はその精算を取り込んでいないこと。写しの精算の日付が scan の窓の中にあること（Plan Review round 2）: 精算の日付が L3 の日 − 30 日以後（取込みの無い開発の DB の窓の始まり、37 §37.9 手順 2。2026-10-18 の精算なら L3 の日が 2026-11-17 まで、それより前の日の精算ならその分だけ早まる）、または開発の DB に、写しの精算の日付以前の `report_date` の `completed` の取込みがある（窓はその日まで戻る）。どちらでもなければ、写しの代わりに L3-1 の合成の tree で代える。合成の tree で代えるときも PC の folder の中（drive の直下でない）に置いたものを選ぶ: Coordinator が L3-1 の生成と同じ回に、出力先を PC の folder の中にしてもう 1 つ書いて渡す（owner が USB メモリから写す手を作らない。日付の条件は L3-1 と同じ） | P1 の (a)（取込み前の原本の精算回数が読める。2026-10-23 に合格）と (b)（L3 の前に合格していること。AC10）。写しが無いときも合成の tree で代える |
| L3-3 | 一覧の状態の label と icon、上の文、予備の経路の 1 つずつの選択・「外す」・置き換えの確認の dialog の見え方 | L3-1 と同じ（L3-2 で抜いた USB メモリを差し直す。Plan Review round 3）。予備の経路は同じタブの「ファイルを選んで取り込む」 | L3-1 の tree に、Z002 の無い組と BOM 付きの組を足した合成の tree（同じ `scan::write_l3_register_sd_tree` が 1 回で書く） | 無い |
| L3-4 | 店の PC で読んだ後に SD をレジへ戻し、次の精算ができる | 店 | 店の SD | 本番の DB が無い今は店の運用に載らない（`docs/project-memory.md`）。導入の時の確認へ送るかを Ready の依頼と同じ回で owner に諮る（既定: 送る。残余は IO-09-D3 の source の検査と読取り専用の open で抑える） |

L3 Eligibility: L3-1〜L3-3 は Windows の実機でしか観測できず（drive の種類・native dialog・app data の folder）、新しい道具を要さず、故障の注入を含まない。故障の系（読めない file・写しの失敗・SD を抜いた後）は自動 test（Matrix）。

## Review Focus

- Ordinary Operation が正常な条件で目的を達成できるか、危険な結果を出さないか（別々に）。
- 37 §37.9 手順 10・45 §45.6a 手順 2a を作らないことが、PLU の runtime の lane の後付けを妨げないか（snapshot の型に field を足すだけで載るか）。
- IO-09-D3 の許可の列の source の検査が、別名の import・glob の import を拾うか。
- IO-09-D5 の実体の path の比較が、root の外を指す link・junction を列挙・読取りの前に止めるか。root そのものの link と root の下を指す link を通す理由、確かめてから開くまでの差し替えを想定しない理由が SD の写しの用途に合うか（closure）。
- Contract Probe P1 の (b) を L3 の前に確かめる段取りが D-111 の Guarantee range・29 §29.7.2 の SD-23 と合うか、owner の手が 1 回で済むか。
- UI-07-D16 の 2026-10-17 の細部（一覧の寿命、重複の拒否、置き換えの確認）が既存の選択の記憶と矛盾しないか。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| D-111 | `docs/decision-log.md` D-111 | S1〜S5 | Matrix 全体 | L3-1〜L3-3 |
| IO-07-D5 | 29 §29.2 の末尾（IO-07-D5、2026-10-17 の `i64` の範囲） | S1 `daily_report_parser.rs` | `settlement_no_*` の 4 行 | 非対象 |
| IO-09-D1 | 29 §29.7.4（手順 1 の error mode、2026-10-23） | S1 `register_sd.rs` | `resolve_*` | L3-1 |
| MNT-01-D10 の folder 名の定数の共有（D-114） | 71 MNT-01-D10、29 §29.7.4 手順 1 | S1（`src-tauri/src/io/mod.rs` の `pub(crate) const`。merge の順に依らない） | `register_sd::register_sd_folder_name_has_one_definition` | 非対象 |
| §29.7.4 手順 4・5 | 29 §29.7.4（`locate_in_register_sd_roots`・`check_selected_file_present`、§29.7.3 の `Io` の注記） | S1 | `locate_in_roots_maps_paths`・`check_selected_file_present_rejects_missing_and_dirs` | 非対象 |
| IO-09-D2 | 29 §29.7.5 | S1 | `list_*` | 非対象 |
| IO-09-D3 | 29 §29.7.6 | S1 | `module_uses_only_allowed_fs_api`・`scan_leaves_tree_unchanged`・`read_rejects_escaping_paths`・`read_stops_at_limit` | L3-4（導入の時へ送る候補） |
| IO-09-D4 | 29 §29.7.6 | S1 | `read_measures_shape` | 非対象 |
| IO-09-D5（closure） | 29 §29.7.6 の IO-09-D5、§29.7.5 手順 6、37 §37.9 手順 3 | S1・S3 | `list_and_read_reject_links_outside_root`・`root_and_read_failures_use_fixed_texts` の `InvalidPath` | 非対象（Windows の junction は Residual Risk） |
| IO-10 | 29 §29.8、`docs/architecture/io-task-specs.md` IO-10 | S1 `pos_source_copy.rs` | `pos_source_copy::*` | L3-2 |
| MNT-03-D15 | 22 §17、pos-tables §12b | S2 | `migration::adds_daily_report_settlement_no_nullable` | 非対象 |
| 24 §14.14・§14.18a | 24 §14.14・§14.18a | S2 `sales_repo.rs` | `sales_repo::*settlement*` | 非対象 |
| BIZ-08-D3 | 37 §37.9 手順 1〜9 | S3 | `scan::*` | L3-1〜L3-3 |
| BIZ-08-D4 | 37 §37.3 手順 8・§37.4 手順 4a・6、pos-tables §12b | S2・S3 | D4 の行 | 非対象 |
| BIZ-08-D5 | 37 §37.4 手順 2b・6、BIZ-08-D5（保証の範囲の 2026-10-17 の文を含む）、45 §45.7 | S3・S4 | D5 の行・`rejected_in_tx_leaves_copy_but_no_copy_path` | L3-2 |
| BIZ-08-D6 | 37 §37.3 手順 4・§37.4 手順 2a・§37.9 手順 7 | S3 | D6 の行 | 非対象 |
| §37.3 手順 1a・9 | 37 §37.3 | S3 | 手順 1a の行・`cached_preview_holds_only_sd_files` | 非対象 |
| CMD-12-D1 | 45 §45.2・§45.3・§45.4 手順 3a・§45.6a（手順 1〜4）・§45.6b・§45.7・§45.8 | S4 | `daily_report_import_cmd::*`、`architecture_test` | 非対象 |
| UI-07-D12 | 55 UI 判断 ID | S5 | UI の行 | 目視（L3-3） |
| UI-07-D15 | 55 UI 判断 ID、55 の利用者フロー手順 2 | S5 | `DailyReportImportPage.sd.test.tsx`・`reducer.test.ts` | L3-1・L3-3 |
| UI-07-D16 | 55 UI 判断 ID（2026-10-17 の (a)〜(f)） | S5 | `DailyReportImportPage.files.test.tsx`・hook の payload | L3-3 |
| 既存 BIZ-08-D1・D2、IO-07-D3、SPEC-SDI（同日追加・insert-only・per-import rollback）、UI-07-D13・D14 | 37 §37.3〜§37.5、55 | 変えない | 既存の test（回帰。`tests.rs` の 8 本は束ごとに精算回数を分け、assert は変えない。S3） | 非対象 |
| 隣接の除外: 37 §37.9 手順 10・45 §45.6a 手順 2a（D-117） | 37・45 | PLU の書出しの見直しの runtime | 非対象 | 非対象 |
| 隣接の除外: Z004 の識別（D-118、32 の新契約）・Z004 タブの一時停止（SPEC-STOP-D4） | 32、55 §55.0 | 変えない | 非対象 | 非対象 |
| 隣接の除外: backup の対象（71 §71.1 の注記） | 71 | 変えない（写しは backup に入らない） | 非対象 | 非対象 |

## Contract Probe

起票の時点で実験していない。以下の 2 つは設計 lane（PR #150）から引き継いだ未確認の前提で、「正常な条件で目的を達成できるか」に効き、P1 は「危険な結果を出さないか」（BIZ-08-D4・D6 の二重取込みの拒否が依る前提、D-111 の Guarantee range）にも効くので、どちらも L3 の前に確かめる（AC10。発注 367）。L3 の前の確かめなので Plan Gate は止めない。

- P1（SD-23・SD-24 の原本の分。取込み前の `XZ` の原本は 3 本とも精算回数が読め、取込みの後の `XZ_BKUP` の同じ file と同じ bytes か）: **(a) 合格（2026-10-23）、(b) 合格（2026-10-31）**（(a) の入力は 2026-10-18 版での SD の写し、(b) の入力は下の (b) の結果）。(a): 精算の後・CV17 の取込みの前の `XZ` にあった同じ日の複数の精算の束（各束の Z001・Z002・Z005）を、今の `io::daily_report_parser` の source を写した一時の program（repo の外）で読んだ。どちらの束も 3 本とも CP932 strict・BOM 無し・CRLF だけ・最終改行ありで（IO-09-D4 の形の flag は全部「外れ無し」）、精算回数は 3 本とも読めてそろい、`i64` に収まり（IO-07-D5 で `Some`）、`parse_daily_report_bundle` の `parse_errors` は空だった。束どうしの精算回数は互いに違った（SD-08 の同じ日の 2 回の精算が 2 つの候補になる前提と合う）。手順: (a) 2026-10-18 版での SD の写し（owner が精算の後、CV17 の取込みをせずに預かる道 1、repo 外の回答台帳 TD-304）の `XZ` の Z001/Z002/Z005 の 1 組を、合成の tree と同じ読み方（IO-09 の形の検査と IO-07 の精算回数）で手元で読み、形の flag と「精算回数が 3 本とも読めてそろうか」だけを出す（中身は出さない）。(b) 取込みの後の同じ file（店 PC の `EcrDatas` か、次の預かりの SD の `XZ_BKUP`）の SHA-256 と、(a) の file の SHA-256 を manifest どうしで比べる。SHA-256 が違う file は、取込みの前後で対象日と精算回数の行が同じかも IO-07 の読み方で比べ、同じ / 違うだけを出す（値も中身も書かない。下の見直しの入力にする）。
  - (b) の入力と owner の手（Plan Review round 2 の Codex 側の是正）: 2026-10-18 版での SD の写しは CV17 の取込みの前の 1 時点の写しで、CV17 の取込みは原本を `XZ` から `XZ_BKUP` へ移す（上の Ordinary Operation）ので、写しの `XZ` の file の取込みの後の版は、この写しにも、その前の 2026-10-06 版での採取にも無い見込み（推論、未実測）。Coordinator が先に手元の写しの manifest（日付と置き場だけ）でこれを確かめ、取込みの後の版が手元にあれば owner の手は要らない。無ければ owner に 1 回頼む（Owner Effort Budget の「P1 の (b) の入力」）: 店 PC の `EcrDatas` の、写しの `XZ` の 1 組と同じ日付の Z001・Z002・Z005 の 3 本の写し（店がその SD を CV17 で取り込んだ後のもの）か、次の預かりの SD の写しの、早く取れる方。owner の手は file の写しを渡すことだけで、比べるのは Coordinator か Writer。L3 の依頼より前に頼み、L3 と同じ回に束ねない（(b) が不一致なら L3 を頼まないので、束ねると L3 の手間が無駄になる）。
  - (b) の結果（2026-10-31、Coordinator）: 入力は次の預かりの SD の写し（2026-10-31 に owner が同じ `sd-copy.ps1` で読取り専用で写した。`sd_read_only_start`・`sd_read_only_end` = True、`copy_identical` = PASS）。この写しの `XZ` に 2026-10 の Z は無く（店が CV17 で取り込んだ後）、(a) の写しの `XZ/2026/10` の 2 つの束の Z001・Z002・Z005（計 6 本。連番なしの束と `A` の束）は、どれも `XZ_BKUP/2026/10` に、束の順の連番（`_0001`・`_0002`）を付けた名前で 1 本ずつあった。対応する 6 組の SHA-256 は 6 組とも一致した（`sha256sum` で両方の file を読み、値は書かない）。SHA-256 が違う組が無いので、行の比較は要らない。下の合否で合格。
  - 合否: (a) が読めて (b) が一致なら合格で、L3 へ進む。(a) が読めない、または (b) が不一致（行の比較が「同じ」でも）なら L3 に進まず、BIZ-08-D4・D6 を見直しに design へ戻す（owner に諮る、予備の介入「P1 が不一致のときの設計の見直しの判断」。D-111 の Guarantee range の「違えば L3 の前に BIZ-08-D4・D6 を見直す」）。(b) を L3 の前の必須にする理由: 取込みの後の file で対象日か精算回数の値が変わると（例: 原本の精算回数 5 の束を SD から取り込んだ後に、CV17 が移した file の精算回数が 6 と読めた）、次の scan でその束は別の精算になり、D4 にも D6 にも当たらず同日追加の確認だけが残る（対象日が変わると確認も出ない）。D4・D6 の拒否はこの前提に依るので、未確認のまま L3・merge へ進めない。手引き: `.local/reports/visit-2026-10-19/sd-copy.md` の 6。
- P4（スマホへの送信の失敗のときにレジが「SD カードへバックアップ」した file が、規則に合わない名前の file として増えるか）: **合格**（2026-10-23、入力は 2026-10-18 版での SD の写しの `p4-other-names.txt` と、その前の採取の写しの名前）。規則外の名前は、前の採取にもあった `CASIO` 直下の PLU の text と、新しく増えた `BACKUP` の下の数字だけの名前の file だけだった。増えた file は `XZ`・`XZ_BKUP` の外にあり、Z の名前の規則（`Znnn_dda`・`.CSV`）に近い形でもないので、IO-09 は列挙しない（`Unknown` にも数えない）。`XZ`・`XZ_BKUP` の Z は全部、置き場の連番の規則（`XZ` は連番なし・`XZ_BKUP` は `_nnnn`）に合っていた。2026-10-18 版での SD の写しの `p4-other-names.txt`（名前の一覧だけ）で、2026-10-06 版での採取に無かった規則外の名前が増えたかだけを見る。設計は分岐しない（`Unknown` として数え、読まない。IO-09-D2）。規則どおりの名前で増えたなら通常の候補になり、同じ bytes は 1 つにまとまる。増えたものが Z の名前の規則に近い別物なら owner に諮る（予備）。
- 観測済みで probe の要らない前提: SD は取外し可能な drive として見える（SD-25）、`XZ_BKUP` = `EcrDatas`（SD-22）、名前の形・大文字の `EJ`・同じ日の複数の Z・未知の名前の file（SD-02・03・07・08）。どれも 2026-10-06 版での SD の採取。`windows-sys` の `Win32_Storage_FileSystem` は `src-tauri/Cargo.toml` の `[target.'cfg(windows)'.dependencies]` で確かめた（2026-10-17。2026-10-23 の main も同じ）。

## Data Safety

- tracked に書かない: 実 SD・`EcrDatas` の file の中身、実の file 名の一覧、hash の値、実 JAN・商品名・金額、店主の発言の原文。P1・P4 の結果は合否と質的な要約だけを本 packet に書く。
- local-only: 2026-10-18 版での SD の写し（owner の手元と `~/downloads/inventory-field-check/approved-readable/raw-private/`）、回答台帳、開発の DB と `pos-sources/`。
- synthetic-only: test と L3-1・L3-3 の SD の tree（名前の形だけを 2026-10-06 版での採取に合わせ、中身は合成）。

## Residual Risk

Plan Review round 1（2026-10-23）で残すと決めたもの。どれも Plan Gate を止めない。

- `cfg(windows)` の FFI が CI と WSL で compile されない（#5）: 初めて compile されるのは owner の L3 の build。Test Plan の FFI の読み（Final Review と、L3 の build の前の Coordinator の照合）で抑える。Linux の `cargo check --release`（AC10）はこれを確かめない。2026-10-23 に windows-sys 0.61.2 の source で、`DRIVE_REMOVABLE` が packet の前の版の feature（`Win32_Storage_FileSystem`）でなく `Win32_System_WindowsProgramming` にあることを見つけ、S1・29 §29.7.4 手順 1 を直した。
- 使い手への押しつけ（#9、AI review から出た要件。owner の確認の前に Human Gate の必須にしない〈AGENTS.md の Working Rules〉）: (a) BIZ-08-D4 の文（37 §37.3 手順 8）と「同じ精算を取込み済み」の行は「前の取込みを取り消してから読み直してください」と案内するが、取消の導線は取込みの直後の結果の画面にしか無い（完了画面を離れた後の取消の導線は `docs/backlog.md` の「日報取込み標準手順の残設計」で未着手。正常な前提では D4 はまず出ない端の場合）。(b) 一覧の「読めません」「ファイルがそろっていません」の行は原因の種類を示すだけで、直し方を名指ししない。(a) は backlog の取消の導線の項目に結んだ。(b) の文を足すかは下の Open Questions で owner に諮る。
- IO-09-D5 の Windows の側（Plan Review の closure）: test の link は Unix の symbolic link で、junction と Windows の symbolic link を `canonicalize` が辿った先の実体にすることは自動 test に無い。Rust の資料（Windows で `CreateFile` と `GetFinalPathNameByHandle`）と 71 §71.4 の「書く先の固定」の同じ前提に依り、L3 に足さない（junction を作る手を owner に頼まない）。Windows の `canonicalize` が SD と PC の folder の中の写しで成功することは L3-1・L3-2 の通常の操作で観測される。確かめてから開くまでの link の差し替えは想定しない（29 IO-09-D5 の理由）。

## 新しい契約の文と境界の 4 観点

起票の commit（2026-10-17）と当て直し（2026-10-23）と Plan Review round 1〜3 と closure の是正（2026-10-23）で足した文を、安全な整数の範囲・0 の表示・失敗の時点と順序・単位や種類の混在で一巡した。

| 文 | 範囲 | 0 | 失敗の時点と順序 | 混在 |
|---|---|---|---|---|
| IO-07-D5 の `i64` に収まらない値は `None`、`0000` は `Some(0)` | `u64` → `i64` の境で `None`（Matrix に `i64::MAX` と + 1） | 0 を値として保つ（表示はしない） | parse の中（D3 の判定の後） | 型は整数だけ |
| §37.3 手順 9 の `sd_source_files` | — | 3 本とも PC なら空（commit は写さない） | 手順 1a の後 | SD と PC の混ざった束は SD の分だけ |
| BIZ-08-D5 の拒んだ束の写し | — | — | 2b（写し）→ 4・4a・5（TX の中の拒否）。正本は `copy_path` | — |
| 22 §17 の migration | 列は NULL を許す INTEGER | 既存行は NULL（0 で埋めない） | 実行器の TX。失敗で版を記録しない。vB・v9・vZ とどちらが先でも止めない（どれも `daily_report_imports` を見ない、2026-10-23） | — |
| UI-07-D16 (a)〜(f) | 一覧は系列ごとに 1 つ（最大 3） | 0 件で「確認する」は押せない | 読めない・大きすぎる file は足す前に止める。preview へ進むと一覧は空 | 系列の決まらない名前・同じ系列の重複を拒む |
| 37 §37.9 手順 10・45 §45.6a 手順 2a の実装の lane | — | — | 本 lane の scan は手順 9 の後に返す | — |
| 29 §29.7.4 手順 1 の thread の error mode（2026-10-23） | — | 空の読取り機の口は root にしない。root が 0 件なら BIZ-08 §37.9 手順 1 の「SD が見つかりません」 | file system を見る前に設定し、返る前（途中で返る場合を含む）に元の値へ戻す（round 2 で 5 関数の guard に広げた。下の段）。設定は呼んだ thread だけで、他の command の thread に効かない | 見るのは取外し可能な drive だけ（固定 disk・network は今のまま見ない） |
| 29 §29.7.4 手順 1 の folder 名の定数の共有（2026-10-23。置き場は round 1 で `crate::io` の `pub(crate) const` に決めた） | — | — | 置き場は merge の順に依らない（先に merge した lane が `io/mod.rs` に置き、後の lane は使う）。MNT → IO の参照で、IO → MNT の参照は作らない | 共有は文字列だけ。IO-09 の自動の発見（drive の root の直下）・`resolve_register_sd_root`（選んだ path から下へ）と、71 MNT-01-D10 の祖先の判定は判定の仕方が違うので関数は共有しない |
| 22 §17 の版の番号（2026-10-23。round 2 で取り直しの範囲を直した） | 次の版。#188 b1 の vZ が v11 を取ったので v12（2026-10-24 に取り直した） | — | 後から merge する側が file 名・`mod` の行・登録を取り直す（最新の版の期待は `app_max_version()` なので直さない）。取り直しを忘れると、同じ版の 2 つの登録で CI が止まる | — |
| 37 §37.8 の写しの行（D-114 で決まった D-111 の B、2026-10-23） | — | — | — | backup・PC の外の控えは DB の 1 file、写しはアプリの data の folder の `pos-sources/` で、混ぜない |
| 55 UI-07-D15 (5) の SD から来たことの持ち越しと recoverTo `sd_list`（round 1、#1） | 持つのは直前の scan の 1 つだけ（新しい scan で置き換え） | 候補 0 件の一覧でも `sd_list` に戻る（(3) の (c) の文） | parse の失敗は `parsing` の時点で決め、commit の失敗は今の `decideRecoverTo` の後に `import_failed` が preview を作り直すときに引き継ぐ。scan_token の期限は CMD だけが見る（UI は確かめない） | SD から来た状態と手で選んだ状態を同じ variant の省略可の field で分ける（手で選んだ経路は field を持たず、今どおり `idle` へ戻る） |
| 29 §29.7.4 手順 4 の path の文字列の判定（round 1、#6） | — | 空の root の列は `None` | file system を見ない純関数（SD を抜いた後の判定は手順 5） | `\` と `/` の区切り、drive 文字の絶対 path と `/` の絶対 path を同じ規則で扱う。要素ごとの一致で、前方一致（`SR500_550_4000X`）を root の下にしない |
| 29 §29.7.4 手順 1 の feature と型（round 1、#5） | `THREAD_ERROR_MODE` と `DRIVE_REMOVABLE` は `u32`（windows-sys 0.61.2） | — | compile は Windows の build が初めて（Residual Risk） | `DRIVE_REMOVABLE` は `Win32_System_WindowsProgramming`、error mode は `Win32_System_Diagnostics_Debug` で、feature が別 |
| IO-09-D3 の source の検査の red（round 1、#11） | — | — | test の時点（source の文字列。`#[cfg(test)]` より前） | `std` の項目の `as` の改名のすべての形を red にする（書く module の名前の列は round 2 で下の段の参照の規則に置き換えた） |
| 失敗定義の D4 の例外と、既存の 8 本の fixture の精算回数（round 1、#2） | 精算回数は 4 桁の合成の値（`0001`〜`0003`） | — | D4（§37.3 手順 8・§37.4 手順 4a）は手順 5 の snapshot の再検査と同日追加の確認より先に止めるので、test ごとに束の番号を分ける | 同じ bytes の束は同じ番号、違う bytes の束は違う番号 |
| L3 の合成の tree の生成（round 1、#10） | 1 組・Z002 の無い組・BOM 付きの組の 3 組 | 環境変数が無ければ何も書かない | `#[ignore]` で CI では回らない | 中身は合成、名前の形だけ 29 §29.7.2 |
| S2・22 §17 の版の期待の形（round 2、#1） | 版は 1 から連番なので、行数の期待も `app_max_version()` と同じ値（連番が崩れると行数の期待で落ちる） | — | vS の登録と同じ commit で 14 本を直す。b1 が先に merge したら、取り込んだ後に S2 の command を回してから vS を登録する | 「最新の版まで当たる」の期待（`app_max_version()`）と「vB の後の形」の期待（`migrate_up_to(&conn, VB_VERSION)`）を混ぜない。vB の後の全 object の SQL を後の migration の列に結びつけない |
| S3・L3-1・L3-2 の日付の条件（round 2、#2） | 合成の組は生成した日の前日・前々日、L3 は生成から 28 日以内。写しは精算の日付が L3 の日 − 30 日以後 | 窓の中が 0 件だと「新しい精算はありません」で FAIL に見えるので、日付を生成の時に決め、L3 の日に依らせない | 日付を決めるのは生成の時（Coordinator）、窓を決めるのは L3 の scan の時（BIZ-08 §37.9 手順 2）。条件を満たさなければ合成の tree で代えてから頼む | 合成の tree の USB メモリ（直下に `CASIO`）と PC の外の控えの媒体を混ぜない（71 MNT-01-D10 の目印で控えが止まる） |
| S5・55 UI-07-D16 (a) の手で選んだ preview の選び直し（round 2、#3） | 新しい一覧は dialog で選んだ分だけ（最大 3） | dialog のキャンセル（0 本）は preview のまま。(c)・(f) で 1 本も足せないときは `idle` の空の一覧と 1 スロットの文 | dialog が返った後に `idle` へ戻し、それから一覧に足す。preview から直接 `parsing` へ進まない | SD から来た preview（`sdOrigin`、一覧へ戻る）と手で選んだ preview（dialog）を `sdOrigin` の有無だけで分ける |
| S5 の flow test の書換え（round 2、#4） | — | 2 本の時点で「確認する」が無効 | parse は 3 本目の後の「確認する」でだけ呼ばれる | 件数の error の文（D16 で無くなる）の assert を、「確認する」が無効の assert に移す（弱めない） |
| Contract Probe P1 の (b) を L3 の前の必須の確かめに戻した（round 1 の #7・round 2 の #5 の「merge の前提にしない」と安全の主張の限定を置き換えた。round 2 の Codex 側の #1） | — | — | (b) の入力の写しを owner に頼むのは L3 の依頼の前で、束ねない。(b) が不一致なら（行が同じでも）L3 を頼まず design へ戻す | bytes の一致を合否にし、行の比較は不一致のときの見直しの入力にだけ使う（合否に混ぜない） |
| 55 UI-07-D15 (6) の結果から一覧へ戻る（round 2 の Codex 側の #2） | `sdOrigin` は直前の scan の 1 つと、選んだ行の `candidateKey` 1 つ | 替えた後に `NotImported` が 0 件なら、一覧の上の文は (3) の (b)・(c) の文（替えた後の状態の数で決める） | 戻るのは `import_succeeded` の後だけ。取消の後は今どおり `idle`。期限は CMD-12 だけが見る（「確認する」の時に固定の文） | 替えるのは UI の表示の scan の 1 行の `status` だけで、CMD の snapshot は替えない。同じ精算の別の bytes の行は scan の時の状態のまま出て、「確認する」で BIZ-08-D4 が止める |
| Matrix の `read_stops_at_limit` の読んだ byte の数（round 2 の Codex 側の #4） | 上限 + 1 byte まで（`max_bytes + 1`、29 §29.7.6 手順 2） | — | 上限を超えた時点で `TooLarge`（読み終えてから比べない） | 数えるのは test の reader の `read` が返した byte の和 |
| 29 §29.7.4 手順 1 の error mode の guard（round 2、#6） | — | — | 5 関数の最初で作り、drop で戻す（途中で返る・`?` の経路を含む）。設定は呼んだ thread だけ | file system を見る 5 関数だけ。純関数の `locate_in_register_sd_roots` は作らない |
| IO-09-D3 の source の検査の参照の規則（round 2、#7） | — | — | test の時点（source の文字列。`#[cfg(test)]` より前） | `crate::`・`super::` の両方の形と `use` の group の中の項目を数え、`super::` の folder 名の定数だけを許す |
| folder 名の検査の対象の file（round 2、#8） | — | — | test の時点 | `#[cfg(test)] mod` で読み込まれる file（`tests.rs`）は全体を test として数えない。本体の file は `#[cfg(test)]` より前だけ |
| L3-2 の写しの置き場の書き方（round 2、Coordinator の追加） | — | — | — | `identifier` の名を直書きせず、アプリのデータの folder（`docs/msi-distribution-runbook.md`）で引く。MSI の固定の lane（PR #191、repo 外の回答台帳 TD-392）で `identifier` が変わっても文が古くならない |
| 29 §29.7.4 手順 1 の folder 名の定数の形 `REGISTER_SD_FOLDER: [&str; 2]`（round 3、#3） | 要素は 2 つで固定（配列の長さを型が持つ） | — | 先に merge した lane が同じ形で置く（後の lane は形を変えない）。形の違いは compile か test の時点で落ちる | 要素ごとに `join` し、`\` を含む 1 つの文字列にしない（Windows と Linux の test で区切りの扱いが違う）。source の検査の数える literal `SR500_550_4000` は定義の 1 か所のまま |
| L3-2・L3-3 の到達経路と L3-2 の置き場の合格基準（round 3、#1） | 写しは選んだ行の置き場のまま（`XZ\…` か `XZ_BKUP\…`、どちらでも合格） | — | L3-1 → USB メモリを抜く → L3-2（「場所を選ぶ」）→ 差し直す → L3-3 の順。抜かないと自動で見つかり「場所を選ぶ」が出ない | 「場所を選ぶ」で選ぶのは PC の folder の中の写し（drive の直下でない、29 §29.7.4 手順 3）で、自動で見つかる USB メモリの直下の tree と混ぜない。合成の tree で代えるときも Coordinator が PC の folder の中に書く |
| S6・AC9 (3) の受入台本の Step 1 の書換え（round 3、#2） | — | AC9 (3) の hit 0 で古い button の名が残らない | 実装と同じ PR で書き換える（merge の後に台本の Step 1 で止まらない） | 予備の経路（PC の上の fixture、写しも精算回数の検査も効かない）だけを使い、SD から読む経路と混ぜない。期待値は変えない |
| S1・Registration Obligations の `scan.rs` の登録（round 3、#4） | — | — | 分けたときだけ、同じ commit で map の 37 の行に足す（足さなければ `design_compliance_test` が落ちる） | 分けないとき（`mod.rs` に置くとき）は既存の `biz::daily_report_import_service` の行で足りる |
| 29 IO-09-D5 の実体の path の比較（closure） | 比べるのは path の要素だけ（数の範囲は無い） | root の下の file が 0 本でも同じ規則（空の一覧は今どおり `Ok`） | 列挙は folder に入る前と file を返す前、読取りは open の前（読んでから比べない）。解けないは `Io`、外は `InvalidPath`。どちらも途中までの一覧・bytes を返さない | 比べる両側を同じ `canonicalize` で解く（Windows の `\\?\` の前置・大文字小文字・短い名前の形をそろえる。片側だけ解くと root の上の link で正しい file を止める）。手で選ぶ経路の `locate_in_register_sd_roots` の文字列の判定とは混ぜない（そちらは file を開かない） |
| 29 IO-09-D5 の root そのものの link と root の下を指す link を通す（closure） | — | — | root は列挙・読取りの各 call の最初に解く（SD を抜いていれば `Io`） | root の指す先は選んだ場所と同じ扱い。root の下を指す link で同じ file が 2 つの path に出ても、同じ bytes の組は 37 §37.9 手順 6 で 1 つになる。71 §71.11.3 の媒体の「reparse point を追わない」（書く側）とは規則を分ける（読む側は包含だけで足りる） |
| 29 IO-09-D5 の確かめてから開くまでの差し替えを想定しない（closure） | — | — | 確かめた解いた path をそのまま開く（元の path で開き直さない）。読む間に別の process が link を差し替える場合は防がない（理由は 29 IO-09-D5、handle から確かめる案は §29.7.8） | — |
| IO-09-D3 の許可の列の `canonicalize`（closure） | — | — | test の時点（source の検査）。許可の列にあるので `fs::canonicalize` は green | 実体の path を得るだけで file system を変えない。書く系の API は今どおり red |
| 37 §37.9 手順 3 の `InvalidPath` の固定の文（closure） | — | — | 列挙（手順 3）と読取り（手順 5）のどちらで出ても scan 全体を止め、候補を返さない | `Io`（差し直せば直る）と文を分け、原因（SD の外を指すリンク）と直し方（SD そのものを差す・リンクの無い写しを選ぶ）を名指す（D-127）。raw の path を文に入れない |

## Open Questions

- owner 向け（今は無い。L3 の回で諮る）: L3-4 を導入の時へ送るか（a 送る〈推奨〉／b 今の L3 で店で行う。答えが無ければ a）。
- owner 向けの候補（AI review 由来〈Plan Review round 1 の #9 (b)〉。L3 の回に同じ依頼で諮るが、判断が 1 つ増えるので Owner Effort Budget の予備から 1 回に数える〈decision point の単位〉。確認の前に gate にしない）: SD の一覧の「読めません」「ファイルがそろっていません」の行に、直し方の 1 文を添えるか。a 添える（例: 「SD を差し直してもう一度読むか、「ファイルを選んで取り込む」で選んでください」。本 lane の中で 55 UI-07-D15 (3) と 37 §37.9 を直す）／b 今のまま（label と icon だけ。直し方は店の手順書に書く）／c 後の lane（画面のデザインの刷新、TD-296 の 11c）で見直す。条件付きの推奨: a（使い手に押しつけない〈D-127〉に合う。ただし L3-3 で見た owner の所感を先に聞く。precondition-dependent）。答えが無ければ c。
- agent 側: migration の番号（merge の順で決める。22 §17。#188 b1 の vZ と、先に merge した側が若い番号を取る）。Contract Audit の 2 本目を採るか（Coordinator）。REQ-401 の部品の列に IO-09・IO-10 を足すか（Writer、S6）。

## Implementation Results

実装（S1〜S6）は `fc43cbf5`〜`8fd5b875`。実装の Writer が見送った 6 点を Coordinator が 2026-10-24 に裁定し（相談役の助言は repo 外）、是正と記録を本 commit で行った。

- 点 1（AC1 の baseline の filtered の数）: 受理。着手時の baseline は `1316 filtered out` で、上の AC1 の `1315` より 1 多い。差は plan-approved の後に取り込んだ main の PR #195（`6457204e`）が足した lib test 1 本（`test_create_product_req101_takkyubin_rejects_custom_code`）。確かめた command と出力: `git diff eed07d37 f140526b -- src-tauri/src | grep -E '^[+-]\s*#\[(tokio::)?test\]' | sort | uniq -c` → `1 +    #[test]`（AC の baseline の head から implementing の head までの lib test の増減はこの 1 本だけ）。AC1 を今の head（`8fd5b875`）で逐語に実行: `cd src-tauri && cargo test --lib -- io::register_sd io::pos_source_copy` → exit 0、`test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 1360 filtered out; finished in 0.36s`。
- 点 2（`sales_repo::latest_completed_daily_report_date` が 24 に無い）: 是正。S2 の計画に無い repo 関数（`pub(crate)`、呼び手は `scan.rs` の手順 2 だけ）を [24](../function-design/24-io-csv-import-repo.md) §14.18c に足した。振舞いは 37 §37.9 手順 2 と同じ（completed の `report_date` の最大、`rolled_back` は数えない）。code は変えない。
- 点 3（45 §45.4 手順 3a の `app_data_dir` を手順 1 より先に取る順）: 受理。code は変えず、[45](../function-design/45-cmd-daily-report-import.md) §45.4 の注記と手順 3a に、実行の順と観測できる差（token が不正かつ `app_data_dir` を取れないときに `internal` が先に返ること）を書いた。
- 点 4（「場所を選ぶ」を SD の読取りの失敗の後はいつも出す）: 受理。[55](../function-design/55-ui-csv-import.md) §55.0 の利用者フロー手順 2 を実装（`DailyReportImportPage.tsx` の `scanError !== null`）に合わせ、理由（wire に「見つからない」の構造化された区別が無く message で分岐しない、2 枚以上・レジの SD でないときも場所を選べば解ける）を書いた。UI-07-D15 (1) の owner 決定（2026-10-06）の文は変えない（実装はその上位集合）。Matrix の UI-07-D15 の行（見つからないときに出ない → red）と SD の scan の状態の行（失敗 → 固定の文と予備の操作）は実装と食い違わないので直さない。
- 点 5（S1 の commit の CI が red）: 受理（記録だけ）。Draft の run `40022365339`（head `4b70a4f5`、S1 と 90 の反映）は Rust tests の `architecture_test` の `layer_dependency_rules` が FAILED。`src-tauri/tests/architecture_test.rs` は本 branch で変えておらず（`git diff --stat f140526b 8fd5b875 -- src-tauri/tests/architecture_test.rs` → 差分なし）、#172 が新設した `src-tauri/src/io/register_sd.rs` を直して次の run `40025822344`（head `519c236e`）と `40029463494`（head `8fd5b875`）は success。commit ごとの緑を求める規則は無く、Ready の条件は最終の head の run（AC4・AC11）。
- 点 6（実装の前の red でなく mutant で test を確かめた）: 受理。実装の Writer は自己申告で mutant 37 件を回したと報告したが、内訳（どの変異をどの test が red にしたか）は保存されていないので、件数・内訳とも未実測として扱う（D-062）。独立の変異確認は Final Review（`docs/DEV_WORKFLOW.md` の Standard Final Review の「Mutation / anti-tautology check」）で行い、Writer の自己実行はその代わりにしない。
- AC10（2026-10-31、Coordinator、head `d7468378`）: `cd src-tauri && cargo check --release` → `Finished \`release\` profile [optimized] target(s) in 5.03s`（exit 0、rustc 1.99.1。WSL なので `cfg(windows)` の本体は compile されない。Test Plan の FFI の照合は L3 の build の前に別に行う）。Contract Probe P1 の (b) は合格（上の Contract Probe）。
- Test Plan の FFI の照合（L3 の build の前、2026-10-31、Coordinator、head `7c096be5`）: `src-tauri/src/io/register_sd.rs` の `cfg(windows)` の本体を、cargo の cache の `windows-sys-0.61.2.crate` を展開した source と照らした。`GetLogicalDrives() -> u32`・`GetDriveTypeW(PCWSTR) -> u32` は `Win32::Storage::FileSystem`、`DRIVE_REMOVABLE: u32 = 2` は `Win32::System::WindowsProgramming`、`SetThreadErrorMode(THREAD_ERROR_MODE, *mut THREAD_ERROR_MODE) -> BOOL`（`BOOL = i32`）・`SEM_FAILCRITICALERRORS: THREAD_ERROR_MODE = 1`・`THREAD_ERROR_MODE = u32` は `Win32::System::Diagnostics::Debug` で、code の `use` の path・引数・戻りの比べ（`!= 0`・`== 0`・`!= DRIVE_REMOVABLE`）と合う。どの項目にも module の中の追加の feature の gate は無く、`src-tauri/Cargo.toml` の features に `Win32_Storage_FileSystem`・`Win32_System_Diagnostics_Debug`・`Win32_System_WindowsProgramming` がある。error mode の guard は file system を見る 5 関数（`find_register_sd_roots`・`resolve_register_sd_root`・`check_selected_file_present`・`list_register_sd_entries`・`read_register_sd_file`）の最初で作り、`Drop` で前の値へ戻す（途中で返る経路を含む）。`locate_in_register_sd_roots` は作らない（純関数）。食い違いは無い。
- 版 11 の衝突（PR #188 も `schema_v11.rs`）は本 commit で触らない。#188 の merge の後に付け替える（上の Open Questions の agent 側の migration の番号）。

## Review Response

先行 round の結果・評価（判定・件数・採否・reviewer の意見）はこの節にだけ書く。前半の節と遷移記録には「round N の是正（Review Response 参照）」だけを書く（独立 review は `## Review Response` より前だけを読む）。

round 1（`3d7c0111`）: Claude 側 fresh Opus 5.5 = reject（P1 0 / P2 3 / P3 8）。Codex は利用上限で使えず、§3.3 に従い Claude 側だけで round 1 とした。Coordinator が P2 3 件を現物で確かめ（`src/features/daily-report-import/reducer.ts` の `dismiss_error`、`src-tauri/src/biz/daily_report_import_service/tests.rs` の fixture の精算回数、S6 の外の注記）、11 件を全部採用した。相談役は省いた（R3、指摘が Scope の漏れ・fixture・注記で筋が明確）。是正は起草役の `881700ca`。

- P2: 一覧へ戻る経路（`sd_list` と `sdOrigin` の持ち越し、`dismiss_error` の分岐、写しの失敗の分岐は変えない）、既存の BIZ test 8 本の束を別の精算回数に（assert は変えず、D4 の拒否は意図した変更と失敗定義に添えた）、S6 の外の注記と AC9 の広げ。
- P3: 版の assert は 3 か所、`cfg(windows)` の FFI を Final Review と L3 の build の前の照合で見る（`THREAD_ERROR_MODE` を許可の列に。起草役が `DRIVE_REMOVABLE` の feature が `Win32_System_WindowsProgramming` にあることを見つけて足した）、path の判定を文字列の規則に、P1 の安全の主張を前提つきに、folder 名の定数を `io/mod.rs` に固定（PR #190 の所有表の文と同じ所）、取消の導線と行の直し方を Residual Risk と Open Questions に（AI review 由来で gate にしない）、L3 の合成の SD の tree を `#[ignore]` の生成の test で、source の検査の red の形を広げた。
- 起草役が発注の外で直したもの: frontend の file の数（directory の中は 8）、上の feature。

round 2（`9f02ddd7`）: Claude 側 fresh Opus 5.5 = reject（P1 0 / P2 2 / P3 6）、Codex = reject（P1 0 / P2 4 / P3 1）。重なりは 2 件（P1 (b) の安全の説明、手で選んだ preview の選び直し）。Coordinator が Claude 側の P2 2 件を現物で確かめ（`src-tauri/src/db/schema_v10.rs` の 3 本と `src-tauri/src/lib.rs` の 2 本が全 migration の後に `VB_VERSION` を期待、37 §37.9 手順 2 の窓）、全件を採用した。Coordinator の追加 1 件（L3-2 の置き場がアプリのデータの folder の名を直書きし、PR #191 の `identifier` の変更〈repo 外の回答台帳 TD-392〉に当たる）も入れた。是正は起草役の `b852071f`（Claude 側 8 件と追加）と `af0f95b0`（Codex 側 5 件。522 で閉じていた所は閉じた文を確かめて残りだけ）。

- Claude 側: 最新の版の期待は `app_max_version()` と比べる形にし（版が増えても直さずに済む。洗い直しで `schema_v2.rs` の 1 本を足した）、vB の後の形は `migrate_up_to`。L3 の合成の日付は生成の日から決める。手で選んだ preview の選び直しは `idle` の一覧へ戻す（55 UI-07-D16 (a)）。flow test の移し方、P1 の (i)・(ii) の分け、error mode の guard を file system を見る 5 関数に、source の検査を `crate::`・`super::` の参照で、folder 名の検査から test module の file を外す。
- Codex 側: P1 (b) を L3 の前の必須の確かめに戻した（D-111 の Guarantee range と 29 SD-23 に合わせ、不一致なら L3 を頼まず design へ戻る）。取込みの結果から同じ scan の一覧へ戻る遷移（55 UI-07-D15 (6)）。読んだ byte の数を数える reader で上限の test を固める。介入は decision point の単位で数える。

round 3（天井、`2b9cabb6`）: Claude 側 fresh Opus 5.5 = reject（P1 0 / P2 1 / P3 3）。Codex 側は未実施。Coordinator が P2 を現物で確かめ（55 UI-07-D15 (1) の「場所を選ぶ」は自動の探索で見つからないときだけ出るのに、L3-2 は「同上 →『場所を選ぶ』」だけ）、全 4 件を採用した。disposition: round 4 は回さず、一括の是正（起草役の `1c154230`）の後に closure へ出す。

- P2: L3-2 の到達経路を「L3-1 の USB を抜く → SD から読む → 場所を選ぶ → PC の folder の中の写し」に、合格基準の置き場は `XZ`・`XZ_BKUP` のどちらでも合格に。L3-3 は USB を差し直す経路に（発注の外の波及）。
- P3: 受入台本の Step 1 と 55 UI-07-D16 の「今の」の 2 文を S6 に、AC9 (3) を足した。folder 名の定数の形を `[&str; 2]` に決めた（PR #190 の packet の書き方と矛盾しない）。`scan.rs` を分けたときの設計照合の登録。

closure（`499da80d`）: Claude 側 fresh Opus 5.5 = approve（P1 0 / P2 0 / P3 1）、Codex（round 3 の Codex 側で closure を兼ねる）= reject（P1 0 / P2 1 / P3 0）。Claude 側の P3（folder 名の定数の名前と形が PR #190 の packet に無い）は Coordinator が #190 の packet の所有表に名前と形を足して閉じた（#190 の branch の `000345f1`）。Codex の P2: path の包含の検査が link 先を解かず、PC の中の SD の写しの `XZ` が root の外への directory の link だと、合法な相対 path が文字列の検査を通り、外の file を読む（Codex が合成の tree で再現）。Coordinator が受理し、是正は起草役の `8fef2a03`（29 IO-09-D5 を新設: 見る path を `std::fs::canonicalize` で実体に解き、解いた root の下かを要素ごとに比べ、外なら `InvalidPath` で開かない。確かめた path をそのまま開く。root の下を指す link は通す。37 §37.9 手順 3 に `InvalidPath` の固定の文。Matrix に link の tree の行と mutant）。Coordinator の inline の review（2026-10-23、fresh context でない）: 規則の筋（両側を同じ関数で解く、確かめた path を開く、確かめから open までの差し替えの余地は SD の写しの用途で受け入れる理由）を読み、書く側の IO-10 を対象外にする判断（書く先はアプリのデータの folder で、link を置けるのは同じ利用者だけ）を受け入れた。

closure 2 回目（`7883545f`、Codex 側だけ。安全境界なので Claude の subagent に出さない）: Codex = approve（P1 0 / P2 0 / P3 0）。X1（link 先の包含）は closed。固定の確認は、IO-09-D5 を写した独立の Rust の probe で、外へ向く directory の link・file の link が列挙・読取りとも `InvalidPath` で開く回数 0・読む byte 0、文字列の比較に戻す mutant・対象だけを解く mutant・検査を読取りの後へ移す mutant がどれも red。Windows の前置・大文字小文字・短い名前の比較、IO-09-D3 の維持、IO-10 を対象外にする判断も妥当とした。Y1 は closed。
