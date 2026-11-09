# Plan Packet: helper の REPO を GitHub の今の名前にし、旧名の record を読むときだけ受ける

wave に属さない単独の lane。merge の helper（`scripts/pr-gate.py`）の repository 名とその test、正本の文（`docs/agent-guidance/merge-evidence.md` の RecordV1、`docs/decision-log.md`）を変える。製品の runtime（`src/**`・`src-tauri/**`）・CI の job・classifier は変えない。

出所: 2026-11-09 の環境の作り直しの点検。owner の事実 TD-598（2026-11-04 の旧 WSL の感染で環境を作り直し、GitHub の user 名を `kosei-w90607` → `higekiri-w` に変えた）と、owner の決定 TD-599（この PR は一回だけの例外として owner が直接 merge する）・TD-600（旧名の record は読むときだけ受け、次に書くときに今の名前で書き直す）・TD-601（この lane を旧パスの書換えより先に進める）。どれも repo 外の回答台帳。

decision-log の番号: 本 lane は D-150 を予約する（起票の時点で open の PR が足す D は D-148〈#227〉・D-149〈#228〉。`git diff origin/main...origin/agent/<branch> -- docs/decision-log.md` の `+## D-` と、各 branch の `docs/plans` の差分の「予約」で確かめた）。

## Workflow State

- Risk: R3
- Plan Commit: pending
- Amendments: none

## Risk

Risk: R3

Reason:
merge の関門（helper の repository の照合と record の受理）を変える。classifier が `workflow=true` を出す `scripts/**` の変更は R2 にできない（`docs/DEV_WORKFLOW.md` Risk Tiers）。R4 にしない理由: 破壊的な操作・実データ・secret・不可逆な git の操作を伴わない（GitHub への書込みは今の `record`・`ready`・`merge` の経路のまま）。

## Goal

Goal Invariant:

### 最小完了条件

- origin が `https://github.com/higekiri-w/inventory-system-desktop.git` の checkout で、helper の `status` が `origin repo mismatch`・`PR repo/base mismatch`・`invalid workflow record author` で止まらず、open な PR の状態と阻害理由を出す。
- `repo` に旧名 `kosei-w90607/inventory-system-desktop` を持つ record がある open な PR で、helper が `record belongs to another repo/PR` で止まらずに record を読み、次にその record を書くときは `repo` を今の名前で書く。
- この PR の merge の後に起票・同期した PR で、helper が通常どおり（D-099 の base の版の照合を含め）動く。この PR 自身は D-150 (3) の一回だけの手順で merge する（下の「この PR の merge の手順」）。

### 失敗定義

- 旧名と今の名前のどちらでもない `repo` の record を受ける。
- origin の URL・PR の base/head の repo・CI の `head_repository` の照合で旧名を受ける（旧名を他人が取った後の repo を本物として扱いうる）。
- 旧名の record を書くときに旧名のまま書き戻し、旧名の受け入れが record の続く限り要る依存になる。
- D-150 (3) の直接の merge がこの PR の外に使われる。

### 非目的

- docs・`CLAUDE.md`・`.codex/**`・`docs/project-memory.md` の旧名の link と旧パスの書換え（後続の lane。TD-601 の順）。
- 旧名 `kosei-w90607` を GitHub で確保するか（owner の判断、未決）。
- 旧名を受ける条件を消すこと（D-150 の Revisit の条件を満たした後の別の lane）。
- RecordV1 の `version` を上げること、record の他の field の変更。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Scope

