# カラーテーマのカスタマイズ対応

**Labels:** enhancement

## 概要

Powerline出力のカラースキームを設定ファイルからカスタマイズできるようにする。現在はハードコードされた固定カラーのみ対応。

## 現状

`toolbox-core/src/color.rs` でセグメント色がハードコードされている：
- ディレクトリ: Blue
- Git（クリーン）: Green
- Git（ダーティ）: Yellow
- ツール: Cyan / Magenta / Gray

## やること

- [ ] `config.toml` に `[theme]` セクションを追加
- [ ] セグメントごとの前景色・背景色の設定
- [ ] プリセットテーマ（dark, light, solarized 等）の提供
- [ ] 256色/TrueColor対応の検討

## 設定例

```toml
[theme]
preset = "dark"  # or "light", "solarized", "custom"

[theme.custom]
directory_bg = "#3465A4"
git_clean_bg = "#4E9A06"
git_dirty_bg = "#C4A000"
tool_bg = ["#06989A", "#75507B", "#555753"]
```

## 関連ファイル

- `toolbox-core/src/color.rs`
- `toolbox-core/src/config.rs`
- `toolbox-core/src/info.rs`
