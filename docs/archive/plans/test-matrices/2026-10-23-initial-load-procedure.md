# Test Design Matrix: 店の PC で本物のデータを入れ始める初期投入の手順を決める

本 lane は docs だけで、自動 test を足さない（code を変えない）。下の行は、手順書 `docs/initial-load-runbook.md` の分岐ごとの期待と、その期待を壊す誤った読み（mutation）を並べたもの。検査の手段は 3 つ: packet の AC の `rg`（機械で数える）、Plan Review・Final Review の読み取り（引く正本の節と code と突き合わせる）、手順書の予行演習（実機。初回の導入の版の後）。手順が引く code の振舞いには既存の自動 test があり、`rg` で実在を確かめた test 名を Evidence に書く。先例は `docs/archive/plans/test-matrices/2026-10-17-msi-distribution-docs.md`（docs だけの手順書の lane の Matrix）。

## Risk

Risk: R4

## Contracts Under Test

- C1 取り込む前の backup を作り、保存先の外の 2 か所へ写してから取り込む。取り込む前の backup から照合までを同じ日に行い（予行演習の流れ 2 も同じ）、照合が全部合ったときだけ業務の入力を始め、合わないかその日に終えられなければ入力を始めずに取り込む前の backup をアプリの復元で戻す。業務の入力を始めた後は戻さず、DB の作り直しで直さない（68 §68.4、71 §71.5、D-123 (5)）
- C2 13 桁の JAN の行は `商品コード` と `JANコード` の両方に JAN を入れる。`商品コード` が 13 桁の数字で `JANコード` が空の行と、`JANコード` が 8 桁・13 桁の数字でないかチェックディジットが合わない行は、初回の導入の版のプレビューが名指しで止め（J5 = b、TD-421・TD-426、Plans の導入の前の関門の 8）、数字だけの `商品コード` が `JANコード` と同じでない行と、`JANコード` が 13 桁で `00000` から始まる行（8 桁の JAN に 13 桁の形式を当てた値。チェックディジットが合う）が 0 であることは表の全部の行で数えて確かめる（今の取込みは `JANコード` を補わず、形もチェックディジットも検査せず warning も出さず、Z004 は `jan_code` で引き当てる。`commit_import`、30 BIZ-01-D1、32 §15）
- C3 プレビューのエラー行と重複行が 0 のときだけ取り込む（確定はエラー行を除いて正常な行だけを入れる。30 §4.9、UI-01c-D8）
- C4 行の種類で商品コードを決める: 種類は上から順に当てはめて 1 行を 1 つの種類にする。部門 11・12 の行は入れない（master-tables の departments）、店が JAN を使っていない行（布の管理 5 パターンのパッチワーク）は JAN で分ける種類より先に独自コードにし、JAN を空にせず保持して `POS在庫連動`・`PLU対象` を `0` にする（D-028、master-tables の `plu_target`、30 の共有 JAN の不変条件）、共有 JAN の群（桁を問わない）は全部の行に独自コードと群の JAN と `POS在庫連動` = `0`（TD-352、ADR SPEC-STK-CNT-D4）、他の行と共有しない 8 桁の JAN の行と、コードの無い行（布の管理 5 パターンの反物・カット生地）は独自コード（J4・J2）、長く動いていないコードの無い品は入れない（D-076）、どれにも当たらない行と接頭辞の無い部門の独自コードの行は 1 行でもあれば手順を止める（外して進まない）。部門の初期データはリリースの前に直す（リリースの後は値を移す migration が要る。D-123 (4)・(5)）
- C5 商品コードと単位の種類はアプリの中で直せない（SP-102-04、30 BIZ-01-D8）。入れた後に見つかったら止めて owner に諮り、予行演習で見つけて入れる前に直す
- C6 商品 CSV は CSV UTF-8（BOM 付きの UTF-8。CP932 の `?` への置換を避ける）で、1 行目が列名で同じ名の列が無く、5 つの数字の列（`商品コード`・`JANコード`・`売価`・`原価`・`初期在庫`）は全部の桁の文字列の値にして、Excel の数値の表示の形に頼らず書く（`商品コード`・`JANコード` の数字は 13 桁なら `"0000000000000"`、8 桁なら `"00000000"`、`売価` と個数の `初期在庫` は `"0"`、`原価` と `m` の `初期在庫` は `"0.00"` の `TEXT` で置き換え、空のセルは空のまま残す）。保存した file は、先頭の 3 byte が UTF-8 の BOM（`EF BB BF`）でなければ行を読まずに保存し直しを名指して止め（アプリの `decode_bytes` の分岐の条件と同じ。BOM の無い file はアプリが CP932 で読み、手順書は CP932 を使わない。BOM 付きで中身が UTF-8 として不正な file は取込みの file の error で止まる）、BOM があれば取り込む前に全部の行で読み、5 つの列が手順 4 で決めた形でない行を行の番号・列・値で名指して止める（Excel の保存がセルの文字のまま書くこと〈P1〉を前提にしない。P1 は実機で確かめない、repo 外の回答台帳 TD-437）。指数の形の値の行と、桁が落ちてチェックディジットが合わない `JANコード` の行は初回の導入の版のプレビューが名指しで止める（J5 = b・TD-426。今の取込みはコードの形を検査しない。26 §18.3、30 §4.8 の手順 4）
- C7 売価は税込の円の整数（Excel の税抜から J1 で換算）、原価は税抜の円を換算せず店の決まり（小数第 3 位で四捨五入して小数第 2 位まで）で丸めた値（空は空のまま）、`m` の初期在庫は m の数、個数の単位の初期在庫は整数で、上限を超える行は止める。丸めで原価が変わった行は 0 の行と同じく一覧にして owner が見る。`TEXT` の値が元の値と違う行（形式の選び違い、`m` の初期在庫の小数 3 桁以上〈丸めの決まりが無い〉、売価・個数の初期在庫の整数でない値、空のセルに値を書いた `TEXT`）は置き換える前に全部の行で数えて止め（比べは両辺を数にして行い、エラー値と数として読めないセルの行も止める）、照合は在庫の数を値を取った元のセルの数式バーの値と、原価をその値を同じ店の決まりで丸めた値と比べる（30 BIZ-01-D8、D-128、SPEC-UNIT-D5、SPEC-STK-VAL-D3）
- C8 初期在庫は取り込んだ日を業務日とする初期在庫の記録になり、どの計数よりも前とみなす。J3 の既定は年末の在庫 Excel の数で、差は次の計数で棚卸しの差として出る（ADR SPEC-STK-CNT-D3）
- C9 流れ 1 は Excel のある Windows で行い、流れ 2・3 の照合では元の在庫 Excel を開ける PC を手元に置く（TD-437）。CSV は開発用の Windows へ写して SHA-256 を照らす。予行演習は店の PC と同じ版（導入の前の関門の confirmed を全部満たした版）・同じ CSV（SHA-256）で開発用の Windows で通し、MSI の手順書の準備と片付けのとおり開発のデータの folder を退避した空の DB で行う。同じ日に MSI の試験用のデータを使うときは、その片付けを終えてから始める
- C10 入れた後の CSV の上書きは `JANコード` を CSV の値で書く（空・列なしは JAN を消す。30 BIZ-01-D8 の上書きの段、30 §4.9.2）ので、上書きの CSV には今の `JANコード` を全部の行に入れる。初回の導入の版のプレビューは上書きの行も検査する（J5・TD-426）ので、商品コードが 13 桁の数字の商品の JAN は消せず、止めて owner に諮る

