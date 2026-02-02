# ツールバージョン検出のキャッシング

**Labels:** enhancement, performance

## 概要

ツールバージョンの検出結果をキャッシュし、不要な再検出を避けてパフォーマンスを向上させる。

## 背景

現在は毎回全ツールのバージョンコマンドを実行している。多くのツールのバージョンは頻繁に変わらないため、キャッシュで改善可能。

## やること

- [ ] バージョン検出結果のインメモリキャッシュ実装
- [ ] TTL（Time-To-Live）設定のサポート（ツールごと）
- [ ] ディレクトリ変更時のキャッシュ無効化
- [ ] `--no-cache` / `--refresh` オプションの追加
- [ ] キャッシュヒット率のログ出力（デバッグ用）

## 設計メモ

```rust
struct VersionCache {
    entries: HashMap<String, CacheEntry>,
}

struct CacheEntry {
    version: String,
    detected_at: Instant,
    working_dir: Option<PathBuf>,
    ttl: Duration,
}
```

## 関連ファイル

- `toolbox-core/src/detector.rs`
- `toolbox-core/src/config.rs`
