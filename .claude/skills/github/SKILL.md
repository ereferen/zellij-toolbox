---
name: github
description: GitHub operations (issues, PRs, repos) with fallback strategy (gh → MCP → curl)
user-invocable: false
---

# GitHub Operations - Fallback Strategy

## Usage

GitHub の issue, PR, リポジトリ操作を行う際に、以下の優先順で手段を選択する。

## Fallback Order

### 1. `gh` コマンド（最優先）

`gh` CLI が使える環境ではこれを使う。

```bash
# 例: issue一覧
gh issue list

# 例: issue取得
gh issue view <number>

# 例: PR一覧
gh pr list

# 例: PR作成
gh pr create --title "title" --body "body"
```

まず `gh --version` で利用可能か確認する。コマンドが見つからない、または認証エラーの場合は次へ。

### 2. MCP GitHub Server

`mcp__github__*` ツールが利用可能な場合はこれを使う。

主なツール:
- `mcp__github__list_issues` - issue一覧
- `mcp__github__get_issue` - issue詳細
- `mcp__github__create_issue` - issue作成
- `mcp__github__list_pull_requests` - PR一覧
- `mcp__github__get_pull_request` - PR詳細
- `mcp__github__create_pull_request` - PR作成
- `mcp__github__search_repositories` - リポジトリ検索

MCP ツールが応答しない、またはエラーの場合は次へ。

### 3. `curl` + GitHub REST API（最終手段）

`gh` も MCP も使えない場合は `curl` で GitHub API を直接叩く。

環境変数 `GITHUB_TOKEN` が設定されている場合は認証ヘッダーを付与する。未設定の場合は認証なしで試行する（rate limit に注意）。

```bash
# リポジトリのオーナー/名前を取得
REPO=$(gh api repos/:owner/:repo --jq '.full_name' 2>/dev/null \
  || git remote get-url origin | sed 's|.*github.com[:/]\(.*\)\.git|\1|')

# 認証ヘッダー（トークンがあれば）
AUTH_HEADER=""
if [ -n "$GITHUB_TOKEN" ]; then
  AUTH_HEADER="Authorization: Bearer $GITHUB_TOKEN"
fi

# issue一覧
curl -s -H "$AUTH_HEADER" \
  "https://api.github.com/repos/${REPO}/issues" | jq '.[] | {number, title, state}'

# 特定のissue取得
curl -s -H "$AUTH_HEADER" \
  "https://api.github.com/repos/${REPO}/issues/<number>" | jq '{number, title, state, body}'

# PR一覧
curl -s -H "$AUTH_HEADER" \
  "https://api.github.com/repos/${REPO}/pulls" | jq '.[] | {number, title, state}'

# issue作成（要認証）
curl -s -X POST -H "$AUTH_HEADER" -H "Content-Type: application/json" \
  "https://api.github.com/repos/${REPO}/issues" \
  -d '{"title": "issue title", "body": "issue body"}'
```

## Fallback の実行フロー

```
1. gh --version を実行
   ├── 成功 → gh コマンドで操作
   └── 失敗 → 2 へ

2. MCP ツール (mcp__github__*) を試行
   ├── 成功 → MCP で操作
   └── 失敗/利用不可 → 3 へ

3. git remote get-url origin からリポジトリ情報を取得
   └── curl + GitHub REST API で操作
       ├── GITHUB_TOKEN あり → 認証付きリクエスト
       └── GITHUB_TOKEN なし → 認証なしリクエスト（読み取りのみ）
```

## Notes

- `curl` フォールバック時の書き込み操作（issue作成、PR作成等）には `GITHUB_TOKEN` が必須
- rate limit: 認証なしは 60 req/hour、認証ありは 5,000 req/hour
- リポジトリ情報は `git remote get-url origin` から自動取得する
