#!/usr/bin/env zsh
# durrrrrenv - zsh hook for automatic environment loading
#
# Add this to your .zshrc:
#   eval "$(durrrrrenv hook)"

# Track the last directory to avoid repeated checks
typeset -g _DURRRRRENV_LAST_DIR=""

# Function to check and load .local_environment
_durrrrrenv_check() {
    local current_dir="$PWD"

    # Skip if we're in the same directory
    if [[ "$current_dir" == "$_DURRRRRENV_LAST_DIR" ]]; then
        return 0
    fi

    _DURRRRRENV_LAST_DIR="$current_dir"

    # Run durrrrrenv check and capture output
    local output
    output=$(durrrrrenv check 2>&1)
    local exit_code=$?

    # If there's output to stderr (warnings/prompts), show it
    if [[ -n "$output" ]] && echo "$output" | grep -q "^durrrrrenv:"; then
        echo "$output" >&2
    fi

    # If check returned shell code to evaluate, do it
    if [[ $exit_code -eq 0 ]] && [[ -n "$output" ]] && ! echo "$output" | grep -q "^durrrrrenv:"; then
        eval "$output"
    fi
}

# Hook into directory changes
autoload -U add-zsh-hook
add-zsh-hook chpwd _durrrrrenv_check

# Also check on shell startup
_durrrrrenv_check