- `scripts/pr-gate.py`:
  - `REPO` を `'higekiri-w/inventory-system-desktop'` にする。`OWNER` は今のまま `REPO` から導く。
  - `LEGACY_REPO = 'kosei-w90607/inventory-system-desktop'` を置き、`ponytail:` comment に受ける範囲（`validate_record` の `repo` の照合だけ。書く record はどれも `repo=REPO` で組む）と消す条件（D-150 の Revisit）を書く。
  - `validate_record`: `value['repo']` が `REPO` か `LEGACY_REPO` なら受ける。他は今のまま `record belongs to another repo/PR`。
  - `Gate.record`: 変えない。旧名の record を今の名前で書き直すのは版が変わった分岐で、そこは今も `dict(version=1, repo=REPO, ...)` で組み、旧 record から broad（`plan_same`）と manual（`carry_manual`）を引き継ぐ。版が同じ分岐（`record = copy.deepcopy(old)`）に旧名の record で着くことは無い。旧名の record はどれもこの PR の merge の前の base を持ち、新しい helper はこの PR の merge の後の base の版でだけ動く（D-099）ので、`(old['head'], old['base'])` が今の head/base と一致しない。書く時の `validate_record`（`Gate.record` の書込みの前）も旧名を受けるが、書く record はどれも版が変わった分岐の `repo=REPO` で組むので、書く時の guard は足さない（D-147。保証は AC3 の test が書いた JSON で pin する）。
  - 変えない所: `Gate.__init__` の `--repo` と origin の URL の照合、`Gate.pr` の base/head の `full_name`、`Gate.ci` の `head_repository`、`snapshot` の `repo`、`gh pr ready`・`gh pr merge` の `--repo`、`record` の書き手の照合（`api('user')['login'] == OWNER`）と `server_record` の author の照合（`item['user']['login'] == OWNER`）。どれも今の名前だけを受ける。
- `scripts/tests/pr-gate.test.py`: 下の Test Plan の 7 本を足す。今の fixture は `g.REPO`・`g.OWNER` で値を引くので書き換えない。
- `docs/agent-guidance/merge-evidence.md`: RecordV1 の直後の段落に、`repo` の意味と旧名を読むときだけ受けることの 1 文（本 commit）。
- `docs/decision-log.md`: 末尾に D-150（本 commit）。
- 本 packet（本 commit）。

呼出し側・隣接の確認（`grep -n -E "\bREPO\b|\bOWNER\b" scripts/pr-gate.py` と `git grep -n -E 'pr-gate\.py|pr_gate' -- scripts .github` で確かめた）: `REPO`・`OWNER` を使うのは `scripts/pr-gate.py` の中だけ。`scripts/check-workflow-git.sh`・`scripts/doc-consistency-check.sh`・`scripts/plan-approve.sh`・`scripts/tests/doc-consistency-plan-packet.test.sh` は helper を module として読み、`plan_origin`・`active_packets`・`packet_form`・`parse_packet`・`workflow_fields` だけを使う（本 lane はそれらを変えない）。`scripts/tests/pr-gate.test.py` は fixture の全部で `g.REPO`・`g.OWNER` から値を引き、repository 名を直書きしない（`grep -n -E "kosei-w90607|higekiri-w" scripts/tests/pr-gate.test.py` が 0 件）。`.github/**` は `REPO` を持たない。

Non-scope: `AGENTS.md` Working Rules の「merge は helper 経由」「直接UI mergeは禁止」の文と `docs/agent-guidance/merge-evidence.md` の手順は変えない（D-150 (3) はこの PR だけの例外で、規則に条件を継ぎ足さない）。`.local/pr-gate/` の旧い capture（旧環境の写し）は capture の照合で古いと分かるので触らない。

### この PR の merge の手順（D-150 (3)、TD-599）

helper は PR の base の版でだけ動き（D-099）、base の版は旧名のままなので、この PR では `status`・`record`・`ready`・`merge` のどれも通らない。次の順で進める。

