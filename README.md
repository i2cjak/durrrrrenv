# durrrrrenv

A zsh alternative to direnv that automatically loads environment configurations when you enter a directory.

## Features

- **Transparent zsh integration** - Hooks into zsh to automatically check for `.local_environment` files
- **Smart parent directory search** - Automatically finds and loads parent directory environments when you cd deep into a project (limited to 5 levels for performance)
- **Blazing fast** - Sub-millisecond directory searches with zero-allocation fast path
- **Automatic cleanup** - Deactivates environments when you leave the directory
- **Security-first** - Requires explicit confirmation before executing any environment file
- **File integrity checking** - Detects when `.local_environment` files change and re-prompts for approval
- **Simple syntax** - Easy-to-understand commands for common tasks
- **Performance monitoring** - Built-in benchmark and verbose modes to measure overhead

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

#### `durrrrrenv bench`
Benchmark the performance of directory search operations.

```bash
durrrrrenv bench                # Run 1000 iterations (default)
durrrrrenv bench -n 10000       # Run 10000 iterations
```

Output shows average time per search, searches per second, and search depth statistics.

#### `durrrrrenv check --verbose`
Check with detailed performance metrics.

```bash
durrrrrenv check --verbose
```

Shows search time, depth found, and total execution time.

## Performance

durrrrrenv is designed for minimal overhead on every directory change:

- **Fast parent directory search**: Limited to 5 levels up (configurable in code via `MAX_SEARCH_DEPTH`)
- **Zero-allocation fast path**: Uses `Path` references instead of cloning PathBufs during search
- **Early termination**: Stops immediately when `.local_environment` is found or max depth reached
- **Optimized zsh hook**: Uses pure zsh built-ins (no external grep/sed/awk processes)
- **Typical performance**: Sub-millisecond search times on modern systems

**Hook Optimizations:**
- Zero external process spawns (no grep, head, cut, etc.)
- Pure zsh pattern matching with `[[ ]]` and parameter expansion
- Line-by-line processing using zsh array flags `${(@f)output}`
- Early returns to avoid unnecessary processing

Use `durrrrrenv bench` to measure performance on your system.

## How It Works

1. When you `cd` into a directory, the zsh hook runs `durrrrrenv check`
2. If leaving a directory with an active environment:
   - Python venv is deactivated automatically
   - Environment is cleaned up
3. `durrrrrenv check` searches for a `.local_environment` file:
   - First checks the current directory
   - If not found, searches up the directory tree to find the nearest parent with a `.local_environment` file
   - This means you can `cd` directly into `my-project/src/lib/utils/` and it will find and load `my-project/.local_environment`
4. If a `.local_environment` file is found:
   - If it's allowed and hasn't changed: commands are executed
   - If it's not allowed or has changed: you're prompted to allow it
5. Allowed directories are tracked in `~/.config/durrrrrenv/allowed.json`
6. File contents are hashed to detect changes
7. Subdirectories inherit the parent's environment (no deactivation when entering subdirectories)

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

# Parent directory search - cd directly into deep subdirectory
cd /tmp
mkdir -p my-project/src/lib/utils
cd my-project/src/lib/utils
# (.venv) is activated! Searched up and found /tmp/my-project/.local_environment
```

---

<p align="center">
  <img src="logo.png" alt="American Embedded Logo" width="200"/>
</p>

<p align="center">
  <strong>PROVIDED BY AMERICAN EMBEDDED</strong><br/>
  Hardtech Consulting
</p>

<p align="center">
  Specialists in low power IoT design, wearable applications, high-speed applications, and RF/SDR solutions.<br/>
  From concept to production, we build the hardware that matters.
</p>

<p align="center">
  <a href="mailto:build@amemb.com">build@amemb.com</a>
</p>

---

## License

MIT