## Failure Modes

- FM1: 照合が合わないのに業務の入力を始める、または取り込む前の backup を作らずに取り込む（C1）
- FM2: 業務の入力を始めた後の誤りを、取り込む前の backup の復元か DB の作り直しで直し、その後の入力が消える（C1）
- FM3: 取り込む前の backup が保存先の中にだけあり、保持日数（既定 3 日）で消える。または照合（予行演習の復元を含む）を日をまたいで延ばし、一覧から取り込む前の backup が消えて戻せない（C1）
- FM4: 13 桁の JAN の行で `JANコード` が空か違う値のまま入り、レジの売上が商品に引き当たらず、`PLU対象` の既定も対象外になる。関門の 8 を満たさない版で予行演習する、または数字だけの `商品コード` が `JANコード` と違う行を抜き取りの照合だけで確かめ、途中の行を見落とす。8 桁の JAN の先頭の 0 が落ちて 7 桁で入る（C2）
- FM5: プレビューにエラー行があるまま取り込み、一部の行だけが入る（C3）
- FM6: 店が JAN を使っていない行の JAN を空にして、請求書の照合と Z004 の参照の値を失う（D-028）、またはその行の `PLU対象` が空で 13 桁の JAN から対象になる。共有 JAN の群に JAN を商品コードにした行が混ざる、群の行が在庫連動のまま入る、または 8 桁の共有 JAN の行が 8 桁の JAN の種類に数えられ在庫連動のまま入る（C4）
- FM7: 接頭辞の無い部門の行に独自コードを作り、独自コードルールの外の形のコードが入る（C4）
- FM8: 商品コードを誤って入れた後に、廃番と登録し直しで直そうとして同じ JAN の商品が 2 つになる（C5）
- FM9: Excel が JAN を指数の形で書いた CSV が 1 つの商品コードとして黙って入る。書式を「文字列」に変えただけで数値のセルが直ったと読む、`TEXT(セル,"0")` の形で先頭の 0 を落とす、8 桁の値に 13 桁の形式を当てて `00000` + 8 桁にする、空のセルに `TEXT` を当てて 0 にする、CP932 で保存して文字が `?` に化ける、BOM の無い file が保存した file の確かめを通り取込みの file の error で初めて止まる、同じ名の列の後ろの値が入る、Excel の保存が書き替えた file を確かめずに取り込む（C6）
- FM10: 売価を税抜のまま入れる、原価を税込に換算する、`m` の初期在庫を cm の数で入れる。原価・`m` の初期在庫が CSV の保存か `TEXT` の形式（選び違いを含む）で表示の桁に丸まり、正常な行として入り、照合の相手の Excel の表示とも合ってしまう。原価の丸めで元の資料の桁の誤りが黙って直る（C7）
- FM11: 初期在庫を入れた日より前の売上が在庫から引かれると読める、または数えた日と入れた日の差が黙って消えると読める（C8）
- FM12: 流れ 1 を Excel の無い PC で行うと読める、照合で元の在庫 Excel を開けない、予行演習を店の PC で行う、別の版・別の CSV で通す、開発のデータの folder か MSI の試験用のデータの DB に当てる（C9）
- FM13: 売価だけを直す CSV（`JANコード` の列なし）で上書きし、黙って JAN が消えてレジの売上が引き当たらなくなる。または、商品コードが 13 桁の数字の商品の JAN を空の上書きで消す手順を示し、プレビューの検査で止まって直せない（C10）