1. 計画の review 2 本（座組表の Plan Reviewer。Codex が使えないときは §3.3）→ P1/P2 0 → `scripts/plan-approve.sh`。
2. 実装（Writer）→ push → この head で hosted の `Merge gate` が成功。
3. Final Review 2 本（座組表の Final Reviewer。gate を変える変更なので、Codex が使えないときは Claude 側を済ませ、Codex の枠が済むまで Ready に進まない）。結果は PR の comment に置く（helper の record は書けない）。Final Review の後に head が変われば、その head で closure の監査（座組表の closure の担当）を受け、P1/P2 0 になってから 4 へ進む。
4. merge の直前に、この PR の head の checkout で、helper の `Gate.pr`・`Gate.rules`・`Gate.ci` を read-only で当てる（`scripts/pr-gate.py` を module として読み、`--pr <N>` の引数で `Gate` を作って 3 つを呼ぶ。`Gate.requirements` の base の版の照合を通らない経路。gh の認証が要るので owner の terminal で流し、command と出力を Coordinator が PR の comment に書く）。`Gate.rules` は main の実効の保護（ruleset の strict な `Merge gate` の required check・bypass なし・方針の file との一致）、`Gate.ci` は同じ head の最新の `ci.yml` の run と `Merge gate` の check の成功を確かめる。どれかが止まる・確かめられないときは merge しない。`Gate.pr`・`Gate.rules`・`Gate.ci` がこの PR で `REPO` の値の外は変わらないことは Final Review で確かめる。
5. owner が Ready にし、`gh pr merge <N> --repo higekiri-w/inventory-system-desktop --squash --match-head-commit <head>` で merge する。`<head>` は、Final Review 2 本（closure があれば最後の closure）の `REVIEWED_HEAD` と、4 で `Gate.ci` が成功を確かめた run の `head_sha` が全部一致する 40 桁の SHA に限る。一致しなければ merge しない。Coordinator は merge の前に、この一致を PR の comment に書く（helper の review の照合の代わり）。
6. merge の後、Coordinator が更新した main の checkout で open な PR ごとに `python3 scripts/pr-gate.py status --pr N ...` を流し、AC6 を確かめる。

## Acceptance Criteria

- AC1: `python3 scripts/tests/pr-gate.test.py` が全 PASS。既存の test を消さず弱めない。
- AC2: `validate_record` が `repo` = `LEGACY_REPO` の record を受け、`repo` = `someone/inventory-system-desktop` と `kosei-w90607/other` の record を `record belongs to another repo/PR` で拒む（`test_record_repo_accepts_legacy_name_on_read`。拒否の例は subTest）。
- AC3: 同期の merge で head と base の両方が変わった（取り込んだ main 側の変更が docs だけで code に触れない）PR で、server の record が旧名で broad と manual の pass を持つとき、`--review-stage sync` の record を書くと、書いた comment の JSON の `repo` が `REPO` になり、broad と manual（`source_head`）が引き継がれる（`test_record_rewrites_legacy_repo_on_sync`。fixture は `SyncRecords` の `test_sync_stage_carries_manual_and_closure` と同じ形で、`docs_sync()` の後に旧名にした server record へ `--review-stage sync`）。この unit の経路は、この PR を取り込む実際の同期（code に触れる同期。AC6）とは別で、書き直しが版の変わった分岐で起きることを固定する。
- AC4: origin の URL が旧名なら、受理の 4 つの形（`git@github.com:…git`・`git@github.com:…`・`https://github.com/….git`・`https://github.com/…`）のどれでも `Gate` が `origin repo mismatch` で止まる（`test_origin_legacy_url_rejected`。形ごとの subTest）。PR の base だけ・head だけの `full_name` が旧名なら、それぞれ `PR repo/base mismatch` で止まる（`test_pr_legacy_full_name_rejected`。base と head を別の subTest）。
- AC5: 旧名の login と旧名の repo を、author・書き手・CI の照合が拒む。login が `kosei-w90607` の MARKER の comment は `invalid workflow record author`（`test_record_author_legacy_login_rejected`）、`user` の login が `kosei-w90607` なら `record writer must be repository owner`（`test_record_writer_legacy_login_rejected`）、CI の run の `head_repository.full_name` が旧名なら `wrong CI workflow/source`（`test_ci_legacy_head_repository_rejected`）。
- AC6（merge の後、上の手順の 6）: 2026-11-09 に open な PR（#194・#216・#227・#228・#229・#235・#236・#237）の `status` が、`origin repo mismatch`・`PR repo/base mismatch`・`invalid workflow record author`・`record belongs to another repo/PR` のどれでも止まらない。base が進んだことによる阻害理由（`stale head/base in workflow record`）は今の helper の規則どおりに出てよい。base が進んでいない PR が `helper differs from base` で止まれば、同期の push で base を進めてから確かめ直す。この PR が main に入れる `scripts/pr-gate.py`・`scripts/tests/pr-gate.test.py` は classifier で `rust`・`frontend`・`env` に当たる（`printf 'scripts/pr-gate.py\n' | bash scripts/ci/classify-changes.sh --files-from-stdin` で確かめた）ので、各 lane のこの PR を取り込む同期は code に触れる同期で、`--review-stage sync` と manual の自動の引継ぎ（MG-D13 の A）は使えない。各 lane の次の一手は、`git -c merge.directoryRenames=false merge origin/main` → push の後、broad が無い lane は今の流れのまま、broad が pass の lane は今の head の監査の closure（座組表の closure の担当）、manual が pass の lane は MG-D13 の B（同期の closure 2 本と owner の承認の後、最初に manual の使い回しを record）か L3 のやり直し。
- AC7: `bash scripts/doc-consistency-check.sh` と `--target plan` が ERROR なし。`rg -n "kosei-w90607" scripts/pr-gate.py` が `LEGACY_REPO` の行と、その `ponytail:` comment だけに当たる。
- AC8: 実装の報告に、各 test が次の mutation で red になることを書く。AC2: `validate_record` から `LEGACY_REPO` を外す、受理を `kosei-w90607/` の prefix の一致へ緩める。AC3: 版が変わった分岐の `repo=REPO` を `repo=old['repo']` にする。AC4: origin の受理の形のどれか 1 つに旧名を足す、base か head の `full_name` の照合に `LEGACY_REPO` を足す。AC5: author・書き手の照合に旧名の login を足す、CI の `head_repository` の照合に `LEGACY_REPO` を足す。

