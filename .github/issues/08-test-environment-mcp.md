# テスト環境の整備（MCP・スナップショットテスト・統合テスト）

**Labels:** enhancement, testing, infrastructure

## 概要

動作確認や見た目のテストを効率的に行えるテスト環境を整備する。MCP（Model Context Protocol）サーバーの活用、スナップショットテスト、統合テスト等を含む。

## 1. MCP テスト環境

### 目的
Claude Code からの開発時にツールの動作確認や出力のプレビューを容易にする。

### やること
- [ ] `.claude/mcp.json` の修正・拡充
  - filesystem MCPサーバーのパス修正（現在typoあり → 修正済み）
  - GitHub MCP サーバーの設定確認
- [ ] テスト実行用のMCPサーバー追加検討
  - `@anthropic-ai/mcp-server-filesystem` でビルド出力の確認
  - カスタムMCPサーバーでCLI出力のプレビュー
- [ ] Claude Code スキルの拡充
  - `/preview` スキル: CLI出力のプレビュー（text/powerline/json各形式）
  - `/test-visual` スキル: Powerline出力のスクリーンショット比較

### MCP サーバー設定案

```json
{
  "mcpServers": {
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": { "GITHUB_PERSONAL_ACCESS_TOKEN": "${GITHUB_TOKEN}" }
    },
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@anthropic-ai/mcp-server-filesystem", "."]
    },
    "shell": {
      "command": "npx",
      "args": ["-y", "@anthropic-ai/mcp-server-shell-exec"]
    }
  }
}
```

## 2. スナップショットテスト（出力の見た目テスト）

### 目的
CLI出力やPowerline表示のレグレッションを自動検出する。

### やること
- [ ] `insta` クレートの導入（Rustスナップショットテスト）
- [ ] 各出力形式のスナップショット
  - テキスト出力（通常・コンパクト・アイコンなし）
  - Powerline出力（シングルライン・マルチライン）
  - JSON出力
- [ ] ANSIカラー出力のスナップショット
- [ ] `cargo insta review` でのスナップショット確認ワークフロー

### テスト例

```rust
#[test]
fn test_powerline_output_snapshot() {
    let info = create_test_toolbox_info();
    let output = info.format_powerline(true, false);
    insta::assert_snapshot!(output);
}

#[test]
fn test_text_output_snapshot() {
    let info = create_test_toolbox_info();
    let output = info.format_display(true, true);
    insta::assert_snapshot!(output);
}
```

## 3. 統合テスト

### 目的
CLI バイナリのエンドツーエンドテストを行う。

### やること
- [ ] `assert_cmd` + `predicates` クレートの導入
- [ ] CLI サブコマンドの統合テスト
  - `toolbox` (デフォルト出力)
  - `toolbox --format json`
  - `toolbox --powerline`
  - `toolbox init`
  - `toolbox show-config`
  - `toolbox list-tools`
- [ ] 設定ファイルとの統合テスト（tempdir使用）
- [ ] エラーケースのテスト（不正な設定ファイル等）

### テスト例

```rust
use assert_cmd::Command;

#[test]
fn test_cli_json_output() {
    Command::cargo_bin("toolbox")
        .unwrap()
        .arg("--format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicates::str::is_json());
}

#[test]
fn test_cli_list_tools() {
    Command::cargo_bin("toolbox")
        .unwrap()
        .arg("list-tools")
        .assert()
        .success()
        .stdout(predicates::str::contains("Python"));
}
```

## 4. Zellijプラグインのテスト

### 目的
WASMプラグインの動作をモックベースでテストする。

### やること
- [ ] Zellij Plugin APIのモック作成
- [ ] プラグインのライフサイクルテスト（load → update → render）
- [ ] コマンド実行結果のパース処理テスト
- [ ] Unicode幅計算の境界値テスト

## 5. テストフィクスチャ

### やること
- [ ] テスト用設定ファイルの整備（`tests/fixtures/`）
- [ ] 各種ツール出力のモックデータ
- [ ] Git リポジトリのテストフィクスチャ
- [ ] テストヘルパー関数の共通化

### ディレクトリ構成案

```
tests/
├── fixtures/
│   ├── config/
│   │   ├── default.toml
│   │   ├── custom_tools.toml
│   │   └── invalid.toml
│   └── tool_outputs/
│       ├── python_version.txt
│       ├── node_version.txt
│       └── ...
├── integration/
│   ├── cli_test.rs
│   └── config_test.rs
└── snapshots/
    ├── text_output.snap
    ├── powerline_output.snap
    └── json_output.snap
```

## 追加する依存関係

```toml
[dev-dependencies]
insta = { version = "1.34", features = ["yaml"] }
assert_cmd = "2.0"
predicates = "3.0"
```

## 関連ファイル

- `toolbox-core/Cargo.toml`
- `toolbox-cli/Cargo.toml`
- `.claude/mcp.json`
- `.claude/skills/`
