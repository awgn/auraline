auraline_pre_cmd() {
    if [[ -n "${EPOCHREALTIME:-}" ]]; then
        printf '%s000' "${EPOCHREALTIME/./}" > "/tmp/auraline_cmd_start.$$"
    else
        printf '%s' "$(date +%s%N)" > "/tmp/auraline_cmd_start.$$"
    fi
}

[[ "${PS0:-}" != *"auraline_pre_cmd"* ]] && PS0="${PS0:-}"'$(auraline_pre_cmd)'
PS1='$(~/.cargo/bin/auraline prompt --exit-code $?)\n\$ '
