setopt PROMPT_SUBST

auraline_preexec() {
    echo -n "$(date +%s%N)" > /tmp/auraline_cmd_start.$$
}

autoload -Uz add-zsh-hook
add-zsh-hook preexec auraline_preexec

PROMPT='$(~/.cargo/bin/auraline prompt --exit-code $?)
\$ '
