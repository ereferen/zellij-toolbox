# エラーハンドリングと診断機能の改善

**Labels:** enhancement

## 概要

ツール検出の失敗理由をよりわかりやすく表示し、トラブルシューティングを容易にする。

## やること

- [ ] `toolbox doctor` サブコマンドの追加（環境診断）
  - PATH の確認
  - 各ツールの検出可否と詳細エラー
  - 設定ファイルのバリデーション
- [ ] `--verbose` / `--debug` フラグの追加
- [ ] ツール検出失敗時のエラーメッセージ改善
  - コマンドが見つからない → PATHの確認を提案
  - パース失敗 → 実際の出力とregexの不一致を表示
- [ ] ログレベル設定（error/warn/info/debug）

## 設計例

```bash
$ toolbox doctor
✅ Python 3.12.1 (/usr/bin/python3)
✅ Node 20.11.0 (/home/user/.asdf/shims/node)
❌ Ruby - command not found (ruby not in PATH)
⚠️  Java - parse error (expected "openjdk ...", got "java 21.0.1 2023-10-17")
```

## 関連ファイル

- `toolbox-cli/src/main.rs`
- `toolbox-core/src/detector.rs`
- `toolbox-core/src/error.rs`
