# Review Packet（Plan Review・Final Review・差分の review）

独立レビューへ、下の対象情報を埋めて渡す。手順・Risk・承認は [DEV_WORKFLOW.md](../DEV_WORKFLOW.md)、重大度と裁定は [code_review.md](../code_review.md) を正本とし、ここに複製しない。

## Role

read-only の独立 reviewer。tracked file の編集、patch適用、git/PRの変更、範囲外の清掃は禁止。subagent を起動しない（depth 1）。作者の説明とvalidationはclaimとして、対象の正本・差分・実行結果で確認する。初回監査か既存findingのclosureかを区別する。まず検出範囲を確保し、correctness・契約・テスト・文書drift・互換性・データ安全の問題を軽微さや不確実さだけで黙って落とさない。採否は既存の裁定へ渡す。合格条件がP1/P2なしのgateで、P2を残したままpassにしない。

## Target

- Risk / stage（計画の review / Final Review / R2 の差分の review / closure）:
- Plan Packet（R3/R4）。R0〜R2 は PR body の 5 行:
- Source design / critical contracts:
- Contract Ledger（旧 packet は Contract Coverage Ledger と Test Design Matrix）:
- 対象差分（branch と範囲）:
- 初回監査 / closure、既存findings:
- Claimed validationと必要な証拠の場所:
- 読ませないもの（同じ round で 2 本以上の review を回すときの各 reviewer。Plan Review・Final Review とも、Double Audit を含む）: 他の reviewer の結果が置かれた場所すべて（例: PR body、PR の comment / review、Coordinator が保存した local の報告 file、是正 commit の件名・本文。旧形式の packet では `## Review Response` の段落も）。その発注の本文にも他の reviewer の結果とそれ由来の観点を書かない（closure・是正の発注は前回 findings から始めてよい）。[AGENT_OPERATING_MANUAL.md](../AGENT_OPERATING_MANUAL.md) §3 の独立性の項

## 報告の先頭の 1 行

読んだ版を報告の 1 行で返す: `REVIEWED_HEAD=<40 桁の小文字 hex>`（`git rev-parse HEAD` の値。1 行だけ）。Coordinator は `python3 scripts/pr-gate.py record --report <報告の file>` で記録し、helper が capture の head と照合する（D-145 (10)）。

## 計画の review の固定の問い

計画の review（Plan Review）では packet の節の代わりに次を必ず答える（[DEV_WORKFLOW.md](../DEV_WORKFLOW.md) Review Rules）。

1. 普通の一日: 計画の通常の操作列（初期状態 → 操作 → 利用者が得る結果、翌日の起動・再試行を含む。workflow の変更は 発注 → 停止と訂正 → review → owner の判断）が目的を達するか。冒頭 1 行で `成立 / 具体的な反例あり / 外部前提が未確認` と答え、不成立なら最短の入力と状態遷移と、期待した結果との差を示す。「正常な条件で目的を達するか」と「危険な結果を出さないか」を別に答える。
2. 登録と生成: Tauri command の登録と bindings の再生成、function-design の新設と対応表、REQ の traceability の再生成、route の生成、operator 画面の到達導線（navigation と到達 test）、文書の新設・改名の索引の義務が Scope と Contract Ledger に漏れていないか。
3. 外の前提: 確かめていない外の前提（library・OS・機器・外の tool・CSV/TSV の形）と、それを確かめる最小の実験。JSON・CSV・設定・cache・command の DTO・生成 bindings に触れるなら producer・consumer・型・精度と範囲・往復・不正な入力・互換。
4. データの安全: 店の実データ・secret を commit しない、local だけ・合成だけの置き場が分かれているか（R4 は packet の `## Data Safety` と照らす）。
5. 前提の出所: 計画の事実と数が実測（command と出力）か出所付きか。owner の要望を AI の要約で裁定していないか。
6. Risk: R2 以下として出された差分では、Risk の判定が正しいか（状態を持つ画面・migration・依存の追加・契約の変更でないか）。

## closure の固定の問い

是正で足した契約文と、是正で見送った経路を全部挙げ、それぞれが今の版で成り立つかを答える（[code_review.md](../code_review.md)）。

## Contract Audit

R3/R4 は [Contract Audit](../DEV_WORKFLOW.md#contract-audit-r3r4) をsource docsから実施する。Ledgerの行が存在するだけでなく、実装とテストが契約に合うこと、未記載の契約、状態遷移、隣接patternの移植漏れ、test oracleの検出力を確認する。実mutationによるred確認は隔離fixtureで行い、tracked成果物を変更しない。自動化できない確認は L3 の条件に従い、対象画面・到達手順・観測可能な合格基準を示す。

状態遷移・証跡・Readyの変更を扱う場合は、[Workflow State](../DEV_WORKFLOW.md#workflow-state) と [ci.md](../ci.md) の該当契約を直接確認する。専用record/head/base・必要broad/closure/manual/R4・実効rules/CIを確認し、廃止済みの SHA 照合を要求しない。

closureは前回指摘と修正差分、影響する契約から確認する。新しい影響や重大欠陥の根拠があれば拡張し、その理由を示す。既読で変更のない全資料の再読を独自に要求しない。Double Audit、Findings Freeze、review round上限は正本に従う。

## Output

findingの書式と重大度は [code_review.md](../code_review.md) の `## Output Shape` と `## Finding Severity` に従う。