## Test Matrix

| Contract | Failure Mode | Test Type | Test Name | Would fail if... |
|---|---|---|---|---|
| C1 | FM1 | rg（packet の AC6） | AC6 | 手順書から「全部が合ったときだけ業務の入力を始める」か「業務の入力を始めた後は、取り込む前の backup へ戻さない」が消える |
| C1 | FM2 | rg（packet の AC5） | AC5 | 手順書に、DB の作り直しで直す文（文ごとに分けて、「作り直さない」「作り直しで行わない」「作り直しの範囲」の外の「作り直」）が 1 文でも出る、または「入れた後のやり直しを DB の作り直しで行わない」が消える |
| C1 | FM1・FM3 | 読み取り（Final Review、68 §68.4・71 §71.5 と突き合わせる） | 流れ 2 の冒頭の後の段落、流れ 3 の前提の後の段落と手順 2・7・9 | 取り込む前の backup が取込みより後にある、2 か所への写しが無い、写しの失敗で取り込みを止めない、復元が業務の入力の前に限られていない、照合か予行演習の復元を日をまたいで延ばせる |
| C2 | FM4 | 読み取り（`src-tauri/src/biz/product_service.rs` の `commit_import` の新規の行の `jan_code: row.jan_code.clone()`、30 BIZ-01-D1、Plans の導入の前の関門の 8 と突き合わせる） | 今の事実の JANコード と黙って入る形の行、J5、流れ 1 の手順 2 の表の 13 桁の JAN の行、手順 4 の確かめの列、流れ 2 の冒頭と手順 4 | 13 桁の JAN の行の `JANコード` が空でよいと読める、数字だけの `商品コード` が `JANコード` と違う行の確かめが表の全部の行でなく抜き取りになる、予行演習の版が関門の 8（TD-426 の桁とチェックディジットを含む）を満たさなくてよいと読める |
| C3 | FM5 | 読み取り（30 §4.9、UI-01c-D8） | 流れ 2 の手順 4、流れ 3 の手順 3 | エラー行か重複行が 1 行でもあるまま取り込む段がある |
| C4 | FM6・FM7 | 読み取り（ADR SPEC-STK-CNT-D4、master-tables の独自コードルール） | 流れ 1 の手順 2 の表と箇条 | 種類が排他でない（上から順の当てはめが無い、8 桁の行が共有しない JAN に限られない）、店が JAN を使っていない行が JAN で分ける種類より下にあり商品コード = JAN になる、その行の `JANコード` が空になるか `PLU対象`・`POS在庫連動` が `0` でない、長く動いていないコードの無い品を入れる、共有 JAN の群の行に JAN の商品コードか `POS在庫連動` の空がある、部門 11・12 の行を入れる、接頭辞の無い部門で止めない、どれにも当たらない行を入れる |
| C5 | FM8 | 読み取り（SP-102-04、30 BIZ-01-D8） | 入れた後に誤りを見つけたらの表 | 商品コードか単位の種類をまたぐ誤りに、アプリの中の直し方（廃番と登録し直しを含む）を手順として示す |
| C6 | FM9 | 読み取り（26 §18.3 と `decode_bytes`、30 §4.8 の手順 4、Plans の導入の前の関門の 8、packet の Contract Probe P3） | 流れ 1 の手順 4・5、流れ 2 の手順 4、今の事実の Excel の CSV の保存と黙って入る形の行 | 文字列の値への置き換え（桁の数の 0 を並べた `TEXT`）が書式の変更だけか `TEXT(セル,"0")` に戻る、`売価`・`原価`・`初期在庫` が文字列の値にする列から外れる、空のセルの扱いが無い、先頭の BOM の確かめが無い（文字コードの外れが取込みまで素通りする）か `Import-Csv` の後に回る、保存した file の確かめが無いか抜き取りになるか 5 つの列のどれかを見ない、外れた行の直し方（全部の列が空の行は sheet のその行を消して保存から、sheet のセルが違えば列ごとに `商品コード` は手順 2・負の値と空の原価は元の資料から手順 3・`TEXT` の当て忘れと形式の選び違いは手順 4 から、sheet が正しく file が違えば取り込まず止める）が無い、全部の列が空の行か行の数の 1 ずれが「sheet が正しく file が違う」の分岐へ進む、値の中の改行がプレビューの行の error で止まると書かれない、流れ 2 の手順 4 に file の error のときの戻り先（流れ 1 の手順 5）が無い、sheet の 1 行目と同じ名の列の段が無い、保存が CSV UTF-8 でなくなる、指数の形の値と JAN の桁・チェックディジットをプレビューが止めることが関門の版に結び付かない |
| C7 | FM10 | 読み取り（30 BIZ-01-D8、D-128、`parse_cost_cell`・`parse_hundredths`） | 流れ 1 の手順 3・4、流れ 2 の手順 6 | 売価の換算、原価を換算しない、`m` は m の数、のどれかが逆になる。原価の丸めが手順 3 に無いか店の決まりと違う、空の原価が丸めで 0 になる、丸めで変わった行の一覧が無い、手順 4 に行ごとに `ROUND` を決める分岐が戻る。`TEXT` の値と元の値の比べが置き換えた後に回る・抜き取りになる・`売価`・`原価`・`初期在庫` のどれかに無い、照合が在庫の数か原価を Excel の表示か文字列にした値と比べる、照合の原価の相手が元の値のまま（丸めた行が合わずに止まり続ける） |
| C7 | FM10 | 読み取り（比べの列の式を入力の種類ごとに評価する） | 流れ 1 の手順 4 の比べの列 | 期待（原価の列、形式 `"0.00"`、式 `=IFERROR(IF(元のセル="",TEXTのセル<>"",IF(TEXTのセル="",TRUE,--TEXTのセル<>--元のセル)),TRUE)`）: 数の `12.34` と `TEXT` の `"12.34"` → FALSE で進む。文字列の数 `"12.34"` → 両辺を数にして FALSE で進む。元も `TEXT` も空 → FALSE で進み、空のまま残る。元が空で `TEXT` に値（`IF` の無い `TEXT` の式の `"0.00"`）→ `TEXT のセル<>""` で TRUE になり止まる。元に値があり `TEXT` が空（その行に `TEXT` の式が無い）→ 内側の `IF` で TRUE になり止まる（元が 0 の行も）。元がエラー値か、どちらかが数として読めない文字 → 外側の `IFERROR` で TRUE になり止まる。Would fail if: 比べの片側だけを数にする（文字列の数が TRUE になり、正しい行で止まる）、元が空の枝が FALSE を返す（空の原価が `0.00` で入る）、`TEXT` が空の枝が無い（元が 0 の行で空の `TEXT` が通る）、`IFERROR` が式の全体を包まないか FALSE を返す（エラー値の行が `COUNTIF` に数えられずに進む） |
| C8 | FM11 | 読み取り（ADR SPEC-STK-CNT-D3 の判定表） | 今の事実の初期在庫の記録の行、J3 | 初期在庫の業務日が数えた日になると読める、差が次の計数で出ることが書かれていない |
| C9 | FM12 | 読み取り（MSI の手順書の確かめる項目と準備と片付け、TD-437） | 流れ 1 の冒頭、流れ 2 の冒頭と手順 1・2・6・8、流れ 3 の前提 | 流れ 1 が Excel の無い PC で行うと読める、流れ 2・3 の照合で元の在庫 Excel を開ける PC が無い、開発用の Windows への写しの SHA-256 の照合が消える、予行演習の版と CSV が店の PC と同じであることを求めない、準備と片付けを引かない、MSI の試験用のデータとの境（片付けの後に始める、商品 0 件で止める）が無い |
| C10 | FM13 | 読み取り（30 BIZ-01-D8 の上書きの段、30 §4.9.2、`commit_import` の上書きの `jan_code: Some(row.jan_code.clone())`、`preview_import` が行の検証を重複の分類より前に行うこと、backlog の関門の 8 の項目の AC の要旨） | 今の事実の上書きの JANコード の行、入れた後に誤りを見つけたらの表 | 上書きの CSV で `JANコード` の空・列なしが今の値を保つと読める、商品名・売価などを直す上書きに今の `JANコード` を入れる段が無い、商品コードが 13 桁の数字の商品の JAN を空の上書きで消せると読める、13 桁の JAN を入れ直す行の `PLU対象` がレジで読むかで `1`・`0` に分かれない（無条件の `1` か空）、レジで読まなくなっただけの商品の直しで `POS在庫連動` の扱いが 2 通りに読める |
| 全体 | 案内から辿れない | rg（packet の AC3・AC4） | AC3・AC4 | PROJECT_HANDOFF か backlog から手順書の link が消える、手順書の節の見出しが欠ける |

