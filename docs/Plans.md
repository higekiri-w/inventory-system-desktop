# Plans.md

導入の前の関門・ブロッカー・製品の決まっていない判断と参照だけを置く（D-145 (12)、TD-487）。完了は merged の PR と git log、lane の現在地は helper の status、session の進捗と次の見通しは handoff、長い候補一覧は [Backlog](backlog.md)。それまでの完了の記録は [archive](archive/harness-context/2026-09-14-Plans.md) と [完了履歴](archive/harness-context/2026-09-30-Plans-completed.md) に残る。正本 repository は `kosei-w90607/inventory-system-desktop`（D-077）。

## ブロッカー

次 lane を止める製品側のblockerはない。単位の拡張の店回答は 2026-09-15 に揃い（POS 数量 1 = 1 m、小数 1 桁で打てる。[聞き取り記録](evidence/hearing-2026-09-14-stock-units.sanitized.md)）、設計（PR #154、2026-10-11、D-113）と単位の runtime（PR #158、2026-10-15）は merge 済み。原価の lane（C）は PR #184（2026-10-21）で merge 済み。残る Z004 と EJ の数量（P）は backlog の「単位の拡張の残りの申し送り」の項目、棚卸しと Z004 の確定は D-118 の (b) の b1（PR #188、2026-10-24）で新しい経路として再開し、残りは b2（backlog の「棚卸しの数の置き場と在庫数の求め直しの runtime」）。

## 導入の前の関門

店の PC に入れて本物のデータが溜まり始めた後では直しにくいものを、初回の導入の前に決め切る（owner の決定、repo 外の回答台帳 TD-345）。導入の時期は関門が揃い次第で、日付を決めず、導入は関門が全部済んだ後の月の初めにする（owner の決定 2026-10-27、repo 外の回答台帳 TD-464）。日報の紙の切替えの日は導入の日と別で、[56](function-design/56-ui-daily-sales.md) UI-09a-D19 が決める。各項目の中身は [backlog](backlog.md) の名前を挙げた項目に置く。

