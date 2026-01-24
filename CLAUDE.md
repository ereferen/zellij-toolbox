# Toolbox - Development Tool Version Display

Zellijプラグインとして、システムにインストールされた開発ツールのバージョンを表示するツール。

## プロジェクト構造

```
toolbox/
├── Cargo.toml              # Workspace root
├── CLAUDE.md               # このファイル
├── toolbox-core/           # コアライブラリ
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs          # エントリポイント
│       ├── config.rs       # 設定管理
│       ├── detector.rs     # ツール検出ロジック
│       ├── error.rs        # エラー型
│       └── info.rs         # 情報構造体
├── toolbox-cli/            # CLIツール
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
└── toolbox-zellij/         # Zellijプラグイン
    ├── Cargo.toml
    └── src/
        └── lib.rs
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

- `Config`: TOML設定ファイルの読み書き
- `ToolDetector`: ツールバージョン検出のメインロジック
- `ToolInfo`, `GitInfo`, `SystemInfo`: 情報を格納する構造体

Features:
- `git`: git2による Git情報取得（デフォルト有効）
- `sysinfo`: システム情報取得（デフォルト有効）
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

### toolbox-zellij

ZellijのWASMプラグイン。CLIを呼び出して結果を表示する。

## TODO

- [ ] ahead/behind の取得（リモートとの差分）
- [ ] pane の working directory 取得と自動更新
- [ ] 定期的な自動更新
- [ ] カスタムツール定義の拡張
- [ ] カラー出力対応

## 注意事項

- Zellijプラグインはwasm32-wasip1ターゲットでビルドする必要がある
- WASMからは直接コマンド実行できないため、CLIツールと連携する
- asdf/mise等のディレクトリ別バージョンに対応するには`--dir`オプションを使う
