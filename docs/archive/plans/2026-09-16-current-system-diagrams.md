# 現行構造の図面同期と設計整合性の点検

## Workflow State

- Evidence Mode: github
- Phase: archive
- Risk: R2
- Execution Mode: codex-only
- Plan Commit: 6ad7576103abf9fd4d5057ca4d298c5acf21c0b7
- Amendments: ab03bacc3f2d4c36536ecb60cf4d7f4c34e91e83, ff64509a75b7cbdbfc8e7cd7a0ced19f7afb25a6
- Coordinator: Astra（D-087、一貫担当。採用・裁定は owner）
- Writer: Astra
- Plan Reviewer: 当初Sonnet PASS / A-1はOpus指摘全採用 + Sonnet限定是正確認PASS
- Final Reviewer: Sonnet + Opus（独立 fresh context、owner指定）
- Final Review Minimum: 2
- Human Gate: ready,merge

2026-09-16: owner の「実情に合わせて図面を更新し、アプリの設計上の矛盾も見つける」という依頼により kickoff → spec-check。既存の仕様・schema・route を調査し、製品契約を変更しない現行構造の記述として Design Readiness を確認して plan-draft へ。計画を先に commit し、独立 Plan Review に提出する。

## Owner Effort Budget

- 介入回数上限: 3
- 実働時間上限: 30分
- relay 往復上限: 2
- Plan Review round 天井: 3

レビューの発注・回収は D-087 により Astra が担当する。今回の図面・検証モデルと別の製品仕様判断が必要になった場合は、根拠付きの所見を残し、製品runtimeの変更へ進めない。

## Consultation Relay

- Review Order Artifact: none
- Review Order Ref: none

## Risk

Risk: R2

現行実装を説明する図面と設計本文の同期、合成データによるtest-onlyのモデルベース検証と調査記録。project-profileのR2 test helperに該当し、DB schema・migration・route/search・業務挙動・既存workflow gateの契約は変更しない。既存実装への疑義は所見として分離し、既存の製品要求を実装に合わせて弱めない。棚卸しの診断FAILは製品問題として保持し、通常suiteのPASSで解消済みとしない。

## Goal

Goal Invariant:

### 最小完了条件

- owner が現行のデータ構造、画面間の到達・戻り、在庫・売上・日報・PLU のつながりを図で確認できる。
- 図から見つかった問題について、根拠・影響・確認済み範囲・残る検証・次の是正候補を追える。
- 入庫・手動販売・返品・廃棄・商品別CSV・棚卸しをまたぐ操作列を、実BIZと独立モデルで実行・比較できる。発生日と取込み時点を分けた診断の成否を記録する。

### 失敗定義

- 古い図を現行として案内する、現行にない FK・画面・取消機能を描く、または実装の欠陥を正しい設計として追認する。
- 数値や図の形を揃えるだけで、任意参照・在庫を動かす条件・戻り経路・失敗時の状態を確認しない。
- テストの期待値を実装の誤動作へ合わせる、または診断FAILを通常suiteのPASSで隠す。

### 非目的

- 製品のruntime挙動、DB schema、店舗データ、依存package、既存CI gateの変更。
- 実店舗運用の安全性全般をこの静的調査だけで保証すること。
- 全部の図を自動生成する汎用基盤の新設。

## Scope

