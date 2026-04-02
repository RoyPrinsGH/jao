_jao_completion() {
    local -a args
    local index

    COMPREPLY=()
    index=$((COMP_CWORD - 1))
    args=("${COMP_WORDS[@]:1}")

    if (( COMP_CWORD == ${#COMP_WORDS[@]} )); then
        args+=("")
    fi

    while IFS= read -r candidate; do
        COMPREPLY+=("$candidate")
    done < <(jao __complete --index "$index" -- "${args[@]}")
}

complete -F _jao_completion jao
