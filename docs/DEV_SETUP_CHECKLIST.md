# 開発環境構築チェックリスト

> **方針**: WSL2 直接開発を現運用とする。Docker 完結（旧案 C）は §A.1 退役記録に履歴保存

---

## 目次

- §1 環境構成
- §2 前提条件（構築済み）
- §3 リポジトリクローン + Git hook セットアップ
- §4 WSL2 直接開発環境セットアップ
- §5 Tauri プロジェクト初期化（参考、Phase 1〜7 で完了済み）
- §6 第1段階完了 — DB 基盤（v0.1.0-db-layer）
- §7 第2段階完了 — 商品管理ロジック（v0.2.0-product-crud）
- §8 第3段階完了 — 入出庫 + 在庫照会バックエンド（v0.3.0-inventory-backend）
- §9 第4段階完了 — POS 連携（v0.4.0-pos-integration）
- §10 第5段階完了 — レポート + 棚卸し + 一括インポート + CMD 層（v0.5.0）
- §11 第6段階完了 — 保守 + 仕上げ + CMD-02〜06 補完（v0.6.0）
- §12 第7段階 — Phase 1 UI 基盤構築（進行中）
- §13 第8段階 — Phase 2 毎日使う 5 画面（完了 / `v0.8.0-ui-daily` tag 済み）
- §14 Phase 3 / Phase 4（画面は実装済み。計画の記録）
- §A.1 退役記録: Docker 完結方針（2026-03-31 採用 → 2026-04-03 退役）

---

## 1. 環境構成

### 1.1 現運用（WSL2 直接開発）

```
Windows 11 Home
├── WebView2 Runtime（Windows 11 にプリインストール済み）
└── WSL2 (Ubuntu)
    ├── Rust 1.83+ (stable)
    ├── Node.js 24 LTS（repository の `.node-version` を local / CI 共通で使用）
    ├── npm
    ├── Tauri 2.0 CLI + Linux 依存ライブラリ
    ├── SQLite3 dev libraries
    ├── Claude Code（WSL2 上で直接作業）
    └── Git
```

### 1.2 必要ツール一覧

| ツール | バージョン | 用途 |
|---|---|---|
| Rust | 1.83+ stable | Tauri バックエンド + scripts |
| Node.js | 24.18.0（`.node-version` が正本） | フロントエンド（React 19 + Vite + TanStack） |
| npm | 12.0.2（D-066 で採用。Node 24 同梱の 11 系から名指し更新） | 依存管理 + lefthook。`allowScripts` / `--allow-git` / `--allow-remote` 既定 off が D-030 の `.npmrc` ガードと併用される |
| ripgrep (`rg`) | apt 経由 | docs / Rust test traceability などの静的チェック |
| WebView2 Runtime | Windows 11 同梱 | Tauri Windows 描画 |
| Tauri 2 Linux deps | apt 経由 | `libwebkit2gtk-4.1-dev` 等 |
| SQLite | rusqlite bundled | DB アクセス（IO-01） |

### 1.3 退役済み構成

**Docker 完結（旧案 C）** は 2026-03-31 採用、2026-04-03 退役。Docker Desktop WSL Integration の `Engine stopping` 固まりが修復不能となり、WSL2 直接運用に切り替えた。退役経緯と再導入条件は **§A.1 退役記録** および [DOCKER_REPAIR_LOG.md](DOCKER_REPAIR_LOG.md) を参照。

### 1.4 Phase 2 以降の Windows native 移行制約

