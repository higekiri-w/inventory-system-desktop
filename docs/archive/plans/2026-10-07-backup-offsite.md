# Plan Packet: backup の PC の外の控えと復元（USB メモリへ自動で写して確かめ、復元の前に控えを確かめる）

wave に属さない単独の lane（design-first、docs だけ。runtime は後続の lane）。2026-10-07 の第 1 段の 3 lane（`agent/stocktake-p1-3`・`agent/unit-extension`・本 lane）の 1 つで、ほかの並走は `agent/npm-audit-1006`（PR #146、package 依存）・`agent/plu-clear`（D-110）。owner の決定は 2026-10-07（repo 外の回答台帳 TD-190 の Q4「PC の外の控えは外付けの保存先へ自動で書き、確かめて画面に出す」）。本 lane の branch は `agent/backup-offsite`。

## Workflow State

- Phase: archive
- Risk: R4
- Plan Commit: 962731ec73db29d0dc9a5a1fdc50674888f3aa7e
- Amendments: e6158aa673a5982c20545e5157a25af7a8be8b01, 7af3804348eb93d6b2f20ba33ac18484ca164350
- Coordinator: Opus 5.5 main session
- Writer: Opus 5.5 subagent（subagent_type: writer）
- Plan Reviewer: fresh Opus 5.5 + Codex（model は発注時に決める）
- Final Reviewer: Fable 5.1（Claude 側、R3 以上）+ Codex（GPT-6.1 Sol 既定）。座組表（docs/AGENT_OPERATING_MANUAL.md ## 座組）どおり
- Final Review Minimum: 2
- Human Gate: ready,merge,r4
- Branch: agent/backup-offsite

遷移の記録:

1. kickoff → spec-check（2026-10-07、起草役）: owner 決定（TD-190 の Q4）と backlog の 2 項目（「backup に、PC の外のコピーと復元の実証が運用として設計されていない」と「保存と起動の守りの follow-up」の (2)〈新しすぎる版の backup の復元の文言〉）を Scope にし、Risk を R4 と記録した（下の Risk）。
2. spec-check → design（2026-10-07、起草役）: 正本（`docs/function-design/71-mnt-backup.md`、`68-ui-backup-restore.md`）に、PC の外の保存先・書けたかの照合・PC の外の成功の表示・復元の予行演習・新しすぎる版の文言が無く、`VACUUM INTO` が正式名へ直接書くので作成途中の file を成功の世代に数えうる。同じ commit で設計正本を更新した（下の Design Readiness）。
3. design のまま止めた（起草の時点、2026-10-07）: owner の判断事項 1〜3（下の「owner の判断事項」）が残り、design → plan-draft の条件「未解決の設計の問いが無い」を満たさない。アプリの仕組みは判断事項 1 の選択肢のどれでも同じに設計したので、決定が設計正本を変えるのは D-114 の Status・71 §71.11.6 の採った型・本 packet の Ordinary Operation と Data Safety だけの見込み（判断事項 2 で写しを含めると決めた場合は 71 §71.1・§71.11 の写す対象も変わる）。
4. design → plan-draft（2026-10-08、起草役）: owner が判断事項 1〜3 に答え（repo 外の回答台帳 TD-196・TD-197）、聞く事実 P5・P6 にも答えた（TD-198・TD-199）。決定を設計正本（71 §71.1・§71.11.6・§71.11.7・§71.13、68 UI-11b-L3-8、`docs/SCREEN_DESIGN.md` の 1 日の動線）、D-114、D-111（未決 B を一部置き換えた `Superseded in part by: D-114` の行）、`docs/backlog.md` に反映した。条件（`docs/DEV_WORKFLOW.md` Workflow State の表: design の出力が source docs にあり、未解決の設計の問いが無い）を満たす。P6 の帰結を和らげる画面の振舞い（持ち帰りの 1 本の古さを出す）は設計に足さず、owner に諮る提案として Coordinator へ渡した。今の設計はその振舞いを持たないと明記してあり（71 §71.11.6・§71.11.7、D-114 の Guarantee range と Revisit）、未解決の問いではなく範囲の追加の提案として扱う。owner が採れば design へ戻る。plan-gate へは Coordinator が進める。
5. plan-draft → plan-gate（2026-10-08、Coordinator、本 commit）: 起草役が最終報告で owner に諮る提案とした「持ち帰りの 1 本の古さを和らげる画面の振舞い」（(a) 足さない／(b) 媒体ごとの最後に写した日を出す／(c) 注意も出す）を owner に諮り、(a) 足さないに決まった（repo 外の回答台帳 TD-202。差し込めば 60 秒の確認で写す今の設計〈71 §71.11.4〉のままでよい）。設計は変えず、Non-scope と D-114 の Revisit の記述のとおり。packet と Matrix（`docs/plans/test-matrices/2026-10-07-backup-offsite.md`）が揃い commit されている（`docs/DEV_WORKFLOW.md` Workflow State の表）。AC1〜AC6 を plan-gate の直前に逐語で再測し一致（AC1 `103:`・`111:`・`706:`・`747:`、AC2 68 `114:`〜`116:` と 53 `213:`、AC3 `207:`〜`210:`、AC4 `939:`、AC5 3 本とも出力なし・exit 1、AC6 出力なし）。Draft PR で Plan Review（fresh Opus + Codex）に出す。
6. plan-gate のまま是正（round 1、2026-10-08、Coordinator）: round 1 の reject を起草役が `7168aae9` で直した（下の Review Response）。設計の方向は変えないので plan-gate に留め、round 2 で再 review する（`docs/DEV_WORKFLOW.md` Workflow State「a plan-gate rejection corrected in place stays at plan-gate」）。Contract Probe の P1 の結果を記録するまで plan-approved へ進めない。
7. plan-gate のまま是正（round 2、2026-10-08、Coordinator）: round 2 の reject を起草役が `f3120ed4` で直した（下の Review Response）。設計の方向は変えないので plan-gate に留め、round 3（上限）で再 review する。Contract Probe の P1 の結果を記録するまで plan-approved へ進めない。
8. plan-gate（round 3、上限、2026-10-08、Coordinator）: round 3 は両者 reject（P1 0）。`docs/DEV_WORKFLOW.md` Review Rules の round 天井に達したので round 4 は回さず、残った findings を disposition「同型指摘の一括是正」とした（どれも正本の書き足りない所を埋める是正で、Goal と設計の方向を変えない）。起草役が `b4c77d98` で直し、Coordinator が現物で確かめた（下の Review Response）。plan-approved は Contract Probe の P1 の結果の記録と owner の承認を待つ。reviewer の再確認は Final Review。
9. plan-gate のまま再開（2026-10-22、Coordinator）: owner がこの lane の再開を選んだ（repo 外の回答台帳 TD-372）。Writer が `origin/main`（`c98fe7dd`）へ同期し（merge `4d702174`、衝突は `docs/backlog.md`・`docs/decision-log.md`）、後から main に入った契約（D-118 (a) の v9、原価の lane C の vB、D-123〜D-127）との食い違いを新しい仕組みを足さずに直した（`91399121`: 71 MNT-01-D9 の保証の範囲、22 §16 の folder の外の場所 (3) と作り直しの手順 3、D-114 の Status、71 §71.10 の引用、AC3・AC4・AC6 の期待の測り直し）。Contract Probe の P1 は確認済み（repo 外の回答台帳 TD-375）。同期で足した契約の文は round 3 の review の後なので、plan-approved の前に fresh の reviewer の closure に出す。plan-approved は closure と owner の R4 の承認を待つ。
10. plan-gate → plan-approved（2026-10-22、Coordinator、本 commit。state だけ）: 同期の closure（fresh Opus 5.5）は reject（P1 0 / P2 1 / P3 5）で、Writer が `962731ec` で 6 件を直した。closure 2（fresh Opus 5.5）が approve（P1 / P2 0、P3 4）。P3 4 件は implementing で直し、Final Review で確かめる（下の Review Response）。owner が R4 の計画の承認をした（repo 外の回答台帳 TD-376）。Plan Commit = `962731ec`。
11. plan-approved → implementing（2026-10-22、Coordinator、本 commit。state だけ）: closure 2 の P3 4 件（N1〜N4）の是正を Writer に出す。design lane で docs だけなので、implementing の成果は P3 の是正と Implementation Results で、その後に Final Review（Minimum 2）。
12. implementing のまま Gated Amendment（2026-10-22、Coordinator、本 commit。state だけ）: Final broad（head `3e3350c1`）の reject を Writer が `e6158aa6` で直し、その closure（Fable 5.1・Codex）の reject と owner の決定（repo 外の回答台帳 TD-381: USB の控えは任意、「PC の外の控えをやめる」を範囲へ）を `7af38043` で直した。どちらも Plan Gate の後の契約の変更（新しい command・error・手順）なので `Amendments` に足す。broad の記録は `Amendments` none の時点のものなので、closure でなくこの head で fresh broad（Minimum 2）を取り直す（`scripts/pr-gate.py` の validate_review は broad の Plan Commit と Amendments が packet と同じことを求める）。TD-381 で介入の上限を 10 から 11 に上げた（下の Owner Effort Budget）。

## Owner Effort Budget

- 介入回数上限: 11（2026-10-22 に 10 から上げた、owner の決定〈repo 外の回答台帳 TD-381〉。理由: Final Review の closure の後に owner が USB の控えを任意にし、「PC の外の控えをやめる」を範囲に入れた判断が 1 回足された）。その前: 10（2026-10-08 に 8 から上げた、owner 承認〈repo 外の回答台帳 TD-205〉。理由: Plan Review round 1 で Contract Probe の P1 を Plan Gate の前に owner が確かめることになり、上限 8 を超えるため）。起票時: 8（既定 6 から上げた。理由: 製品の運用の判断が 3 点〈owner の判断事項 1〜3〉と、owner に聞く機器・訪問の事実が 1 点〈Contract Probe の P5・P6、1 回の問い合わせにまとめる〉ある。1 回の問い合わせにまとめても decision point の数で数える）
- 実働時間上限: 30 分（既定）
- Plan Review round 天井: 3（既定）

| 種別 | 上限 | 消費（時点） | 残りの見込み | 予備 | 合計 |
|---|---|---|---|---|---|
| 介入 | 11 | 11（closeout 時点。2026-10-23: R4 の承認と Ready 1〈TD-383〉、merge 1〈TD-385〉。2026-10-22: USB の控えを任意にし「PC の外の控えをやめる」を範囲へ 1〈TD-381〉、R4 の計画の承認 1〈TD-376〉。2026-10-08: 判断事項 1〜3 の 3〈TD-196・TD-197〉、機器・訪問の事実 1〈P5・P6、TD-198・TD-199〉、持ち帰りの 1 本の古さを扱う仕組みの提案 1〈TD-202〉、Contract Probe の P1 の依頼 1〈TD-204。結果の受領は同じ decision point〉、上限の改定 1〈TD-205〉。TD-190 の Q4 は lane の選択の前の決定で本 lane に数えない。再開の選択〈TD-372〉と P1 の結果の受領〈TD-375〉は TD-204 と同じ decision point で数えない。次の lane の候補の決定〈TD-384〉は本 lane の後の判断で数えない） | 0 | 0 | 11 = 11 + 0 + 0 |

## Risk

Risk: R4

Reason:
backup の作り方（作業名 → 確かめ → 正式名）、復元の前の確かめ、PC の外への書込み（取外し可能な媒体へ書き、レジの SD に書かない）を決める設計で、`docs/DEV_WORKFLOW.md` Risk Tiers の R4「backup restore」とデータ安全の境界に当たる。本 lane は docs だけだが契約を決めるので R4（Final Review Minimum 2、Human Gate `r4`）。classifier の workflow の一覧（`scripts/ci/classify-changes.sh` の `AGENTS.md|CLAUDE.md|docs/DEV_WORKFLOW.md|…|docs/templates/*|…` の行）に当たる file を Scope に持たない（下の Scope の表は `docs/function-design/`・`docs/architecture/`・`docs/SCREEN_DESIGN.md`・`docs/ARCHITECTURE.md`・`docs/FUNCTION_DESIGN.md`・`docs/decision-log.md`・`docs/backlog.md`・`docs/plans/` だけ）。required gate の green / red は変わらない: docs だけで、既存の function-design の file に未実装の関数の signature を足すが、`src-tauri/tests/design_compliance_test.rs` は「設計書にあるがコードにない関数」を INFO として失敗にしない（同 file の冒頭の説明）。function-design の新しい file を作らない。manual は足さない（本 lane は docs だけ。L3 は runtime の lane の Human Gate、68 §68.12 の UI-11b-L3-6〜9）。

