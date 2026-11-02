# 日報の紙（印刷）の L3 の合成 fixture

日報の紙の runtime（[Plan Packet](../../../docs/plans/2026-10-27-daily-report-sheet-runtime.md) の S7、[Test Design Matrix](../../../docs/plans/test-matrices/2026-10-27-daily-report-sheet-runtime.md) の Manual）の Windows L3 の入力物です。

すべて合成の値です。実店舗の CSV 本文・売上・商品名・JAN・部門名を含みません。名称は `項目03`・`支払A`・`部門01`・`合成P01` のような合成の語で、`その他小物` は作り直した空の DB にある初期の部門（`src-tauri/src/db/schema_v1.rs` の `departments`）の名前です。全て CP932・CRLF です。

## 取り込む順

作り直した空の DB で、次の順に取り込みます（Matrix の Manual の (1)〜(6)。owner は商品の一括 import と売上データ取込みの画面で file を選ぶだけで、DB へ直接書きません）。

1. 商品の一括 import で `products-sheet.csv`
2. 売上データ取込みの日報で `tests/fixtures/daily-report-z001/` の `2026-03-24` の 2 束（`_0001` → `_0002`。2 束目は「追加で取り込む」）
3. 売上データ取込みの Z004 で `Z004_260324.CSV`
4. 日報で `2026-03-25` の 2 束（`Z001_260325_0001.CSV`・`Z002_260325_0001.CSV`・`Z005_260325_0001.CSV` → 同じ形の `_0002`。2 束目は「追加で取り込む」）
5. 日報で `2026-03-26` の 1 束（`Z001_260326_0001.CSV`・`Z002_260326_0001.CSV`・`Z005_260326_0001.CSV`）
6. Z004 で `Z004_260326.CSV`

(5) の束の Z005 の 60 行（`部門01`〜`部門60`）は合成の部門名なので、空の DB の部門に当たりません。日報の取込みの確認の画面に「確認が必要な部門があります」の知らせが 60 件並び、日次売上にも「部門マスタと対応していない部門が 60 件あります（部門名のまま表示しています）」が出ます。合成の名前のためで想定どおりなので、そのまま取り込みます（異常ではありません）。

## file の役

| file | 日付 | 精算回数 | 役 |
| --- | --- | --- | --- |
| `products-sheet.csv` | — | — | 合成の商品 80（`SHEET-P01`〜`SHEET-P80`、商品名 `合成P01` の形、部門ID 1、売価 100、原価 50.00、税率 10、在庫単位 `pcs`、初期在庫 1000、POS在庫連動 1）。b1 の `products-new.csv` と同じ形 |
| `Z004_260324.CSV` | 2026-03-24 | 0001 | 2 回取り込んだ日（`daily-report-z001/` の `2026-03-24`）の Z004。商品に当たる行 5（`SHEET-P01`〜`P05`、個数 1〜5、金額 100〜500）と、どの商品にも当たらない JAN の行 1（紙と画面に `数えていない PLU の行が 1 行あります` が出る） |
| `Z001_260325_*.CSV`・`Z002_260325_*.CSV`・`Z005_260325_*.CSV` | 2026-03-25 | 0001・0002 | 支払集計を足さない日（SALES-SHEET-D2 の 3）。Z002 は 1 束目が `支払A`・`支払A` の 2 行、2 束目が `支払X`・`支払A`・`支払A` の 3 行で、行がずれ 1 回の精算の中に同じ名称が 2 行ある。Z001（総売・純売）と Z005（`その他小物` の 1 行）は 2 束で組がそろい、足す |
| `Z001_260326_0001.CSV`・`Z002_260326_0001.CSV`・`Z005_260326_0001.CSV` | 2026-03-26 | 0001 | 改ページの日。Z001 30 行（1・2 行目は `総売`・`純売`、他は `項目03`〜`項目30`）、Z002 80 行（1 行目 `現金`、他は `支払02`〜`支払80`）、Z005 60 行（`部門01`〜`部門60`）。名称の番号は行の位置で、紙の境の前後の行が続いているかを番号で見る |
| `Z004_260326.CSV` | 2026-03-26 | 0002 | `SHEET-P01`〜`P80` の 80 行（全部商品に当たる。個数 1、金額 100）。`Z004_260324.CSV` の次の精算回数 |

