use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn toolbox_cmd() -> assert_cmd::Command {
    assert_cmd::cargo::cargo_bin_cmd!("toolbox")
}

// --- Default execution (tool version display) ---

#[test]
fn test_default_output_succeeds() {
    toolbox_cmd().assert().success();
}

#[test]
fn test_default_output_with_no_icons() {
    toolbox_cmd().arg("--no-icons").assert().success();
}

#[test]
fn test_default_output_compact() {
    toolbox_cmd().arg("--compact").assert().success();
}

// --- JSON output ---

#[test]
fn test_json_output_is_valid_json() {
    let output = toolbox_cmd()
        .args(["--format", "json"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(stdout.trim());
    assert!(parsed.is_ok(), "Output is not valid JSON: {}", stdout);
}

#[test]
fn test_json_output_has_tools_field() {
    let output = toolbox_cmd()
        .args(["--format", "json"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    assert!(
        parsed.get("tools").is_some(),
        "JSON should have 'tools' field"
    );
}

#[test]
fn test_json_pretty_output_is_valid() {
    let output = toolbox_cmd()
        .args(["--format", "json-pretty"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(stdout.trim());
    assert!(parsed.is_ok(), "Output is not valid JSON: {}", stdout);
}

// --- Powerline output ---

#[test]
fn test_powerline_output_succeeds() {
    toolbox_cmd().arg("--powerline").assert().success();
}

#[test]
fn test_powerline_single_line_succeeds() {
    toolbox_cmd()
        .args(["--powerline", "--single-line"])
        .assert()
        .success();
}

#[test]
fn test_powerline_with_color_always() {
    toolbox_cmd()
        .args(["--powerline", "--color", "always"])
        .assert()
        .success();
}

#[test]
fn test_powerline_with_color_never() {
    toolbox_cmd()
        .args(["--powerline", "--color", "never"])
        .assert()
        .success();
}

// --- Subcommands ---

#[test]
fn test_list_tools_subcommand() {
    toolbox_cmd()
        .arg("list-tools")
        .assert()
        .success()
        .stdout(predicate::str::contains("Available tools"))
        .stdout(predicate::str::contains("Python"));
}

#[test]
fn test_list_tools_shows_enabled_status() {
    let output = toolbox_cmd()
        .arg("list-tools")
        .output()
        .expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("enabled") || stdout.contains("disabled"),
        "list-tools should show enabled/disabled status"
    );
}

#[test]
fn test_show_config_subcommand() {
    // show-config will use default config if no config file exists
    toolbox_cmd().arg("show-config").assert().success();
}

#[test]
fn test_init_subcommand_creates_config() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();
    // Remove the file so init can create it
    std::fs::remove_file(&path).unwrap();

    toolbox_cmd()
        .args(["--config", path.to_str().unwrap(), "init"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created config file"));

    // Verify file was created
    assert!(path.exists(), "Config file should be created");

    // Verify it's valid TOML
    let content = std::fs::read_to_string(&path).unwrap();
    let parsed: Result<toml::Value, _> = toml::from_str(&content);
    assert!(parsed.is_ok(), "Created config should be valid TOML");
}

#[test]
fn test_init_subcommand_refuses_overwrite_without_force() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "existing content").unwrap();
    let path = temp_file.path().to_path_buf();

    toolbox_cmd()
        .args(["--config", path.to_str().unwrap(), "init"])
        .assert()
        .success()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_init_subcommand_force_overwrite() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "old content").unwrap();
    let path = temp_file.path().to_path_buf();

    toolbox_cmd()
        .args(["--config", path.to_str().unwrap(), "init", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created config file"));
}

// --- Custom config file ---

#[test]
fn test_custom_config_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
use_default_tools = false

[display]
show_icons = false
compact = true

[[custom_tools]]
name = "Echo"
command = "echo v1.0.0"
parse_regex = 'v?(\d+\.\d+\.\d+)'
enabled = true

[extras]
git_branch = false
git_status = false
current_directory = false
virtual_env = false
system_memory = false
system_cpu = false
"#
    )
    .unwrap();

    let path = temp_file.path().to_path_buf();

    let output = toolbox_cmd()
        .args(["--config", path.to_str().unwrap(), "--format", "json"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    let tools = parsed["tools"].as_array().unwrap();

    assert_eq!(tools.len(), 1, "Should have exactly one tool");
    assert_eq!(tools[0]["name"], "Echo");
    assert_eq!(tools[0]["version"], "1.0.0");
    assert!(tools[0]["available"].as_bool().unwrap());
}

#[test]
fn test_invalid_config_file_errors() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "invalid toml {{{{").unwrap();
    let path = temp_file.path().to_path_buf();

    toolbox_cmd()
        .args(["--config", path.to_str().unwrap()])
        .assert()
        .failure();
}

// --- --help and --version ---

#[test]
fn test_help_flag() {
    toolbox_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Display development tool versions",
        ));
}

#[test]
fn test_version_flag() {
    toolbox_cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("toolbox"));
}

