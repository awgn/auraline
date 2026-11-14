_auraline_ps0=1
_auraline_start=""
_auraline_prompt=""

auraline_pre_cmd_fallback() {
    printf '%s' "$(date +%s%N 2>/dev/null)" > "/tmp/auraline_cmd_start.$$"
}

auraline_prompt_cmd() {
    local exit_code=$?
    local -a dur_arg=()

    if [[ -n "${_auraline_start:-}" && -n "${EPOCHREALTIME:-}" ]]; then
        local now="${EPOCHREALTIME/[.,]/}000"
        local dur_ns=$(( now - _auraline_start ))
        if (( dur_ns >= 0 )); then
            dur_arg=(--duration-ns "$dur_ns")
        fi
        _auraline_start=""
    fi

    _auraline_prompt="$(__AURALINE_BIN__ prompt --shell bash --exit-code "$exit_code" ${dur_arg[@]+"${dur_arg[@]}"})"
    return "$exit_code"
}

if [[ -n "${EPOCHREALTIME:-}" ]]; then
    if [[ "${PS0:-}" != *"_auraline_start="* ]]; then
        PS0="${PS0:-}"'${_auraline_ps0:$((_auraline_start=${EPOCHREALTIME/[.,]/}000, 0)):0}'
    fi
else
    [[ "${PS0:-}" != *"auraline_pre_cmd_fallback"* ]] && PS0="${PS0:-}"'$(auraline_pre_cmd_fallback)'
fi

if [[ "${PROMPT_COMMAND:-}" != *"auraline_prompt_cmd"* ]]; then
    PROMPT_COMMAND="auraline_prompt_cmd${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
fi

PS1='${_auraline_prompt}\n\$ '