手順が引く振舞いの既存の自動 test（2026-10-23 に `rg -n 'fn test_…' src-tauri/src` で実在を確かめた）:

- 文字コード・ヘッダ・引用符（C6）: `test_parse_csv_req104_cp932_normal`、`test_parse_csv_req104_utf8_bom_normal`、`test_parse_csv_req104_decode_failure`、`test_parse_csv_req104_double_quote_handling`（`src-tauri/src/io/product_csv_importer.rs`）
- プレビューの分類と行の error（C3）: `test_preview_import_req104_validation_errors`、`test_preview_import_req104_duplicates`、`test_preview_import_req104_mixed`、`test_preview_import_req104_product_code_length_limit`（`src-tauri/src/biz/product_service.rs`）
- 単位・`m` の初期在庫（C7）: `test_preview_import_req104_stock_unit_code_and_label_normalized`、`test_preview_import_req104_initial_stock_in_meters_for_m_rows`、`test_commit_import_req104_revalidates_initial_stock_of_new_rows`
- 初期在庫の記録と写し（C8）: `test_commit_import_req104_initial_stock_movement`、`test_commit_import_req104_initial_stock_movement_and_copy_match`
- `PLU対象` の既定（C2）: `test_commit_import_req907_blank_plu_target_derives_for_new_and_preserves_overwrite`
- 確定の戻し（C3）: `test_commit_import_req104_rollback_on_failure`
- 独自コードの発番が CSV の番号を飛ばす（C4）: `test_create_product_req101_custom_code_after_csv_import`、`test_generate_custom_code_req101_no_prefix`
- Z004 の `jan_code` の引き当て（C2）: `test_parse_and_validate_req401_normal_matching`、`test_parse_and_validate_req401_unmatched_product`（`src-tauri/src/biz/csv_import_service/tests/parse_tests.rs`）
- 復元で内容が置き換わる（C1）: `test_restore_backup_req901_replaces_data`（`src-tauri/src/mnt/backup.rs`。suppliers だけを見る）。取り込む前の backup の復元で商品と入出庫の記録が 0 件に戻ることは、packet の Contract Probe P2 の一時 test で確かめた（test は残していない）