Z004 は layout A（マシンNo. `0001`、精算回数は 4 桁の 0 埋め、時刻 `20:00`）で、精算の識別メタを持ちます（b1 の後の Z004 は精算回数の無い file を拒む）。JAN は `29` と 0 で埋めた 10 桁の番号の 12 桁に、EAN-13 の check digit（左から重み 1・3 の交互）を足した 13 桁です（当たらない行は番号 `9999999999`）。

`2026-03-26` が必ず改ページを起こす理由（56 UI-09a-D17 の見積りの前提: 行の高さ 4.4〜5 mm、印刷できる高さ 277 mm で 1 枚に約 55〜63 行）: 左の段（Z001 30 行と Z005 60 行と見出しの 4 行）と右の段（Z002 80 行と見出しの 2 行）がどちらも 1 枚目を越えて 2 枚目に続き、PLU の表（80 行と見出しの 2 行）は 2 枚目の左の段の後から始まり 3 枚目へ続きます。行の高さが 3.5 mm（1 枚に約 75 行）でも、両段と PLU がどれも紙をまたぎます。

## 生成手順

下の Python 3（標準ライブラリだけ）の script を repo の外の file に保存し、repo の root で次を実行して 12 file を書き出します。CSV を手で編集したり、エディタで UTF-8 として上書き保存したりしないでください。中身を変えるときは script を直して再生成し、この README の表を合わせます。

```sh
python3 -I <script> tests/fixtures/daily-report-sheet
```

