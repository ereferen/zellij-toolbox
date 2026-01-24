# /run - Run the toolbox CLI

Run the toolbox CLI with various options.

## Usage

- `/run` - Run with default settings
- `/run json` - Output in JSON format
- `/run --dir /path/to/dir` - Run with specific directory

## Instructions

When the user runs this skill:

1. First ensure the release binary exists, if not build it:
   ```bash
   cargo build --release -p toolbox-cli
   ```
2. Run the CLI:
   - Default: `./target/release/toolbox`
   - With json: `./target/release/toolbox --format json-pretty`
   - Pass through any additional arguments from the user
3. Display the output
