# Toolbox - Development Tool Version Display

Zellijプラグインとして、システムにインストールされた開発ツールのバージョンを表示するツール。

## プロジェクト構造

```
toolbox/
├── Cargo.toml              # Workspace root
├── CLAUDE.md               # このファイル
├── README.md               # ユーザー向けドキュメント
├── .devcontainer/          # VS Code DevContainer設定
├── .claude/                # Claude Code開発設定
│   ├── settings.json       # 権限設定
│   ├── mcp.json            # MCP サーバー設定
│   └── skills/             # カスタムスキル (/build, /test, /run, /check, /add-tool)
├── toolbox-core/           # コアライブラリ
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # エントリポイント（公開API）
│       ├── config.rs       # 設定管理（24ツールのデフォルト定義含む）
│       ├── detector.rs     # ツール検出ロジック
│       ├── error.rs        # エラー型
│       ├── info.rs         # 情報構造体と表示フォーマット
│       └── color.rs        # ANSIカラーとPowerlineレンダリング
├── toolbox-cli/            # CLIツール
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
└── toolbox-zellij/         # Zellijプラグイン
    ├── Cargo.toml
    └── src/
        └── main.rs         # WASMプラグイン（Unicode幅・ANSI対応）
```

## ビルド方法

### CLIツール

```bash
cargo build --release -p toolbox-cli
# バイナリ: target/release/toolbox
```

### Zellijプラグイン

```bash
# WASM targetが必要
rustup target add wasm32-wasip1

# ビルド
cargo build --release -p toolbox-zellij --target wasm32-wasip1
# 出力: target/wasm32-wasip1/release/toolbox_zellij.wasm
```

## 開発コマンド

```bash
# 全体ビルド（CLIのみ）
cargo build

# テスト
cargo test

# フォーマット
cargo fmt

# リント
cargo clippy
```

## 設定ファイル

設定は `~/.config/toolbox/config.toml` に保存される。

```bash
# デフォルト設定を生成
toolbox init

# 設定を表示
toolbox show-config

# 利用可能なツール一覧
toolbox list-tools
```

## アーキテクチャ

### toolbox-core

- `Config`: TOML設定ファイルの読み書き（24ツールのデフォルト定義、カスタムツール追加、オーバーライド対応）
- `ToolDetector`: ツールバージョン検出のメインロジック（asdf/mise対応、Git ahead/behind追跡）
- `ToolInfo`, `GitInfo`, `SystemInfo`: 情報を格納する構造体
- Powerlineスタイルの表示フォーマット（シングルライン・マルチライン）
- ANSIカラー出力（auto/always/never切替）

Features:
- `default`: `git` + `sysinfo`（デフォルト有効）
- `git`: git2による Git情報取得（ブランチ、ステータス、ahead/behind）
- `sysinfo`: システム情報取得（メモリ、CPU使用率）
- `wasm`: WASM向けビルド（上記を無効化）

### toolbox-cli

CLIインターフェース。clap使用。

サブコマンド:
- `init`: 設定ファイル生成
- `show-config`: 現在の設定を表示
- `list-tools`: 利用可能なツール一覧

オプション:
- `-c, --config`: 設定ファイルパス
- `-d, --dir`: 作業ディレクトリ（asdf等のため）
- `-f, --format`: 出力形式（text/json/json-pretty）
- `--compact`: コンパクト表示
- `--no-icons`: アイコン非表示
- `--powerline`: Powerlineスタイル出力
- `--single-line`: シングルライン表示（powerline使用時）
- `--color`: カラーモード（auto/always/never）

### toolbox-zellij

ZellijのWASMプラグイン。CLIを呼び出して結果を表示する。

- タイマーイベントによる自動更新（デフォルト5秒間隔）
- `run_command()`経由でCLIを呼び出し
- Unicode文字幅の正確な計算（`unicode-width`クレート使用）
- ANSIエスケープシーケンスのスキップ処理
- シングルライン／マルチライン表示モード

## 実装済み機能

- [x] ahead/behind の取得（リモートとの差分） - `git2`の`Branch::upstream()`で実装
- [x] 定期的な自動更新 - Zellijプラグインのタイマーイベントで実装（デフォルト5秒）
- [x] カスタムツール定義の拡張 - カスタムツール追加、オーバーライド対応
- [x] カラー出力対応 - Powerlineスタイル、ANSIカラー（auto/always/never）
- [x] マルチライン／シングルラインPowerline表示
- [x] Unicode文字幅の正確な計算
- [x] ANSIエスケープシーケンスの幅計算スキップ
- [x] 24ツールのデフォルト定義
- [x] 仮想環境検出（Python venv, Conda）
- [x] DevContainer設定

## TODO

- [ ] pane の working directory 取得と自動更新（Zellijイベントから自動取得）
- [ ] CI/CD パイプライン（テスト・ビルド・リリース自動化）
- [ ] テスト環境の整備（MCP、スナップショットテスト等）
- [ ] Zellijプラグインの統合テスト
- [ ] カラーテーマのカスタマイズ対応
- [ ] ドキュメントの多言語化（日本語・英語）

## 注意事項

- Zellijプラグインはwasm32-wasip1ターゲットでビルドする必要がある
- WASMからは直接コマンド実行できないため、CLIツールと連携する
- asdf/mise等のディレクトリ別バージョンに対応するには`--dir`オプションを使う