Rollback / recovery notes（R4、`docs/DEV_WORKFLOW.md` Risk Tiers の R4 の行）: 本 lane は docs だけで、merge の後に戻すときは本 lane の commit の revert で足りる（code・DB・設定に触れない）。runtime の lane への申し送り: 古い版（本設計の前の版）のアプリへ戻しても、PC 側の `offsite-backup.json`・媒体の `InventoryBackup\`（目印と写し）・規約に合う作業名の `.partial` は古い版から見えない・使われないだけで、DB・`app_settings`・既存の backup の一覧と復元は変わらない（古い版の一覧・掃除は規約の完全一致〈`mnt/backup.rs:155`・`:171`〉、今日の判定は `.db` の後方一致〈`:446`〉で、どれも `.db.partial` を見ない）。戻した後に残る file は消さなくてよく、再び新しい版へ上げると同じ目印と状態から続く。runtime の lane は、この扱いを自分の packet の rollback notes に書き、古い版へ戻した後に起きることを L3 か test で 1 回確かめる。

## Goal

Goal Invariant:

### 最小完了条件

- 後続の runtime の lane の Writer が、チャットの履歴を見ずに source docs だけで「USB メモリを用意する → 用意した媒体が見えている間に確かめ済みの最新の backup を写して読み戻しで照合する → 最後に PC の外へ写せた日時を画面とホームに出す → 復元の前に控えを確かめて新しすぎる版・壊れた控えを止める → 別の PC・新しい PC で媒体の控えから戻す」を実装できる（MNT-01-D7〜D10、71 §71.11〜§71.13、UI-11b-D14〜D16、UI-00-D12、43 §43.8.2〜§43.8.7）。
- owner の決定（TD-190 の Q4）が設計正本と D-114 に反映され、運用の型（差しっぱなし・入れ替え 等）の比較が盗難・火事・ランサムウェアへの耐性で並び、owner の判断事項 1〜3 の決定（2026-10-08、repo 外の回答台帳 TD-196・TD-197）とその帰結が D-114 と設計正本に反映されている。
- 新しすぎる版の backup の復元で、利用者に「新しい版のアプリが要る」と伝わる設計がある（MNT-01-D9、UI-11b-D15）。
- USB の控えは任意（owner 決定 2026-10-22、repo 外の回答台帳 TD-381）: USB を用意しない・やめた店でも、PC の中の backup とアプリの全機能が今までどおり動き、PC の外の控えの知らせは出ない（71 §71.11.5・§71.11.5b、UI-00-D12）。

### 失敗定義

- runtime の Writer が、媒体の見分け方・写す対象・照合の方法・失敗の扱い・レジの SD に書かない保証・状態の置き場所・知らせの条件のどれかを推測しないと書けない。
- 設計が、drive 文字の変化でレジの SD や別の drive に backup を書く経路、作成途中・壊れた file を成功の控えと数える経路、媒体が抜けたまま黙って何日も過ぎる経路、復元で状態が巻き戻って黙って写さなくなる経路のどれかを残す。
- 復元の契約（MNT-01-D1〜D6、`CmdErrorKind` の restore の 3 値、UI-11b-D2〜D5）を壊す。

### 非目的

- runtime の code・test・fixture・bindings（後続の lane）。
- クラウドへの backup（外部のサービスに頼らない owner の方針、TD-188 の文脈）。network（UNC）の保存先（`docs/backlog.md` の保留の項）。
- 更新・migration の前の保全の gate（更新を保全の成否で止める仕組み。MSI の手順書〈`docs/msi-distribution-runbook.md` の流れ 3〉は merge 済みで、更新の前の backup を手で取る手順だけを持つ。D-114 の Revisit、`docs/backlog.md` の同項の残り）。
- 写しの暗号化、媒体を 1 本だけ登録から外す操作（全部をやめる操作は範囲に入れた、owner 決定 TD-381、71 §71.11.5b）、持ち帰った媒体の古さをアプリで扱うこと（D-114 の Revisit）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## Ordinary Operation

設計を含む変更なので、店の毎日の列を置く。本 lane は docs だけで、表の通常運用は runtime の lane の後に成り立つ。**この文書を完了できる**（設計正本がそろう）ことと、**通常運用を達成できる**（店で PC の外の控えが毎日取れる）ことは別で、後者は本 lane では未達。運用の型は C（2 本の入れ替え。owner 決定 2026-10-08、repo 外の回答台帳 TD-196）で書く。

| 初期状態 | 操作 | 利用者が得る結果 | 次へ進む条件 | 未確認の前提／probe参照 |
| --- | --- | --- | --- | --- |
| 導入時、PC の外の控えは用意の前 | owner が USB メモリ（札「控え 1」）を差し、バックアップ画面の「この USB メモリを控えの保存先にする」で選ぶ | 媒体に `InventoryBackup\` と目印ができ、続けて最新の backup が写り、card に「最後に PC の外へ写した控え: 今日の日時（控え 1、確かめ済み）」 | card に今日の日時 | P1（USB メモリが取外し可能な drive に見える。確認済み 2026-10-22、TD-375）。P5: 空きの USB の口は 1 つ（TD-198）で、控え 1 がそれを占める |
| 控え 1 が差してある | owner が控え 1 を抜き、同じ口に 2 本目（札「控え 2」）を差して同じく用意し、控え 2 を抜いて持ち帰り、控え 1 を差し戻す（空きの口が 1 つなので 2 本を同時に差さない、P5） | 控え 2 にも同じ控えが写る。控え 1 を差し戻すと 60 秒以内に「差してある控え: 控え 1」 | — | 持ち帰りは owner が受けた前提（TD-011 の見直しと露出の受容、TD-196） |
| 毎朝、PC を起動する | 店主は何もしない | 起動時の自動 backup（71 §71.8）の後、共通レイアウトの起動直後の確認が控え 1 へ写す（UI-11b-D16） | — | — |
| 営業中 | 何もしない | 設定時刻の backup ができれば、60 秒以内に控え 1 へ写る | — | 店を閉める前に PC を閉じる日は、その日の入力は翌朝の backup まで PC の中だけ（71 §71.11.7） |
| 控え 1 が抜けた・壊れた | 何もしない | 写せない。媒体が見えないだけなら toast は出ない。最後に写せた日から 3 日でホームに「PC の外の控えが 3 日写せていません。USB メモリが差してあるか確かめてください。」 | 店主が差し直す、または owner に連絡 | 写す処理の失敗（空きなし・照合の不一致）は toast を 1 回（UI-11b-D16） |
| ホームに知らせが出ている | 店主が控え 1 を差し直す | 60 秒以内に写り、知らせが消える | — | — |
| PC の中の backup が作られていない（自動バックアップを切った・保存先に書けない）まま 3 日 | 何もしない | ホームに「PC の外の控えが {N} 日前のままです。PC の中の新しいバックアップが作られていません。…」（USB の文でない。UI-00-D12） | 店主か owner がバックアップ画面の「今すぐバックアップを作成」を押す、または設定を直す → 60 秒以内に写り、知らせが消える | 差し直しでは直らない（写す元が古いまま、71 §71.11.5） |
| PC の外の控えの記録の file が壊れた | 何もしない | ホームに「PC の外の控えの記録を読めません。USB メモリを差してバックアップ画面の「記録を作り直す」を押すか、使わないならバックアップ画面の「PC の外の控えをやめる」を押してください。」 | 「記録を作り直す」は、読めない file を退避名へ写して、差してある控えを登録し直した記録で置き換えて写す（71 §71.11.5a。USB が差さっていなければ止まり、2 つの直し方を名指す）。持ち帰りの控えは次の訪問で登録し直す | — |
| USB の控えを使わない（用意しない・やめた） | 用意しない。または owner がバックアップ画面の「PC の外の控えをやめる」を押し、確認で「やめる」 | PC の中の backup とアプリの全機能は今までどおり動く。PC の外の控えの知らせは出ない。やめたときは状態の file が退避名に残り、USB メモリの上の控えと PC の中の backup は消えない（71 §71.11.5b） | また使うときは「この USB メモリを控えの保存先にする」（目印から同じ媒体として使える） | 持ち帰りの控えが守る盗難・火事の守りは無くなる（owner が選ぶ） |
| owner の訪問 | owner が控え 1 を抜き、持ち帰っていた控え 2 を差す。差した直後（今日の控えが写る前）に控え 2 の最新の file を「控えを選んで確かめる」で見る。写しが先に済んでいたら 2 番目に新しい file を見る | 控え 2 が前回の入れ替えの時点まで読めることが分かり（短い確かめの (1)、71 §71.13。確かめの間は 60 秒の確認が止まる）、詳細を閉じて画面を戻すと確認が再開し、60 秒以内に今日の控えが控え 2 へ写る（同 (2)）。控え 1 を持ち帰る | card に「差してある控え: 控え 2」と今日の日時 | drive 文字が変わっても目印で見つかる（MNT-01-D8）。同じ口で抜いてから差す（P5）。訪問の間隔は決まっていない（P6、TD-199） |
| 新しすぎる版の控え・壊れた控えを選んだ | 一覧の行か「控えを選んで確かめる」 | 確認の手順へ進まずに固有の文言（UI-11b-D15）。事前バックアップも作らない | 別の控えを選ぶ | — |
| PC を失った（故障・盗難・火事） | owner が新しい PC にアプリを入れ、「控えを選んで確かめる」で持ち帰っていた控えを選び、2 段の確認で戻し、「今すぐバックアップを作成」を押してから、媒体を用意し直す（71 §71.13 の本番の復元） | 最後の入れ替えの時点（店に残った控えが無事なら前日）まで戻り、媒体の最新が戻した内容の控えになる（新しい PC の空の DB の控えを写さない） | 媒体の最新の控えを確かめ、商品の数が戻した結果と合う | 盗難・火事で失う期間は最後の入れ替えからの日数で、上限が無い（P6、下の残るリスク） |

残るリスク（owner の決定 TD-196 と、P5・P6 の答えの帰結）:

- 持ち帰りの 1 本が守る範囲: 盗難・火事・水害、または差している控え 1 も暗号化するランサムウェアで、PC と控え 1 を同時に失うと、戻れるのは持ち帰りの控えの時点まで。失う日数は最後の入れ替えからの日数で決まり、訪問の間隔は決まっていない（TD-199）ので上限が無い。店主の許容（3 日程度前まで、L-220）を超えうる。アプリは持ち帰りの 1 本の古さを見ず、ホームの 3 日の知らせ（UI-00-D12）は差してある 1 本についてだけ出る（71 §71.11.6・§71.11.7、D-114 の Guarantee range）。持ち帰りの 1 本の古さをアプリで扱うことは今の設計に無い（D-114 の Revisit）。
- PC の故障・DB の破損・誤操作の後に古い時点へ戻すことは、差しっぱなしの控え 1（前日まで、30 世代）が守り、訪問の間隔に依らない。
- owner は「実際に運用できるかは別」と添えた（TD-196）。入れ替えが途絶えても控え 1 への毎日の写しとホームの知らせは続き、守りは A（差しっぱなし）と同じに戻るだけで、アプリの振舞いは変わらない（MNT-01-D8 は型に依らない）。
- 空きの USB の口が 1 つ（P5、TD-198）: 控え 1 がその口を占める。SD の読取り機とバーコードリーダーは自分の口を使い続けるので、毎日の SD の読取り（D-111）とスキャンは変わらない。2 本を同時に差せないので、用意と入れ替えは「抜いてから差す」順にする（上の表、68 UI-11b-L3-8）。ほかの USB 機器（例: アプリの更新を運ぶ USB メモリ）を差す間は控え 1 を抜く。差し戻し忘れは 3 日でホームが知らせる（UI-00-D12）。常に差す機器が増えれば USB の hub が要る（D-114 の Revisit）。ノート PC を持ち帰る日（TD-009）は控え 1 も PC と一緒に動き、差したまま運ぶと口や媒体を傷めうる。

Plan Review は、この列が「正常な条件で目的を達成できるか」と「危険な結果を出さないか」を別々に答える（`docs/DEV_WORKFLOW.md` Review Rules）。

## owner の判断事項（決定済み、2026-10-08）

確認済み事実（repo 外の回答台帳）: PC の外の保存先は今使っていない（L-217）。外部に保存するなら置き場所は店（TD-011）。ノート PC を家へ持ち帰って作業する日がある（TD-009）。USB メモリは owner が持っていて渡せるが、店主が毎回手で取れるかは分からない（TD-187）。店主は 3 日程度前まで戻れれば許容（L-220）。毎営業日に店主が USB へ手で取る手順は owner が 2026-09-28 に「今は考えない」とした（TD-031）。決定済み: 外付けの保存先へ自動で書き、確かめて画面に出す（TD-190 の Q4）。

決定（owner 2026-10-08、repo 外の回答台帳 TD-196・TD-197）: 下の表の「決定」の列。アプリの設計（71 §71.11・§71.12、UI-11b-D14〜D16、UI-00-D12）はどの選択肢でも同じで、決定は運用の手順・残るリスク・D-114 の Status を決めた。選択肢・起草時の推奨・根拠は記録として残す。

| # | 判断事項 | 選択肢 | 起草時の推奨（区分） | 根拠 | 決定 |
|---|---|---|---|---|---|
| 1 | 運用の型 | A 差しっぱなし（1 本、毎日自動）／ B 週 1 回など店主が手で差す（1 本）／ C 2 本の入れ替え（1 本を差しっぱなし、1 本を owner が持ち帰り、訪問時に入れ替え）／ C' 2 本の入れ替え（外した 1 本を店の中の PC と別の場所に置く） | **C**（precondition-dependent: owner が店の外へ店のデータを持ち出すことを認め〈TD-011「置き場所は店」の見直し〉、持ち出した媒体を失くしたときの露出〈商品・原価・売上の記録。暗号化しない〉を受け、訪問の間隔が分かること）。前提が立たなければ **C'**（candidate）。A は C の最初の 1 本で、単独では推奨しない。B は推奨しない | 71 §71.11.6 の比較: A は盗難（差した媒体ごと持ち去られやすい）・火事・ランサムウェア（差している媒体も暗号化されうる）に弱い。B は店主の毎週の操作が要り（TD-187・TD-031）、PC の故障で最大 1 週間を失い許容の 3 日を超える。C は店主の手間なしで、毎日の故障は前日まで、盗難・火事・ランサムウェアは持ち帰りの 1 本で守る（失うのは最後の入れ替えからの日数）。C' は火事を守らない。ノート PC を家へ持ち帰る日（TD-009）、差してある 1 本は PC と一緒に動く（C でも持ち帰りの 1 本は owner の所にある） | **C**（TD-196）。前提の TD-011 の見直し（店のデータを店の外へ持ち出す）と、失くしたときの露出（暗号化しない）を受けた。実際に運用できるかは別との添え書きがある（Ordinary Operation の残るリスク） |
| 2 | D-111 の残りの判断 B（SD から読んだ原本の写し `pos-sources/` を backup・PC の外の控えに含めるか） | 含めない ／ PC の外の控えに folder ごと写す（足すだけの mirror） ／ backup の file にまとめる | **含めない**（candidate） | 取り込んだ内容は DB にあり、DB の控えで戻せる。原本は (b) の lane まで店が CV17 の取込みを続ける間（D-111 の運用の制約）、レジの SD の `XZ_BKUP` に同じ bytes が残り、SD は PC と別の機器（レジ）にある。写しの runtime（SD 直読みの runtime の lane）が未実装で量も未実測。見直し: CV17 の取込みをやめるとき（D-111 の (b) の lane）、写しの容量が分かったとき。レシート画像（`images/`、IO-06）も同じく今は含めない（71 §71.1）が、同じ時に見直す | **含めない**（TD-197）。D-111 に `Superseded in part by: D-114` の行を足した |
| 3 | 復元の予行演習の実施者と頻度 | 71 §71.13 の推奨どおり ／ 頻度を変える ／ 通しの演習をしない | **71 §71.13 の推奨どおり**（candidate）: 短い確かめは入れ替えのたび（A なら月 1 回）に owner、通しの演習は go-live の前と migration を含む更新の後に owner が別の PC で | 「控えがある」と「戻せる」を分ける（`docs/backlog.md` の同項）。店の PC で実際の復元をしない（店の記録を巻き戻さない）。別の PC に店のデータが残る間の扱いは owner の判断（71 §71.13） | **71 §71.13 のとおり**（TD-197） |

店主に聞く事項: 無い（台帳で確かめた: 戻れる許容〈L-220〉、手で取れるか〈TD-187〉、置き場所〈TD-011〉、PC を持ち帰ること〈TD-009〉は回答済み）。判断事項 1 は C で店主の手の操作を要しないので、店主が USB を差して抜けるか（open-questions の Q-017、保留〈TD-031〉）は聞かない。

owner に聞いた事実（判断ではない、2026-10-08 に 1 回で）: P5 バーコードリーダー・SD の読取り機と別に空いている USB の口は 1 つ（TD-198）。P6 訪問の間隔は決まっていない（TD-199）。帰結は Ordinary Operation の残るリスクと Contract Probe。

### runtime の置き場所（決定済み）

- **後続の別の lane**（採る）: 本 lane は docs の設計だけにし、runtime は本 lane の merge の後に 1 本の lane として起票する。理由: runtime は Rust（`mnt::offsite` の新設、`create_backup` の作業名、`inspect_backup`、6 command）・bindings・frontend（card・restore の確かめ・ホームの Alert・共通の確認の hook）にまたがる R4 で、実 USB の L3（68 §68.12 の UI-11b-L3-6〜9）が要り、本 lane の設計の review（R4 の Final Review 2 本）と分けた方が review の対象が小さい。判断事項 1〜3 の決定（2026-10-08）は runtime の code を変えない（判断事項 2 は「含めない」と決まったので runtime の Scope も増えない）。
- 同じ lane の後段にする案は採らない: plan-first（設計と packet を先に commit し、独立の Plan Review を通す）を runtime の Scope でもう一度回すことになり、本 lane の packet が設計と runtime の両方の AC を持って重くなる。

## Scope

本 lane は docs だけを変える（予定 file の全部。所有の列は並走 lane との重なり）。

| file | 変更 | 他 lane との重なり |
|---|---|---|
| `docs/function-design/71-mnt-backup.md` | §71.1 の module と写しの注記、§71.4 の手順 0・3・4a・4b とエラーハンドリング、MNT-01-D7・D10 の新設、§71.4.1（上書きしない公開）の新設、§71.8 手順 3 の判定、§71.10 の D-114 の追加の表、§71.11（PC の外の控え、MNT-01-D8）・§71.12（`inspect_backup`、MNT-01-D9）・§71.13（復元の予行演習）の新設。main への同期の後（2026-10-22）、MNT-01-D9 に作り直しの起点の migration より前の版の開発・試験 DB の控えの扱い（D-123 との整合）。Final Review の是正（2026-10-22）: §71.4 の書く先の固定と MNT-01-D10 の祖先の判定、§71.5 手順 0、§71.4 の `VACUUM INTO` の失敗の作業名、§71.11.1 の `StateUnsupported`、§71.11.5 の `local_backup_old`、§71.11.5a（記録の作り直し）の新設、§71.11.7、MNT-01-D9 の確かめの後の入れ替え | なし |
| `docs/function-design/68-ui-backup-restore.md` | 前文、UI-11b-F8〜F10、§68.3 の 6 command（`resetOffsiteState` は Final Review の是正、`stopOffsiteBackup` は owner の決定 TD-381）、§68.7 の `offsite_resetting`、§68.4 の PC の外の控えと復元の手順 1・1a・2、UI-11b-D14〜D16、§68.6、§68.7 の `offsite_preparing`・`restore_inspecting`・`restore_blocked` と `ready`・`restore_detail` の行、§68.9、§68.10、§68.11、§68.12 の UI-11b-L3-6〜9、§68.13 | なし |
| `docs/function-design/53-ui-home.md` | §53.1 の `OffsiteBackupWarning` の行、§53.5 の query 失敗の行と UI-00-D12（原因ごとの文は Final Review の是正）、更新履歴 | なし |
| `docs/function-design/43-cmd-settings-log.md` | §43.8.2〜§43.8.7（6 command・エラーの写像・DTO の形。§43.8.6 `reset_offsite_state` と §43.8.4 の `backup_dir` は Final Review の是正）、§43.11 の登録の注記 | なし |
| `docs/function-design/22-mnt-migration.md` | MNT-03-D11 の「復元への波及」。main への同期の後（2026-10-22）、§16 の作り直しの手順の「folder の外でアプリが読む場所」(3) と手順 3 に、用意した媒体の `InventoryBackup`（D-123 との整合。同じ項の「folder の外の場所を足す変更は、この項に足す」による） | §16 は原価の lane C・棚卸しの runtime (a) が書いた節（どちらも merge 済み） |
| `docs/architecture/cmd-task-specs.md` | CMD-11 の表に 5 行 | なし |
| `docs/architecture/mnt-task-specs.md` | MNT-01 に PC の外の控えの処理構造 | なし |
| `docs/SCREEN_DESIGN.md` | 1 日の動線のバックアップの文、ホーム画面の利用者配慮に 1 行、バックアップ・復元画面のレイアウト判断に 2 行 | なし |
| `docs/ARCHITECTURE.md` | MNT-01 の行 | なし |
| `docs/FUNCTION_DESIGN.md` | MNT-01 の索引の行 | なし |
| `docs/decision-log.md` | D-114 を番号の順（D-113 と D-116 の間）に置き、D-111 の末尾に `Superseded in part by: D-114` の 1 行を足す | 全 lane が末尾に追記（`agent/plu-clear` = D-110、本 lane = D-114。ほかの 2 lane は依頼文が指定した番号）。merge 順で両方を残す |
| `docs/backlog.md` | 「backup に、PC の外のコピーと…」の entry に着手の sub-bullet、「保存と起動の守りの follow-up」の (2) に着手の注記 | 全 lane が自 lane の entry だけ |
| `docs/plans/2026-10-07-backup-offsite.md`、`docs/plans/test-matrices/2026-10-07-backup-offsite.md` | 新設 | なし |

触らない: `Plans.md`（D-097）、`docs/function-design/90-traceability.md`（生成物）、`docs/spec/requirements.md`・`requirements-coverage.md`（QR-04 / QR-05 / REQ-901 の行は変えない。D-114 は REQ-901 の範囲の中）、`docs/db-design/`（DB の schema・`app_settings` の key を変えない）、`docs/project-memory.md`（「PC の外の保存先は今使っていない」は事実のまま）。

### runtime の lane への申し送り（形が変わる型と、作る所・読む所）

本 lane は code を書かない。触る型・関数の作る所・読む所を `rg` で挙げる（2026-10-07、`95c0aeb0` の現物。行番号は定義・呼出しの行）。runtime の lane はこの一覧を Scope の出発点にし、起票時の現物で数え直す。

| 型・関数 | 変更 | 作る所 | 読む所・test |
|---|---|---|---|
| `create_backup`（作業名 → 確かめ → 正式名、MNT-01-D7、手順 0 の MNT-01-D10）と `cleanup_old_backups`（手順 0） | 中の処理だけ。signature と `BackupResult` は変えない。手順 0 で書く先を固定し（`pin_to_volume`・`pin_backup_dir`。`GetFullPathNameW` で正規化 → 在る最も近い祖先を `CreateFileW` で開いて `GetFinalPathNameByHandleW(VOLUME_NAME_GUID)` → 無い末尾を付ける、71 §71.4 の「書く先の固定」）、正規化した元の path と実体の path の両方の祖先で判定し、以後の全部の操作を実体の path で行う。媒体の root の固定（§71.11.3）も同じ関数で、test の注入点も 1 つ。CMD の `create_backup` は先に `pin_backup_dir` を呼んで原因を名指す（43 §43.6）。既存の test `:1217`（`fail_any_metadata()`）は手順 4a で先に失敗するので、注入を公開の後の正式名だけに当てる形へ移し、oracle は変えない（71 §71.10 の D-114 の表） | `src-tauri/src/mnt/backup.rs:218` | 呼出し: `cmd/settings_cmd.rs:192`、`mnt/backup.rs:377`・`:420`（`check_auto_backup` の中）。test: `mnt/backup.rs:666`・`:683`・`:722`・`:744`・`:960`・`:1018`・`:1217`、`cmd/settings_cmd.rs:560`・`:579` |
| `collect_today_backup_names` | 判定を `extract_datetime_from_backup`（`mnt/backup.rs:171`）と同じ規約の完全一致にし、日付が今日の名前だけを返す（71 §71.8 手順 3、MNT-01-D7）。signature は変えない。呼出し側の `backup_time` 以降の判定（`mnt/backup.rs:410` の `strip_prefix`・`strip_suffix`）は、完全一致の名前だけが来るので変えなくてよい | `mnt/backup.rs:425`（判定は `:446`） | 呼出し: `mnt/backup.rs:361`（`check_auto_backup`）。test: `mnt/backup.rs:1191`。規約外の名前（`inventory_backup_{今日}_manual.db`）と作業名を数えない test を足す（Matrix の MNT-01-D7 の行） |
| `extract_datetime_from_backup`・`extract_date_from_backup` | `extract_datetime_from_backup` は長さ・`_` の位置・ASCII の確認を slice より前に移す（今は 15 bytes の確認の後に `&stem[..8]` を切るので、`inventory_backup_あああああ.db` で文字境界の panic になる。71 §71.3。今の `list_backups` にもある潜在の panic）。`extract_date_from_backup` は `_` の位置を先に見るので形を変えない。signature は両方変えない。作業名の掃除（71 §71.4 手順 3、§71.11.4 手順 4b）は `.partial` を除いた名前をこの関数に通して、規約に合う作業名だけを消す | `mnt/backup.rs:171`・`:155` | 一覧（`list_backups`、`:309`）・掃除（`cleanup_old_backups`）・新しい作業名の掃除・今日の判定 |
| file 操作の差し替え（restore の `RestoreFileOps` と同じ形） | backup の公開に `publish_no_replace`（71 §71.4.1。Windows は `MoveFileExW` を `MOVEFILE_REPLACE_EXISTING` なしで、Windows 以外は `hard_link` + 作業名の削除）を足す。目印・状態の置き換えは既存の `replace` の形 | `mnt/restore.rs:99`（trait）・`:118`〜`:153`（`StdRestoreFileOps` の `rename`・`replace`）。backup 側に同じ形を置くか trait を共有するかは runtime の lane が決める（本番と test で別の判定を作らない） | test の注入: `mnt/restore.rs:1062`（操作の記録の `record`） |
| `src-tauri/Cargo.toml` の `windows-sys` の feature | `Win32_System_Diagnostics_Debug` を足す（`SetThreadErrorMode`、71 §71.11.3）。新しい依存ではなく、既存の `windows-sys = "=0.61.2"` の feature | `src-tauri/Cargo.toml:48` | `mnt::offsite` の媒体を探す・読み書きする関数 |
| `read_current_version_without_ddl` | private を `pub(crate)` にして `inspect_backup` と共有（MNT-01-D9、22 MNT-03-D11） | `src-tauri/src/db/migration.rs` の `read_current_version_without_ddl` | 呼出し: 同 file の `migrate`、新しい `inspect_backup` |
| `app_max_version` | 変えない | `src-tauri/src/db/migration.rs` の `app_max_version` | 新しい `inspect_backup` |
| 新 `inspect_backup` と `BackupInspection` | MNT-01-D9。このアプリの backup かの検査（版 0・必須の表なしを `Err`、71 §71.12 手順 2a）と「使える控え」の判定を持つ | `mnt/backup.rs`（71 §71.12） | `create_backup` の手順 4a、`mnt::offsite` の写す元の検査（§71.11.4 手順 3a）、新 command `inspect_backup` |
| 新 module `mnt::offsite`（`prepare_offsite_medium`・`check_offsite_backup`・`offsite_status`〈`backup_dir` を受け `local_backup_old` を返す〉・`reset_offsite_state`・`stop_offsite_backup`、型 71 §71.11.1。`OffsiteFailureKind` 7 値、`OffsiteError::MarkerUnsupported`・`StateUnsupported`・`NoMediumAttached`・`StateResetFailed`・`StopFailed`） | MNT-01-D8 | `src-tauri/src/mnt/offsite.rs`、`src-tauri/src/mnt/mod.rs` に `pub mod offsite` を足す | `src-tauri/tests/design_compliance_test.rs` の `build_doc_to_modules_map()` の `71-mnt-backup.md` の行に `mnt::offsite` を足す |
| レジの SD の root の判定（`CASIO\SR500_550_4000`） | MNT-01-D10 | SD 直読みの runtime の lane（IO-09、`io::register_sd`）と同じ folder 名の定数を使う。先に入る lane が置き、後の lane が使う（文字列を二重に持たない）。IO-09 の `resolve_register_sd_root` は選んだ path から下へ解決する関数なので流用しない（判定は 71 MNT-01-D10 の祖先の判定） | `create_backup`・`cleanup_old_backups` の手順 0、`mnt::offsite` |
| 新 command 6 つと request DTO 2 つ（`PrepareOffsiteMediumRequest`・`InspectBackupRequest`） | 43 §43.8.2〜§43.8.7。6 つとも `#[tauri::command(async)]`（main thread で写さない。既存の command は全部 `#[tauri::command]` の sync、`cmd/settings_cmd.rs:124` 等） | `cmd/settings_cmd.rs`、`lib.rs` の `collect_commands!`（`:277`、restore は `:356`）と `generate_handler!`（`:1272`、restore は `:1352`） | `src/lib/bindings.ts`（再生成。今の backup 系は `:311`・`:317`・`:333`） |
| `useAutoBackupCheck` | `checkAutoBackup` の後に `checkOffsiteBackup`、mount 時に 1 回、結果の種類に依らず status を invalidate（UI-11b-D16）。新しい export `suspendAutoBackupCheckAndWait(): Promise<void>`（停止して世代番号を進め、実行中の回の完了を待つ。UI-11b-D15） | `src/features/backup-restore/useAutoBackupCheck.ts:33`（hook）・`:23`（`suspendAutoBackupCheck`）・`:29`（`resumeAutoBackupCheck`） | mount: `src/components/layout/RootLayout.tsx:48`。test: `src/features/backup-restore/useAutoBackupCheck.test.tsx`（停止の test は `:131`〜`:141`）。module を mock する test は新しい export も mock に足す: `src/features/inventory-records/OtherRecordDetailRoutes.test.tsx:9`〜`:12`、`src/lib/app-router.test.tsx:37`（どちらも `vi.mock` で export を置き換える）。実物を import する test: `RootLayout.test.tsx:13`、`BackupRestorePage.flow.test.tsx:29`。`BackupRestorePage.test.tsx:17`・`:175` は実物を import する |
| `BackupRestorePage` の復元の選択（`selected: BackupInfo \| null`） | 選んだ file（`BackupInfo` でない）も詳細に入る。確かめの結果を持つ。控えを選んだ時点で `suspendAutoBackupCheckAndWait()` を待ってから `inspectBackup`、`ready` へ戻る・unmount（復元の実行中でも fatal の後でもないときだけ）で `resumeAutoBackupCheck()`（UI-11b-D15） | `src/features/backup-restore/BackupRestorePage.tsx:93`（型）・`:227`（`selectRestoreBackup`）。今の停止・再開の呼出しは `:273`・`:279`・`:303`（import は `:56`） | test: `BackupRestorePage.test.tsx`・`BackupRestorePage.flow.test.tsx` |
| MSI の手順書の流れ 3（`docs/msi-distribution-runbook.md`） | 更新の前の backup を写す「PC の外の媒体」が D-114 の媒体なら、`InventoryBackup\` の外の folder に置くと書く（中の規約の名前の file は写しとして数えられ、新しい 30 本を超えると消える。71 §71.11.7） | 本 lane では直さない。PC の外の控えの runtime の lane（害は runtime ができてから生じる） | — |
| ホームの知らせ | UI-00-D12 | 新 `src/features/home/components/OffsiteBackupWarning.tsx`、`src/features/home/HomePage.tsx`（前日未取込みの Alert は `:78`） | `HomePage.test.tsx`、`src/lib/query-keys.ts:116`（`backupRestore` の key に status を足す） |

## Non-scope

- runtime の code・test・fixture・bindings・`90-traceability.md`。
- クラウド・network（UNC）の保存先、写しの暗号化、媒体を 1 本だけ登録から外す操作（全部をやめる操作は範囲、71 §71.11.5b）、持ち帰った媒体の古さの管理（D-114 の Revisit）。
- 更新・migration の前の保全の gate（`docs/backlog.md` の同項の残り。MSI の手順書の流れ 3 は手で取る手順だけを持つ）。
- `pos-sources/`・`images/` を控えに含めること（判断事項 2 で含めないと決めた、TD-197。見直しは D-114 の Revisit）。
- `backup_path` に取外し可能な drive を選ばせない変更（MNT-01-D10 の棄却案。レジの SD の上だけを止める）。
- 既存の restore の契約（MNT-01-D1〜D6、UI-11b-D2〜D5・D11〜D13）の変更。ただし UI-11b-D13 の共通の確認の停止は、UI-11b-D15 が控えの確かめの開始の時点へ早める（復元の結果での再開・fatal の後の停止は変えない）。

## Acceptance Criteria

baseline は本 worktree（起草の時点の HEAD は `95c0aeb0` の上の未 commit の編集）で逐語に実行した出力。2026-10-08 に owner の決定の反映の後（HEAD `3c2df223` の上の未 commit の編集）で全 AC を逐語に再実行し、値の変わった所を「2026-10-08 の再実行」として足した。「main の値」は同じ検索を `git show 95c0aeb0:<file> | rg …` で数えた値（出力なし = 0 件）。

- AC1（MNT-01 の決定がある）: `rg -n '^\*\*MNT-01-D(7|8|9|10):' docs/function-design/71-mnt-backup.md` が 4 行（D7・D10・D8・D9）。起草時の実測: `103:**MNT-01-D7: …`・`111:**MNT-01-D10: …`・`705:**MNT-01-D8: …`・`746:**MNT-01-D9: …` の 4 行。main の値 0。2026-10-08 の再実行: `103:`・`111:`・`706:`・`747:` の 4 行（§71.11.7 に 1 行足したため D8・D9 が 1 行ずれた）。 Plan Review round 1 の是正の後の再実行（2026-10-08）: `104:`・`112:`・`729:`・`773:` の 4 行（§71.4 の手順と §71.4.1・§71.11 の追記で行がずれた）。 Plan Review round 2 の是正の後の再実行: `106:`・`114:`・`752:`・`796:` の 4 行（§71.3 の名前の解析と §71.11 の追記で行がずれた）。 同期の closure の是正の後の再実行（2026-10-22）: `106:`・`114:`・`753:`・`797:` の 4 行（§71.11.7 に `InventoryBackup\` の扱いの 1 行を足したため）。 Final Review の是正の後の再実行（2026-10-22）: `108:`・`116:`・`784:`・`828:` の 4 行（§71.4 の書く先の固定・§71.5 手順 0・§71.11.5a の新設で行がずれた）。 closure の是正と owner の決定 TD-381 の反映の後の再実行（2026-10-22）: `114:`・`122:`・`819:`・`863:` の 4 行（書く先の固定の置き換え・§71.11.5a の組み替え・§71.11.5b の新設で行がずれた）。
- AC2（画面とホームの決定がある）: `rg -n '^\| UI-11b-D1[456] \|' docs/function-design/68-ui-backup-restore.md` が 3 行、`rg -n '^#### PC の外の控えの知らせ（UI-00-D12' docs/function-design/53-ui-home.md` が 1 行。起草時の実測: 68 は `114:`・`115:`・`116:` の 3 行、53 は `213:#### PC の外の控えの知らせ（UI-00-D12、D-114）` の 1 行。2026-10-08 の再実行: 同じ。 Final Review の是正の後の再実行（2026-10-22）: 68 は `116:`〜`118:` の 3 行（§68.3 に `resetOffsiteState`、§68.4 に手順 4 を足したため）、53 は `213:` のまま。 closure の是正と TD-381 の反映の後の再実行（2026-10-22）: 68 は `118:`〜`120:` の 3 行（§68.3 に `stopOffsiteBackup`、§68.4 に手順 5）、53 は `213:` のまま。
- AC3（command がある）: `rg -n '^\| 43\.8\.[2-7] \|' docs/function-design/43-cmd-settings-log.md` が 6 行（`prepare_offsite_medium`・`check_offsite_backup`・`get_offsite_backup_status`・`inspect_backup`・`reset_offsite_state`・`stop_offsite_backup`。Final Review の是正で §43.8.6、owner の決定 TD-381 で §43.8.7 を足したので、式を `[2-5]`・4 行から替えた〈2026-10-22〉。下の 2026-10-22 より前の記録は旧い式の出力）。起草時の実測: `207:`〜`210:` の 4 行。2026-10-08 の再実行: 同じ。 Plan Review round 2 の是正の後の再実行: `209:`〜`212:` の 4 行（§43.8.2〜§43.8.5 の前文に「実行の thread」を足したため）。 main（`c98fe7dd`）への同期の後の再実行（2026-10-22）: `205:`〜`208:` の 4 行（main が 43 の冒頭の節を D-118 の契約へ置き換えて 4 行縮めたため）。 closure の是正と TD-381 の反映の後の再実行（2026-10-22、替えた式）: `211:`〜`216:` の 6 行。
- AC4（決定の記録）: `rg -n '^## D-114' docs/decision-log.md` が 1 行で、D-114 が owner の決定 1〜3・Guarantee range・Revisit を持つ。起草時の実測: `938:## D-114: …` の 1 行。2026-10-08 の再実行: `939:## D-114: …` の 1 行（D-111 に未決 B の決着を 1 行追記したため）。 main（`c98fe7dd`）への同期の後の再実行（2026-10-22）: `981:## D-114: …` の 1 行（D-114 を番号の順に D-113 と D-116 の間へ置いたため）。
- AC5（旧い記述・未決の書き方の live な残り 0）: `rg -c 'restore に版の事前検査は足さない（同じ判定の二重化）' docs -g '!docs/archive/**' -g '!docs/plans/2026-10-07-backup-offsite.md'`、`rg -c 'PC の外へのコピーを1日のどの時点で行うかは決めていない' docs -g '!docs/archive/**' -g '!docs/plans/2026-10-07-backup-offsite.md'`、`rg -c 'D-114 の owner の判断事項|owner の判断事項（未決|決まるまでは含めない|D-114 の判断事項' docs -g '!docs/archive/**' -g '!docs/plans/2026-10-07-backup-offsite.md'` の 3 つが出力なし（0 件、exit 1）。packet を除くのは、この AC の文自身が検索語を含むため（2026-10-07 の起草時の「出力なし」は packet 自身の 1 件を見落としていた。2026-10-08 に除外を足した）。2026-10-08 の再実行（plan-draft の commit の前の worktree）: 3 つとも出力なし・exit 1。main の値: 1 つ目は 22 に 1、2 つ目は SCREEN_DESIGN に 1。3 つ目は起票の commit（`3c2df223`）で SCREEN_DESIGN に 1・71 に 4・decision-log に 1（owner の判断を待つ書き方。2026-10-08 に決定の文へ直した）。新しい文は 22 の「restore の中に版の事前検査は足さない」と SCREEN_DESIGN の「PC の外の控えは、用意した USB メモリが差してある間に」で、それぞれ `rg -c` で 1 件。
- AC6（code を変えない）: `git diff --name-only c98fe7dd HEAD -- src src-tauri` が出力なし（base は同期の基点。main への同期〈2026-10-22〉で `95c0aeb0` から替えた。同期の前の再実行の記録は旧い base の出力）。起草時の実測: 出力なし。2026-10-08 の再実行: 出力なし。 Plan Review round 1 の是正の後の再実行: AC2〜AC6 は値が同じ（AC2 68 `114:`〜`116:` と 53 `213:`、AC3 `207:`〜`210:`、AC4 `939:`、AC5 3 本とも出力なし・exit 1、AC6 出力なし・exit 0）。 Plan Review round 2 の是正の後の再実行: AC2 は 68 `114:`〜`116:` と 53 `213:`、AC4 `939:`、AC5 3 本とも出力なし・exit 1、AC6 出力なし・exit 0（AC1・AC3 は上）。 Plan Review round 3 の後の是正の後の再実行: AC1 `106:`・`114:`・`752:`・`796:`、AC2 68 `114:`〜`116:` と 53 `213:`、AC3 `209:`〜`212:`、AC4 `939:`、AC5 3 本とも出力なし・exit 1、AC6 出力なし・exit 0（どれも round 2 の後と同じ）。 main（`c98fe7dd`）への同期と、後から入った契約に合わせる是正の後の再実行（2026-10-22）: AC1 `106:`・`114:`・`752:`・`796:`、AC2 68 `114:`〜`116:` と 53 `213:`、AC3 `205:`〜`208:`、AC4 `981:`、AC5 3 本とも出力なし・exit 1、AC6（替えた base）出力なし・exit 0。 同期の closure の是正の後の再実行（2026-10-22）: AC1 `106:`・`114:`・`753:`・`797:`、AC2 68 `114:`〜`116:` と 53 `213:`、AC3 `205:`〜`208:`、AC4 `981:`、AC5 3 本とも出力なし・exit 1、AC6 出力なし・exit 0。 Final Review の是正の後の再実行（2026-10-22）: AC1 `108:`・`116:`・`784:`・`828:`、AC2 68 `116:`〜`118:` と 53 `213:`、AC3 `205:`〜`208:`（43.8.6 は AC3 の式の外）、AC4 `981:`、AC5 3 本とも出力なし・exit 1、AC6 出力なし・exit 0。 closure の是正と TD-381 の反映の後の再実行（2026-10-22）: AC1 `114:`・`122:`・`819:`・`863:`、AC2 68 `118:`〜`120:` と 53 `213:`、AC3（`[2-7]`）`211:`〜`216:`、AC4 `981:`、AC5 3 本とも出力なし・exit 1、AC6 出力なし・exit 0。
- AC7（検査）: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-10-07-backup-offsite.md` が ERROR 0（WARN は報告）。

## Design Readiness

- 引用する設計正本（節まで）: `docs/function-design/71-mnt-backup.md` §71.1・§71.4（MNT-01-D7・D10）・§71.8 手順 3・§71.10・§71.11（MNT-01-D8）・§71.12（MNT-01-D9）・§71.13／`68-ui-backup-restore.md` UI-11b-F8〜F10・D14〜D16・§68.4・§68.7・§68.9〜§68.13／`53-ui-home.md` §53.5 UI-00-D12／`43-cmd-settings-log.md` §43.8.2〜§43.8.7／`22-mnt-migration.md` MNT-03-D11／`docs/decision-log.md` D-114・D-111（未決 B・IO-09-D3 の SD に書かない）・D-032（復元前の強制バックアップ、変えない）。
- 必要な設計成果物: function-design（MNT-01・CMD-11・UI-11b・UI-00） = updated in this PR／DB = existing sufficient（schema・`app_settings` を変えない。状態は DB の外の file、71 §71.11.1）／SCREEN_DESIGN = updated in this PR／decision-log = D-114 を追加／Boundary / Wire Contract = 下に記入。
- plan にしかない durable な判断の昇格先: すべて D-114 と上の正本へ置いた。owner の決定 1〜3 は D-114 の「owner の決定」に書き、D-111 の未決 B の決着は D-114 の「owner の決定」2 に書き、D-111 に `Superseded in part by: D-114` の行を足した。runtime の置き場所の推奨だけは packet にあり、runtime の起票時に Coordinator が使う（durable な製品の判断ではない）。
- 前提・制約と、延期した design gap: Windows の drive の種類の判定（P1）は確認済み（2026-10-22、repo 外の回答台帳 TD-375、Contract Probe。`Removable` を `DRIVE_REMOVABLE` と同じ区分と見なす前提は runtime の lane の L3 で確かめる）。空の読取り機の扱い（P3）は設計で前提から外した（媒体を探す間は常に thread の error mode で dialog を抑える、71 §71.11.3）。読み戻しの照合は file cache を通りうる（P2、保証の範囲を狭めて書いた）。延期: 更新・migration の前の保全、媒体の登録の解除、持ち帰った媒体の古さ、暗号化（Non-scope、D-114 の Revisit）。延期が安全な理由: どれも今の backup と復元の振舞いを変えず、PC の外の控えが無い今より悪くしない。
- 絶対保証の自己点検: 「レジの SD に書かない」の例外: SD が取外し可能な drive でなく固定 disk に見える読取り機でも、書く先の folder とその祖先の `CASIO\SR500_550_4000` の判定（MNT-01-D10）は drive の種類に依らず効く。`backup_path` に SD の上の folder を選んだ既存の設定は `create_backup` と `cleanup_old_backups` の手順 0 が止める（自動 backup は失敗し、既存の失敗の toast が出る）。判定の後に drive 文字が SD へ移っても、書込みは判定した volume に固定した path で行うので SD に書かない（71 §71.4 の「書く先の固定」）。junction・symbolic link を含む保存先も、正規化した元の path と実体の path の両方の祖先で判定するので、SD を指す link・SD の上の link のどちらでも書かない。固定できない保存先（network・Mount Manager の外）には書かない（手動の backup で原因を名指す、43 §43.6）。「用意した媒体にしか書かない」の例外: 同じ目印を別の媒体へ手で写した場合（利用者の手の操作。同じ `medium_id` の 2 本になり、どちらにも写る。害は札が同じになることだけ）。「作成途中を成功に数えない」: 正式名は共有の検査（このアプリの backup・版・`quick_check`）の後の上書きしない公開でしか作らない。例外は MNT-01-D6 の metadata の失敗と公開の後の親 directory の sync の失敗で、正式名の file は検査済み。「確かめ済みの backup を上書きしない」: 例外なし（`MoveFileExW` を `MOVEFILE_REPLACE_EXISTING` なしで呼ぶ。71 §71.4.1）。「写す元が使える控え」: 写す直前に共有の検査を通す（71 §71.11.4 手順 3a）。例外は検査の後・copy の前に PC の中の file が替わった場合で、読み戻しの hash は替わった後の bytes と一致しうる（同じ process の中で backup の file を書き換える経路は無く、利用者の手の操作に限る）。「照合した媒体にしか書かない」: 1 回の確認の中は volume GUID path に固定し、copy・公開・掃除の直前に目印を読み直し、媒体の `InventoryBackup` とその中の entry の reparse point を追わない（71 §71.11.3）。例外は目印の読み直しと書込みの間の抜き差しで、volume GUID path が別の媒体を指さないので書込みは失敗する。「黙って写さなくならない」: 状態の file が壊れたら知らせ、画面の「記録を作り直す」で直す（71 §71.11.5a）。写す元が古いときは USB でなく PC の中の backup の直し方を知らせる（`local_backup_old`）。例外: 状態の file が消えた（利用者が消した・PC を替えた）とき、`NotPrepared` になり知らせない（用意の前と区別できない）。PC の外の控えをやめる操作（71 §71.11.5b）は、この状態を画面から作る（状態の file は退避名に残す）。PC を替えたときは本番の復元の手順の最後で用意し直す（71 §71.13）。
- 判定: ready（plan-draft、2026-10-08）。owner の判断事項 1〜3 は決定し、設計正本・D-114・D-111 に反映した。持ち帰りの 1 本の古さをアプリで扱うことは、owner が (a) 足さないに決めた（repo 外の回答台帳 TD-202。遷移の記録 5）。D-114 の Revisit に見直し契機として残し、Non-scope に置く。Plan Review round 1 の是正（2026-10-08）は設計正本の中で直し、新しい module・command を足していない（下の Contract Probe の P1 は Plan Gate の前の owner の確かめ）。

## Registration / Generation Obligations

本 lane（docs だけ）は該当なし（function-design の新しい file・REQ の増減・route・画面の新設をしない）。runtime の lane の義務: 新しい 6 command に `#[tauri::command(async)]`（43 §43.8.2 の「実行の thread」）+ `#[specta::specta]` を付け、`lib.rs` の `collect_commands!` と `generate_handler!` の両方に登録する（`scripts/check-command-drift.sh` が 4 つの集合の一致を見る）、`cd src-tauri && cargo run --bin generate_bindings`、`src-tauri/tests/design_compliance_test.rs` の `build_doc_to_modules_map()` の `71-mnt-backup.md` の行に `mnt::offsite` を足す、test に REQ-901 と決定 ID を付けて `cargo run --bin generate_traceability`（REQ-901 の coverage は既に required、`docs/spec/requirements.md:38`）。画面の新設は無い（既存の `/settings/backup` とホームの中）。

