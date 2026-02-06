---
name: sync
description: "GitHub Issueの状態と実装状況を同期する。マージ済みPRでクローズされるべきissueを特定し、CLAUDE.mdのTODOリストも更新する"
---

# /sync - GitHub Issue 同期

GitHub Issue のオープン/クローズ状態と、実際の実装状況を同期させる。

## Instructions

以下の手順で同期を行い、結果をテーブル形式で報告する。

### 1. 情報収集

以下を並列で取得する:

1. **オープンissue一覧**: `mcp__github__list_issues` (owner=ereferen, repo=zellij-toolbox, state=open)
2. **クローズ済みissue一覧**: `mcp__github__list_issues` (owner=ereferen, repo=zellij-toolbox, state=closed)
3. **マージ済みPR一覧**: `mcp__github__list_pull_requests` (owner=ereferen, repo=zellij-toolbox, state=closed) → merged_at が non-null のもの
4. **オープンPR一覧**: `mcp__github__list_pull_requests` (owner=ereferen, repo=zellij-toolbox, state=open)
5. **CLAUDE.md の TODO セクション**: ファイルを読んで実装済み `[x]` / 未実装 `[ ]` を確認

### 2. 同期判定

各オープンissueについて以下を判定:

- **マージ済みPRが存在**: PR の body に `Closes #N` / `Fixes #N` / `(#N)` が含まれている → **クローズ対象**
- **CLAUDE.md で `[x]` 済み**: issue の内容が CLAUDE.md の実装済み項目に対応している → **クローズ候補**（ユーザーに確認）
- **オープンPRが存在**: 作業中 → **進行中**として報告
- **上記いずれでもない**: **未着手**として報告

### 3. 同期実行

1. マージ済みPRでクローズされるべきissueを `mcp__github__update_issue` で state=closed にする
   - コメントも追加: "Closed by PR #N"
2. CLAUDE.md の TODO リストで状態が食い違っている項目を更新

### 4. 結果報告

テーブル形式で報告:

```
| # | タイトル | 状態 | アクション |
|---|---------|------|-----------|
| #15 | カラーテーマ対応 | PR #23 マージ済み | → Closed |
| #17 | キャッシング | PR #22 マージ済み | → Closed (既にクローズ) |
| #18 | エラーハンドリング | 未着手 | - |
```

## Notes

- クローズ対象が見つかった場合、実行前にユーザーに確認する
- CLAUDE.md の変更は最小限にとどめる（TODO のチェックマーク更新のみ）
- `#15` のように issue body 内のチェックリスト `[x]`/`[ ]` も参考にする
