#compdef jao

_jao_completion() {
    local -a args candidates
    local index

    index=$((CURRENT - 2))
    args=("${words[@]:2}")

    if (( CURRENT > ${#words[@]} )); then
        args+=("")
    fi

    candidates=("${(@f)$(jao __complete --index "$index" -- "${args[@]}")}")
    _describe 'values' candidates
}

compdef _jao_completion jao
