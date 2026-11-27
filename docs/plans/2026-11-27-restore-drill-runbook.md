# Plan Packet: 復元の通しの演習と PC の引越しの手順を手順書に入れる（導入の前の関門 3 の (4)）

起票: 2026-11-27。導入の前の関門 3 の残りの (4)（[Plans の導入の前の関門](../../Plans.md#導入の前の関門)、`docs/backlog.md` の「backup に、PC の外のコピーと復元の実証が運用として設計されていない」の「導入の前の関門」の段の (4)。owner の決定 TD-675・TD-842、まとめは D-162）の起票。docs だけの lane で、製品の runtime（`src/**`・`src-tauri/**`）は変えない。手順書の本文は、下の「owner・店に決めてもらう事項」の答えと計画の固定の後に書く（本 packet の commit には手順書の本文を入れない）。

## Workflow State

- Risk: R4
- Plan Commit: pending
- Amendments: none

`Plan Commit`・`Amendments` は `scripts/plan-approve.sh` が書く（手で写さない）。

## Risk

Risk: R4

Reason:
手順書が、店の本物の商品のデータ（初期投入の予行の CSV を取り込んだ data folder とその backup・PC の外の控え）の復元（Risk Tiers の R4 の「backup の復元」）と、PC の入れ替えでの data folder の移し替えと古い PC の片付け（同じく「破壊的なデータの lifecycle」）を管理者にさせる。手順の誤りは、店の記録を戻せない形で失うか、開発用の Windows と媒体に店のデータを残す。先例: 初期投入の手順書の lane（`docs/archive/plans/2026-10-23-initial-load-procedure.md`、R3 で起票し round 1 で R4 へ上げた）と MSI の手順書の lane（`docs/archive/plans/2026-10-17-msi-distribution-docs.md`、R2 で起票し R3 へ上げた）は docs だけで R3 以上になった。

## Goal

Goal Invariant:

### 最小完了条件

- 管理者が手順書だけを見て、初期投入の予行（[initial-load-runbook.md](../initial-load-runbook.md) の流れ 2）の中で 71 §71.13 の通しの演習を流し、合格（戻した後の画面の商品の数が、戻す前に控えた数と合う）か不合格かを決め、退避した data folder を戻して流れ 2 の残り（手順 7・8）を続けられる。
- 71 §71.13 の「通しの演習」の行の「関門 3 の残りの (4) の手順書が決める。それまでこの行だけでは流さない」が、手順書の該当の手順への link に置き換わる。
- 管理者が手順書だけを見て、予定した PC の入れ替え（古い PC が動く）と、PC を失ったとき（71 §71.13 の本番の復元）のそれぞれで、新しい PC へ入れる MSI の版と置き場、DB の外の物の扱い（写すか捨てるか）、古い PC の片付け、記録の置き場を決められる。
- 手順書の画面の名前・ボタンの文字が、書いた時点の `src/**` の `*.tsx`・`*.ts` と一致する（`rg` で照らした所を packet か PR に併記する）。

### 失敗定義

- 演習のために、店の PC で復元させる（71 §71.13 の「店の PC で通しの演習をしない」に反する）。
- 開発用の Windows の data folder（`%APPDATA%\com.inventory-system.desktop`）を、退避せずに演習の空の起動で上書き・混在させる。または退避した folder を戻す前に演習の folder を消す。
- 演習の空の起動の前に、ショートカットの作業フォルダーの `inventory.db` を確かめさせない（作業フォルダーの `inventory.db` を取り込む、22 §12。msi-distribution-runbook.md の「予行演習の準備と片付け」の 2 つ目の段）。
- 流れ 2 の「手順 3〜7 は同じ日に続けて行う」を崩す（取り込む前の backup が日をまたいで保持日数で消え、手順 7 で戻せない）。
- 合格を、ボタンが押せない・文が出ないことから言い切らせる（TD-839・TD-837）。または合格の基準を「戻す前に控えた数と合う」以外の、画面で見えない条件にする。
- 引越しの手順で、DB の外の物（下の「DB の外の物」の表）を黙って失わせる。または古い PC の片付けを、新しい PC で照合が合う前にさせる。
- 新しい PC に、戻す控えより古い版（扱える DB の版が控えの DB の版より小さい）の MSI を入れさせる（「この控えは、より新しい版のアプリで作られています。…」で止まる。MNT-03-D11）。
- 手順書に、owner の手元の clone を操作する手順（`git restore .`・`checkout-index -f`・`reset --hard`）を入れる。
- 実店舗の実値（商品の数・日付・媒体の札の割当て）を repo に書く。

### 非目的

- アプリの機能の追加（保存先を既定へ戻す手段など。下の「足りないアプリの機能」へ出す）。
- 関門 3 の (3)（更新・migration の前の保全。別の起票 1158）と関門 12 の受入。
- 71 §71.13 の短い確かめ・本番の復元の手順の中身の変更（本番の復元の行は引越しの手順書から引く）。
- 店の PC での演習。自動更新（updater）。

Priority: `Goal Invariant > Acceptance Criteria > supporting evidence`。AC や証跡作業が Goal Invariant を前進させない場合は、Goal を置き換えず簡略化・defer・削除する。

## 現状の調べ（2026-11-27、clone の HEAD `11b1d793`）

- 71 §71.13 の通しの演習の行は、流す場所・媒体の用意・退避した folder を戻して続けることを本手順書へ預け、「それまでこの行だけでは流さない」とする。本番の復元の行は、新しい PC での復元・「今すぐバックアップを作成」・保存先の選び直し・媒体の登録し直しまでを持つ。予定した PC の入れ替え・DB の外の物・古い PC の片付け・演習の記録の置き場は、どの文書にも無い。
- initial-load-runbook.md の流れ 2 は、手順 3（取り込む前の手動バックアップ）→ 4・5（取込み）→ 6（照合）→ 7（取り込む前の backup へ復元して 0 件）→ 8（片付け）。手順 3〜7 は同じ日。手順 1 と 8 は msi-distribution-runbook.md の「予行演習の準備と片付け」（`%APPDATA%` と `%LOCALAPPDATA%` の `com.inventory-system.desktop` を退避・保全・戻す）を引く。
- 画面と文字（`rg` で照らした所）: 画面の名は「バックアップ・復元」（`src/config/navigation.ts` の label、`BackupRestorePage.tsx` の title）。ボタンは「今すぐバックアップを作成」「保存先を選ぶ」（`BackupRestorePage.tsx`）、「この USB メモリを控えの保存先にする」「控えを選んで確かめる」（`OffsiteBackupPanel.tsx`）。確かめの結果の文は「この控えは戻せます」（`BackupRestorePage.tsx`）。媒体の card の文は「差してある控え: 控え N」（`offsite-status-text.ts`）。保存先が未設定のときの文は「保存先が未設定のためアプリ既定フォルダに保存されます」（`BackupRestorePage.tsx`）。商品の数は「商品検索・一覧」の「全 N 件のうち…」（`src/components/patterns/Pagination.tsx`。一覧の既定の絞り込みは廃番以外、`ProductListPage.tsx` の `discontinued === "active"`）。新しすぎる控えの文は「この控えは、より新しい版のアプリで作られています。…」（`BackupRestorePage.tsx`）。
- 演習の空の起動は媒体に書かない: アプリは目印の `medium_id` が PC 側の状態（`offsite-backup.json`）の `media` にある媒体だけに写す（71 §71.11.3）。退避の後の空の folder には状態の file が無いので、差した媒体へ空の DB の控えは写らない。
- 媒体の登録し直しは目印を引き継ぐ（71 §71.11.2 手順 4）。新しい PC で `offsite-backup.json` を写さずに登録し直す道は、71 §71.13 の本番の復元が既に持つ。
- アンインストールは data folder と WebView の保存の folder を残す（msi-distribution-runbook.md の M7）。古い PC の片付けでアンインストールだけでは店のデータが消えない。
- 保存先の既定は `{app_data_dir}\backups`（`src-tauri/src/mnt/backup.rs` の `resolve_backup_dir`。`backup_path` が空のとき）。既定へ戻すボタンは無い（backlog の「PC の外の控えの runtime」の (a) の (4)）。今できるのは「保存先を選ぶ」で新しい PC の中の folder（既定と同じ folder を含む）を選ぶこと（71 §71.13 の本番の復元の行と同じ）。
- 並走の PR #307（PLU の runtime、未 merge）は 71 §71.7 に MNT-01-D13 を足す: 復元した DB から PLU の最終照合日時を消し、次に SD を読んで照合するまで PLU の書出しが止まる。本 lane は §71.7 を変えないが、#307 が先に merge すれば、引越しと演習の手順書に「復元の後は SD を読むまで PLU書出しが止まる」を書く（後なら #307 の側の同期で足す）。

## DB の外の物（引越し・本番の復元が扱わない物）

| 物 | 場所 | 何か | 本番の復元（PC を失った）で | 予定した入れ替え（古い PC が動く）で |
|---|---|---|---|---|
| レシートの写真 | `images/receipts/`（`src-tauri/src/io/image_manager.rs`） | 返品・交換の記録の添付。DB は相対 path で指す | 失う（owner の決定 TD-690。EJ の控え画面が正の道） | O4-a の答え |
| SD の原本の写し | `pos-sources/`（`src-tauri/src/io/pos_source_copy.rs`） | SD から読んだ原本の bytes | 失う（D-114 の owner の決定 2、TD-197。取り込んだ内容は DB にある） | O4-b の答え |
| PC の外の控えの状態 | `offsite-backup.json`（と `offsite-backup.json.stopped-*`） | 用意した媒体の一覧と最後に写せた日時 | 無い。媒体を登録し直す（71 §71.13 の本番の復元） | O4-c の答え |
| 診断ログ | `logs/`（`src-tauri/src/lib.rs` の `log_dir`） | 起動・失敗の診断 | 失う | O4-d の答え |
| WebView の localStorage | `%LOCALAPPDATA%\com.inventory-system.desktop` | 表示サイズ（`inventory.displayScale.v1`）、日報の前回の folder（`inventory:daily-report-import:last-dir:v1`）、PLU の保存済み未確認（`inventory:plu-export:pending:v1`。#307 で変わりうる） | 失う | O4-e の答え |
| 保存先の設定 | DB の `backup_path` | 戻した DB が古い PC の path を持ちうる | 「保存先を選ぶ」で選び直す（71 §71.13）。既定へ戻すボタンは無い | 同じ |
| MSI | 配布の記録（repo の外）と保管の MSI（msi-distribution-runbook.md の「配布の記録」） | 新しい PC へ入れる版 | O5 の答え | O5 の答え |

## Scope

- `docs/initial-load-runbook.md` の流れ 2: 手順 6（照合）と手順 7（取り込む前の backup へ戻す）の間に、通しの演習の手順を足す（手順の番号を振り直すか 6a〜6g とするかは書く時に決め、流れ 3 から流れ 2 の手順を引く所を全部 `rg` で直す）。中身: (1) 媒体の用意（O2 の答え。取り込む前、手順 3 の前に「この USB メモリを控えの保存先にする」で用意するか、手順 6 の後に用意するか）、(2) 手動バックアップ（「今すぐバックアップを作成」）と、媒体の card が「差してある控え: 控え N」と今の日時になるのを待つこと、(3) 「商品検索・一覧」の既定の絞り込みで「全 N 件」を控える（売上のある data なら最後の営業日の売上も。流れ 2 の data は商品だけ）、(4) アプリを全部閉じ、`%APPDATA%` と `%LOCALAPPDATA%` の `com.inventory-system.desktop` を別の名へ退避（msi-distribution-runbook.md の「予行演習の準備と片付け」の書き方に合わせる）、(5) 作業フォルダーの `inventory.db` が無いことを確かめて起動 → 「バックアップ・復元」→「控えを選んで確かめる」→ 媒体の `InventoryBackup` の最新の file → 「この控えは戻せます」と作成日時が (2) の控えと合うことを見る → 2 段の確認で戻す、(6) 戻った後の「全 N 件」が (3) と合えば合格、合わなければ不合格として控え、(7) アプリを閉じ、演習の folder を別の名へ変えて保全し、`com.inventory-system.desktop` の名の folder が無いことを確かめてから (4) で退避した folder を戻し、起動して「全 N 件」が (3) と同じことを見てから手順 7 へ進む。演習が不合格でも (7) まで行い、流れ 3 へ進まない（流れ 2 の「全部が合ったときだけ流れ 3 へ進む」に含める）。「手順 3〜7 は同じ日に続けて行う」に演習を含める。
- `docs/function-design/71-mnt-backup.md` §71.13 の「通しの演習」の行だけ: 「関門 3 の残りの (4) の手順書が決める。それまでこの行だけでは流さない」を、initial-load-runbook.md の該当の手順への link に置き換える（§71.1・§71.7・§71.10 は #307 が触るので触らない。§71.11 は触らない）。
- 引越しの手順書（置き場は O1 の答え。推奨は `docs/msi-distribution-runbook.md` の流れ 5 の後に「流れ 6: PC を入れ替える」を足す）: 予定した入れ替えと PC を失ったときの 2 つの場面。中身: 新しい PC へ入れる MSI の選び方（配布の記録の「扱える DB の版」が戻す控えの DB の版以上。既定は古い PC の今の版と同じ MSI）、msi-distribution-runbook.md の流れ 2 での導入、71 §71.13 の本番の復元の行を引いた復元、上の「DB の外の物」の表の物ごとの扱い（O4 の答え）、新しい PC での照合、古い PC の片付け（O6 の答え。アンインストールでは data folder が残る〈M7〉ことを書く）、記録の置き場（O3 の答え）。
- 演習の記録の置き場（O3 の答え）を、演習の手順と引越しの手順の両方に書く。実値は repo に書かない。
- `docs/backlog.md` の (4) の段: 済んだ範囲（手順書の lane）を書く。関門 3 の行（Plans.md）は (3) が残るので外さない。
- `docs/PROJECT_HANDOFF.md`: 引越しの手順の置き場が新しい file になるときだけ、案内の 1 行。
- D を足すなら D-174（予約）。O1〜O6 の答えが設計の判断（DB の外の物を引越しで写す・捨てる）を含むので、答えのまとめを D-174 に置くかは答えの後に決める。

Non-scope: `src/**`・`src-tauri/**`、`docs/migration-recovery-runbook.md`（1158）、initial-load-runbook.md の流れ 3（#310 が手順 10・11 を触る。流れ 2 の手順の番号を振り直すなら、流れ 3 が流れ 2 を引く所だけを直し、手順 10・11 の本文は触らない）、msi-distribution-runbook.md の流れ 3・流れ 4（1158 が触りうる）。

## owner・店に決めてもらう事項

推奨は confirmed / candidate / precondition-dependent を区別する。答えは repo 外の回答台帳に残し、手順書には答えの ID と要旨を書く。答えが出るまで手順書では「未決（〈項目〉）」とする。

- O1 引越しの手順の置き場: (a) msi-distribution-runbook.md に流れ 6 を足す、(b) 新しい file（例 `docs/pc-move-runbook.md`）、(c) 71 §71.13 の表に段を足す。推奨 (a)（candidate。新しい PC への導入は流れ 2、MSI の選び方は配布の記録で、どちらも同じ手順書にある。(c) は設計正本に管理者の手順を足すことになる）。
- O2 演習の媒体: (a) 演習だけに使う USB メモリを別に用意し、終わったら演習の folder と同じく保全する、(b) 店の控えの USB（用意済みの 2 本の 1 本）を使う、(c) 管理者の手元の USB を使う。推奨 (a)（precondition-dependent: 空きの USB メモリがあること。(b) は店の媒体に予行の data の控えと目印が残り、店の PC で登録したときに媒体の一覧に予行の控えが混ざる。(c) は管理者の媒体に店の商品のデータが残る）。
- O3 演習と引越しの記録の置き場: (a) msi-distribution-runbook.md の配布の記録と同じ repo の外の場所に 1 行、(b) repo 外の回答台帳、(c) PR の comment（実値を書かない形）。推奨 (a)（candidate。数の控えと日時は実値なので repo に書かない。配布の記録は既に管理者の保管場所）。
- O4 DB の外の物を予定した入れ替えで写すか: 項目ごとに「写す / 捨てる」。
  - O4-a `images/receipts/`: (a) data folder の中の同じ path へ写す、(b) 捨てる。推奨 (a)（candidate。DB は相対 path で指すので、同じ path へ写せば添付が開く。PC を失ったときは TD-690 のとおり失う）。
  - O4-b `pos-sources/`: (a) 写す、(b) 捨てる。推奨 precondition-dependent（取り込んだ内容は DB にあり、同じ bytes の原本はレジの SD の `XZ_BKUP` にも残る〈D-114 の決定 2 の理由〉。写しを後で読む機能が無いなら (b)、読む機能を入れる予定があれば (a)）。
  - O4-c `offsite-backup.json`: (a) 写さず、新しい PC で媒体を登録し直す（71 §71.13 の本番の復元と同じ道）、(b) 写す。推奨 (a)（candidate。登録し直しは目印を引き継ぐ〈71 §71.11.2 手順 4〉ので札は変わらず、手順が 1 つで済む）。
  - O4-d `logs/`: (a) 捨てる（古い PC の片付けまで古い PC に残す）、(b) 写す。推奨 (a)（candidate。診断ログは新しい PC で新しく積もる）。
  - O4-e WebView の localStorage: (a) 写さず、入れ替えの前に古い PC で PLU の保存済み未確認を済ませ（PLU書出し の画面に保存済みの確認が残っていないこと）、表示サイズと日報の folder は新しい PC で選び直す、(b) `%LOCALAPPDATA%` の folder を丸ごと写す。推奨 (a)（candidate。WebView の保存の形は WebView2 の内部で、丸ごとの写しが新しい PC で読めるかは未確認〈未実測〉。#307 の後の PLU の画面で「保存済み未確認」の形が変わるかは書く時に確かめる）。
- O5 新しい PC へ入れる MSI の版と置き場: (a) 古い PC に今入っている版（配布の記録の最後の行）と同じ MSI を保管の場所から写す、(b) 最新の版の MSI。推奨 (a)（candidate。(b) は扱える DB の版が控え以上なら戻せるが、復元と版の更新（migration）を同じ時に重ねる。更新は流れ 3・4 で後に分けて行う）。
- O6 古い PC の片付け: (a) 新しい PC で照合が合い、何日か（日数は owner）業務を続けた後に、古い PC の data folder と `%LOCALAPPDATA%` の folder を消す（Windows の「この PC をリセット」の「すべて削除」を含めるかは owner）、(b) 照合が合ったらすぐ消す、(c) 古い PC をそのまま保管する。推奨 (a)（candidate。新しい PC で不具合が出たときに古い PC から引き直せる間を残す。消す日まで古い PC を店の外へ出さない）。
- O7 演習の日: (a) 初期投入の予行（流れ 2）の日に組み込む、(b) 流れ 2 を別の日に手順 1 からもう一度流す。推奨 (a)（candidate。流れ 2 の手順 3〜7 の同じ日に収まり、予行の data を作り直さない。関門 12 の受入とは別の日でよい、TD-842）。日付そのものは owner。

店への問いの案（Coordinator が台帳を引いてから、無いものだけを送る）:

- 空いている USB メモリ（演習だけに使う 1 本）がありますか（O2）。
- 返品・交換でレシートの写真を撮って付けていますか。付けているなら、PC を入れ替えるときに写真も新しい PC へ移したいですか（O4-a）。
- PC を入れ替える予定（買い替えの時期）はありますか（O6 の日数の目安）。

## 足りないアプリの機能

- 保存先を既定（`{app_data_dir}\backups`）へ戻す手段が画面に無い（backlog の「PC の外の控えの runtime」の (a) の (4)）。手順書は「保存先を選ぶ」で新しい PC の中の folder を選ぶことだけを書く（71 §71.13 と同じ）。
- 予定した入れ替えで DB の外の物（`images/receipts/` など）を写す機能が無い。O4 で「写す」を選んだ物は管理者が folder を写す手順になる（店の人の操作にしない）。写す機能を入れるかは backlog へ出す（O4 の答えの後）。
- 演習の合格を数で見る所が「全 N 件」の表示だけで、DB 全体の照合（記録の件数・最後の記録の日時）を画面で並べる所は無い。合格の基準は 71 §71.13 の「商品の数（売上のある data なら最後の営業日の売上も）」のままにする。

## Acceptance Criteria

- AC1: `bash scripts/doc-consistency-check.sh` と `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-11-27-restore-drill-runbook.md` が ERROR なしで終わる（WARN の行数を PR に書く）。
- AC2: 変えた `*.md` に `npx prettier --check` が exit 0。`bash scripts/check-workflow-git.sh` が exit 0。
- AC3: 71 §71.13 の通しの演習の行に「それまでこの行だけでは流さない」が無く、initial-load-runbook.md の演習の手順への link がある（`rg -n "それまでこの行だけでは流さない" docs/` が 0 件）。
- AC4: 手順書の画面の名・ボタンの文字・文が、書いた時点の `src/**` に `rg -F` で 1 件以上あり、その command と一致した path を PR に併記する（上の「現状の調べ」の文字を含む）。
- AC5: 演習の手順が、退避の前に数を控える・作業フォルダーの `inventory.db` を確かめる・退避した folder を戻す前に `com.inventory-system.desktop` の名の folder が無いことを確かめる・戻した後に数を見直す、の 4 点を持ち、流れ 2 の「同じ日」に演習を含める。
- AC6: 引越しの手順が、「DB の外の物」の表の 7 行を全部、場面（PC を失った・予定した入れ替え）ごとに扱いを書き、未決の所を「未決（O*n*）」と書く。手順書に `git restore`・`checkout-index`・`reset --hard` が無い（`rg`）。
- AC7: backlog の (4) の段に本 lane の済んだ範囲があり、Plans.md の関門 3 の行が残る。
- AC8: 手順書と packet に実店舗の実値（商品の数・日付・媒体の札の割当て）が無い（`git diff origin/main...HEAD` を目で確かめ、PR に書く）。

## Design Readiness

- 引用する設計正本（節まで）: 71 §71.1（backup は DB だけ）・§71.11.2 手順 4（目印の引継ぎ）・§71.11.3（用意済みの媒体だけに写す）・§71.13（通しの演習・本番の復元）、22 §12（作業フォルダーの `inventory.db` の取込み）・§3.2 MNT-03-D11（新しすぎる控えの拒否）、68 UI-11b-D15（確かめの間の 60 秒の確認の停止）、D-114（決定 2）・D-162・TD-675・TD-690・TD-842、msi-distribution-runbook.md の「予行演習の準備と片付け」・配布の記録・M7、initial-load-runbook.md の流れ 2。
- 必要な設計成果物: 振舞いの契約は existing sufficient（アプリを変えない）。手順書は updated in this PR（計画の固定の後の実装の commit）。71 §71.13 の 1 行は updated in this PR。
- plan にしかない durable な判断の昇格先: O1〜O7 の答えのうち、DB の外の物の扱い（O4）と古い PC の片付け（O6）は手順書と、要れば D-174。
- 前提・制約と延期した design gap: 保存先を既定へ戻す手段（backlog の (a) の (4)）、DB の外の物を写す機能、WebView の保存を丸ごと写せるか（未実測）。#307 の MNT-01-D13 の merge の順。
- 絶対保証の例外と抜け道: 演習の空の起動は媒体に書かない（71 §71.11.3。状態の file が無ければ用意済みの媒体が無い）。例外は、退避の漏れで状態の file が残ったとき（AC5 の退避の確かめで防ぐ）。
- 判定: not ready（O1・O2・O4・O6 の答えが手順書の形を決める。答えの後に ready にして計画を固定する）。

## Contract Ledger

| 契約 ID | 設計正本の節 | 実装（Scope） | 自動 test | L3・非対象 |
|---|---|---|---|---|
| 71 §71.13 通しの演習 | 71 §71.13 の表の 2 行目 | initial-load-runbook.md の流れ 2 の演習の手順、71 の行の link | なし（docs） | 演習そのものは owner が開発用の Windows で流す（本 lane の merge の条件にしない） |
| 71 §71.13 本番の復元 | 71 §71.13 の表の 3 行目 | 引越しの手順が引く（中身は変えない） | なし | 非対象（変えない） |
| 71 §71.13 店の PC で演習しない | 71 §71.13 の箇条 1 | 演習の手順の前提 | なし | 非対象 |
| backup は DB だけ | 71 §71.1、D-114 決定 2、D-162 (2) | 「DB の外の物」の表 | なし | 非対象 |
| 用意済みの媒体だけに写す | 71 §71.11.3 | 演習の空の起動の安全の根拠 | 既存の offsite の test（変えない） | 非対象 |
| 目印の引継ぎ | 71 §71.11.2 手順 4 | O4-c の推奨の根拠 | 既存（変えない） | 非対象 |
| 作業フォルダーの DB の取込み | 22 §12、msi-distribution-runbook.md の準備の 2 つ目の段 | 演習の空の起動の前の確かめ | なし | 非対象 |
| 新しすぎる控えの拒否 | 22 §3.2 MNT-03-D11 | O5 の MSI の選び方 | 既存（変えない） | 非対象 |
| 確かめの間の確認の停止 | 68 UI-11b-D15 | 演習の (5) の確かめの後に詳細を閉じる | 既存（変えない） | 非対象 |
| 流れ 2 の同じ日 | initial-load-runbook.md の流れ 2 の前文 | 演習を同じ日に含める | なし | 非対象 |
| 予行演習の準備と片付け | msi-distribution-runbook.md の同名の段 | 演習の退避と戻し | なし | 非対象 |
| アンインストールで残る | msi-distribution-runbook.md の M7 | 古い PC の片付け | なし | 非対象 |
| PLU の照合日時を消す | 71 §71.7 MNT-01-D13（#307、未 merge） | merge の順で演習・引越しの手順に 1 文 | #307 の test | 非対象 |
| D-162・TD-675・TD-842 | decision-log D-162 | 本 lane の出所 | なし | 非対象 |

## Test Plan

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| docs の整合 | 壊れた link・節の参照 | CLI | `bash scripts/doc-consistency-check.sh` | 手順書・71・backlog の link が切れる |
| packet の形 | 新形式の節の欠け | CLI | `bash scripts/doc-consistency-check.sh --target plan docs/plans/2026-11-27-restore-drill-runbook.md` | Workflow State・必須の節・R4 の Recovery / Data Safety が欠ける |
| 書式 | prettier の差 | CLI | `npx prettier --check <変えた md>` | 表・箇条の書式が崩れる |
| 画面の文字 | 手順書の文字が画面に無い | CLI（手で流す） | AC4 の `rg -F` | ボタンの名を推測で書く・画面の文字が変わった |
| 預けの行の置換 | 71 の行が手順書を指さない | CLI | AC3 の `rg` | 「それまでこの行だけでは流さない」が残る |
| 押しつけない | owner の clone の操作の混入 | CLI | AC6 の `rg` | 手順書に `git restore` 等が入る |
| 演習の安全 | 退避の漏れ・作業フォルダーの DB | review | 計画の review と Final Review の固定の問い | AC5 の 4 点のどれかが欠ける |

## Recovery

- 本 lane は docs だけで、merge を revert すれば手順書と 71 の 1 行が元に戻る。DB・migration・状態の file は変えない。
- 演習の失敗（不合格・途中で止まった）: 演習の folder は別の名へ変えて保全し、退避した folder を戻す（Scope の (7)）。退避した folder は演習の間に触らないので、戻せば演習の前の状態に戻る。戻した後に「全 N 件」が (3) と違えば、流れ 2 の手順 7 へ進まずに止め、owner に見せる。
- 引越しの失敗（新しい PC で照合が合わない）: 古い PC を片付けていない間（O6）は、古い PC で業務を続けられる。新しい PC の data folder は消さずに別の名へ変えて保全する。PC を失ったときは、媒体の別の世代・もう 1 本の控え（型 C）から 71 §71.13 の本番の復元をやり直す。
- 失う範囲: 予定した入れ替えで「捨てる」を選んだ DB の外の物は、古い PC を片付けた後は戻らない。

## Data Safety

- 演習の data は店の本物の商品のデータ（流れ 1 の CSV）で、開発用の Windows の data folder・演習の folder・演習の媒体（O2）に残る。消す時期は owner（71 §71.13 の箇条 2、initial-load-runbook.md の流れ 2 の手順 8）。repo の checkout（ignored の `.local/` を含む）・cloud の同期の folder に置かせない。
- 店の PC では演習しない。復元は開発用の Windows の演習の folder の中だけ。
- 引越しで写す物は data folder の中の名指した folder だけで、写す先は新しい PC の data folder。古い PC の data folder は O6 の日まで消さない。
- commit しないもの: 実の DB・backup・媒体の file・`offsite-backup.json`・診断ログ・CSV・演習と引越しの記録の実値（商品の数・日付・札の割当て）。owner の答えは repo 外の回答台帳に残し、repo には ID と要旨だけを書く。