Tauri 2 on Linux は日本語 IME のインライン入力が未対応（[tauri#11412](https://github.com/tauri-apps/tauri/issues/11412) OPEN）。Phase 1 P0 IPC 疎通までは WSL2 で英字入力検証で進行可能だが、Phase 2 着手時（UI-00 ホーム画面以降、商品名 / 取引先名 / 部門名の日本語入力を伴う全画面）には Windows native ビルドへ移行する。最終デリバリ先は Windows 11 Home（手芸店実機）なので、Phase 2 完了時に Windows ネイティブで MSI ビルドを生成して動作確認する。

根拠: memory `tauri2-linux-ime-limitation.md` / `dev-environment-policy.md`、Plans.md Backlog 「Phase 2 以降 Windows native 開発移行」項目。

---

## 2. 前提条件（構築済み）

- [x] Windows 11 Home
- [x] WSL2 が有効化されている
- [x] Ubuntu ディストリビューションがインストール済み
- [x] Claude Code がインストール済み
- [x] Git がインストール済み
- [x] GitHub アカウント＋SSH キー設定済み

> **不要**: Docker Desktop for Windows（§A.1 退役記録参照）

---

## 3. リポジトリクローン + Git hook セットアップ

### 3.1 GitHub リポジトリ初期化（完了済み）

- [x] GitHub にプライベートリポジトリ作成（`inventory-system`）
- [x] `.gitignore` 生成（Rust + Node.js + Tauri 用）
- [x] WSL2 側でクローン

```bash
cd ~
git clone git@github.com:{username}/inventory-system.git
cd inventory-system
```

### 3.2 Git hook セットアップ

クローン直後に 2 種類の hook をインストールする。

#### pre-push hook（push 増分の fast gate + Ready push guard）

push 前に以下を `scripts/pre-push.sh` の順に実行し、いずれか失敗で push をブロックする（Rust の test の実行は Draft の hosted の rust_test が担う）:

- ① workflow git checks（PK5）
- ② Rust 変更時の `cargo fmt --check` / `cargo clippy --all-targets --all-features -- -D warnings`
- ③ `./scripts/doc-consistency-check.sh`（設計書整合 19 項目）
- ④ `./scripts/check-env-safety.sh`（.env / `src/lib/env.ts` 安全性、UI_TECH_STACK §6.9 準拠）
- ⑤ Rust / frontend test / function design / requirements 変更時の traceability check
- ⑥ frontend 変更時の `npm run generate:routes` / `npm run typecheck` / `npm run lint`

pre-push は push 増分の L0 gate であり、PR 全差分の merge evidence ではない。push ごとに走る Draft の hosted の run が `Merge gate` を出し、Ready で回し直さず同じ head の run を merge の CI 根拠にする（D-145 (9)）。手元で全 gate を回したいときだけ `bash scripts/local-ci.sh full`（任意）を使う。

Ready PR への push は stale green 防止のため block する。修正は PR を Draft に戻してから行う。緊急 bypass は raw `--no-verify` ではなく、docs/ci.md が定義する固定 reason token を環境変数で渡して hook を実行し、`.local/quality-check.log` に `BYPASS` を残す。

```bash
INVENTORY_PRE_PUSH_BYPASS_REASON=owner-approved git push
```

```bash
ln -s ../../scripts/pre-push.sh .git/hooks/pre-push
```

複製ではなく symlink にするのは、複製だと後の `scripts/pre-push.sh` の改善が hook に届かないため。`scripts/pre-push.sh` は tracked の mode が実行可能なので `chmod` は要らない。hook は `git rev-parse --show-toplevel` で push した worktree の root を使うので、本体 checkout の版の hook でも検査対象は push した側になる。

実行ログは `.local/quality-check.log` に記録される（`.gitignore` 済み）。

#### lefthook（frontend の pre-commit 自動修正）

commit 時に staged された `*.{ts,tsx,js,jsx}` へ `eslint --fix`、`*.{ts,tsx,js,jsx,json,md,yml,yaml}` へ `prettier --write` を適用する。Rust / 設計書は pre-push が担当するため lefthook は frontend 限定。

```bash
npm install                   # lefthook を含む devDep をインストール
npx lefthook install          # .git/hooks/pre-commit を書き込む（初回のみ）
```

緊急時は `git commit --no-verify` で bypass 可能。`merge` / `rebase` 中は自動 skip される。

### 3.3 Claude Code auto-memory sandbox 設定（オプション）

Claude Code の auto-memory 機能（`MEMORY.md` / 個別 memory ファイル / `.last_audit` sentinel など）は `~/.claude/projects/<sanitized-cwd>/memory/` に保存される。本リポジトリでは sanitize 後の path が `~/.claude/projects/-home-kosei-projects-inventory-system-public/memory/` で固定だが、ここはデフォルトの sandbox writable mount に含まれないため、Claude 側からの書き込み (`touch .last_audit` や新規 memory ファイル `Write`) が `Read-only file system` で失敗する。

auto-memory を使う場合、project local settings (`./.claude/settings.local.json`、repository `.gitignore` で管理外) に sandbox writable mount を 1 行追加する:

```json
{
  "sandbox": {
    "enabled": true,
    "autoAllowBashIfSandboxed": true,
    "filesystem": {
      "allowWrite": [
        "~/.claude/projects/-home-kosei-projects-inventory-system-public/memory"
      ]
    }
  }
}
```

- `-home-kosei-projects-inventory-system-public` は cwd の絶対パスを `/` → `-` 変換した sanitize 文字列。別ホームディレクトリで作業する場合は適宜置換（例: `/home/alice/inventory-system-public` なら `-home-alice-inventory-system-public`）
- 検証: `touch ~/.claude/projects/-home-kosei-projects-inventory-system-public/memory/.last_audit` で `Read-only file system` が出なくなれば反映済（現セッションで即時反映、restart 不要）
- gitignored ファイルなので team 共有されず、他マシン / 他 user は本節の手順で個別に設定する
- global settings (`~/.claude/settings.json`) に書く方法もあるが、`<sanitized-cwd>` 部分がマシン依存のため project local の方が再利用性は低くスコープが明確
- `denyWithinAllow` 設定で `settings.json` 系ファイル自体は引き続き Bash 経由から保護される（`Edit` tool 経由のみ書ける）

### 3.4 agmsg sandbox 設定（オプション）

`$agmsg` は `~/.agents/skills/agmsg/scripts/` の script だけを使って、Claude Code / Codex / Gemini CLI などの agent 間メッセージを SQLite に記録する。daemon や network は不要だが、Claude Code sandbox 下では `~/.agents/skills/agmsg` が writable mount に含まれないため、受信既読状態や送信記録の更新が失敗する。

`$agmsg` を使う場合、project local settings (`./.claude/settings.local.json`、global gitignore でリポジトリ管理外) に sandbox writable mount を追加する:

```json
{
  "sandbox": {
    "enabled": true,
    "autoAllowBashIfSandboxed": true,
    "filesystem": {
      "allowWrite": [
        "~/.agents/skills/agmsg"
      ]
    }
  }
}
```

- `settings.local.json` は machine-specific で team 共有されない。他マシン / 他 user は本節の手順で個別に設定する
- 操作は必ず `~/.agents/skills/agmsg/scripts/` 配下の script 経由で行う。`teams/` や `db/` を直接読んだり編集したりしない
- tracked `.claude/settings.json` のproject hook inventoryは0本で、`claude-code-harness`と`ponytail`（D-147）もproject scopeでは無効。agmsg delivery modeはrepository hookに依存させず、Claude Code / Codexとも`turn`（ターン終端チェック）または`off`のみを使う
- Claude Code の `Edit` で monitor mode の hook 設定を書こうとすると、auto-mode classifier が self-modification と判定して deny することがある。その場合は user が内容を確認して手作業で配置する
- sandbox が invocation ごとに PID namespace を分ける環境では、monitor の pidfile 生存判定が stale と誤判定されることがある。これは monitor 配信周辺の注意点で、script 経由の inbox / send 自体は引き続き使える

---

## 4. WSL2 直接開発環境セットアップ

旧 §10「WSL2 直接移行チェックリスト」を本セクションに昇格。Phase 5 時点で WSL2 直接運用に切り替え済み（memory `dev-environment-policy.md`）。

### 4.1 Rust インストール

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup component add clippy rustfmt
```

### 4.2 Node.js 24 LTS（repository pin）

```bash
mise install node@$(tr -d '\r\n' < .node-version)
mise exec -- node --version
mise exec -- npm --version
```

> Node version の正本は repository root の `.node-version`。ローカル検証と `actions/setup-node@v6` は同じ exact version を読み、`package.json#engines.node`、`package.json#devEngines.runtime`、`@types/node` は同じ Node 24 major に揃える。`devEngines.runtime.onFail = "error"` により、Node 24 以外での通常の `npm install` / `npm ci` / `npm run` は fail-fast する。user-wide default の変更は必須ではなく、WSL2 では `mise exec -- <command>` で repository pin を明示して実行できる。
>
> npm は D-066 により local で 12 系を採用（`npm install -g npm@12.0.2 --prefix ~/.local/share/mise/installs/node/<version>` で mise の node ディレクトリへ導入。既定 prefix `~/.npm-global` は PATH で mise に隠されるため使わない）。**mise で node を再 install / 更新すると npm は同梱の 11 系に戻るため、npm 12 を再導入すること**。CI は Node 24 同梱の npm 11 系のまま（lockfile は npm 11 / 12 どちらでも `npm ci --ignore-scripts` 差分ゼロを 2026-08-12 に実測済み）。

### 4.3 Tauri 2 Linux 依存ライブラリ

```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  ripgrep \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

### 4.4 Tauri CLI

```bash
cargo install tauri-cli --version "^2"
```

### 4.5 動作確認

- [x] `rustc --version` → 1.83 以上
- [x] `node --version` → v24.18.0（`.node-version` / CI 整合）
- [x] プロジェクトディレクトリで `cargo tauri dev` → GUI ウィンドウが表示される
- [x] Claude Code の作業ディレクトリは WSL2 上の `/home/{user}/inventory-system-public` に固定（プロジェクト外保存禁止、CLAUDE.md 「やってはいけないこと」参照）

> fresh checkout / 環境再構築（`npm ci --ignore-scripts`）直後は、frontend gate（typecheck / lint / test）実行前に `npm run generate:routes` を明示実行する。`.npmrc` の `ignore-scripts=true` が `pretypecheck` / `prelint` / `pretest` の `tsr generate` 実行を抑止するため、routeTree 未生成のままだと typecheck / lint が fail する（2026-08-26 実測）。

### 4.6 Phase 2 着手時の Windows native ビルド確認

Tauri 2 on Linux IME 制約のため、Phase 2 (UI-00 ホーム画面以降) の日本語入力検証は Windows native ビルドで行う（§1.4 参照）。Phase 2 完了時に Windows 11 Home で MSI ビルドを生成して動作確認する。

#### L3 利用者デモのコード同期手順（merge 前の feature ブランチを Windows native clone へ取り込む）

Phase 2 以降の L3 利用者デモ（8-0 gate）は WSL2 とは別の Windows 側 clone `C:\Users\Owner\projects\inventory-system-public` で実施する（IME インライン入力のため）。デモ前に GitHub 経由で feature ブランチを取り込む（PowerShell、merge 前なので feature ブランチを触る・main ではない）:

```powershell
cd C:\Users\Owner\projects\inventory-system-public
git remote get-url origin                # public repo（inventory-system-desktop）を指しているか必ず確認
git fetch origin
git restore package-lock.json            # ローカル変更で出る、捨ててOK（reset --hard で origin 版に揃う）
git checkout <feature-branch>
git reset --hard origin/<feature-branch> # git log -1 で期待 SHA と一致確認
npm ci --ignore-scripts                  # 環境再構築の標準（D-030 常設ガード、install script 実行なし）
npm run tauri dev                        # ウィンドウ起動 = 取り込み成功
```

origin が旧 private repo（`inventory-system`）のままだと `git fetch` は成功するのに feature ブランチが見つからず取り込みが止まる。旧 private のままなら public repo へ張り替える: `git remote set-url origin git@github.com:kosei-w90607/inventory-system-desktop.git`。

**毎回ハマる落とし穴（判断は固定）**:

1. `package-lock.json` がローカル変更扱いで checkout をブロックする → `git restore package-lock.json` で破棄（どのみち reset --hard で origin 版に揃うので保持価値ゼロ）
2. `npm ci --ignore-scripts` 後の `npm audit` 警告 → **無視して進む**（テスト系 devDep 由来が大半）。**`npm audit fix` は走らせない**（一括更新は D-030 でも禁止のまま。更新は WSL 側で名指し package の意図的更新のみ）。読み取り専用 `npm audit` までは可
3. **再 migration は schema 変更がなければ不要**（既存 Windows DB の seed 流用）。デモデータが古い/空なら `cargo run --bin seed_demo_data -- --reset`（決定的 seed）

取り込み前に WSL 側で `git diff --stat <前回tag>..HEAD -- package.json package-lock.json` で依存変更（`npm ci` 要否）、migration / schema 差分（再 migration 要否）を裏取りする（「git pull だけで動かない」事故防止）。

> 出典: PR #67 UI-06a L3 デモ運用（2026-05-21）で確立。詳細判断軸は memory `windows-native-l3-runbook` を参照。

---

## 5. Tauri プロジェクト初期化（参考、Phase 1〜7 で完了済み）

> **状態**: 本プロジェクトは初期スキャフォールド + 第1〜第6段階のバックエンド全層 + 第7段階 Phase 1 UI 基盤の構造整備が完了済み。本セクションは新規環境への再構築時の参考として履歴を残す。

### 5.1 create-tauri-app 手順（履歴）

```bash
cargo install create-tauri-app
cargo create-tauri-app inventory-system \
  --template react-ts \
  --manager npm
```

### 5.2 生成されたディレクトリ構成（初期形）

```
inventory-system/
├── src-tauri/           # Rust（バックエンド）
│   ├── src/
│   │   └── main.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                 # React + TypeScript（フロントエンド）
│   ├── App.tsx
│   ├── main.tsx
│   └── ...
├── package.json
├── tsconfig.json
├── vite.config.ts
└── .gitignore
```

### 5.3 現状のディレクトリ構成（拡張後）

ARCHITECTURE.md の 5 層分割（UI / CMD / BIZ / IO / MNT）に従い、`src-tauri/src/{db,biz,cmd,mnt}` および `src/{components,routes,lib,config,features}` 等に拡張済み。詳細は [ARCHITECTURE.md](ARCHITECTURE.md) §1〜§2 を参照。

---

## 6. 第1段階完了 — DB 基盤（v0.1.0-db-layer）

**実装範囲**: IO-01 SQLite データアクセス層初期化 + MNT-03 マイグレーション
**入力ドキュメント**: [FUNCTION_DESIGN.md](FUNCTION_DESIGN.md) §IO-01 / §MNT-03 / [DB_DESIGN.md](DB_DESIGN.md)

- [x] migration.rs: schema_versions テーブル + 初期スキーマ（18 テーブル CREATE TABLE）+ 初期データ INSERT（departments 21 件, app_settings 初期値）
- [x] db/mod.rs: init_database()（PRAGMA foreign_keys=ON + WAL + busy_timeout + migrate 呼び出し）
- [x] CHECK 制約 / FK 制約 / 10 インデックス すべて反映
- [x] テスト 10 本通過（DB 初期化 + 再マイグレーションスキップ + 全 18 テーブル + CHECK 制約）
- [x] cargo clippy 警告ゼロ / cargo fmt --check 準拠
- [x] git tag: `v0.1.0-db-layer`

> 詳細: [archive/v0_tag_history.md](archive/v0_tag_history.md) §v0.1.0-db-layer

---

## 7. 第2段階完了 — 商品管理ロジック（v0.2.0-product-crud）

**実装範囲**: IO-01 product_repo / inventory_repo / stocktake_repo / system_repo / migration + BIZ-01 product_service
**主要 PR**: PR #1 (product_repo) / PR #2 (サポートリポジトリ群) / PR #3 (BIZ-01 product_service)

- [x] product_repo: 11 関数（find_by_product_code / search_products / insert_product / update_product / find_by_jan_code / find_department_by_id / list_departments / increment_next_seq / list_suppliers / find_or_create_supplier / insert_price_history）
- [x] inventory_repo: insert_movement
- [x] system_repo: insert_operation_log
- [x] stocktake_repo: find_active_stocktake / insert_stocktake_item
- [x] product_service: create_product / update_product / toggle_discontinue / search_products / generate_custom_code
- [x] テスト累計 92 本通過（バリデーション + 重複 product_code + price_history + plu_dirty + 廃番反転 + 棚卸し自動追加 + 独自コード発番連番 + 検索ページング/部門フィルタ）
- [x] cargo clippy 警告ゼロ / cargo fmt --check 準拠
- [x] git tag: `v0.2.0-product-crud`
- [x] CMD 層 + UI 層は未実装（バックエンドのみ）

> 詳細: [archive/v0_tag_history.md](archive/v0_tag_history.md) §v0.2.0-product-crud

---

## 8. 第3段階完了 — 入出庫 + 在庫照会バックエンド（v0.3.0-inventory-backend）

**実装範囲**: IO-01 inventory_repo 分割 + receiving / return / manual_sale / disposal_repo + sales_repo + migration v2 / BIZ-02 inventory_service
**主要 PR**: PR #4 (FUNCTION_DESIGN 分割) / PR #5 (IO 層 入出庫 + migration v2) / PR #6 (inventory_repo 分割) / PR #7 (BIZ-02) / PR #8 (inventory_service 分割)

- [x] IO 層: inventory_repo を 8 ファイルに分割 + receiving / return / manual_sale / disposal_repo + sales_repo + 共通型
- [x] migration v2: 冪等性カラム追加
- [x] BIZ-02 inventory_service: apply_stock_change（共通在庫処理）+ 4 業務関数（入庫 / 返品 / 手動販売 / 廃棄）
- [x] 冪等性テスト + 整合性テスト
- [x] テスト累計 205 本通過（PR #5 +57 / PR #7 +56 含む）
- [x] cargo clippy 警告ゼロ / cargo fmt --check 準拠
- [x] git tag: `v0.3.0-inventory-backend`
- [x] CMD-02〜06（入出庫 + 在庫照会コマンド）と UI 層は未実装（バックエンドのみ、CMD は v0.6.0 で補完）

> 詳細: [archive/v0_tag_history.md](archive/v0_tag_history.md) §v0.3.0-inventory-backend

---

## 9. 第4段階完了 — POS 連携（v0.4.0-pos-integration）

**実装範囲**: IO-02 z004_parser / IO-04 plu_formatter / csv_import_repo / BIZ-03 csv_import_service / BIZ-04 plu_export_service / CMD-07 csv_import_cmd / CMD-08 plu_export_cmd
**主要 PR**: PR #9 (第4段階 POS 連携関数設計書) / PR #10 (共有土台 + E-4 契約固定 + IO-04 設計) / PR #11 (設計-コード突合テスト) / PR #12 (PLU レーン IO-04 + BIZ-04) / PR #13 (CSV IO 層 IO-02 + csv_import_repo) / PR #14 (BIZ-03 CSV 取込みパイプライン) / PR #15 (CMD 層 CMD-07/08 + AppState)

- [x] IO-02 z004_parser: CP932 デコード + 改行正規化（ / \r\n / \n / \r）+ メタ行日付抽出 + 5 フィールド CSV パース + JAN 正規化（純関数、DB 非依存）
- [x] IO-04 plu_formatter: カシオレジスター CV17 互換 TSV / CP932（E-4 オンライン調査確定、2026-04-08）
- [x] csv_import_repo + csv_import_errors テーブル
- [x] BIZ-03 csv_import_service: parse / validate / commit / rollback の 4 段階パイプライン + 重複チェック + ロールバック + csv_import_errors 記録
- [x] BIZ-04 plu_export_service: 全件 / 差分モード + plu_dirty 抽出 + 5000 件上限 + plu_dirty=0 + plu_exported_at 更新
- [x] CMD-07 csv_import_cmd: 4 コマンド / CMD-08 plu_export_cmd: 2 コマンド
- [x] AppState + CmdError 整備
- [x] テスト累計 320 本通過（PR #12 +28 / PR #13 +38 / PR #14 +35 / PR #15 +9 含む）
- [x] cargo clippy 警告ゼロ / cargo fmt --check 準拠
- [x] git tag: `v0.4.0-pos-integration`
- [x] UI 層は未実装（バックエンドのみ）。実機 POS レジでの TSV → SD カード → レジ読込みの一連フローは UI-08 着手時（Phase 4 / Plans.md 第10段階 10-3a）に確認

> 詳細: [archive/v0_tag_history.md](archive/v0_tag_history.md) §v0.4.0-pos-integration

---

## 10. 第5段階完了 — レポート + 棚卸し + 一括インポート + CMD 層（v0.5.0）

**実装範囲**: IO-03 product_csv_importer / BIZ-05 sales_service / BIZ-06 stocktake_service / BIZ-07 integrity_service / BIZ-01 一括インポート拡張 + D-2 統合 / CMD-01 / CMD-09 / CMD-10 / CMD-11 部分
**主要 PR**: PR #19 (IO-03 商品 CSV インポーター) / PR #20 (BIZ-05 売上集計) / PR-4 (BIZ-06 棚卸し + DB 拡張) / PR-5 (BIZ-07 整合性 + BIZ-01 Import + D-2) / PR-6 (CMD 層一括 CMD-01 / CMD-09 / CMD-10 / CMD-11 部分)

- [x] IO-03 product_csv_importer: ParsedRow 対応 + エンコーディング自動判定（BOM → UTF-8 / それ以外 → CP932、D-1 確定仕様）
- [x] BIZ-05 sales_service: get_daily_sales / get_monthly_sales（is_voided=0 のみ対象、商品別 / 部門別 / ランキング / 前月比較）
- [x] BIZ-06 stocktake_service: start_stocktake / update_count / complete_stocktake（差異補正 inventory_movements 記録）
- [x] BIZ-07 integrity_service: run_integrity_check / fix_integrity（stock_quantity 突合 + 利用者確認フロー）
- [x] BIZ-01 拡張: preview_import / commit_import + D-2 plu_dirty / plu_exported_at 統合
- [x] CMD-01 product_cmd: 7 コマンド / CMD-09 sales_cmd: 2 コマンド / CMD-10 stocktake_cmd: 4 コマンド / CMD-11 部分 integrity_cmd: 2 コマンド
- [x] テスト累計 448 本通過（PR #19 +12 / PR #20 +22 / PR-4 +51 / PR-5 +35 / PR-6 +6 / design_compliance +2 含む）
- [x] cargo clippy 警告ゼロ / cargo fmt --check 準拠
- [x] git tag: `v0.5.0`

> 詳細: [archive/v0_tag_history.md](archive/v0_tag_history.md) §v0.5.0

---

## 11. 第6段階完了 — 保守 + 仕上げ + CMD-02〜06 補完（v0.6.0）

**実装範囲**: MNT-01 backup / MNT-02 log_manager / MNT-04 diagnostic_log / IO-05 report_csv_exporter / IO-06 image_manager / system_repo 拡張 / CMD-11 settings_cmd / CMD-02〜05 入出庫コマンド / CMD-06 inventory_cmd
**主要 PR**: PR #25 (MNT-04 診断ログ、Issue #24 自動 Close) / Phase 6 PR-1 (設計書) / PR-2 (IO-05/06 + system_repo 拡張) / PR-3 (MNT-02 ログ管理) / PR-4 (MNT-01 バックアップ) / PR-5 (CMD-11 settings) / CMD-02〜05 補完 / CMD-06 補完

- [x] MNT-01 backup: create_backup（VACUUM INTO）/ restore_backup / check_auto_backup / list_backups
- [x] MNT-02 log_manager: cleanup_old_logs（起動時自動削除、365 日超）
- [x] MNT-04 diagnostic_log: tracing 基盤 + ファイルログ（日次ローテーション、30 日保持、REQ-700）
- [x] IO-05 report_csv_exporter: export_csv（UTF-8 BOM 付き、Excel 対応）
- [x] IO-06 image_manager: save_receipt_image（相対パス管理、`images/receipts/...`）
- [x] system_repo 拡張: 操作ログ取得 + ページング
- [x] CMD-11 settings_cmd: 8 コマンド（settings CRUD / list_logs / backup / restore / save_receipt_image）
- [x] CMD-02〜05: 7 コマンド + BIZ list ラッパー 3 関数（受入 / 返品 / 手動販売 / 廃棄）
- [x] CMD-06 inventory_cmd: 3 コマンド（stock_detail / low_stock / movements）+ IO 3 + BIZ 3
- [x] テスト累計 546 本通過（MNT-04 +13 / Phase 6 PR-2 +24 / PR-3 +6 / PR-4 +16 / PR-5 +14 / CMD-02〜05 +13 / CMD-06 +12 含む）
- [x] cargo clippy 警告ゼロ / cargo fmt --check 準拠
- [x] git tag: `v0.6.0`
- [x] バックエンド全層（IO / BIZ / CMD / MNT）実装完了

> 詳細: [Plans.md](../Plans.md) §Tag Guarantees `v0.6.0` / §Test Count

---

## 12. 第7段階 — Phase 1 UI 基盤構築（進行中）

**実装範囲**: Tailwind CSS 4 + shadcn/ui + TanStack Router + tauri-specta + invoke ラッパ + .env 構成 + デモデータ seed
**入力ドキュメント**: [UI_TECH_STACK.md](UI_TECH_STACK.md) §1〜§7 / [SCREEN_DESIGN.md](SCREEN_DESIGN.md) §1〜§6
**主要 PR**: PR #41 (UI_TECH_STACK 策定 + UI-00/UI-13 追加) / PR #42 (Tailwind + shadcn/ui + Preflight) / Phase B-1 PR #44/#45/#46 (フロント CI 品質基盤) / PR #48 (7-5c invoke ラッパ ADR-004、`c5f3786`) / PR #50 (UI-12 共通レイアウト、`d512d01`) / PR #52 (7-9 + 7-10、`0ed76ca`)

### 12.1 P0 着手前判定事項（完了）

- [x] 7-1 Tailwind CSS 4 導入（`@tailwindcss/vite` プラグイン方式、`src/styles/globals.css` に @theme 定義、stone palette + warm tones、commit `9747545`）
- [x] 7-2 shadcn/ui 初期化（`components.json` + `src/lib/utils.ts` + 18 コンポーネント導入: Button / Input / Label / Dialog / AlertDialog / DropdownMenu / Select / Checkbox / RadioGroup / Tabs / Card / Table / Sonner（Toast）/ Form / Badge / Skeleton / Separator / ScrollArea、commit `abf587c`）
- [x] 7-2.5 Task 0 Preflight（4/4 項目: WSL2 + Tauri GUI 動作 / pre-push hook 挙動 / React 19 + TanStack Router peer dep / doc-consistency-check.sh の docs/research/ 許容、commit `f0cab79`、詳細: [research/preflight-2026-04-20.md](research/preflight-2026-04-20.md)）
- [x] 7-4 P0 ルーティング — TanStack Router v1.168.23 採用（[ADR-001](research/2026-04-20-router-adr.md)、commit `2bd4876`、比較 spike: spike/router-tanstack `5f66acd` / spike/router-react-router `3324891` を remote 保持）
- [x] 7-5a P0 invoke 型定義方式 — tauri-specta v2.0.0-rc.24 採用（[ADR-002](research/2026-04-20-invoke-type-adr.md)、commit `6e892e2`、search_products + get_product の 2 コマンドに適用済、残 43 commands は Phase 2 以降で段階展開）
- [x] 7-5b P0 TanStack Query キャッシュ戦略表 — Phase 1 確定値採用（[ADR-003](research/2026-04-20-query-cache-adr.md)、commit `8d54c3c`、UI_TECH_STACK §2.5 に補強 6 項目）

### 12.2 UI 基盤実装（完了）

- [x] 7-3 UI-12 共通レイアウト — PR #50 マージ完了（`d512d01`、squash）。関数設計 + 実装 + 設計書同期を 1 PR 統合（UI 関数設計テンプレ 2 段階化を初導入、業務ロジックなし版）。サイドバー（4 エリア × 19 項目）+ メイン 2 カラム + ウィンドウタイトル動的更新（`useRouterState` + Tauri `getCurrentWindow().setTitle()` 併用、`core:window:allow-set-title` capability）。Codex Round 1 P2 1 件対応 + Round 2 全 0 マージ可能判定。設計合意書: [archive/plans/2026-04-21-ui-12-design-agreement.md](archive/plans/2026-04-21-ui-12-design-agreement.md)
- [x] 7-5c invoke ラッパ C 案 — PR #48 マージ完了（`c5f3786`、squash）。`src/lib/invoke.ts` 薄ラッパ + `InvokeError` Error 派生クラス + `unwrapResult` / `src/lib/invoke-fallback.ts` `FallbackCommand` literal union（撤去リスト兼用、初期値 `never`）+ `typedInvoke` / `scripts/check-typedinvoke-count.sh` 件数監視 + eslint 境界ルール（barrel 経由抜け道対策込み）+ Phase 1 toy command `greet` 削除 + index route を `search_products` demo に差し替え。Codex Round 1 P2 × 2 + P3 × 1 対応 + Round 2 全 0 判定。撤去期限 = `v0.8.0-ui-daily` タグ gate。設計根拠: [ADR-004](research/2026-04-20-invoke-wrapper-adr.md)。Phase 2 closeout で fallback 実体・件数 CI・eslint 境界ルールは撤去済み
- [x] 7-9 P1 デモデータ seed — PR #52 commit `814dc86`。`src-tauri/src/bin/seed_demo_data.rs` + `src/seed_demo.rs` で 6 部門 × uniform 100 商品 / suppliers 5 / sale_records 300 / inventory_movements 400、`StdRng::seed_from_u64(42)` で決定的、冪等 `ON CONFLICT DO NOTHING` + `--reset` flag、integration test 5 本
- [x] 7-10 P1 環境変数・.env 構成 — PR #52 commit 3-5。[UI_TECH_STACK.md §6.9](UI_TECH_STACK.md) 設計原則新設 + .env ファイル 4 本（.env.example / .env.{development,test,production}）+ `src/vite-env.d.ts` 拡張 + `src/lib/env.ts` + `scripts/check-env-safety.sh` + CI frontend job step + pre-push ④ section。第三者レビュー P1 3 件反映（bypass 経路狭め / subfolder + 大文字対応 / 静的検査限界明示 + keyring 導線）

### 12.3 Phase 1 follow-up

- [x] 7-6 Storybook 導入判断 — **不採用で確定（D-068、2026-08-12）**。コンポーネント 41 個でトリガー条件（10 超）は満たすが、1人運用で閲覧者不在 + RTL / mutation / L3 の品質三段が成立済み + D-030 下の大型 devDep 追加が便益に見合わないため導入しない
- [x] 7-7a Vitest 初期化 — PR #64 (`2b30f43`) で `vitest` / `@testing-library/react` / `user-event` / CI integration 完了
- [ ] 7-7b `@axe-core/react` or hooks accessibility coverage 組込み
- [x] 7-8a P1 Error Boundary 戦略文書化・実装 — UI_TECH_STACK §6.10。router 既定 fallback + root 全画面 fallback の 2 層と共通回復 UI を UI 安全網 batch で適用
- [ ] 7-8b 横断 UI 要素標準化 — Toast / Dialog / EmptyState / ErrorState のテンプレート実装
- [x] 7-8c P1 unsaved changes ガード — `useUnsavedChangesWarning` hook + `isDirty` 連動を対象 6 画面へ適用し、全 Page 分類 sweep を追加
- [x] 7-11 P1 AI/UI 開発 workflow 文書化 — PR #72 (`022b8ae`) で `docs/DEV_WORKFLOW.md` / `docs/code_review.md` / repo-local Skills に統合
- [ ] git tag: `v0.7.0-ui-foundation`（必要なら履歴 tag として扱う。Phase 2 completion gate ではない）

### 12.4 Phase B-1 全体クローズ（フロント CI 品質基盤、完了）

- [x] PR #44 Prettier + editorconfig
- [x] PR #45 ESLint 9 flat config + typescript-eslint strict-type-checked + typecheck 独立化
- [x] PR #46 lefthook pre-commit + CI frontend ジョブ拡張 + npm audit warn-only

> 申し送り Backlog: `tsr.config.*` 一本化 / npm audit 残件 / CI frontend job 名と branch protection rule の整合（Node runtime の一本化は Node 24 toolchain alignment change で対応、[Plans.md](../Plans.md) 参照）

---

## 13. 第8段階 — Phase 2 毎日使う 5 画面（完了 / `v0.8.0-ui-daily` tag 済み）

**実装範囲**: UI-00 ホーム / UI-07 CSV 取込み / UI-09a 日次売上 / UI-06a 在庫照会 / UI-09b 月次売上
**入力ドキュメント**: [SCREEN_DESIGN.md](SCREEN_DESIGN.md) 「毎日使う 5 画面」 / [UI_TECH_STACK.md §7.2](UI_TECH_STACK.md) 後続フェーズ判定事項
**実装プラン (archive)**: [archive/plans/2026-05-09-phase-2-ui-00.md](archive/plans/2026-05-09-phase-2-ui-00.md)（UI-00 ホーム画面、PR #56 squash merge `e6da3d8` 完了 2026-05-09）

### 13.1 着手前条件

- [x] PR-B（DEV_SETUP_CHECKLIST.md 書き直し）マージ完了 — Phase 2 着手の gate（PR #54、merge commit `5b4316f`、2026-05-08）
- [x] Phase 2 着手時の Windows native ビルド移行 — §1.4 + memory `tauri2-linux-ime-limitation.md`。PR #56（UI-00、squash merge `e6da3d8`、2026-05-09）で実施完了。Tooltip fix 2 件（`7e82176` TooltipProvider wrap / `56852f5` aria-disabled パターン）は Windows native 実機検証で発見した bug を fix
- [x] Phase 2 着手前の specta 化対象 commands 精査 — `get_daily_sales` / `list_low_stock` / `list_plu_dirty` / `list_csv_imports` を PR #56 commit `2c1ac37` で specta 化完了 + bindings.ts 再生成（[ADR-004](research/2026-04-20-invoke-wrapper-adr.md) §2.3 / §3）

### 13.2 5 画面実装タスク

- [x] 8-1 UI-00 ホーム画面（REQ-301/302, SP-102-07）— PR #56 (`e6da3d8`)
- [x] 8-2 UI-07 CSV 取込み（REQ-401）— PR #62 (`b8db619`)
  - [x] 8-2a 着手時判定: CSV 取込みフローの状態管理 — `useReducer + discriminated union` 採用（[UI_TECH_STACK §7.2](UI_TECH_STACK.md)）
  - [x] 8-2b 着手時判定: IPC ストリーミング（Tauri channel）採否 — Phase 2 では不採用（[UI_TECH_STACK §7.2](UI_TECH_STACK.md)）
- [x] 8-3 UI-09a 日次売上レポート（REQ-501）— PR #65 (`8c2be51`)
- [x] 8-4 UI-06a 在庫照会（REQ-301/302/303 統合）— PR #67 (`cf89082`) + 高視認性 follow-up PR #74 (`ae0c68f`)
- [x] 8-5 UI-09b 月次売上レポート（REQ-502）— PR #66 (`caf7d57`) + seed / card overflow follow-up PR #70 (`aeeee2a`)

### 13.3 Phase 2 完了時の追加実装 / 判定

- [x] 8-6 ショートカット一覧ダイアログ（Ctrl+/ 対応、グローバル + 画面固有）— PR #58 (`452fe0a`)
- [x] 8-7 ファイルエクスポート後 UX 共通化（成功通知 + エラー処理）— UI-09a/b で `useExportFile` 共通化済み
- [x] 8-0 利用者デモ（必須 gate）: 5 画面を実機で触ってもらい操作フロー合意を確認済み。商品コードは小さいが、他の視認性問題はなし。商品コード readability は表示スケール option follow-up PR #77 で対応済み
- [x] 8-9 Phase 2 完了時判定事項:
  - [x] E2E テスト範囲: Phase 2 tag gate では C（見送り）を採用。Vitest + React Testing Library と Windows native H-6 で代替し、smoke E2E は Phase 3 / Phase 4 の横断 regression 状況で再評価
  - [x] 視覚回帰テスト採否: Phase 2 tag gate では見送り。display-scale option は PR #77 で導入済みだが visual regression gate は追加しない。Storybook / E2E / Phase 3 以降の画面横断 regression 状況で再評価
- [x] git tag: `v0.8.0-ui-daily`（PR #75 closeout merge `f44f99a` に作成・push 済み。typedInvoke 段階撤去は完了済み = `invoke-fallback.ts` 削除 + 件数 CI / eslint ルール撤去）

---

## 14. Phase 3 / Phase 4（画面は実装済み。計画の記録）

> **状態**: Phase 3・4 の画面は実装済み（[SCREEN_DESIGN](SCREEN_DESIGN.md) の画面一覧、`src/routes/`）。下の check の表は着手前の計画の記録で、印は更新していない。v1.0 の release は自動在庫連動の完成まで待つ（D-070）ので、完了と書かない。tag は public repo に無い（[履歴境界](PROJECT_HANDOFF.md#履歴境界public-snapshot)）。

### 14.1 第9段階 — Phase 3 商品管理 + 入出庫画面

**着手前判定**: Phase 2 8-9 では E2E / 視覚回帰を tag gate にしない判断で通過済み（[UI_TECH_STACK §7.2](UI_TECH_STACK.md)）。display-scale option は PR #77 で導入済み。Phase 3 / Phase 4 着手時は、画面横断 regression の状況を踏まえて smoke E2E / visual regression を再評価する。

- [ ] 9-1 UI-01a 商品検索・一覧（REQ-103）
- [ ] 9-2 UI-01b 商品登録・編集（REQ-101, 102）
- [ ] 9-3 UI-01c 商品一括インポート（REQ-104）
- [ ] 9-4 UI-02 入庫記録（REQ-201）
- [ ] 9-5 UI-03 返品・交換（REQ-202）
- [ ] 9-6 UI-04 手動販売出庫（REQ-203）
- [ ] 9-7 UI-05 廃棄・破損（REQ-204）
- [ ] git tag: `v0.9.0-ui-product-inventory`

### 14.2 第10段階 — Phase 4 在庫特殊 + システム管理

**着手前判定**: Q40（障害時の対応）の具体化を UI-13 整合性検証画面とエラーバウンダリ実装と合わせて確定（[Plans.md](../Plans.md) Backlog 参照）。

- [x] 10-1 UI-06b 在庫少一覧（REQ-302 個別）: D-047 により本行の独立画面実装タスクとしては解消。`/stock/low` は作らず UI-06a `status=low_stock` フィルタへの deep-link で対応する。サイドバー nav 変更（`navigation.ts` の `search`/`activeMatch` field）は Public PR #9（squash merge `70dc4d1`）で実装済み、詳細は [archive/plans/2026-07-16-sidebar-pending-links.md](archive/plans/2026-07-16-sidebar-pending-links.md) の scope
- [ ] 10-2 UI-06c 在庫変動履歴（REQ-303）
- [ ] 10-3 UI-08 PLU 書出し（REQ-402）
  - [ ] 10-3a 実機動作確認（CV17 `.txt` 取込み → PCツールSD書込み → SR-S4000設定読み → レジ呼出し）
- [ ] 10-4 UI-10 棚卸し（REQ-205、中断再開 UI が特殊）
  - [ ] 10-4a 着手時判定: IPC ストリーミング（Tauri channel）採否 — 長時間処理の進捗通知方式（[UI_TECH_STACK §7.2](UI_TECH_STACK.md)）
- [ ] 10-5a UI-11a 閾値設定画面
- [ ] 10-5b UI-11b バックアップ画面
- [ ] 10-5c UI-11c 操作ログ画面
- [ ] 10-6 UI-13 在庫整合性検証画面（REQ-904、BIZ-07 との連携。REQ-403 POS 部門別売上照合は別 deferred task）
- [ ] git tag: `v1.0.0`

### 14.3 後続フェーズの追加判定事項

Phase 2〜4 の着手 / 完了タイミングで再評価する判定事項は [UI_TECH_STACK.md §7.2](UI_TECH_STACK.md) に集約。本書ではトリガーのみ列挙する。

| トリガー | 判定対象 | 詳細 |
|---|---|---|
| UI-07 着手時（8-2a/b） | CSV 取込みフロー状態管理 + IPC ストリーミング採否 | [UI_TECH_STACK §7.2](UI_TECH_STACK.md) |
| Phase 2 完了時（8-9） | E2E テスト範囲 / 視覚回帰テスト採否 | [UI_TECH_STACK §7.2](UI_TECH_STACK.md) |
| UI-10 着手時（10-4a） | IPC ストリーミング採否（棚卸し長時間処理） | [UI_TECH_STACK §7.2](UI_TECH_STACK.md) |
| 各 Phase 末（継続） | UI デザイン哲学探索の再訪 | [UI_TECH_STACK §7.3](UI_TECH_STACK.md) |

---

## A.1 退役記録: Docker 完結方針（2026-03-31 採用 → 2026-04-03 退役）

### 経緯

- **2026-03-31**: 案 C「Docker 完結（第1〜第2段階）→ UI 実装フェーズで WSL2 直接（案 A）に移行」を採用
- **2026-04-03**: Docker Desktop WSL Integration の `Engine stopping` 固まりが修復不能となり退役決定。WSL2 直接運用に全面移行
- **退役後**: 開発コマンドは WSL2 上の Ubuntu で直接実行する。`docker compose` 系コマンドは inventory-system では使用しない

### 修復試行のサマリ

[DOCKER_REPAIR_LOG.md](DOCKER_REPAIR_LOG.md) §「試したこと」に 8 件の試行が記録されている。要点のみ再掲:

- socat workaround 残骸削除 / WSL Integration ON-Apply / `wsl --unregister docker-desktop` / 完全クリーンインストール / Resource Saver OFF / 別 distro (Ubuntu-24.04) / 4.59.0 ダウングレードまで全て同一症状（Engine 再起動の無限ハング）で失敗
- バージョン非依存 + distro 非依存と確定。Docker Desktop の VM 層（`docker-desktop` distro / `wsl-bootstrap` / VHDX マウント）の再起動シーケンスが根本原因と推定
- 最終回避策: docker-ce 29.3.1 へ移行することで Windows 全体の Docker 利用は復旧（別プロジェクトのポートフォリオ web アプリ用、memory `docker-portfolio-dependency.md`）
- 本プロジェクトでは Docker 自体不要のため、回避策ではなく WSL2 直接運用への切替を選択

### 退役時の Docker 構成（履歴）

参考として、退役時の Dockerfile と docker-compose.yml を残す。再導入時の参照用。

```dockerfile
FROM rust:1.83-bookworm

# システム依存パッケージ
RUN apt-get update && apt-get install -y \
    sqlite3 \
    libsqlite3-dev \
    pkg-config \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Node.js 20 LTS
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# Rust コンポーネント
RUN rustup component add clippy rustfmt

# 作業ディレクトリ
WORKDIR /workspace

# Rustのビルドキャッシュを永続化するためのボリュームマウントポイント
ENV CARGO_HOME=/cargo-cache
```

```yaml
services:
  dev:
    build: .
    volumes:
      - .:/workspace
      - cargo-cache:/cargo-cache
      - node-modules:/workspace/src-ui/node_modules
    working_dir: /workspace
    stdin_open: true
    tty: true
    command: /bin/bash

volumes:
  cargo-cache:
  node-modules:
```

### 再導入を検討する条件

- Docker Desktop WSL Integration バグの修復（upstream 対応 or 公式回避手順）
- CI（GitHub Actions）等で multi-arch ビルドが必要になった場合は別途検討（インフラ層のみ Docker 化、開発環境は WSL2 直接維持）

### 関連ドキュメント

- [DOCKER_REPAIR_LOG.md](DOCKER_REPAIR_LOG.md) — 退役時の復旧試行ログ
- memory `dev-environment-policy.md` — WSL2 直接開発採用の判断軸
