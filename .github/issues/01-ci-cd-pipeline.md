# CI/CD パイプラインの構築

**Labels:** enhancement, infrastructure

## 概要

GitHub Actionsを使ったCI/CDパイプラインを構築し、テスト・ビルド・リリースを自動化する。

## やること

### CI（Pull Request / Push）
- [ ] `cargo test` の自動実行
- [ ] `cargo fmt --check` によるフォーマットチェック
- [ ] `cargo clippy -- -D warnings` によるリントチェック
- [ ] `cargo build -p toolbox-cli` のビルド確認
- [ ] `cargo build -p toolbox-zellij --target wasm32-wasip1` のWASMビルド確認

### CD（Release）
- [ ] GitタグをトリガーにしたWASMプラグインのリリース自動化
- [ ] CLI バイナリのクロスコンパイルとリリース（Linux/macOS）
- [ ] GitHub Releases へのアーティファクト添付

### その他
- [ ] Dependabot による依存関係更新の自動化
- [ ] キャッシュ設定（Cargo registry, target directory）

## ファイル構成案

```
.github/
├── workflows/
│   ├── ci.yml       # テスト・リント（PR/push時）
│   └── release.yml  # リリース（タグ時）
└── dependabot.yml
```
