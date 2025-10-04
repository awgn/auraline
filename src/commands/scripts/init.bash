auraline_pre_cmd() {
   echo -n "$(date +%s%N)" > /tmp/auraline_cmd_start.$$
}

trap 'auraline_pre_cmd' DEBUG

PS1='$(~/.cargo/bin/auraline prompt --exit-code $?)\n\$ '
