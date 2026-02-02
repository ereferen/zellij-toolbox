# 設定ファイルのホットリロード

**Labels:** enhancement

## 概要

設定ファイルの変更を自動検出し、再起動なしで反映する。

## やること

- [ ] 設定ファイルの変更監視（タイムスタンプまたはnotify）
- [ ] Zellijプラグインでの設定再読み込み
- [ ] CLIでの `--watch` モード（設定変更時に自動再表示）
- [ ] 設定変更時のバリデーションとエラーレポート

## 設計メモ

WASM環境では `notify` クレートが使えないため、タイマーイベント内でファイルの更新日時を比較する方式が現実的。

```rust
// Zellijプラグインでのアプローチ
fn check_config_update(&mut self) {
    let current_mtime = fs::metadata(&self.config_path)
        .and_then(|m| m.modified()).ok();
    if current_mtime != self.last_config_mtime {
        self.reload_config();
        self.last_config_mtime = current_mtime;
    }
}
```

## 関連ファイル

- `toolbox-core/src/config.rs`
- `toolbox-zellij/src/main.rs`