## State Lifecycle Matrix

手順の状態（店の PC の DB）の移り変わり。

| State / subject | Initial | Pending | Success | Invalidate | Refetch | Revisit | Restart | Failure | Retry | Evidence |
|---|---|---|---|---|---|---|---|---|---|---|
| 店の PC の DB | 商品 0 件（MSI の手順書の流れ 2 の後） | 取り込む前の backup を 2 か所へ写した | 照合が全部合い、業務の入力を始める | — | — | 開き直してホームが出る（流れ 3 の手順 5） | 開き直した後に照合（手順 6） | 照合が合わない → 入力を始めずに取り込む前の backup へ復元 → 商品 0 件 | CSV を直して流れ 1 から（予行演習を含む） | 流れ 3 の手順 2〜7 |
| 商品 CSV | 在庫 Excel の写し | 表で確かめた（流れ 1 の手順 4） | CSV と SHA-256 | 直したら SHA-256 が変わる | 予行演習と店の PC で SHA-256 を照らす | — | — | 確かめで止まる行がある → 直す | 流れ 1 の手順 3 から | 流れ 1、流れ 3 の手順 1 |
| プレビュー | ファイルを選ぶ | プレビュー（DB を変えない） | エラー行と重複行が 0 → 取り込む | — | — | — | — | エラー行がある → 取り込まない | CSV を直して選び直す | 流れ 2 の手順 4 |