## Impact Review Lenses

| Lens | Question to answer | Evidence home | Applicability / finding | Follow-up artifact |
|---|---|---|---|---|
| Adapter / core boundary | Which concepts belong to replaceable external adapters, and which concepts are stable app-core contracts? | Architecture, function design, decision-log, Plan Packet | 媒体の発見（Windows の drive の列挙）とレジの SD の判定は OS・機種に依る部分で、`mnt::offsite` の中の小さな関数に閉じる。core は「確かめてから正式名」「目印で見分ける」「状態は DB の外」「3 日で知らせる」 | レジの機種が変わったら MNT-01-D10 の folder を IO-09 と一緒に替える |
| Fact check / design decision split | Which claims are observed facts from hardware/tool/files, and which are app decisions that need source-doc promotion? | Investigation doc, source design docs, decision-log | 事実: 店の回答（L-217・L-220・TD-009・TD-011・TD-187・TD-190）、今の実装（`VACUUM INTO` が正式名へ直接書く、`backup.rs:218` 以降）。外部前提: P1 は確認済み（TD-375）、P2〜P4 は Contract Probe の扱い（probe しない・設計で前提から外した・runtime の test で実測）。判断: D-114・MNT-01-D7〜D10・UI-11b-D14〜D16・UI-00-D12 | Contract Probe |
| Lifecycle / retry | What happens before, during, after, and after failure for import/export, duplicate input, rollback, retry, cancellation, and re-run? | Function design, DB design, UI design, Test Matrix | 作成の途中の失敗は作業名だけが残り次回に消える。同じ名前の正式名があれば既存を残して失敗。写しの途中の抜去は作業名が媒体に残り、次の写しで消える。照合の不一致は作業名を消して `last_failure`。写す元が検査に通らなければ何も書かず `last_failure`。媒体の入れ替えは目印で追い、1 回の確認の中は volume の識別に固定する。復元は status の file を変えない。用意の再実行は同じ `medium_id` を使う。新しい PC での復元の後は手動の backup を作ってから媒体を登録し直す（71 §71.13） | Matrix の State Lifecycle |
| Operator workflow | What does the operator do in the real sequence across app, external tool, media, print/export, backup, and recovery? | Screen/UI design, function design, Plan Packet manual checks | 店主の操作は無し（差しっぱなしの 1 本を抜かない）。owner が用意・入れ替え・短い確かめ・通しの演習・本番の復元を行う（Ordinary Operation、71 §71.13） | runtime の lane の L3（UI-11b-L3-6〜9） |
| Replacement path | If the external system changes, which files/modules/docs are replaced and which app-core contracts remain stable? | Architecture, function design, decision-log | 保存先を USB の HDD・network に広げるときは §71.11.3 の発見だけを替え、写し・照合・状態・知らせは残る | D-114 の Revisit |
| Data safety / evidence | How can the claim be supported by anonymized shape/count/hash/procedure evidence without committing real store data? | Plan Packet Data Safety, investigation doc, review evidence | 媒体には店の DB の控えが入る（暗号化しない）。持ち出す（判断事項 1 = C、owner が露出を受けた、TD-196）。repo には実データ・backup file・媒体の中身を置かない | Data Safety |
| Reporting / accounting semantics | Are totals, summaries, item records, returns, corrections, and inventory movements modeled separately enough to avoid false business meaning? | DB design, function design, report design, Test Matrix | not applicable: 業務の数値・集計を変えない（`inspect_backup` の商品の数・最後の記録は控えの確かめの表示だけ） | — |
| Manual verification | Which assertions cannot be proven by automated tests and require Windows native L3, external tool import, or real-device confirmation? | Plan Packet, Test Matrix, PR body | 実 USB の drive の種類の判定・用意・写し・入れ替えで drive 文字が変わっても見つかること・レジの SD の拒否・媒体から読む確かめ（68 §68.12 の UI-11b-L3-6〜9）。3 日の知らせと新しすぎる版の控えは L3 Eligibility の条件 (3) に当たるので自動 test | runtime の lane の Human Gate に manual |
| 環境・再現性 | 新設の環境依存（toolchain / CI runner / OS 差異等）を repo-pinned config で強制するか、明示的に defer するか | repo-pinned config, Plan Packet | drive の列挙は Windows だけ（既存の `windows-sys` の feature `Win32_Storage_FileSystem`、`src-tauri/Cargo.toml:48`。`GetLogicalDrives`・`GetDriveTypeW` が同じ feature に入ることは Codex の Plan Review round 1 が同版の crate の source で確かめた。書く先の固定〈71 §71.4〉の `GetFullPathNameW`・`CreateFileW`・`GetFinalPathNameByHandleW` も同じ module にある見込みで、runtime の lane の build で確かめる）。`SetThreadErrorMode` は `windows_sys::Win32::System::Diagnostics::Debug`（docs.rs の windows-sys 0.61.2）で、feature `Win32_System_Diagnostics_Debug` を既存の `windows-sys` に足す（runtime の lane の `Cargo.toml` の変更。新しい依存ではない）。Windows 以外は空の列で、test は root の列を受ける内部関数に一時 directory を渡す（71 §71.11.3）。SHA-256 は既存の `sha2`（`Cargo.toml:33`）、id は既存の `uuid`（`:37`）。新しい依存を足さない | runtime の lane |