- `docs/inventory_system_erd.html`: 現行の全 application table と migration 管理表、カラム・PK/FK・NULL許容・有限状態を同期。重複線を整理し、物理 FK とアプリ側の論理参照を区別する。巨大な図でも拡大・スクロールして確認できる表示と source の表示を備える。
- `docs/diagrams/current-system.md`: 図面の入口、層と外部連携、現行画面の構成・主要遷移、業務記録の詳細と戻り、在庫・売上・日報・PLU のデータフローを Mermaid で記述。query 条件と戻り先の区別、失敗・取消・再実行の境界を注記する。
- `docs/research/2026-09-16-diagram-audit.md`: 基準の code ref、図ごとの照合先、解消した文書不一致、残る製品設計・実装上の所見を根拠付きで記録する。問題なしと判断した境界も確認範囲を限定して記載する。
- `docs/db-design/transaction-tables.md`: migration で追加済みの冪等性列をカラム定義へ反映。将来の取消・訂正設計は未実装のまま区別する。
- `docs/DB_DESIGN.md` / `docs/SCREEN_DESIGN.md` / `docs/ARCHITECTURE.md` / `docs/PROJECT_HANDOFF.md` / `docs/architecture/ui-task-specs.md`: 現行図への参照、historical mockup の位置付け、既存 UI task 一覧の欠落を同期する。
- `docs/screen_mockups.html`: 既存内容は初期提案資料として保持し、旧画面遷移図を現行と誤認しない可視注記と現行図へのリンクを加える。
- `docs/function-design/55-ui-csv-import.md` / `56-ui-daily-sales.md` / `57-ui-monthly-sales.md`: 既存図の明白な転記不一致を本文・型定義・実装に照合して同期。競合する業務契約を一方的に変更しない。
- 本 packet と `docs/Plans.md` の進捗を同期する。
- `docs/diagrams/cross-feature-verification.md`: 標準的な図の用途、シーケンス図・ステートマシン図・簡略アクティビティ図、独立モデルの契約と実行結果を記録する。既存current-systemの入口から参照する。
- `src-tauri/src/biz/csv_import_service/tests/cross_feature_tests.rs` と `tests/mod.rs`: 既存test-only境界内へモデルベーステストを追加。CSV fixture builderと一時DBを再利用し、実BIZと日次売上consumerまで検証する。新規診断の `#[ignore]` はXFA-D4どおり明示実行用に限定する。
- `src-tauri/src/biz/csv_import_service/test_support.rs` と `tests/commit_tests.rs` / `tests/rollback_tests.rs`: 重複する `build_cached` をtest_supportへ挙動不変で移設する。既存testのassert、ケース、実行条件は変更しない。
- `docs/plans/test-matrices/2026-09-16-cross-feature-model-audit.md`: oracle・失敗条件・範囲・既知FAILの扱いを先に定義する。
- `docs/function-design/90-traceability.md`: 必要なgenerated更新をcanonical generatorで行う。

## Non-scope

- runtime・schema・migration・DTO・navigationの製品挙動変更、既存テストの削除・弱体化。
- 既存 mockup の全画面再デザインや提案資料の現行画面化。
- CI への新しい必須 gate、汎用 generator、外部サービスの新設。
- Ready / merge（owner の別指示が必要）。

## Acceptance Criteria

- AC-1: `src-tauri/src/db/migration.rs` が登録する schema を照合し、ER の table/column/PK/FK/nullable が一致する。SQL をメモリ内 SQLite に適用して比較し、実店舗 DB は開かない。enum / CHECK / index は省略範囲を明示し、図に書く値は実 schema と一致する。
- AC-2: `src/config/navigation.ts` と `src/routes/` の具体的な page route を図・対応表で網羅する。layout/index の重複、画面内展開、検索条件つき deep-link は独立画面と混同しない。各主要矢印は実装の Link / navigate / callback へ遡れる。
- AC-3: 在庫を動かす商品別売上と、在庫を動かさない公式日報を区別する。CSV rollback、通常業務記録の追跡、棚卸し、PLU の app 内確認と外部レジ反映の境界が図と説明から分かる。
- AC-4: 図から見つかった矛盾は、文書の更新漏れ・設計と実装の不一致・追加検証が必要な設計懸念に分類し、source と実装の位置、影響、再現または静的根拠、未検証範囲、是正候補を記録する。重大な欠陥は黙って実装追認せず owner へ返す。
- AC-5: `bash scripts/doc-consistency-check.sh`、`git diff --check`、変更分類に応じた `bash scripts/local-ci.sh changed` が通る。Mermaid を render し、構文エラーがなく文字・線・図の範囲が読めることを確認する。新しい dependency を repository に追加しない。
- AC-6: 独立 Final Review を通し、未解決 finding は owner に明示する。Windows native L3 はアプリ表示・挙動を変更しないため対象外。
- AC-7: XFA-D1の通常suiteが実BIZ→DB→日次売上を独立期待値と比較し、再送・対象importの取消・POS非連動・拒否時の状態維持・DB再接続を含む有限の操作列を実行する。最初の失敗prefixと期待/実値を出す。
- AC-8: XFA-D2のカウント後の移動、過去販売の遅い取込み、正常対照を実行する。診断の不一致は非0終了のまま結果へ記録し、既知P1の解決・運用安全性の証明とはしない。記録上のモデルと現物モデルを混同しない。
- AC-9: モデル検査が、在庫とmovementを同時に誤らせた状態も検出できることを合成DBへのmutationで確認する。モデルがDBから期待値を逆算していないことをレビューする。
- AC-10: Rust fmt/clippy、通常cargo test、design compliance、traceability生成/check、local changedを実行する。通常suiteの成功と明示診断の失敗は別の証拠とし、失敗を隠すscriptやgate変更を加えない。最終レビューはSonnetとOpusの独立パスを実施する。