## Adjacent Pattern Audit

| Source pattern / contract | Repository sites inspected | Ported sites | Explicit exclusions and reason | Test / evidence |
|---|---|---|---|---|
| 管理者の手順書の形（対象・引く正本・未確認の点・今の事実の表・流れ・確かめる項目） | `docs/msi-distribution-runbook.md`、`docs/migration-recovery-runbook.md` | 手順書の冒頭の引用・今の事実の表・流れ・未確認（P*n*） | 配布の記録の表は持たない（管理者の記録は流れ 3 の手順 9 の項目だけ） | AC4 |
| 取り込む前の backup を 2 か所へ写して SHA-256 を照らす | MSI の手順書の流れ 3 の手順 3・4 | 流れ 3 の手順 2・9 | — | 読み取り |

## Negative Paths

- missing input: 必須列が無い CSV は file ごと拒否（30 §4.8 の手順 3）。`JANコード` の空は今は拒否されないので流れ 1 の手順 2 の表で埋め、13 桁の数字の `商品コード` の行は関門の 8 の版のプレビューが止める（C2）。上書きの行の `JANコード` の空・列なしは JAN を消す（13 桁の数字の `商品コード` の商品では関門の 8 の版のプレビューが止める。C10）
- invalid input: 行の error はプレビューに出て取り込まない（C3）。指数の形の値と、`JANコード` の桁かチェックディジットが合わない値は今は拒否されず、関門の 8 の版のプレビューが行の error として名指す（C2・C6、TD-426）
- duplicate/ambiguous input: CSV の中の重複は行の error、既存のコードは重複行（新しい DB では 0）。共有 JAN の群は独自コードに分ける（C4）
- unknown reference: `部門ID` が無い部門・`取引先ID` が無い取引先は行の error（30 §4.8）。`取引先ID` は空にする
- dependency missing: 予行演習の版が店の PC の版と違う → 流れ 3 の前提で止める（C9）
- permission/write failure: 取り込む前の backup か写しが失敗 → 取り込まない（C1）
- dry-run side effect: プレビューは DB を変えない（30 §4.8）