## Boundary / Wire Contract

- producer: MNT-01（71 §71.11・§71.12）、CMD-11（43 §43.8.2〜§43.8.7）
- consumer: UI-11b（`getOffsiteBackupStatus`・`prepareOffsiteMedium`・`checkOffsiteBackup`・`inspectBackup`・`resetOffsiteState`・`stopOffsiteBackup`）、UI-00（`getOffsiteBackupStatus`）
- wire type: `PrepareOffsiteMediumRequest { selected_path: String }`、`InspectBackupRequest { backup_path: String }`、`OffsiteMediumView { label, drive_root, newest_copy? }`、`OffsiteCheckResult`（`kind` の tagged enum 5 値）、`OffsiteBackupStatus { prepared, last_success_at?, last_success_label?, days_since_last_success?, stale, attached[], last_failure_kind?, local_backup_old }`、`OffsiteFailureKind`（7 値、snake_case）、`BackupInspection { file_name, created_at?, size_bytes, schema_version, app_max_version, newer_than_app, quick_check_ok, product_count?, last_operation_at? }`。error は既存の `CmdError`（kind `validation` / `internal` だけ、`CmdErrorKind` に値を足さない）
- internal type: `OffsiteMediumMarker`（媒体の `InventoryBackup\offsite-medium.json`）と `OffsiteState`（`{app_data_dir}\offsite-backup.json`）。どちらも `format: 1` の JSON で wire に出さない
- precision/range: 日時は `YYYY-MM-DD HH:MM:SS`（ローカル時刻）の文字列。`days_since_last_success`・`product_count`・版は i64 だが値は小さく、JS の安全な整数の範囲に収まる。SHA-256 は 64 桁の小文字 hex（状態の file だけ）
- round-trip path: 用意（目印・状態を書く）→ 確認（目印を読み、写し、状態を書く）→ status（状態と見えている媒体を読む）→ 画面・ホーム。復元: 一覧の行 / 選んだ file の path → `inspectBackup` → 既存の `restoreBackup({ backup_path })`
- invalid input: 空の `selected_path` / `backup_path` は `validation`。取外し可能でない drive・レジの SD・`format` が 1 でない目印の媒体・`InventoryBackup` か目印が reparse point の媒体は `validation`（固定の文、43 の表）。`format` が 1 で `medium_id` か `label` の読めない目印・JSON として読めない目印は作り直す（写しは消さない）。形の違う状態の file は `StateUnreadable`（上書きしない。画面の作り直しが元の bytes を退避してから作る）。`format` が 1 でない数の状態の file の作り直しは `StateUnsupported`（`validation`、何もしない）。用意済みの媒体が見えない作り直しは `NoMediumAttached`（`validation`、何もしない。やめる操作も名指す）。やめる（`stop_offsite_backup`）は入力なしで、移せなければ `StopFailed`（`internal`、何もしない）。固定できない・レジの SD の上の保存先は、手動の backup で `validation`（43 §43.6）。このアプリの backup でない file（空・別用途・版 0・必須の表なし）は `inspect_backup` が `Err`
- compatibility: 既存の command・DTO・`CmdErrorKind`・`app_settings`・DB の schema は変えない。新しい版のアプリの file を古い版が上書きしない: 目印は `format` が 1 でなければ書き換えず、その媒体に書かない（用意は `MarkerUnsupported`）。状態の file は `format` が 1 でなければ `StateUnreadable` で止まる（71 §71.11.1・§71.11.2 手順 4・§71.11.3）