## Design Sources

- [DB 設計](../../DB_DESIGN.md) と配下のテーブル定義、[migration](../../function-design/22-mnt-migration.md)。実装は `src-tauri/src/db/schema_v*.rs` と `migration.rs`。
- [アーキテクチャ](../../ARCHITECTURE.md)、[画面設計](../../SCREEN_DESIGN.md)、`src/config/navigation.ts`、`src/routes/`、各画面の Link / navigate。
- [共通規則](../../function-design/10-common-rules.md)、[在庫処理](../../function-design/31-biz-inventory-service.md)、[商品別 CSV](../../function-design/32-biz-csv-import-service.md)、[PLU](../../function-design/33-biz-plu-export-service.md)、[棚卸し](../../function-design/35-biz-stocktake-service.md)、[日報](../../function-design/37-biz-daily-report-import-service.md)、[記録追跡](../../function-design/65-inventory-record-traceability.md)。
- [図とコードの照合基準](../../code_review.md)、[文書の書式](../../DOC_STYLE_GUIDE.md)。
- [横断検証モデル XFA-D1〜D5](../../diagrams/cross-feature-verification.md)、[Test Design Matrix](test-matrices/2026-09-16-cross-feature-model-audit.md)。

## Required Design Artifacts

| 対象 | 資料 | 状態 |
|---|---|---|
| 現行 DB の図示 | schema / migration / DB design | 既存 sufficient。本文の転記漏れのみ同期 |
| 画面・業務フローの図示 | route / navigation / function-design | 既存 sufficient。不一致を調査所見に分離 |
| 図面の位置付け・読み方 | current-system.md と親文書 | 同じ変更で追加 |
| 新しい製品契約 | なし | 非対象 |
| test-only oracleと診断の判定 | cross-feature-verification XFA-D1〜D5 / Matrix | 本Gated Amendmentで設計済み |

## Registration / Generation Obligations

- 新設図面の入口を DB / SCREEN / ARCHITECTURE / PROJECT_HANDOFF から参照する。
- 新しいTauri command、page route、function-design文書はない。bindings / route treeの変更は不要。新しいREQ付きRust testによるtraceabilityはcanonical generatorで再生成・checkし、生成物は手編集しない。
- `cross_feature_tests` を既存csv_import_serviceのtest-only moduleとして登録する。production visibility、feature flag、依存packageを増やさない。

## Design Readiness

既存設計とsourceは図示・通常の操作モデルを作るために十分。棚卸し時点の業務期待はXFA-D2で診断専用oracleとして定義し、既存実装の期待値へ合わせない。未解決の製品設計は所見として残し、修正方式を採用しない。test-only scopeの実装前に決める製品Human Gateはない。新しいmoduleの実装は本Gated AmendmentのPlan Gate通過後に開始する。

## Impact Review Lenses

- Adapter/core: POS 固有形式、app 内の売上・日報モデル、外部反映の確認限界を区別する。
- Fact/design: コードの実在、設計意図、現在の不一致、将来案を混ぜない。
- Lifecycle/operator: 入力・確定・取消・再試行・詳細・戻りの実経路を追い、図で途切れを隠さない。
- Reporting/data safety: 商品別明細と公式集計の意味を分離し、NULL参照・冪等性・stock と movement の関係を確認する。
- Manual/environment: 実店舗情報は扱わず、図の表示だけを browser で確認する。製品の Windows / レジ実機検証を代替したとしない。

## Contract Probe

R2の図面・test-only検証。外部環境の挙動を製品契約へ昇格させないためR3/R4 Contract Probeは非対象。図のrenderと明示診断の非0終了は実際に検証する。

## Test Plan

- メモリ内 SQLite の schema introspection と図の entity / column / FK を比較する一時検証。production migration の制御コード全体を実行したとは主張しない。
- navigation / concrete route と図面対応表を集合で比較し、主要矢印は実 source で確認する。
- Mermaid render と可視確認、リンクの実在、doc checker、diff check、変更分類に従う local gate。
- 製品の不具合が疑われた場合、既存テストと実経路を調査し、必要なら作業用の合成データによる再現検証を行う。製品テストや仕様を弱めない。
- 追加の実行条件は [Test Design Matrix](test-matrices/2026-09-16-cross-feature-model-audit.md)。通常 `cargo test --lib cross_feature_tests` と、明示診断 `cargo test --lib cross_feature_tests -- --ignored --nocapture` を別々に実行する。
- 既存テストが同じものを保護するか調査したうえで、主目的を複数機能・時点の組合せに限定する。件数・所要時間は未実測。

