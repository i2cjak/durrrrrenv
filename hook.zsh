#!/usr/bin/env zsh
# durrrrrenv - zsh hook for automatic environment loading
#
# Add this to your .zshrc:
#   eval "$(durrrrrenv hook)"

# Track the last directory to avoid repeated checks
typeset -g _DURRRRRENV_LAST_DIR=""
# Track the directory where we have an active environment loaded
typeset -g _DURRRRRENV_ACTIVE_DIR=""

# Function to unload environment from a directory
_durrrrrenv_unload() {
    # Deactivate Python venv if active
    if typeset -f deactivate > /dev/null; then
        deactivate
    fi
}

# Function to check and load .local_environment
_durrrrrenv_check() {
    local current_dir="$PWD"

    # Skip if we're in the same directory
    if [[ "$current_dir" == "$_DURRRRRENV_LAST_DIR" ]]; then
        return 0
    fi

    # If we're leaving a directory with an active environment, unload it
    if [[ -n "$_DURRRRRENV_ACTIVE_DIR" ]] && [[ "$current_dir" != "$_DURRRRRENV_ACTIVE_DIR" ]]; then
        # Check if we're not in a subdirectory of the active dir
        if [[ "$current_dir" != "$_DURRRRRENV_ACTIVE_DIR"/* ]]; then
            _durrrrrenv_unload
            _DURRRRRENV_ACTIVE_DIR=""
        fi
    fi

    _DURRRRRENV_LAST_DIR="$current_dir"

    # Fast-path: If we're still within the active environment directory tree,
    # we don't need to do anything (environment is already loaded)
    if [[ -n "$_DURRRRRENV_ACTIVE_DIR" ]] && [[ "$current_dir" == "$_DURRRRRENV_ACTIVE_DIR"/* ]]; then
        return 0
    fi

    # Fast-path: Check if .local_environment exists anywhere in the tree
    # before spawning the durrrrrenv process. Avoids process spawn overhead.
    local check_dir="$current_dir"
    local found_env=0
    local depth=0

    while [[ $depth -lt 5 ]]; do
        if [[ -f "$check_dir/.local_environment" ]]; then
            found_env=1
            break
        fi

        # Move to parent directory
        local parent_dir="${check_dir:h}"
        [[ "$parent_dir" == "$check_dir" ]] && break  # Reached root
        check_dir="$parent_dir"
        ((depth++))
    done

    # If no .local_environment file found in tree, skip durrrrrenv entirely
    if [[ $found_env -eq 0 ]]; then
        return 0
    fi

    # Run durrrrrenv check and capture output
    local output
    output=$(durrrrrenv check 2>&1)
    local exit_code=$?

    # Early exit if no output
    [[ -z "$output" ]] && return 0

    # Check if output is an error message (starts with "durrrrrenv:")
    # Using zsh pattern matching instead of grep for speed
    if [[ "$output" == durrrrrenv:* ]]; then
        echo "$output" >&2
        return 0
    fi

    # Only proceed if check succeeded
    [[ $exit_code -ne 0 ]] && return 0

    # Extract DURRRRRENV_DIR if present using zsh string manipulation
    local env_dir=""
    local script_output=""

    # Process output line by line using zsh built-ins
    local line
    for line in "${(@f)output}"; do
        if [[ "$line" == DURRRRRENV_DIR=* ]]; then
            # Extract directory using parameter expansion
            env_dir="${line#DURRRRRENV_DIR=}"
        else
            # Accumulate script lines
            script_output="${script_output}${line}"$'\n'
        fi
    done

    # Evaluate the script if we have output
    if [[ -n "$script_output" ]]; then
        eval "$script_output"

        # Set the active directory
        if [[ -n "$env_dir" ]]; then
            _DURRRRRENV_ACTIVE_DIR="$env_dir"
        else
            _DURRRRRENV_ACTIVE_DIR="$current_dir"
        fi
    fi
}

# Hook into directory changes
autoload -U add-zsh-hook
add-zsh-hook chpwd _durrrrrenv_check

# Also check on shell startup
_durrrrrenv_check
