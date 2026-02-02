# /preview - Preview toolbox CLI output

Preview the toolbox CLI output in various formats.

## Usage

- `/preview` - Preview all output formats
- `/preview text` - Preview text output
- `/preview powerline` - Preview powerline output
- `/preview json` - Preview JSON output

## Instructions

When the user runs this skill:

1. First ensure the release binary exists, if not build it:
   ```bash
   cargo build --release -p toolbox-cli
   ```
2. Run the CLI in each format and display the output:
   - Text: `./target/release/toolbox`
   - Compact: `./target/release/toolbox --compact`
   - Powerline: `./target/release/toolbox --powerline --color always`
   - Powerline single-line: `./target/release/toolbox --powerline --single-line --color always`
   - JSON: `./target/release/toolbox --format json-pretty`
3. Display each output with a heading indicating the format
4. If specific format is requested, only show that format