## Boundary / Wire Contract

既存契約の説明だけで変更なし。物理 FK と app 側の論理関連、在庫単位と売上集計、URL と画面内 state をそれぞれ区別する。

## Review Focus

- 図を読むだけで誤った必須関係・在庫更新・レジ反映・取消機能を想像しないか。
- source と図の差が既存欠陥なのか単なる省略なのか、根拠で区別しているか。
- 健全性を保証する範囲を過大に書いていないか。図を更新し続ける参照先が明確か。

## Data Safety

実 POS / 店舗 CSV、実 DB、backup、log、receipt、secret は読取・記録対象外。メモリ内 SQLite には repository の schema と必要な合成データのみを使う。

## Implementation Results

ERの物理関係と全カラム、現行画面の到達・戻り、在庫・日報・PLU・棚卸しの図を更新した。本文の冪等性列・navigation一覧・個別図も同期し、初期mockupを歴史資料として明示した。

初回の図面更新では実schemaのメモリ内DDL比較、page routeの集合照合、Mermaidのrenderで転記を確認。棚卸しの時点問題は合成DBで実BIZ関数を呼び再現し、在庫照会の戻りは既存hookテストで受け側動作を確認した。初回の一時検証moduleは取り除き、所見・再現方法を `docs/research/2026-09-16-diagram-audit.md` に保持した。

A-1ではtest-onlyのモデルと操作列を追加した。通常suiteと正常対照はPASS、明示診断は数量の不一致によりFAIL（非0）であり、入力不足・fixtureエラーではない。カウント後の移動が消える既知経路に加え、POS販売→カウント→確定→遅いCSVで二重減算する経路を確認した。独立モデルは、在庫と台帳の同時誤り、および総計が一致する売上sourceの取り違えも検出した。具体的な反例と実行方法は `docs/diagrams/cross-feature-verification.md` に記録。runtime修正は未実施で、通常のRust gateとSonnet / Opusの最終レビューを進めた。

