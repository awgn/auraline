auraline
========
[![Crates.io](https://img.shields.io/crates/v/auraline.svg)](https://crates.io/crates/auraline)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub repository](https://img.shields.io/badge/github-repo-blue.svg)](https://github.com/awgn/auraline)

`auraline` is a utility that renders information into a string suitable to be embedded
in the shell prompt. It is written in Rust.

Build
-----

To build the package run the following commands:

```bash
cargo install --path .
```

This will build and install the binary `auraline` in the Cargo bin directory (e.g., `~/.cargo/bin/`).

Building auraline Statically
-----------------------------

To statically build auraline, you can use the following command:

```bash
RUSTFLAGS='-C target-feature=+crt-static' cargo build --release
cargo install --path .
```

This command will instruct Cargo to build auraline with static linking, which will create a static binary.
The binary will include all necessary dependencies, making it more self-contained and easier to distribute.

Usage
-----

```text
Usage: auraline [OPTIONS]

Options:
  -p, --path <PATH>    Specify a path where to run the line ($PWD by default)
  -t, --theme <THEME>  Specify the theme color
  -s, --short-mode     Enable short mode
  -f, --fast           Fast mode
  -n, --nerd-font      Use Nerd Fonts
  -h, --help           Print help
  -V, --version        Print version
```

Theme Color
-----------

The `--theme` option allows you to customize the color of the prompt. You can use one of the predefined color names or a true color value.

### Predefined Colors

You can use any of the following predefined color names:

- `black`
- `red`
- `green`
- `yellow`
- `blue`
- `magenta`
- `cyan`
- `white`
- `purple`
- `bright_black`
- `bright_red`
- `bright_green`
- `bright_yellow`
- `bright_blue`
- `bright_magenta`
- `bright_cyan`
- `bright_white`

Example:
`auraline --theme red`

### True Color

You can also specify a true color value in the format `r,g,b`, where `r`, `g`, and `b` are integers from 0 to 255.

Example:
`auraline --theme 128,0,128`

Nerd Fonts
----------

The `-n` or `--nerd-font` option enables the use of Nerd Fonts for icons and symbols in the prompt.
Make sure you have a Nerd Font installed and configured in your terminal for the symbols to render correctly.

Bash
----

To use it with bash, configure the shell prompt as follow:

`PS1='\u@\h :: \[\033[1;32m\]\w\[\033[0m\] $(~/.cargo/bin/auraline --theme blue -n)\n-> '`

Zsh
---

For zsh, try the following configuration in .zshrc:

```zsh
autoload -U colors && colors
setopt promptsubst
local git_prompt='$(~/.cargo/bin/auraline --theme blue -n)'
PS1="%{$fg[green]%}%n@%m %{$fg[blue]%}%c ${git_prompt} %# "
```

Fish
----

For fish shell, define the following function in
~/.config/fish/functions/fish`_prompt.fish:

```fish
function fish_prompt --description 'Write out the prompt'

    set -l last_status $status
    set -l git_prompt (~/.cargo/bin/auraline --theme blue -n)

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