## Test Plan

Test Design Matrix: [2026-10-07-backup-offsite](test-matrices/2026-10-07-backup-offsite.md)（runtime の lane が実装する test の設計。本 lane は docs の検査だけ）。

- targeted tests: 本 lane は `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-10-07-backup-offsite.md` と full。
- negative tests: Matrix の Negative Paths（runtime）。
- compatibility checks: 既存の restore の契約と backup 系の command の wire（Matrix）。
- data safety checks: 本 lane の差分に実データ（店の DB・backup file・媒体の中身・実 JAN・商品名・金額）を入れない。
- main wiring/integration checks: runtime の lane（共通の確認の hook から `checkOffsiteBackup` まで、ホームの query まで）。

## Review Focus

- Ordinary Operation の列が、正常な条件で目的（店主の手間なしに PC の外の控えが毎日取れ、取れない日が続けば知らされ、PC を失っても戻せる）を達成できるか。
- レジの SD・目印の無い媒体・別の PC で用意した媒体・`InventoryBackup\` の外に書く経路が無いか（MNT-01-D8・D10、71 §71.11.3）。
- 作成途中・照合の失敗・抜去の途中の file が、PC の中でも媒体の上でも成功の控えと数えられないか（MNT-01-D7、71 §71.11.4）。
- 状態を DB の外に置いたことで、復元・状態の file の破損・PC の入れ替えのどれでも「黙って写さなくなる」経路が残らないか（71 §71.11.5・§71.11.7、Design Readiness の絶対保証の自己点検）。
- 復元の前の確かめ（MNT-01-D9）が、restore の契約（MNT-01-D1〜D6、`CmdErrorKind` の 3 値、UI-11b-D2〜D5）を変えずに新しすぎる版の文言を出せるか。MNT-03-D11 の改めた文と矛盾しないか。
- 目印と状態の file の版の扱い（71 §71.11.1・§71.11.2 手順 4）で、新しい版のアプリが置いた file を古い版が上書き・削除する経路が無いか。
- 運用の型の比較（71 §71.11.6）が、盗難・火事・ランサムウェア・PC の持ち帰り（TD-009）で正しいか。採った C の前提（TD-011 の見直し・露出の受容）と、訪問の間隔が決まっていないこと（TD-199）・空きの USB の口が 1 つ（TD-198）の帰結が、Ordinary Operation の残るリスクと 71 §71.11.6・§71.11.7 に漏れなく書かれているか。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3 / 非対象 |
|---|---|---|---|---|
| D-114 | `docs/decision-log.md` D-114 | 本 lane は設計の記録。runtime は後続 | runtime（Matrix） | — |
| MNT-01-D7 | 71 §71.4 手順 3・4・4a・4b、エラーハンドリング、§71.8 手順 3 | runtime: `mnt/backup.rs` の `create_backup` | Matrix の MNT-01-D7 の行 | L3: UI-11b-L3-6（`.partial` が残らない） |
| MNT-01-D8 | 71 §71.11.1〜§71.11.7（§71.11.5a の作り直し・§71.11.5b のやめるを含む） | runtime: `mnt/offsite.rs` | Matrix | L3: UI-11b-L3-6・L3-8 |
| 71 §71.3 名前の解析 | 71 §71.3（規約外の多バイトの名前で panic しない） | runtime: `mnt/backup.rs` の `extract_datetime_from_backup` | Matrix の §71.3 の行 | 非対象 |
| MNT-01-D9 | 71 §71.12、22 MNT-03-D11 | runtime: `mnt/backup.rs` の `inspect_backup`、`db/migration.rs` の共有 | Matrix | L3: UI-11b-L3-9 |
| MNT-01-D10 | 71 §71.4 手順 0 と「書く先の固定」、§71.5 手順 0、§71.11.2 手順 3、§71.11.3 | runtime: `mnt/backup.rs`・`mnt/offsite.rs` | Matrix | L3: UI-11b-L3-7 |
| CMD 43.8.2〜43.8.7 | 43 §43.8.2〜§43.8.7（エラーの写像・DTO・lock を放す時点、作り直しの前の backup） | runtime: `cmd/settings_cmd.rs`・`lib.rs` | Matrix | 非対象 |
| UI-11b-F8・D14 | 68 §68.2・§68.4・§68.5・§68.9・§68.11 | runtime: `features/backup-restore`（`OffsiteBackupPanel`） | Matrix | L3: UI-11b-L3-6・L3-8、目視の確認 |
| UI-11b-F9・D15 | 68 §68.4 手順 1・1a・2、§68.5、§68.7 | runtime: `BackupRestorePage` | Matrix | L3: UI-11b-L3-9 |
| UI-11b-F10・D16 | 68 §68.5、§68.10 | runtime: `useAutoBackupCheck` | Matrix | 非対象 |
| UI-00-D12 | 53 §53.5 | runtime: `features/home` | Matrix | 目視の確認（3 日の状態は fixture、L3 Eligibility の条件 (3) により自動 test） |
| 22 §16 の作り直しの手順 (3)・手順 3 | 22 §16「folder の外でアプリが読む場所」(3) と手順 3（媒体の `InventoryBackup` を移す） | 手順の文（code なし） | — | 非対象（開発者の作り直しの手順） |
| MNT-03-D11（改めた文） | 22 §3.2 の MNT-03-D11「復元への波及」 | 版の判定の関数を共有するだけ。restore の中は変えない | 既存の MNT-03-D11 の test（runtime が回帰で回す） | 非対象 |
| 隣接の除外: MNT-01-D1〜D6、UI-11b-D2〜D5・D11〜D13、D-032 | 71 §71.4〜§71.8、68 §68.5 | 変えない（確かめは restore の前に足すだけ。共通の確認の停止・世代番号を共有する）。UI-11b-D13 の停止の始まりだけ、UI-11b-D15 が控えの確かめの開始へ早める（結果での再開・fatal の後の停止は D13 のまま、unmount でも D13 が優先） | 既存の test（runtime が回帰で回す） | 非対象 |
| 隣接の除外: UI-11b-D8（`backup_path` は picker だけ） | 68 §68.5 | 変えない（取外し可能な drive を選ばせない変更は Non-scope） | 既存 | 非対象 |
| 隣接の除外: D-111 の (8)・残りの判断 A・B、IO-09-D3 | D-111、29 §29.7.6 | 変えない。残りの判断 B は判断事項 2 で「含めない」と決め、D-111 に `Superseded in part by: D-114` の行を足した（TD-197）。A は変えない。MNT-01-D10 は IO-09-D3 と同じ folder で SD を見分ける | — | 非対象 |

## Contract Probe

- P1 店で使う USB メモリが、Windows の `GetDriveTypeW` で `DRIVE_REMOVABLE` に見える: **確認済み（2026-10-22、repo 外の回答台帳 TD-375）**: owner が控え用に用意した USB メモリを owner の PC に差し、`[System.IO.DriveInfo]::GetDrives()` で USB の drive が `Removable`（内蔵の drive は `Fixed`）と出た。以下は起票の時点の記述（設計の前提。外れると用意が `NotRemovable` で拒まれ、Ordinary Operation の 1 行目から進めない）。Plan Gate の前に owner が確かめる。結果は Coordinator が packet に記録する。確かめ方: owner の USB メモリを owner の PC に差し、PowerShell で `[System.IO.DriveInfo]::new('E').DriveType`（`E` は差した drive の文字）を実行して `Removable` と出るかを見る（エクスプローラーの表示は場所により「USB ドライブ」等になり、合否を決めにくい。.NET の `DriveInfo.DriveType` の値は `CDRom` / `Fixed` / `Network` / `NoRootDirectory` / `Ram` / `Removable` / `Unknown`、https://learn.microsoft.com/en-us/dotnet/api/system.io.driveinfo.drivetype 。資料はこれが `GetDriveTypeW` を呼ぶとは書いていないので、`Removable` を `DRIVE_REMOVABLE` と同じ区分と見なす前提を runtime の lane の L3〈UI-11b-L3-6〉で確かめる）。外れたら USB の HDD と同じく D-114 の Revisit（固定 disk を許す条件の設計）へ戻す。
- P2 写した直後の読み戻しが Windows の file cache から返るか: **probe しない**（設計は保証の範囲を「OS が返す bytes の一致」に狭めて書き〈71 §71.11.7〉、媒体の記憶素子の故障は差し直した後の短い確かめ〈§71.13〉が受け持つ）。
- P3 媒体の入っていない読取り機（SD の読取り機の空の口）の root を見たとき、Windows がエラーの dialog を出すか: **設計で前提から外した**（probe しない）。媒体を探す・読み書きする間は常に、その thread の error mode を `SetThreadErrorMode(SEM_FAILCRITICALERRORS)` にして終わったら戻す（71 §71.11.3。Microsoft の資料: critical-error-handler の message box を出さず error を呼んだ thread へ返す。thread だけの設定で、process 全体の `SetErrorMode` より推奨。https://learn.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-setthreaderrormode ）。dialog が出る機種でも出ない機種でも同じ振舞いになる。
- P4 SQLite の `immutable=1` の読取り専用の open が file と folder に何も書かない: SQLite の公式資料（URI の `immutable` の項: 読取り専用で開き、lock と変更の検出をしない）に依る。runtime の lane の test が前後の hash と folder の一覧で実測する（Matrix の MNT-01-D9 の行）。
- P5 店の PC に、バーコードリーダー・SD の読取り機と別に USB の口が空いている: **観測（owner 回答 2026-10-08、repo 外の回答台帳 TD-198）**: 空きは 1 つ。差しっぱなしの控え 1 がその口を占めるので、2 本を同時に差さず、用意と入れ替えは抜いてから差す。SD の読取り機は自分の口のままで、毎日の SD の読取りは変わらない。扱いは Ordinary Operation の残るリスク、68 UI-11b-L3-8。
- P6 owner の訪問の間隔: **観測（owner 回答 2026-10-08、TD-199）**: 決まった間隔は無い。持ち帰りの 1 本が守る範囲（盗難・火事で失う日数）は最後の入れ替えからの日数で、上限が無い（Ordinary Operation の残るリスク、71 §71.11.6・§71.11.7、D-114 の Guarantee range）。
- P7 `VACUUM INTO` が volume GUID path（`\\?\Volume{GUID}\…`）の作業名へ書ける（71 §71.4 の「書く先の固定」の前提、Final Review の是正 2026-10-22）: **未確認**。runtime の lane が Windows の test か L3（UI-11b-L3-4・L3-7）で、固定した path へ作業名ができ正式名へ公開されることを確かめる。外れたら（SQLite が path を書き換える・開けない）、書く先の固定の形を設計へ戻す（runtime の lane の packet で止める）。
- P8 FAT32・exFAT の取外し可能な drive（店の USB メモリ）で、root を `CreateFileW` で開いた handle の `GetFinalPathNameByHandleW(…, VOLUME_NAME_GUID)` が成功する（71 §71.4 の「書く先の固定」を媒体の root にも使う前提、closure の是正 2026-10-22）: **未確認**。資料の Remarks は Mount Manager に無い volume では失敗しうるとする。runtime の lane が Windows の test か L3（UI-11b-L3-6）で確かめる。外れたら、P7 と同じく媒体の root の固定の形を設計へ戻す（runtime の lane の packet で止める）。
- 観測済みで probe の要らない前提: `sha2`・`uuid`・`windows-sys` の `Win32_Storage_FileSystem` は既存の依存（`src-tauri/Cargo.toml:33`・`:37`・`:48`）。今の `create_backup` は `VACUUM INTO` で正式名へ直接書く（71 §71.4 の旧い手順 3・4、`mnt/backup.rs:218` 以降）。今日の backup の判定は前方一致と `.db` の後方一致（`mnt/backup.rs:446`。規約外の名前を数えるので、runtime の lane で完全一致に替える。申し送りの表）。新しすぎる版の backup の復元は差し替えの後の open で拒否され `restore_failed_recovered` の固定の文になる（22 MNT-03-D11、`src/features/backup-restore/BackupRestorePage.tsx:305`）。

## Data Safety

- tracked に書かない: 店の DB・backup file・媒体の中身・状態の file の中身、実 JAN・商品名・金額、店主の発言の原文、repo 外の回答台帳の細部（TD・L の番号と要旨だけを書いた）。
- local-only: repo 外の回答台帳（TD-009・TD-011・TD-031・TD-187・TD-190・TD-196〜TD-199、L-217・L-220）。owner の USB メモリと、通しの演習で別の PC に戻した店のデータ（演習の後に消す、71 §71.13）。
- synthetic-only: runtime の lane の test は一時 directory に合成の DB の控え・目印・状態の file・`CASIO\SR500_550_4000` の folder を作る。L3 は試しの DB で行い、レジの SD の拒否は試しの媒体に同じ folder を置いても確かめられる（68 UI-11b-L3-7）。
- 媒体の露出: 控えは暗号化しない。C で媒体を店の外へ持ち出すので、失くしたときに店の記録が読める。owner はこの露出を受けた（判断事項 1、TD-196）。

## Implementation Results

design lane で docs だけ（code・test・bindings は変えない、AC6）。PR: [#155](https://github.com/higekiri-w/inventory-system-desktop/pull/155)。

成果物:

- 設計正本: `docs/function-design/71-mnt-backup.md`（§71.1、§71.3 の名前の解析、§71.4 の MNT-01-D7・D10 と §71.4.1、§71.8 手順 3、§71.10 の D-114 の表、§71.11 の MNT-01-D8、§71.12 の MNT-01-D9、§71.13）、`68-ui-backup-restore.md`（UI-11b-F8〜F10・D14〜D16、§68.3〜§68.13 の該当の行、UI-11b-L3-6〜9）、`53-ui-home.md`（§53.1、UI-00-D12）、`43-cmd-settings-log.md`（§43.8.2〜§43.8.7、§43.11）、`22-mnt-migration.md`（MNT-03-D11 の「復元への波及」、§16 の作り直しの手順の folder の外の場所 (3) と手順 3）、`docs/architecture/cmd-task-specs.md`（CMD-11）・`mnt-task-specs.md`（MNT-01）、`docs/ARCHITECTURE.md`・`docs/FUNCTION_DESIGN.md` の MNT-01 の行、`docs/SCREEN_DESIGN.md`（1 日の動線、ホーム、バックアップ・復元画面）。
- 決定: `docs/decision-log.md` の D-114 と、D-111 の末尾の `Superseded in part by: D-114` の行（未決 B）。
- Test Design Matrix: `docs/archive/plans/test-matrices/2026-10-07-backup-offsite.md`（runtime の lane が実装する test の設計）。
- `docs/backlog.md`: 本 lane の項（着手・導入の前の関門・設計の Draft と再開・残り）と「保存と起動の守りの follow-up」の (2) の着手。

AC1〜AC6 の最終の出力（closure の是正と TD-381 の反映の後、逐語）:

- AC1: `114:**MNT-01-D7: …`・`122:**MNT-01-D10: …`・`819:**MNT-01-D8: …`・`863:**MNT-01-D9: …` の 4 行
- AC2: 68 は `118:| UI-11b-D14 |`・`119:| UI-11b-D15 |`・`120:| UI-11b-D16 |` の 3 行、53 は `213:#### PC の外の控えの知らせ（UI-00-D12、D-114）` の 1 行
- AC3: `211:| 43.8.2 |`〜`216:| 43.8.7 |` の 6 行（式は `[2-7]`）
- AC4: `981:## D-114: …` の 1 行
- AC5: 3 本とも出力なし・exit 1
- AC6: `git diff --name-only c98fe7dd HEAD -- src src-tauri` は出力なし・exit 0

