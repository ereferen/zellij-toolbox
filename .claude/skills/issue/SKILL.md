---
name: issue
description: "GitHub Issue ワークフロー: issue取得 → ブランチ作成 → 実装 → テスト → コミット → PR作成"
argument-hint: "<number>|list|status"
---

# /issue - GitHub Issue ワークフロー

GitHub Issue を取得し、ブランチ作成 → 実装 → テスト → コミット → PR作成 の一連の流れを実行する。

リポジトリ情報: owner=ereferen, repo=zellij-toolbox (git remote origin から取得)

## Usage

- `/issue <number>` - 指定されたissue番号の作業を開始
- `/issue list` - オープンissueの一覧を表示
- `/issue status` - 現在のブランチに関連するissueの状態を表示

## Instructions

### `/issue list` の場合

1. `mcp__github__list_issues` で owner=ereferen, repo=zellij-toolbox のオープンissue一覧を取得
2. 番号、タイトル、ラベルを見やすく表示

### `/issue status` の場合

1. 現在のブランチ名から issue 番号を推定
2. issue の状態を取得して表示
3. 関連する PR があればその状態も表示

### `/issue <number>` の場合

以下のワークフローを順番に実行する。各ステップで進捗を報告し、問題があればユーザーに確認する。

#### 1. Issue の取得と分析

1. `mcp__github__get_issue` で issue を取得 (owner=ereferen, repo=zellij-toolbox)
2. issue の内容を分析し、作業内容をユーザーに提示
3. 実装方針をユーザーに確認（必要に応じて EnterPlanMode を使用）

#### 2. ブランチ作成

1. main ブランチが最新か確認: `git fetch origin main`
2. ブランチ名を決定: `feat/issue-<number>-<short-description>` or `fix/issue-<number>-<short-description>`
   - issue のラベルに "bug" があれば `fix/`、それ以外は `feat/`
3. main から新しいブランチを作成: `git checkout -b <branch-name> origin/main`

#### 3. 実装

1. issue の要件に基づいてコードを実装
2. CLAUDE.md のルールに従う（テスト必須、フォーマット、リント）
3. 実装中は進捗をユーザーに報告

#### 4. テスト・検証

1. `cargo fmt --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace`
4. 全てパスすることを確認。失敗した場合は修正してリトライ

#### 5. コミット

1. 変更ファイルを確認 (`git status`, `git diff`)
2. コミットメッセージのフォーマット:
   - `feat: <description> (#<issue-number>)`
   - `fix: <description> (#<issue-number>)`
   - `test: <description> (#<issue-number>)`
3. `Co-Authored-By: Claude <model> <noreply@anthropic.com>` を含める
4. `git add` で関連ファイルをステージング（`.claude/settings.local.json` は除外）
5. `git commit`

#### 6. PR 作成

1. `git push -u origin <branch-name>`
2. PR を作成:
   - title: コミットメッセージと同じ
   - body: `Closes #<issue-number>` を含める
   - base: main
3. PR の URL をユーザーに報告

## Notes

- コミット前に必ず全テスト（fmt, clippy, test）をパスさせること
- CLAUDE.md のテストルールに従うこと
- 実装方針が不明確な場合はユーザーに確認すること
- `.claude/settings.local.json` は絶対にコミットしない
