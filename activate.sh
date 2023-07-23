ORIGINAL_PATH=$PATH

function set_python_version() {
    local version_path
    version_path=$(tamago find 2>/dev/null)
    if [[ $? -eq 0 ]] && [[ -n "$version_path" ]]; then
        # Only do this if we are not in a virtual environment.
        # TODO: There might be a cleaner way of doing this. I think
        #       I see why pyenv intercepts python and pip now.
        if [[ -z "$VIRTUAL_ENV" ]]; then
            PATH="${version_path}/bin:$PATH"
        fi
    else
        # TODO: probably just strip tamago out
        PATH=$ORIGINAL_PATH
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