// --- Working directory ---

#[test]
fn test_working_dir_option() {
    toolbox_cmd()
        .args(["--dir", "/tmp", "--format", "json"])
        .assert()
        .success();
}

// --- Color modes ---

#[test]
fn test_color_auto() {
    toolbox_cmd().args(["--color", "auto"]).assert().success();
}

#[test]
fn test_color_always() {
    toolbox_cmd().args(["--color", "always"]).assert().success();
}

#[test]
fn test_color_never() {
    toolbox_cmd().args(["--color", "never"]).assert().success();
}

// --- Doctor subcommand ---

#[test]
fn test_doctor_subcommand_succeeds() {
    toolbox_cmd()
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Toolbox Doctor"))
        .stdout(predicate::str::contains("Tool Status:"))
        .stdout(predicate::str::contains("tools checked"));
}

#[test]
fn test_doctor_shows_config_info() {
    toolbox_cmd()
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Config:"));
}

#[test]
fn test_doctor_json_output_is_valid() {
    let output = toolbox_cmd()
        .args(["doctor", "--json"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(stdout.trim());
    assert!(
        parsed.is_ok(),
        "Doctor JSON output is not valid: {}",
        stdout
    );

    let value = parsed.unwrap();
    assert!(value.get("total").is_some());
    assert!(value.get("ok_count").is_some());
    assert!(value.get("tools").is_some());
}

#[test]
fn test_doctor_json_has_tool_details() {
    let output = toolbox_cmd()
        .args(["doctor", "--json"])
        .output()
        .expect("failed to execute");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap();
    let tools = parsed["tools"].as_array().unwrap();

    // With default config, there should be tools
    assert!(!tools.is_empty());

    // Each tool should have required fields
    for tool in tools {
        assert!(tool.get("name").is_some());
        assert!(tool.get("status").is_some());
        assert!(tool.get("command").is_some());
        assert!(tool.get("enabled").is_some());
    }
}

#[test]
fn test_doctor_with_custom_config() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
use_default_tools = false

[[custom_tools]]
name = "Echo"
command = "echo v1.0.0"
parse_regex = 'v?(\d+\.\d+\.\d+)'
enabled = true
"#
    )
    .unwrap();

    let path = temp_file.path().to_path_buf();

    toolbox_cmd()
        .args(["--config", path.to_str().unwrap(), "doctor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Echo"))
        .stdout(predicate::str::contains("1 tools checked: 1 ok"));
}

#[test]
fn test_doctor_shows_unavailable_tools() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(
        temp_file,
        r#"
use_default_tools = false

[[custom_tools]]
name = "FakeTool"
command = "nonexistent_fakecmd_12345 --version"
enabled = true
"#
    )
    .unwrap();

    let path = temp_file.path().to_path_buf();

    toolbox_cmd()
        .args(["--config", path.to_str().unwrap(), "doctor"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ERR"))
        .stdout(predicate::str::contains("FakeTool"))
        .stdout(predicate::str::contains(
            "1 tools checked: 0 ok, 0 warning, 1 error",
        ));
}