計画からの外れ:

- main への同期（遷移の記録 9）: 衝突は `docs/backlog.md`・`docs/decision-log.md`。後から main に入った契約（D-118 (a) の v9、原価の lane C の vB、D-123〜D-127）との食い違いを 4 か所で直した: 71 MNT-01-D9 の保証の範囲（作り直しの起点の migration より前の版の開発・試験 DB の控え）、22 §16 の folder の外の場所 (3) と作り直しの手順 3（Scope の 22 の行が §16 に広がった）、D-114 の Status から実装の状態を外した（D-126）、71 §71.10 の行番号の引用（D-125）。AC3・AC4 の期待を測り直した。
- 同期の closure の是正（Review Response、遷移の記録 10）: D-111 は本文への差し込みをやめて `Superseded in part by` の形にした。§71.11.7 に手で置いた file の扱い。backlog の待ちの文。Non-scope の MSI の句と Scope の decision-log の行。AC6 の command の base を同期の基点 `c98fe7dd` に替えた。MNT-01-D9 の確かめの対象。
- closure 2 の P3 の是正（遷移の記録 11、Review Response）: N1 P1 の状態の文、N2 §71.11.4 手順 4a と §71.11.7 の例外、N3 MNT-01-D9 の止まる検査を節で引く、N4 MSI の手順書の流れ 3 の申し送り先。
- Final Review の是正（2026-10-22、Codex 492 と Fable 5.1 の broad、Coordinator が全件を受理）: MNT-01-D10 の書く先の固定と祖先の判定（`cleanup_old_backups` を含む）、`VACUUM INTO` の `Err` の後の作業名、読めない記録の作り直し（command `reset_offsite_state`〈43 §43.8.6〉・`OffsiteError::StateUnsupported`・UI の `offsite_resetting` を足した）、`OffsiteBackupStatus.local_backup_old`（`offsite_status` が `backup_dir` を受ける）と原因ごとの知らせの文、MNT-01-D9 の確かめの後の入れ替えの例外、Contract Ledger の 2 行、行番号の引用、Contract Probe の P7。
- closure の是正（2026-10-22、Codex 496 と Fable 5.1 の closure、Coordinator が全件を受理）: 書く先の固定を `pin_to_volume`（正規化 → 実体の path）へ置き換え、元の path と実体の path の両方の祖先で判定し、媒体の root の固定も同じ関数にした（junction を含む保存先、固定できない保存先の名指し、Contract Probe の P8）。作り直しの順を「媒体を探す → 新しい記録の作業名 → 原本の copy → 置き換え」へ組み替え、`NoMediumAttached`・`StateResetFailed` を足した（途中の失敗と媒体 0 本で知らせと再試行の入口が残る）。「4 command / §43.8.5」の引用の sweep、AC3 の式。
- owner の決定（2026-10-22、repo 外の回答台帳 TD-381。USB の控えは任意）: 「PC の外の控えをやめる」（71 §71.11.5b、43 §43.8.7、68 §68.4 手順 5・`offsite_stopping`、`OffsiteError::StopFailed`）を範囲に入れ、Non-scope の「媒体を登録から外す操作は持たない」を置き換えた。媒体が無いときの作り直しの文が、作り直しとやめるの 2 つを名指す。Goal と Ordinary Operation に USB を使わない店の行。
- 上の Final Review の是正のほかに、新しい module・command・DTO・error kind は計画から足していない。

