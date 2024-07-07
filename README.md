prompt-rs
==========

prompt-rs is an utility that, from a local git repository, renders information
into a string suitable to be embedded in the shell prompt. It is written in Rust.

Build
-----
To build the package run the following commands:

```
cargo build --release
cargo install --path .
```

This will build and install the binary `prompt-rs` in the Cargo bin directory (e.g., `~/.cargo/bin/`).

Building prompt-rs Statically
-----------------------------

To statically build prompt-rs, you can use the following command:

```
RUSTFLAGS='-C target-feature=+crt-static' cargo build --release
cargo install --path .
```

This command will instruct Cargo to build prompt-rs with static linking, which will create a static binary.
The binary will include all necessary dependencies, making it more self-contained and easier to distribute.

Usage
-----

```
Usage: git-prompt-rs [OPTIONS]

Options:
  -p, --path <PATH>    Specify the git-repository path ($PWD by default)
  -t, --theme <THEME>  Specify the theme color
  -s, --short-mode     Enable short mode
  -h, --help           Print help
  -V, --version        Print version
```

Bash
----

To use it with bash, configure the shell prompt as follow:

`PS1='\u@\h :: \[\033[1;32m\]\w\[\033[0m\] $(~/.cargo/bin/prompt-rs --theme blue)\n-> '`

Zsh
---

For zsh, try the following configuration in .zshrc:

```
autoload -U colors && colors
setopt promptsubst
local git_prompt='$(~/.cargo/bin/prompt-rs --theme blue)'
PS1="%{$fg[green]%}%n@%m %{$fg[blue]%}%c ${git_prompt} %# "
```

Fish
----

For fish shell, define the following function in
~/.config/fish/functions/fish`_prompt.fish:

```
function fish_prompt --description 'Write out the prompt'

    set -l last_status $status
    set -l git_prompt (~/.cargo/bin/prompt-rs --theme blue)

    if not set -q __fish_prompt_normal
        set -g __fish_prompt_normal (set_color normal)
    end

    # PWD
    set_color $fish_color_cwd
    echo -n (prompt_pwd)
    set_color normal

    printf '%s ' "{$git_prompt}"

    if not test $last_status -eq 0
        set_color $fish_color_error
    end

    echo -n '$ '

end
```