## Design Readiness

- 引用する設計正本（節まで）: `docs/agent-guidance/merge-evidence.md` の「状態と非CI記録」の RecordV1 と、owner 名義の専用 comment の段落、「Helperの境界」の base の版の照合（D-099）。`docs/decision-log.md` D-099。`AGENTS.md` Working Rules の merge の項目。
- 必要な設計成果物: merge-evidence の RecordV1 の後の 1 文と D-150 を本 commit で書く（updated in this PR）。helper の他の契約は existing sufficient。
- plan にしかない durable な判断の昇格先: TD-599・TD-600 の判断を D-150 へ。旧名を受ける範囲の規則の文を merge-evidence へ。
- 前提・制約と延期した design gap:
  - P1: 名前を変えた後も、GitHub の API が旧名の時に書いた comment の `user.login` を今の名前（`higekiri-w`）で返す。`server_record` の author の照合はこれを前提にする。確かめ: `gh api repos/higekiri-w/inventory-system-desktop/issues/227/comments --jq '.[].user.login'`（owner の terminal。sandbox の中では gh を使えない）。plan の固定の前に結果を PR の comment に置く。
  - P2: PR の API が base/head の `repo.full_name` を今の名前で返し、CI の run の API が `head_repository.full_name` を今の名前で返す（名前を変える前の run を含む）。確かめ: `gh api repos/higekiri-w/inventory-system-desktop/pulls/227 --jq '.base.repo.full_name, .head.repo.full_name'` と `gh api 'repos/higekiri-w/inventory-system-desktop/actions/runs?per_page=5' --jq '.workflow_runs[].head_repository.full_name'`。P1 と同じ扱い。
  - 旧名の record を書き直すのは、その PR で次に `record` を書くとき。書かれない PR の record は旧名のまま残り、旧名の受け入れはそれまで要る（D-150 の Revisit）。