残りの危険:

- runtime は後続の lane。店で PC の外の控えが毎日取れる通常運用は runtime の後に成り立つ（Ordinary Operation）。
- `Removable` を `DRIVE_REMOVABLE` と同じ区分と見なす前提は、runtime の lane の L3（UI-11b-L3-6）で確かめる（Contract Probe の P1）。
- `VACUUM INTO` が volume GUID path へ書けることは runtime の lane で確かめる（Contract Probe の P7。外れたら書く先の固定の形を設計へ戻す）。FAT32・exFAT の取外し可能な drive で `GetFinalPathNameByHandleW(VOLUME_NAME_GUID)` が成功することも runtime の lane で確かめる（Contract Probe の P8）。
- 確かめた後に同じ drive 文字の媒体が入れ替わると、このアプリのものでない DB へ復元しうる（71 MNT-01-D9。restore の契約は Non-scope で、事前バックアップから戻せる）。
- 読み戻しの照合は Windows の file cache から返りうる（P2、71 §71.11.7）。
- 持ち帰りの 1 本の古さはアプリから見えず、訪問の間隔が決まっていないので、PC と差しっぱなしの 1 本を同時に失ったときに失う日数に上限は無い（Ordinary Operation の残るリスク、D-114 の Guarantee range）。
- 利用者が `InventoryBackup\` に手で置いた規約の名前の file は、検査を通らずに写しと数えられうる（71 §71.11.7。復元の前の確かめが止める）。MSI の手順書の流れ 3 の直しは runtime の lane の起票で行う（申し送りの表、backlog）。
- 更新・migration の前の保全の gate は無い（Non-scope、backlog の同項の残り）。導入の前の関門で足された 3 点のうち、PC の中の保持日数と日付での掃除は本 lane の外（backlog）。
- 作り直しの起点の migration より前の版の開発・試験 DB の控えは、確かめを通って復元で止まる（71 MNT-01-D9。店の初回の版はそれらの migration を含むので、店の控えには当たらない）。

closeout（2026-10-23）:

- 結果: 確かめ済みの最新の backup を用意した USB メモリへ自動で写して読み戻しで照合し、PC の中と PC の外の成功を分けて画面とホームに出し、復元の前に控えを確かめて新しすぎる版・壊れた控えを止め、別の PC で媒体の控えから戻す手順までを設計正本と D-114 に置いた。USB の控えは任意で、やめる操作を持つ（TD-381）。code・migration・bindings は変えず、店で PC の外の控えが毎日取れる通常運用は後続の runtime の lane の後に成り立つ。
- review・CI・merge: Plan Review は round 1〜3（上限）と同期の closure（reject → 是正）・同期の closure 2（fresh Opus、approve）で、owner が R4 の計画を承認した（TD-376）。Final Review（Minimum 2、Claude 側 Fable 5.1 と Codex GPT-6 Astra）は broad と closure の reject と是正を重ね、途中の契約の変更を Gated Amendment として記録して broad を取り直し、closure の 2 回目で両者が approve（P3 だけ）した。helper の専用 record は review pass・r4 pass で、owner の R4 の承認と Ready（TD-383）の後の run の `Merge gate` は success、owner の merge の指示（TD-385）の後に helper 経由で squash merge した。PR: [#155](https://github.com/higekiri-w/inventory-system-desktop/pull/155)
- 後続: `docs/backlog.md` の「PC の外の控えの runtime」（申し送りは上の「runtime の置き場所」と申し送りの表、下の Review Response の closeout の段落の (a)〜(c)、Contract Probe の P7・P8）と、自動 backup の確認の失敗の知らせの候補（AI の review 由来）。導入の前の関門 3 の残りは、その runtime の lane と復元の予行演習（71 §71.13）。

## Review Response

round 1（`76eed0a4`）: Claude 側 fresh Opus 5.5 = reject（P2 2 / P3 6）、Codex GPT-6 Astra（発注 248）= reject（P1 4 / P2 8 / P3 1）。重なりは 4 件（Opus #1 = Codex #6、Opus #3 = Codex #9、Opus #5 = Codex #3、Opus #7 = Codex #13）。Coordinator が採否を決め、全件を採用した。是正は起草役の `7168aae9`。設計の方向（新しい module・command）は変えず、runtime の lane の Scope に `collect_today_backup_names` の完全一致・上書きしない公開・既存の `windows-sys` の feature の追加・失敗の種類の値の追加が加わる（packet の申し送りの表）ので、plan-gate に留める。

- P1（Codex #1）正式名への rename が同じ秒の既存の backup を上書きする。是正: 71 §71.4.1「上書きしない公開」を新設（Windows は `MOVEFILE_REPLACE_EXISTING` を付けない `MoveFileExW`、衝突は既存を残して失敗）。
- P1（Codex #2・#3、Opus #5）写す元を検査せず、空・別用途の DB を「戻せます」と判定する。是正: §71.12 に「このアプリの backup である」検査（版 0・必須の表の欠けを拒否）を足し、作成・PC の外へ写す元・復元の詳細の 3 つで共有する。不合格の写す元は `SourceUnverified` で、写しの公開・成功の記録・掃除へ進まない。
- P1（Codex #4）媒体の検査と書込みが drive 文字でしか結び付かない。是正: §71.11.3 で 1 回の確認の中の読み書きを volume GUID path に固定し、copy・公開・掃除の直前に目印の `medium_id` を読み直す。
- P2（Codex #5〜#11、Opus #1・#2）: 目印・状態の file の公開に親 directory の sync を足した（#5）。確認の結果に依らず status を invalidate する（#6・Opus #1）。未知の `format` の目印を作り直さず `MarkerUnsupported`（#7）。`.partial` の削除を規約に合う名前だけに（#8）。今日の判定を完全一致に（#9・Opus #3）。入れ替え時の短い確かめを 2 段に分けた（#10）。中断点ごとの Matrix の行と WAL の fixture（#11）。新しい PC での復元の後、媒体を登録し直す前に手動の backup を入れ、合格に「媒体の最新が戻した結果と合う」を足した（Opus #2）。
- P2（Codex #12）外部前提の probe を Plan Gate の後に送っていた。是正: P1（USB メモリが `DRIVE_REMOVABLE`）を Plan Gate の前に owner が確かめる前提にした。owner は新しい USB メモリを用意してから確かめる（repo 外の回答台帳 TD-204）。結果を記録するまで plan-approved へ進めない。P3（空の読取り機の dialog）は、媒体を探す間は常に `SetThreadErrorMode(SEM_FAILCRITICALERRORS)` を設定する設計にして前提から外した。
- P3: 目印の札を最大の N + 1 に（Opus #4）。抜去の途中・`Io`・`StorageFull` の行（Opus #6）。Design Readiness の決定済みの書き方（Opus #7・Codex #13）。Risk に Rollback / recovery notes（Opus #8）。
- 起草役の判断で Coordinator が受けたもの: 空の DB の控えを「今日の backup あり」と数えない設計は採らない（店の記録の有無の経験則を backup に持ち込み、復元の時に自動で backup を作る案は restore の契約〈Non-scope〉を変えるため）。`windows-sys` の feature 名は module path から推した未確認の事実で、runtime の lane の build で確かめる。

round 2（`48698c68`）: Claude 側 fresh Opus 5.5 = reject（P2 1 / P3 6）、Codex GPT-6 Astra（発注 250）= reject（P1 1 / P2 3）。Coordinator が採否を決め、全件を採用した。是正は起草役の `f3120ed4`（自己点検で同型の穴も直した）。

- P1（Codex #1）媒体の `InventoryBackup` が reparse point（junction 等）だと PC の中や別の媒体へ転送され、成功の表示のまま PC の外の控えにならない。是正: 71 §71.11.3 で reparse point を追わず、媒体の `InventoryBackup` が reparse point なら用意済みに数えず、中の entry が reparse point なら目印・写し・作業名に数えず開かず消さない。用意は `OffsiteError::Redirected` で拒む。
- P2（Codex #2）規約外の多バイトの名前で名前の解析が文字境界の panic を起こす（今の `list_backups` にもある潜在の panic）。是正: 長さ・区切り・ASCII の確認を slice の前に置き、`extract_datetime_from_backup` を直す申し送りと回帰の行。
- P2（Codex #3）復元の控えの確かめの間に 定期の確認（71 §71.11.4）の掃除が選んだ控えを消しうる。是正: UI-11b-D15 で確かめの前に確認を止めて実行中の回の完了を待ち、取消・復元の終わりまで止める。
- P2（Codex #4）公開の後の失敗の oracle が `UpToDate` と矛盾。是正: 公開の前と後で oracle を分けた。
- P2（Opus #1）4 command が main thread で動き、写しの間に画面が固まる（Tauri v2 の公式資料）。是正: 43 §43.8.2 で `#[tauri::command(async)]` で定義し、Matrix に attribute の確かめの行。
- P3: 既存の test の注入点の移し替え（Opus #2）、`OffsiteError` の 9 値の enum（Opus #3）、今写した file を残す保持の case（Opus #4）、P1 の確かめ方を PowerShell の 1 行に（Opus #5）、新しい PC で札が重ならない手順（Opus #6）、新しい PC の保存先と状態の file が読めないときの手順と文（Opus #7）。
- 起草役の判断で Coordinator が受けたもの: 状態の file が読めないときのホームの文は店主向けに「バックアップ画面で確かめてください」とする。`backup_dir` 自身は reparse point かを見ない（既存の設定を壊さない）。自己点検で、用意が状態の file を読めないときに目印だけ書く順序の誤りを直し、定期の確認（71 §71.11.4）と競合しうる 5 つの操作の排他の表を 71 §71.11.4 に置いた。

