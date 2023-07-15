
function set_python_version() {
    local version_path
    version_path=$(tamago find 2>/dev/null)
    if [[ $? -eq 0 ]] && [[ -n "$version_path" ]]; then
        PATH="${version_path}/bin:$PATH"
        alias python="${version_path}/bin/python3"
    else
        # Set default Python path here
        PATH="/usr/local/bin:$PATH"
    fi
}

function cd() {
    builtin cd "$@" && set_python_version
}

# If PROMPT_COMMAND is empty, just set it to your function
# Otherwise, append your function to the existing PROMPT_COMMAND
if [[ -z "$PROMPT_COMMAND" ]]; then
    PROMPT_COMMAND=set_python_version
else
    PROMPT_COMMAND="${PROMPT_COMMAND};set_python_version"
fi