## Boundary Checks

- threshold: 売価 999999、原価 999999.99 円、初期在庫 999999999（`m` は cm にした後）、商品コード 100 文字、file 20MB
- null/default: `在庫単位` の空は `個`、`初期在庫` の空は 0、`POS在庫連動` の空は連動する、`PLU対象` の空は 13 桁の `jan_code` で決まる
- empty/non-empty: 初期在庫の空と 0 は取り込んだ後に区別できない（packet の Contract Audit の 0 の表示）
- min/max: 上の threshold
- status/policy enum: 税率 `10`・`8`・`0`
- wire type: 変えない
- internal type: 変えない（原価はアプリが 1/100 円にする）
- producer/consumer: 在庫 Excel（店）→ CSV（管理者）→ 取込み（アプリ）
- round-trip token: 該当なし
- precision/range: 原価と `m` の初期在庫は小数 2 桁まで（原価の 3 桁以上は流れ 1 の手順 3 で店の決まりで丸め、変わった行を一覧にする。`m` の初期在庫の 3 桁以上の行は手順 4 で止める。CSV には `"0.00"` の文字列で書く）
- cross-language parse: Excel の数値の書式と CSV の文字列（C6）

## Compatibility Checks

- old schema/input: 該当なし（新しい DB に入れる）
- new schema/input: 初回の導入の版（Plans の導入の前の関門の confirmed を全部満たす版）
- output order: 該当なし
- optional field behavior: 上の null/default

## Data Safety Checks

- source-derived data: 在庫 Excel と CSV は repo の外
- generated outputs: 管理者の記録は repo の外
- secrets: 該当なし
- local-only files: 予行演習の folder（消す時期は owner）
- synthetic sample boundaries: 本 lane は試験用のデータを作らない

## Main Wiring / Integration Checks

- helper connected to main path: 該当なし
- output reaches manifest/report: PROJECT_HANDOFF から手順書へ（AC3）
- effective config reaches runtime: 該当なし
- CLI arg reaches implementation: 該当なし

## Mutation-style Adequacy Questions

docs だけの lane で、mutation は手順書の文を誤った読みに替えることを指す（上の Test Matrix の Would fail if）。