round 3（上限、`7a38423c`）: Claude 側 fresh Opus 5.5 = reject（P2 1 / P3 4）、Codex GPT-6 Astra（発注 254）= reject（P2 2 / P3 1）。P1 は 0。重なりは Opus #1 = Codex #2。round 天井に達したので次の round を開始せず、全件を disposition「同型指摘の一括是正」とした（Coordinator が採用。是正は起草役の `b4c77d98`、同型の穴の自己点検を含む）。Coordinator の確かめ: 68 UI-11b-D15 の unmount の再開が「復元の実行中でも fatal の 2 種の後でもないとき」に限られている、71 §71.13 の短い確かめの (1) と (2) の間に「詳細を閉じて `ready` に戻す（確認が再開する）」がある、43 の新しい 4 command の attribute がすべて `#[tauri::command(async)]`（43 に残る `#[tauri::command]` は既存の command の §43.1〜§43.10 だけ）、71 D7 の Compatibility が公開の後の失敗で検査済みの正式名が残りうることと合っている、を実読で確かめた。

- P2（Codex #1）入れ替えの短い確かめの後に定期の確認を再開する操作が無い。是正: 71 §71.13・Ordinary Operation・Matrix に「詳細を閉じて ready に戻す」遷移。
- P2（Opus #1・Codex #2）unmount で無条件に再開し、D13 の停止と衝突する。是正: 上のとおり限り、Matrix に fatal の各 kind・復元の実行中の離脱の停止の oracle。
- P3: `stale` を写した日と写した控えの日付の古い方から数える（Opus #2、文と wire は変えない）。状態の file の直し方に先に手動の backup（Opus #3）。lock は公開関数の入口で 1 回（Opus #4）。登録の節の attribute を async に（Opus #5）。D7 の Compatibility（Codex #3）。
- 自己点検で直したもの: 定期の確認の停止と再開の組を全経路で並べて閉じた。§71.11.7 の bytes の一致の例外 2 つ、Non-scope・Contract Ledger・D-114 の「UI-11b-D11〜D13 は変えない」の言い切り（D15 が停止の始まりを早める）を直した。

同期の closure（2026-10-22、`d189560a`）: Claude 側 fresh Opus 5.5 = reject（P1 0 / P2 1 / P3 5）。Coordinator が F1 を decision-log の頭の「Status の書き方」と D-111 の本文で確かめ、全件を採用した。是正は Writer の `962731ec`。

- P2（F1）merge 済みの D-111 の本文に「未決 B の決着」の行を差し込んでいた。是正: 行を消し、D-111 の末尾に `Superseded in part by: D-114` の 1 行を足し、消した中身を D-114 の owner の決定 2 へ移した。
- P3: §71.11.7 に手で置いた file も写しとして数えて掃除する旨（F2）、backlog の待ちの文（F3）、Non-scope の MSI の句と Scope の decision-log の行（F4）、AC6 の base（F5）、MNT-01-D9 の確かめの対象と止まる条件の引用（F6）。

同期の closure 2（2026-10-22、`962731ec`）: Claude 側 fresh Opus 5.5 = approve（P1 / P2 0、P3 4）。F1〜F6 は全部 closed。P3 は implementing で直す（Writer）: N1 Design Readiness に「P1 は Plan Gate の前に owner が確かめる」が残る、N2 71 §71.11.4 の 4a の括弧と保証の範囲の文が手で置いた file を例外に挙げていない、N3 MNT-01-D9 の止まる条件を手順の列挙でなく 22 §16・§19 の節で引く、N4 MSI の手順書の流れ 3 の申し送り先を runtime の lane だけにするか backlog の同項の「残り」に 1 行足す。

Final Review と closeout（2026-10-23）: Final Review は Minimum 2（Claude 側 Fable 5.1 と Codex GPT-6 Astra、各 high）。

- broad: 両者 reject（Fable P2 1 / P3 4、Codex 492 P1 1 / P2 3 / P3 1）。是正は書く先の volume への固定と祖先の判定、読めない記録の画面での作り直し、知らせの原因の区別ほか（Implementation Results の「Final Review の是正」）。
- closure: 両者 reject（Fable P2 2 / P3 4、Codex 496 P2 2 / P3 1）。是正は書く先の固定の `pin_to_volume` への置き換えと作り直しの順の組み替えで、同時に owner の決定 TD-381（USB の控えは任意、「PC の外の控えをやめる」を範囲へ）を反映した。ここまでの契約の変更を Gated Amendment として `Amendments` に記録し（介入の上限 10 → 11）、broad を取り直した（helper は broad の Plan Commit・Amendments が packet と同じことを求める）。
- broad の取り直し: 両者 reject（Fable P2 1 / P3 6、Codex 499 P2 2 / P3 1）。是正は `RegisterSd` で見つけた folder を名指すことと、失敗の後の画面を取り直した status で決めること。
- closure: Fable approve（P3 6）、Codex 502 reject（P2 2 / P3 3）。是正は作り直しの手順 7 の後の status を §71.11.5 の式どおりにする、やめる操作の画面から届かない再試行の経路を消す、`io` の文を (5) に戻す、PC 側の状態の失敗を `Io` にする。
- closure の 2 回目: 両者 approve（Fable P3 2、Codex 505 P3 1）。helper の record は review pass、r4 pass（owner の R4 の承認と Ready、TD-383）。Ready の後の run の `Merge gate` は success で、owner の merge の指示（TD-385）の後に merge した。

P3 の disposition（Coordinator の裁定）:

- packet の errata（packet を変えると Gated Amendment になるので merge の前に直さず、ここに記録する）: Scope の `docs/architecture/cmd-task-specs.md` の行の「CMD-11 の表に 5 行」は 6 行（43 §43.8.2〜§43.8.7）。Contract Ledger の MNT-01-D10 の行に 43 §43.6（手動の backup の `pin_backup_dir` と `BackupDirError` の写像）が無い。Boundary / Wire Contract の precision/range の、版も値が小さく JS の安全な整数の範囲に収まるとする文は、43 §43.8 の DTO の形の文が正本（版は範囲を仮定せず、UI は判定にも表示にも使わない）。AC1・AC2 の期待の行の位置は merge の時点で AC1 `114:`・`122:`・`827:`・`871:`、AC2 68 `119:`〜`121:`・53 `213:`（数と中身は Implementation Results の最終の出力と同じ）。
- 後続の runtime の lane へ申し送る（closure の 2 回目の P3）: (a) 用意済みの媒体が差してある（`attached` が空でない）のに `io` が続くとき（書込み禁止の媒体ほか）、53 UI-00-D12・68 UI-11b-D14 の (5) が「USB メモリが差してあるか確かめてください」を出して副情報と食い違う。`attached` で文を分ける。(b) 手順 7 の中で成功の記録を置き換えた後の親 directory の sync の失敗で、card に成功の日時と「PC の外の控えを写せませんでした」が並ぶ（71 §71.11.4 の表の「状態の file の書込みの失敗 → `Io`」は merge より前からの契約）。置き換えの後の sync の失敗を掃除と同じ「成功のまま・warn」にするかを runtime の lane で決める。(c) Matrix の原因の文の行に `stale = true` の前提が明記されていない。
- backlog（AI の review から出た候補。出所を書き、owner の確認の前に要件にしない）: 自動 backup の確認（UI-11b-D13、owner 決定 2026-09-25 (a)）の失敗は連続失敗の最初の 1 回の toast だけで原因を名指さず、失敗のときに backup の一覧を取り直さない。USB を使わない店では、自動 backup が止まり続けても原因が届きにくい（PC の外の控えを用意した店はホームの `local_backup_old` の知らせと手動 backup の文が名指す）。`docs/backlog.md` に 1 項目として置いた。

packet の本文（Scope・Contract Ledger・AC・Contract Probe）は当時の契約の記録として書き換えない。Owner Effort Budget の表は closeout の時点の値（上限 11、消費 11）に直した。