- 関門（confirmed。番号は他の文書から引くので固定し、済んだ関門は外す。済んだことは git log と PR が持つ。D-145 (12)）:
  - 関門 2: D-118 の (b)（D-070 で v1.0 の条件）。今年の年末の棚卸しは在庫 Excel を埋めれば終わる（TD-347）ので、年末に間に合わせる必要は無い。backlog の「棚卸しの数の置き場と在庫数の求め直しの runtime」。→ b1・b2 の 2 つの lane に分けた。b1 は [PR #188](https://github.com/higekiri-w/inventory-system-desktop/pull/188) で 2026-10-24 に main へ入った（[packet](archive/plans/2026-10-22-stocktake-count-runtime-b1.md)）。→ 済み: b2 は [PR #193](https://github.com/higekiri-w/inventory-system-desktop/pull/193) で 2026-10-27 に main へ入った（[packet](plans/2026-10-23-stocktake-count-runtime-b2.md)）。
  - 関門 3: backup の PC の外の控えと復元の予行（[PR #155](https://github.com/higekiri-w/inventory-system-desktop/pull/155)、D-114）。他の機能の前提ではないので、導入の直前（店でアプリに本物のデータが溜まり始める前）まででよい。外の控えの USB は用意済み（TD-346）。USB の控えは任意（USB が無くてもアプリは動き、繋ぐことを必須にしない。TD-381）で、PC の外の控えの runtime は優先を下げた（TD-401）。PC の外の控えが間に合わないときに導入を待つかは未回答。backlog の「backup に、PC の外のコピーと復元の実証が運用として設計されていない」。→ 設計は済み: PR #155（2026-10-23、D-114、[packet](archive/plans/2026-10-07-backup-offsite.md)）。残りは PC の外の控えの runtime（runtime の L3 で USB の用意と写しを確かめる。backlog の「PC の外の控えの runtime」。2 つの lane に分け、(a) を起票した: [packet](plans/2026-10-23-backup-offsite-runtime-a.md)）と復元の予行演習（[71 §71.13](function-design/71-mnt-backup.md)）と、TD-345 で足した 3 点のうち D-114 の設計が扱っていない PC の中の保持日数と PC の日付での掃除（71 MNT-01-D3。同じ backlog の項目の「導入の前の関門」の段）。
  - 関門 4: 店の PC での MSI の予行演習 M9（[手順書](msi-distribution-runbook.md#確かめる項目) の M9。初回の導入の前）。
  - 関門 6: 棚卸しの runtime b1（PR #188）の L3 で owner が挙げた画面の 2 点（owner の決定 2026-10-24、repo 外の回答台帳 TD-433。小さな別 lane か画面の要素の見直しの lane〈PR #202〉で直す）: 商品フォームの在庫連動の拒否の Alert が画面に無い語「在庫連動」で欄（「POS販売で在庫を減らす」）を指すこと、棚卸しの一覧の「最終カウント」が保存の時刻を出し「前の日の分を入れる」で入れた日付が一覧に出ないこと。backlog の「棚卸しの runtime b1 の L3 の所感（画面の語と最終カウントの日付）」。→ 済み: [PR #210](https://github.com/higekiri-w/inventory-system-desktop/pull/210) で 2026-10-28 に main へ入った。
  - 関門 8: 商品 CSV のプレビューが、`商品コード` が数字 13 桁なのに `JANコード` が空の行と、指数の形（`E+`）に崩れた値の行と、`JANコード` が 8 桁・13 桁の数字でないかチェックディジットが合わない行を名指しで止め、直し方を出す（owner の決定 2026-10-24、repo 外の回答台帳 TD-421・TD-426。出所は PR #196〈初期投入の手順〉の起票の調べと Plan Review）。今はどれも error も warning も無く入り、1 つ目と 3 つ目はレジの売上に引き当たらず、2 つ目は誤ったコードのまま変えられない。backlog の「商品 CSV のプレビューで黙って入る形の行を止める」。→ 済み: [PR #209](https://github.com/higekiri-w/inventory-system-desktop/pull/209) で 2026-10-28 に main へ入った。
  - 関門 9: 在庫の振替（詰め替え・切り分け）を導入の前に入れる（owner の決定 2026-10-27、repo 外の回答台帳 TD-463。種類を足すには `inventory_movements` の作り直しが要り、導入の前なら開発・試験 DB の作り直しで済む）。backlog の「在庫の振替（詰め替え・切り分け）」。
  - 関門 10: 返品・交換の `register_processed` の既定（[63](function-design/63-ui-return-exchange.md) UI-03-D8）（owner の決定 2026-11-02、repo 外の回答台帳 TD-570 で candidate から足した）。差額だけの戻しの交換と部門キーで売った商品の返品は既定のまま保存すると在庫が黙って動かない。b1 の起票で b1 の外の小さな lane と決めた（TD-377〜TD-379 の (iii)）。backlog の「返品・交換の `register_processed` の既定と差額だけの戻しの交換」。
  - 関門 11: 取込み済みを `XZ_BKUP` へ移す (b) の lane（D-111 (1)(b)）（owner の決定 2026-11-02、repo 外の回答台帳 TD-570 で candidate から足した）。owner は v1.0 の前と答えていた（TD-296 の 11a）。backlog の「取込み済みの Z・EJ を `XZ_BKUP` へ移す操作（(b) の lane）」。
  - 関門 12: 日報・月次の画面の導入前受入（[56](function-design/56-ui-daily-sales.md) UI-09a-D13・[57](function-design/57-ui-monthly-sales.md) UI-09b-D9）（owner の決定 2026-11-02、repo 外の回答台帳 TD-570 で candidate から足した）。手順は [受入の手順書](daily-monthly-acceptance-runbook.md)（[packet](plans/2026-11-02-daily-monthly-acceptance.md)、owner の答え TD-578）。場は開発用の Windows で、店へ入れる版の MSI を開発の data folder を退避した後の空の DB に入れ、店の SD の写しを読んで綴りの紙と並べる（店の PC では行わない。O1）。初期投入の流れ 3 の前に終え、全部が合ったときだけ流れ 3 へ進む。前提は 2 つの R3 の lane（O3）: A（日報の取込みの完了の画面を離れた後に取り消す入口。backlog の「日報取込み標準手順の残設計」）と B（月次の画面に月の中の取り込んだ日を出す。backlog の「月次の画面に月の中の取り込んだ日を出す」）。両方が merge されるまで全体の判定は未了。下の precondition-dependent の go-live の検証の流れ（受入台本の第 2 版）には束ねない（O6）。backlog の「日報画面の Excel 印刷・バインダー代替受入判定」。
- 関門（candidate、owner が選んで足す）:
  - 初期投入の手順（backlog の「初期投入の手順」）: 関門（confirmed）の 8 と管理者の手順書 [initial-load-runbook.md](initial-load-runbook.md) が [PR #196](https://github.com/higekiri-w/inventory-system-desktop/pull/196) で 2026-10-27 に入った（[packet](archive/plans/2026-10-23-initial-load-procedure.md)）。残りは初回の導入の版の後の予行演習と店の PC での投入（手順書の流れ 2・流れ 3）で、CSV 作りと照合に Excel のある Windows と店の値札が要る（手順書の流れ 1、流れ 2 の手順 6。TD-437）。
  - デザインの刷新。owner は v1.0 の前に入れると決めた（TD-296）が、関門を決めた TD-345 より前の答えで、関門に足すかは owner が選ぶ。設計の lane は機能の関門が済んでから始める（owner の決定 2026-11-02、TD-569）。backlog の「デザインの刷新（v1.0 の前）」。
  - PLU をレジへ書く時期（PLU を書く前の導入では Z004 の自動在庫連動はほとんど動かない）は、owner が次の訪店の C3（店の PC の CV17 の画面で部門リンクの値を確かめる、TD-459）の後に A / B / C から選び、B・C は D-070 の「初日から」を改める新しい D になる（TD-462）。backlog の「PLU をレジへ書く時期と D-070 の自動在庫連動」。
- 関門（precondition-dependent）:
  - SD 直読み（PR #172）を初回の版に入れるのは、導入の初日からアプリで日報を取り込む場合（#172 より前の経路の取込みは精算回数が NULL のまま二重取込みの拒否の外に残る）。backlog の「SD 直読みの runtime」。→ 2026-10-31 に [PR #172](https://github.com/higekiri-w/inventory-system-desktop/pull/172) で main へ入り、初回の版に入る。
  - 共有 JAN の色違いの群と JAN の無い品の v1.0 の扱い。今は店内コードができるまで在庫連動の外（ADR SPEC-STK-CNT-D4、TD-296）。レジとリーダーの実機の確かめ（C1、次の訪店、TD-454）の後に owner が決める（TD-461）。backlog の「JAN の無い商品の店内コードと色ごとのバーコードの表」。
  - Z004 の経路の受入台本の第 2 版と go-live の検証の流れ（D-070 の配布判定の入口条件。[受入台本](ACCEPTANCE_SCENARIO.md) の第 1 版は Z004 の経路を対象外とする）。台本の形は b2 と、上の candidate の PLU をレジへ書く時期で決まる。backlog の「go-live の検証の流れ（受入台本の第 2 版）」。
- 導入の後でよいと確かめたもの: ECR+ のサービス終了（2028 年 12 月末予定）、二重起動・途中の電源断（対策済み）、v8 より後の migration を足す lane の義務（初回の版の後から当たる）。

## 製品の未決判断

L8-2/L8-5は旧⑩laneからの記録・実機観測の申し送りで、判断待ちではない。いずれもハーネス整備を止めるgateではなく、採用や新規backlog起票を行わずに元の状態を保持する。

- L8-2（badge 無色、⑦ 待ち）・L8-5（記録日時 font 差。[backlog](backlog.md) の L8-5 の項目）は対象外（参照のみ）
- STK-1 / STK-2の直し方は、時点証拠 ADR（PR #85、D-116）を置き換えた[棚卸しの数の置き場と在庫の求め直しの ADR](adr/2026-10-16-stocktake-count-placement.md)（PR #167、2026-10-17 merge、D-118）が正本。記録から在庫を求め直すので、確定が数えた後の入出庫を打ち消す・後着の Z004 が二重に減らす形が構造上起きない。runtime の (a)（在庫数を記録から求め直して書く形と計数の置き場の migration）は PR #176（2026-10-19）で入り、(b) の b1（PR #188、2026-10-24）で停止 ADR の入口を外し、時点証拠の code を撤去し、STK-1・STK-2 の診断を新しい経路で通る回帰 test にした。残る b2 は [backlog](backlog.md) の「棚卸しの数の置き場と在庫数の求め直しの runtime」が扱う。
- SD 直読み（D-111）の未決 A（読んだ原本の写しの保持期間と削除）は消さないと決めた（owner の決定、repo 外の回答台帳 TD-296。D-111 の未決 A の答えの行）。B（写しを backup に含めるか・PC の外へ出すか）は含めないと決めた（D-114、repo 外の回答台帳 TD-197。D-111 の `Superseded in part by: D-114`）。どちらも runtime の既定（消さない・backup に入れない）のままで、後続の「SD 直読みの runtime」の lane を止めない。

元の文脈は [移送前のPlans](archive/harness-context/2026-09-14-Plans.md)。関連する製品作業でownerの判断を得る。

## 参照

- [安定した製品前提](project-memory.md)、[引き継ぎ案内](PROJECT_HANDOFF.md)
- [未了項目・保留・受容済みリスク](backlog.md)
- [移送前のdashboard全文](archive/harness-context/2026-09-14-Plans.md)