- 「全部が合ったときだけ業務の入力を始める」を消すと AC6 が落ちる。
- 「作り直しで行わない」を「作り直しで直す」に替えると AC5 が落ちる（文ごとに分けるので、同じ行の「作り直さない」に隠れない。round 1 の是正で `$TMPDIR` の写しに当てて確かめた。packet の AC5）。
- 流れ 1 の手順 2 の表の 13 桁の JAN の行の `JANコード` を空にすると、Final Review の C2 の読み取りが落とす（機械の検査は無い。Residual Test Gaps）。

## Residual Test Gaps

- CSV の中の商品コードの重複の行の error（`CSV内で商品コードが重複しています`）を直接確かめる自動 test は見つからなかった（`rg -n 'CSV内で商品コードが重複' src-tauri/src` は `preview_import` の 1 行だけ）。手順書は CSV の側で重複を確かめず、プレビューのこの行の error と、エラー行が 0 のときだけ取り込む確認（流れ 2 の手順 4、流れ 3 の手順 3）に頼る。この error が退行すると手順書の側に代わりの確かめは無い。直接の回帰 test を足すのは code を変える lane（backlog の「商品 CSV のプレビューで黙って入る形の行を止める」の runtime の lane）で行う（残課題）。
- 手順書の表の値（J1〜J4 の既定と J5 の答え、行の種類）は機械で検査しない。Plan Review と Final Review の読み取りと、予行演習で確かめる。
- P1（Excel の CSV の保存がセルの文字のまま書く）は実機で確かめない（安全を Excel の保存の確かめに置かず、保存した file を手順 5 で確かめる。repo 外の回答台帳 TD-437 の後の裁定）。流れ 1 の手順 5 が、先頭が UTF-8 の BOM でない file（CP932・BOM の無い UTF-8・UTF-16）を行を読む前に止め（BOM 付きで中身が UTF-8 として不正な file は流れ 2 の手順 4 の file の error で止まる）、保存した file の 5 つの数字の列を全部の行で読み、手順 4 で決めた形でない行（表示の桁の丸め、先頭の 0 の落ち、指数の形、桁区切り）を名指して止め、確かめた行の数を手順 2 の和と比べる。形を保ったまま違う値を書く外れ方はこの段で拾えず、照合の抜き取り（流れ 2 の手順 6）でしか拾わない（packet の Contract Probe P1）。手順 5 の block は是正の前と後の形の両方を Windows PowerShell 5.1 で回した（packet の Contract Probe P3）。PowerShell 7 では回していない。
- 原価・`m` の初期在庫の表示の桁や `TEXT` での丸めはプレビューが止めない（丸まった値も正しい形の数）。原価は手順 3 で店の決まりで丸めて変わった行を一覧にし、`TEXT` の丸め（形式の選び違いを含む）は手順 4 の比べの列が、Excel の保存の書き替えは手順 5 の保存した file の確かめが、全部の行で名指す。この 2 つを飛ばすと、気づく手段は照合の抜き取り（流れ 2 の手順 6。在庫の数を元のセルの数式バーの値と、原価をその値を店の決まりで丸めた値と比べるので、選んだ行が表示の桁か `TEXT` で丸まっていれば合わない）だけになる。アプリの側で見つける形（プレビューに原価 × 初期在庫の合計を出し、Excel の合計と比べる）は backlog の関門の 8 の項目の起票で決める候補 (5) に置いた（AI の review から出た案で、owner の確認の前に必須にしない）。
- CSV の上書きが `JANコード` の空・列なしで JAN を消す振舞いを直接確かめる自動 test は見つからなかった（`test_commit_import_req104_overwrite_preserves_optional_fields` は在庫単位だけを見る）。Plan Review round 2 の Claude 側が一時 test で再現した。test を足すのは関門の 8 の runtime の lane（backlog の項目の候補 (3)）で行う。P2（復元で 0 件に戻る）は code の上で確かめ（packet の Contract Probe）、Windows の画面の上の復元は予行演習の手順 7 で確かめる。
