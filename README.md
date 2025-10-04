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

Help
----

```text
Usage: auraline <COMMAND>

Commands:
  prompt
  init

Options:
      --help     Print help information
  -V, --version  Print version
```

Prompt
------
The `prompt` command generates the prompt string that can be embedded in your shell prompt.

```text
Usage: auraline prompt [OPTIONS]

Options:
      --help                   Print help information
  -u, --user                   Basic user info
  -r, --realname               Basic realname info
  -h, --hostname               Basic hostname info
  -d, --device-name            Basic devicename info
  -D, --distro                 Basic distro info
  -w, --pwd                    Current working directory
  -W, --full-pwd               Current working directory (full path)
  -v, --vcs                    Show VCS info (git, hg, jj, etc.)
  -s, --ssh                    Show SSH info
  -o, --os                     Show OS info
  -V, --virt                   Show virtual env info
  -n, --netif                  Show network interfaces
  -N, --netns                  Show network namespace info
  -m, --memory                 Show memory usage info
  -H, --huge-pages             Show HugePages info
  -M, --manifest               Show development package info in the current directory
  -e, --duration               Show the duration of the last command)
      --exit-code <EXIT_CODE>  Specify the exit-code of the last command to show
      --timings                Enable timings mode (dev)
      --theme <THEME>          Specify the theme color
      --nerd-font              Use Nerd Fonts
```

Integration and environment variables
-------------------------------------
You can integrate `auraline` into your shell prompt by adding the command to your shell configuration file (e.g., `.bashrc`, `.zshrc`, etc.).

## Bash (~/.bash_profile)
```
export AURALINE_PROFILE=nerdy
export AURALINE_THEME=blue
eval "$(auraline init bash)"
```

## Zsh
```
export AURALINE_PROFILE=nerdy
export AURALINE_THEME=blue
eval "$(auraline init bash)"
```

Profiles
--------

The `AURALINE_PROFILE` environment variable allows you to select a predefined profile for the prompt.

You can choose from the following profiles:
- `minimal`: A minimalistic prompt with only the most essential information (used as default)
- `lean`: A balanced prompt with a moderate amount of information. Doesn't require Nerd Fonts.
- `nerdy`: A more detailed prompt with additional information and Nerd Font icons.

Theme Color
-----------

The `--theme` option (`AURALINE_THEME` asl well) allows you to customize the color of the prompt.
You can use one of the predefined color names or a true color value.

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
