zmodload zsh/datetime
setopt PROMPT_SUBST

typeset -g _auraline_start=""
typeset -g _auraline_prompt=""

auraline_preexec() {
    _auraline_start=$EPOCHREALTIME
}

auraline_precmd() {
    local exit_code=$?
    local -a dur_arg=()

    if [[ -n "$_auraline_start" ]]; then
        local -i dur_ns=$(( (EPOCHREALTIME - _auraline_start) * 1000000000.0 ))
        (( dur_ns >= 0 )) && dur_arg=(--duration-ns "$dur_ns")
        _auraline_start=""
    fi

    _auraline_prompt="$(__AURALINE_BIN__ prompt --shell zsh --exit-code "$exit_code" "${dur_arg[@]}")"
}

autoload -Uz add-zsh-hook
add-zsh-hook preexec auraline_preexec
add-zsh-hook precmd auraline_precmd

PROMPT='${_auraline_prompt}
\$ '