```python
"""日報の紙の runtime の L3 の合成 fixture を書き出す（標準ライブラリだけ。CP932・CRLF）。

使い方: python3 -I gen_sheet_fixtures.py tests/fixtures/daily-report-sheet
"""
import pathlib
import sys

OUT = pathlib.Path(sys.argv[1])


def write(name, lines):
    (OUT / name).write_bytes(("\r\n".join(lines) + "\r\n").encode("cp932"))


def jan(index):
    # 本体は 29 と 0 で埋めた 10 桁の番号の 12 桁。重みは左から 1・3 の交互（l3_fixtures.rs の jan と同じ）
    body = f"29{index:010d}"
    total = sum(int(d) * (1 if i % 2 == 0 else 3) for i, d in enumerate(body))
    return body + str((10 - total % 10) % 10)


def report(source, date, settlement, time, rows):
    third = "個数        " if source == "Z005" else "個数/件数   "
    lines = [
        '"マシンNo.   ","01","",""',
        '"ファイル    ","synthetic","",""',
        '"モード      ","精算","",""',
        f'"精算回数    ","{settlement}","",""',
        f'"日付        ","{date}","",""',
        f'"時刻        ","{time}","",""',
        "",
        f'"レコード    ","キャラクター","{third}","金額        "',
    ]
    for position, (label, value, amount) in enumerate(rows, start=1):
        lines.append(f'"{position:04d}","{label}","{value}","{amount}"')
    return lines


def z004(settlement, date, rows):
    lines = [
        '"マシンNo.   ","0001"',
        '"ファイル    ","Z004"',
        f'"精算回数    ","{settlement}"',
        f'"日付        ","{date}"',
        '"時刻        ","20:00"',
        "",
        '"レコード    ","ｽｷｬﾆﾝｸﾞｺｰﾄﾞ ","キャラクター","個数        ","金額        "',
    ]
    for line, (code, name, quantity, amount) in enumerate(rows, start=1):
        lines.append(f'"{line}","{code}","{name}","{quantity}","{amount}"')
    return lines


# 商品の一括 import（b1 の products-new.csv と同じ形。部門ID 1、在庫単位 pcs、POS在庫連動 1）
products = ["商品コード,商品名,部門ID,売価,原価,税率,JANコード,在庫単位,初期在庫,POS在庫連動"]
for index in range(1, 81):
    products.append(f"SHEET-P{index:02d},合成P{index:02d},1,100,50.00,10,{jan(index)},pcs,1000,1")
write("products-sheet.csv", products)

# 2026-03-24 の Z004（daily-report-z001 の 2 回取り込んだ日）: 商品に当たる 5 行と当たらない 1 行
rows = [(jan(i), f"合成P{i:02d}", i, i * 100) for i in range(1, 6)]
rows.append((jan(9999999999), "合成未登録", 1, 100))
write("Z004_260324.CSV", z004("0001", "2026-03-24", rows))

# 2026-03-25: 支払集計を足さない日（D2 の 3。行がずれ、1 回の精算の中に同じ名称が 2 行）
bundles = [
    ("0001", "12:00", [("総売", "2", 3000), ("純売", "2", 3000)],
     [("支払A", "1", 1000), ("支払A", "1", 2000)], [("その他小物", "2", 3000)]),
    ("0002", "19:00", [("総売", "3", 7500), ("純売", "3", 7500)],
     [("支払X", "1", 500), ("支払A", "1", 3000), ("支払A", "1", 4000)], [("その他小物", "3", 7500)]),
]
for settlement, time, z001_rows, z002_rows, z005_rows in bundles:
    for source, data in (("Z001", z001_rows), ("Z002", z002_rows), ("Z005", z005_rows)):
        write(f"{source}_260325_{settlement}.CSV", report(source, "2026-03-25", settlement, time, data))

# 2026-03-26: 改ページの日（Z001 30 行・Z002 80 行・Z005 60 行。名称の番号は行の位置）
z001_rows = [("総売", "30", 30000), ("純売", "30", 30000)]
z001_rows += [(f"項目{p:02d}", str(p % 5), p * 100) for p in range(3, 31)]
z002_rows = [("現金", "10", 20000)]
z002_rows += [(f"支払{p:02d}", str(p % 3), p * 10) for p in range(2, 81)]
z005_rows = [(f"部門{p:02d}", str(p % 4), p * 50) for p in range(1, 61)]
for source, data in (("Z001", z001_rows), ("Z002", z002_rows), ("Z005", z005_rows)):
    write(f"{source}_260326_0001.CSV", report(source, "2026-03-26", "0001", "20:00", data))

# 2026-03-26 の Z004: SHEET-P01〜P80 の 80 行（全部商品に当たる）。精算回数は 0002
write("Z004_260326.CSV", z004("0002", "2026-03-26",
                             [(jan(i), f"合成P{i:02d}", 1, 100) for i in range(1, 81)]))
```

## 生成後の verification

`tests/fixtures/z004/README.md` の「生成後の verification」と同じ 3 つを確かめます。

1. encoding: `file tests/fixtures/daily-report-sheet/*` が CP932 の系（`Non-ISO extended-ASCII text`）で、`iconv -f CP932 -t UTF-8` で文字化けせず読める。
2. CRLF: 全 file で `\n` の数と `\r\n` の数が同じ。
3. 読める: 商品の CSV の 80 行の JAN が全部 13 桁で check digit が合い、重複しない。日報の束は parser と取込みを通る（2026-10-31 に一時の test で、`2026-03-25` の 2 束を取り込むと紙の `unsummed_sources` が `[Z002]`・支払が 5 行、`2026-03-26` の束は紙の行が 30・80・60 行、Z004 の 2 file は誤り 0 で 6 行と 80 行、商品の CSV は誤り 0 で 80 行を確かめた。一時の test は残していない）。
