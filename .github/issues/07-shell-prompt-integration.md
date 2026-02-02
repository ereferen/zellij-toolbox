# シェルプロンプト統合

**Labels:** enhancement

## 概要

toolboxの出力をシェルプロンプト（bash/zsh/fish）に統合できるようにする。starship的な使い方を可能にする。

## やること

- [ ] `toolbox prompt` サブコマンドの追加
- [ ] プロンプト用の軽量出力モード（最小限のツール情報のみ）
- [ ] シェルごとの統合スクリプト生成
  - bash: `PS1` 用
  - zsh: `PROMPT` / `RPROMPT` 用
  - fish: `fish_prompt` 用
- [ ] `toolbox init bash/zsh/fish` による設定スニペット生成

## 設計例

```bash
# .bashrc に追加
eval "$(toolbox init bash)"

# または手動設定
export PS1='$(toolbox prompt --single-line) \$ '
```

## 注意事項

- プロンプト表示は高速でなければならない（100ms以下が理想）
- キャッシング機能（#04）との組み合わせが重要
- starship等の既存ツールとの差別化を意識する

## 関連ファイル

- `toolbox-cli/src/main.rs`（新サブコマンド追加）
