# durrrrrenv

A zsh alternative to direnv that automatically loads environment configurations when you enter a directory.

## Features

- **Transparent zsh integration** - Hooks into zsh to automatically check for `.local_environment` files
- **Automatic cleanup** - Deactivates environments when you leave the directory
- **Security-first** - Requires explicit confirmation before executing any environment file
- **File integrity checking** - Detects when `.local_environment` files change and re-prompts for approval
- **Simple syntax** - Easy-to-understand commands for common tasks

## Installation

### Build from source

```bash
cargo build --release
sudo cp target/release/durrrrrenv /usr/local/bin/
```

### Set up zsh integration

Add this to your `~/.zshrc`:

```bash
eval "$(durrrrrenv hook)"
```

## Usage

### Creating a .local_environment file

Create a `.local_environment` file in your project directory with commands to execute:

```bash
# Example .local_environment
source ~/scripts/setup.sh
python_venv .venv
source <(west completion zsh)
```

### Supported Commands

#### `source <file>`
Source a shell script file.

```bash
source ~/.bashrc
source ./scripts/setup.sh
```

#### `python_venv [path]`
Activate a Python virtual environment. Defaults to `.venv` if no path is provided.

```bash
python_venv           # Uses .venv
python_venv venv      # Uses venv directory
```

#### `source <(command)`
Process substitution - execute a command and source its output.

```bash
source <(west completion zsh)
source <(kubectl completion zsh)
```

### Allowing a directory

When you `cd` into a directory with a `.local_environment` file for the first time, you'll see:

```
durrrrrenv: .local_environment file found but not allowed
durrrrrenv: Run 'durrrrrenv allow' to allow it
durrrrrenv: File contents:
---
source ~/setup.sh
python_venv .venv
---
```

To allow the file and execute it immediately:

```bash
eval "$(durrrrrenv allow)"
```

You'll be prompted to confirm:

```
Contents of .local_environment:
---
source ~/setup.sh
python_venv .venv
---
Allow this file to be executed? [y/N]: y
Allowed .local_environment in /home/user/project
```

The environment will be loaded immediately after you confirm. If you don't want to execute it immediately, just run `durrrrrenv allow` without the `eval` wrapper.

### CLI Commands

#### `durrrrrenv check`
Check if the current directory has an allowed `.local_environment` file and execute it. This is called automatically by the zsh hook.

```bash
durrrrrenv check
```

#### `durrrrrenv allow`
Allow the `.local_environment` file in the current directory. After allowing, it outputs the shell script to stdout, which you can execute immediately with `eval`.

```bash
eval "$(durrrrrenv allow)"  # Allow and execute immediately
# or
durrrrrenv allow            # Just allow without executing
```

#### `durrrrrenv deny`
Remove permission for the `.local_environment` file in the current directory.

```bash
durrrrrenv deny
```

#### `durrrrrenv status`
Show the status of the current directory's `.local_environment` file.

```bash
durrrrrenv status
```

Output:
```
Directory: /home/user/project
Status: Allowed

Commands to execute:
  Source { path: "~/setup.sh" }
  PythonVenv { path: ".venv" }
```

#### `durrrrrenv hook`
Output the zsh hook script (used in `eval "$(durrrrrenv hook)"`).

## How It Works

1. When you `cd` into a directory, the zsh hook runs `durrrrrenv check`
2. If leaving a directory with an active environment:
   - Python venv is deactivated automatically
   - Environment is cleaned up
3. If a `.local_environment` file exists in the new directory:
   - If it's allowed and hasn't changed: commands are executed
   - If it's not allowed or has changed: you're prompted to allow it
4. Allowed directories are tracked in `~/.config/durrrrrenv/allowed.json`
5. File contents are hashed to detect changes
6. Subdirectories inherit the parent's environment (no deactivation when entering subdirectories)

## Security

- **Explicit approval required** - No environment file is executed without your confirmation
- **Change detection** - If a `.local_environment` file changes after being allowed, you'll be prompted again
- **Transparent** - Always shows you what will be executed before asking for approval

## Configuration

Allowed directories are stored in: `~/.config/durrrrrenv/allowed.json`

This file contains:
- Directory hashes (for privacy)
- Canonical paths
- File content hashes
- Timestamps

## Example Workflow

```bash
# Create a new project
mkdir my-project
cd my-project

# Create a .local_environment file
cat > .local_environment << 'EOF'
python_venv .venv
source <(west completion zsh)
EOF

# On next cd, you'll be prompted
cd ..
cd my-project
# durrrrrenv: .local_environment file found but not allowed
# durrrrrenv: Run 'durrrrrenv allow' to allow it

# Allow it and execute immediately
eval "$(durrrrrenv allow)"
# Allow this file to be executed? [y/N]: y
# Allowed .local_environment in /home/user/my-project
# (.venv) is now activated and west completion is loaded immediately!

# It also auto-loads on cd
cd ..
# (.venv) is automatically deactivated when leaving!

cd my-project
# (.venv) is activated again

# Subdirectories keep the environment active
cd subdir
# (.venv) still active

cd ../..
# (.venv) deactivated when fully leaving the project

cd my-project
# Check status anytime
durrrrrenv status
# Directory: /home/user/my-project
# Status: Allowed
```

## License

MIT