[PR #69](https://github.com/higekiri-w/inventory-system-desktop/pull/69) で実装し squash merge 済み（`183af827`、2026-09-16）。Coordinator / Writer = Astra（D-087、codex-only）。Plan Review = Sonnet PASS、A-1 は Opus 指摘全採用 + Sonnet 限定確認 PASS。Final Review = Sonnet PASS / Opus P2 2・P3 4 → 全採用 / Sonnet closure PASS（`ee72ceea`）。merge phase は Fable が代行: 監査 3 本を PR comment に転記して helper record、`origin/main`（wave 11 #70〜#73）単段 merge `a3e44edc`、closure（Sonnet fresh）で P2 1 件〈図面の廃棄保存結果リンクが #71 と矛盾〉→ `6eba236f` で是正、Ready → CI pass → helper merge。Human Gate は ready,merge（manual 非対象）。

## Review Response

- Findings Freeze: frozen after A-1 initial Broad Audit（Sonnet / Opus）。以後は既存指摘の是正確認。
- Plan Review: Sonnet fresh context、PASS（P1/P2 なし）。軽微な補足提案は下記 narrative で採用した。再レビュー不要の任意補足で、契約・Scope の変更はない。

### Plan Review の補足と実装への移行

2026-09-16: Sonnet の独立 Plan Review PASS を受領。`plan-gate → plan-approved → implementing` をこの順で記録する。実装前の Plan Commit を保持し、通常作業は D-087 の一貫担当で行う。

| reviewer の補足 | 採用した根拠・対応 | 検証の対応先 |
|---|---|---|
| 個別図の具体的な転記不一致 | 55 §55.8 は任意 state の reset が未図示。56 §56.3 の返却構造、57 §57.3 の派生値名は現行 hook と異なる | AC-2/4、図と reducer / hook を直接照合 |
| task 一覧の欠落の特定 | ui-task-specs UI-12 の旧一覧は一括価格改定・入出庫履歴・取引先管理を欠く。ARCHITECTURE の UI 一覧も UI-14/15 を欠く | AC-2、navigation / route の対応表 |
| Scope と検証の対応 | ER・DB本文 = AC-1、画面・task・個別図 = AC-2、業務フロー = AC-3、audit = AC-4、入口・旧図注記・表示 = AC-5、独立レビュー = AC-6 | Test Plan の一時検証・doc gate・render・Final Review |

### 横断業務検証への拡張依頼

2026-09-16: owner が「状態を持つ処理のテストを作って確かめる」「複数機能を跨ぐ実際の業務の検証」を依頼し、追加でOpusレビューを指定した。新しいテストのoracle・範囲・既知不具合の扱いを設計するため、`implementing → design` へ戻す。既存の図面更新と検証履歴は保持する。追加の実装はGated Amendmentの独立Plan Gate通過後に開始する。

### Gated Amendment A-1: 横断業務モデル検証

ownerの上記依頼を範囲拡張の承認として継承し、XFA-D1〜D5とMatrixを設計した。`design → plan-draft → plan-gate` としてOpusの独立レビューへ提出する。追加はtest-only moduleと図・監査結果、traceabilityに限定し、既存製品の修正やgate緩和は含めない。従来の図面成果は保持。原Plan Commitは書き換えず、本改訂commitをAmendmentsへ追記してから実装へ進む。最終レビューはowner指定によりSonnet + Opusとする。

Opusの初回Plan Reviewは実行時間上限で終了し、判定未受領。同vendorのfresh contextで追加A-1の範囲だけを再確認する。Phaseはplan-gateのまま維持し、未受領を承認に読み替えない。

Opusの再実行でCHANGES_REQUESTEDを受領（P1なし、是正後は追加の全面レビュー不要との条件付き判断）。P2の売上source別oracle、棚卸し確定のfixture前提、診断のcoverage扱いをすべて採用し、XFA-D1/D2/D4とMatrixへ反映した。P3の既存tests/配置とCachedPreview helper共用も採用した。Sonnetのfresh contextでこの是正に限ったPlan Gateの確認を行う。最終レビューのSonnet + Opus要件は維持する。

A-1 Plan Gate: Sonnetの独立した限定是正確認で全指摘の解消と実装開始可のPASSを受領。`plan-gate → plan-approved → implementing` を順に記録し、原Plan Commitを保持したままA-1と是正commitをAmendmentsへ追記する。追加Rust実装はこの記録より後に開始する。

A-1 Final Review: SonnetはPASS、Opusは索引とREQ帰属のP2および説明補足のP3を提示した。全て採用し、STK-2の独立追跡、mutationテストのREQ-904/203への改名とcanonical再生成、測定コマンドとPR/CI証拠への参照、固定tail・時点比較範囲の明記、システム管理ラベル同期を行う。検査条件・製品契約は変更しない。数値の正本はEvidence Ownershipに従ってPR本文/CIとし、最終の限定是正確認へ進む。

### merge phase（2026-09-16、Fable 代行）

- Final Review pass A（Sonnet、broad、Astra session で実施、転記）: PASS（P1/P2/P3 なし）。https://github.com/higekiri-w/inventory-system-desktop/pull/69#issuecomment-5695998506
- Final Review pass B（Opus、broad、Astra session で実施、転記）: CHANGES_REQUESTED（P1 0 / P2 2 / P3 4）→ 全件採用して是正済み。https://github.com/higekiri-w/inventory-system-desktop/pull/69#issuecomment-5695998796
- Final Review closure（Sonnet、Astra session で実施、転記、head `ee72ceea`）: PASS。https://github.com/higekiri-w/inventory-system-desktop/pull/69#issuecomment-5695999081
- closure（Sonnet fresh context、base 同期後、head `6eba236f`）: `a3e44edc` = `ee72ceea` + `origin/main`（wave 11 の #70〜#73）の単段 merge で #69 側 file への手編集なし、意味的衝突・conflict marker なし。P2 1 件（`docs/diagrams/current-system.md` L107 / L140 が「廃棄は保存結果から直接リンクなし」のままで #71 の link と矛盾）→ `6eba236f` で 2 行是正し再確認 pass。review = pass（broad 2 @ `ee72ceea` + closure @ `6eba236f`）、manual / R4 = not-required（Human Gate は ready,merge）。https://github.com/higekiri-w/inventory-system-desktop/pull/69#issuecomment-5696084118
- 以後 Ready → CI pass → helper merge（squash `183af827`）。