- 絶対保証の例外と抜け道: 旧名の受け入れは record の `repo` の文字列だけ。`Gate.server_record` は author の照合（今の login）を `validate_record` より先に当て、comment はこの repo の issue の comment だけから取るので、他人の comment を record として受けることにならない。この順と、author・書き手・CI の照合が旧名を拒むことは AC5 の test で固定する。D-150 (3) の PR では、helper の review・manual の機械の確かめが無く、上の手順の 3〜5（closure、head の helper の `Gate.pr`・`Gate.rules`・`Gate.ci`、head の一致）を Coordinator と owner が PR の comment で確かめる。
- 判定: ready。正本の該当節は 2 か所と新しい D 1 件で、helper の変更は 1 つの定数と 2 か所の条件に留まる。P1・P2 は plan の固定の前に確かめる。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| RecordV1 の `repo` | merge-evidence「状態と非CI記録」の RecordV1 と直後の段落、D-150 (2) | `validate_record` の旧名の受理（書き直しは今の版が変わった分岐のまま） | `test_record_repo_accepts_legacy_name_on_read`・`test_record_rewrites_legacy_repo_on_sync` | 非対象 |
| origin・PR・CI の repository の照合 | merge-evidence「Helperの境界」、D-150 (1) | `REPO` の値だけを変え、照合の形は変えない | `test_origin_legacy_url_rejected`・`test_pr_legacy_full_name_rejected`・`test_ci_legacy_head_repository_rejected` | 非対象 |
| owner 名義の専用 comment（author と書き手） | merge-evidence「状態と非CI記録」の専用 PR comment の段落 | 変更なし（`OWNER` は `REPO` から導く） | `test_record_author_legacy_login_rejected`・`test_record_writer_legacy_login_rejected`、今の `login='attacker'` の test | 前提 P1 |
| helper の base の版の照合 | merge-evidence「Helperの境界」、D-099 | 変更なし | 今の `helper differs from base` の test | この PR の merge は D-150 (3) |
| merge は helper 経由・直接 UI merge の禁止 | AGENTS.md Working Rules、D-150 (3) | 変更なし（この PR だけの例外） | 非対象（手順） | 「この PR の merge の手順」、AC6 |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| 旧名の record を読む | 旧名の record がある open な PR で `status`・`record` が `record belongs to another repo/PR` で止まる。逆に、無関係の repo の record を受ける | unit（`validate_record`。拒否の例は subTest） | `test_record_repo_accepts_legacy_name_on_read` | `LEGACY_REPO` を受理の集合から外す。受理を末尾の名前（`inventory-system-desktop`）か `kosei-w90607/` の prefix の一致へ緩める |
| 旧名を書き戻さない | 同期の後の最初の record で旧名が残り、旧名の受け入れが永続の依存になる。旧名の old から broad・manual の引継ぎが落ちる | unit（`SyncRecords`、合成 repo。docs だけの同期で head と base が変わった後、旧名の server record に `--review-stage sync`） | `test_record_rewrites_legacy_repo_on_sync` | 版が変わった分岐の `repo=REPO` を `repo=old['repo']` にする。`plan_same`・`carry_manual` の引継ぎを旧名で落とす |
| origin の照合で旧名を受けない | 旧名を指したままの clone が helper を通り、旧名を他人が取った後の repo を本物として扱う | unit（`Gate.__init__`、合成の git repo の origin。受理の 4 形を subTest） | `test_origin_legacy_url_rejected` | origin の受理の形のどれか 1 つに旧名を足す |
| PR の repo の照合で旧名を受けない | 旧名の full_name を返す PR を本物として扱う | unit（`Gate.pr`、合成の API の応答。base だけ・head だけを subTest） | `test_pr_legacy_full_name_rejected` | base か head の `full_name` の照合に `LEGACY_REPO` を足す |
| record の author で旧名を受けない | 旧名を取った他人の MARKER の comment を record として受ける | unit（`Gate.server_record`、合成の comment） | `test_record_author_legacy_login_rejected` | author の照合に `kosei-w90607` の login を足す |
| record の書き手で旧名を受けない | 旧名の login で record を書く | unit（`Gate.record`、合成の `user` の応答） | `test_record_writer_legacy_login_rejected` | 書き手の照合に `kosei-w90607` の login を足す |
| CI の source で旧名を受けない | 旧名の repo の run を Merge gate の根拠にする | unit（`Gate.ci`、合成の run） | `test_ci_legacy_head_repository_rejected` | `head_repository` の照合に `LEGACY_REPO` を足す |
