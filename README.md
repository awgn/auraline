auraline
========
[![Crates.io](https://img.shields.io/crates/v/auraline.svg)](https://crates.io/crates/auraline)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![GitHub repository](https://img.shields.io/badge/github-repo-blue.svg)](https://github.com/awgn/auraline)

`auraline` is a fast, concurrent shell prompt generator written in Rust. It gathers system, environment, development, and VCS metadata asynchronously and renders a compact, single-line prompt string ready to be embedded in `bash` or `zsh`.

Build & Install
---------------

To build and install `auraline` into your Cargo bin directory (`~/.cargo/bin/`):

```bash
cargo install --path .
```

### Static Build

To build a self-contained static binary:

```bash
RUSTFLAGS='-C target-feature=+crt-static' cargo build --release
```

Usage
-----

```text
Usage: auraline <COMMAND>

Commands:
  prompt
  init

Options:
      --help     Print help information
  -V, --version  Print version
```

### `auraline prompt`

Generates the prompt string to embed in your shell prompt:

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
  -S, --sudo                   Show cached sudo credentials status
  -o, --os                     Show OS info
  -V, --virt                   Show virtual env info
  -n, --netif                  Show network interfaces
  -N, --netns                  Show network namespace info
  -m, --memory                 Show memory usage info
  -k, --disk                   Show disk usage info
  -l, --load                   Show system load average info
  -H, --huge-pages             Show HugePages info
  -M, --manifest               Show development package info in the current directory
  -e, --duration               Show the duration of the last command
      --exit-code <EXIT_CODE>  Specify the exit-code of the last command to show
      --timings                Enable timings mode (dev)
      --theme <THEME>          Specify the theme color
      --nerd-font              Use Nerd Fonts
      --profile <PROFILE>      Specify the prompt profile to use: minimal, lean, nerdy
```

Shell Integration & Environment Variables
-----------------------------------------

Add the initialization snippet to your shell configuration file:

### Bash (`~/.bash_profile` or `~/.bashrc`)

```bash
export AURALINE_PROFILE=nerdy
export AURALINE_THEME="0,102,204"
eval "$(auraline init bash)"
```

### Zsh (`~/.zshrc`)

```zsh
export AURALINE_PROFILE=nerdy
export AURALINE_THEME="0,102,204"
eval "$(auraline init zsh)"
```

### Environment Variables

- **`AURALINE_PROFILE`**: Selects a base profile (`minimal`, `lean`, or `nerdy`).
- **`AURALINE_OPTIONS`**: Space-separated extra flags merged on top of the active profile (e.g. `export AURALINE_OPTIONS="--full-pwd --realname"`).
- **`AURALINE_THEME`**: Sets the accent color (named ANSI color or `R,G,B` truecolor tuple).

Profiles
--------

| Profile   | Enabled Providers | Nerd Fonts |
|-----------|-------------------|------------|
| `minimal` | `user`, `hostname`, `pwd` *(default)* | No |
| `lean`    | `user`, `hostname`, `pwd`, `memory`, `disk`, `load`, `ssh`, `sudo`, `netif`, `duration`, `exit_code` | No |
| `nerdy`   | `user`, `hostname`, `pwd`, `os`, `virt`, `memory`, `disk`, `huge_pages`, `load`, `ssh`, `sudo`, `netif`, `netns`, `manifest`, `vcs`, `duration`, `exit_code` | Yes |

Theme Colors
------------

The `--theme` option (or `AURALINE_THEME` environment variable) customizes the prompt accent color using either a predefined ANSI color name or a 24-bit RGB tuple.

### Predefined Colors

- Standard: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, `purple`
- Bright: `bright_black`, `bright_red`, `bright_green`, `bright_yellow`, `bright_blue`, `bright_magenta`, `bright_cyan`, `bright_white`, `bright_purple`

```bash
auraline prompt --profile nerdy --theme cyan
```

### True Color (`R,G,B`)

Specify `r,g,b` components (`0..=255`):

```bash
export AURALINE_THEME="0,102,204"
auraline prompt --profile nerdy --theme 128,0,128
```

Supported Providers
-------------------

All enabled providers execute concurrently on a Tokio runtime; non-VCS providers run in parallel while VCS root discovery walks the directory tree.

### Basic Information
- **User** (`-u`, `--user`): Current username
- **Realname** (`-r`, `--realname`): User's real name
- **Hostname** (`-h`, `--hostname`): System hostname (`@hostname`)
- **Device Name** (`-d`, `--device-name`): System device/computer name
- **Distro** (`-D`, `--distro`): OS / Linux distribution name
- **PWD** (`-w`, `--pwd`): Current working directory (with `~` home shortening)
- **Full PWD** (`-W`, `--full-pwd`): Current working directory (full path)

### Version Control Systems (`-v`, `--vcs`)
Automatically walks parent directories to detect the active repository (prioritizing colocated Jujutsu `.jj` repositories over `.git`):
- **Git**:
  - Current branch or detached `HEAD`
  - Active repository state / action (`rebase ²⁄₅`, `merge`, `cherry-pick`, `revert`, `bisect`)
  - Working tree status counts with superscripts: conflicts (`≠`), staged (`●`), modified (`○`), untracked (`⁇`)
  - Stash count (`≡`), linked worktrees (`⌂`), tag/commit describe (`v0.9.0▴22∷6adea710`) with `git name-rev` fallback (`↪main↓1`), and upstream divergence (`↑` ahead / `↓` behind)
- **Jujutsu (`jj`)**: Bookmarks/branches, working-copy status, change/commit description, and divergence
- **Mercurial (`hg`)**: Branch name, commit hash, and status
- **Pijul**: Channel name, change hash, and status
- **Darcs**: Commit info and status

### System & Networking
- **OS** (`-o`, `--os`): OS / distribution Nerd Font icon (supports 60+ Linux distributions, BSD variants, macOS, Windows)
- **Sudo** (`-S`, `--sudo`): Displays an indicator (`⚡` or `󰌋` with `--nerd-font`) when the current session is running as `root` or has active cached `sudo` credentials
- **Memory** (`-m`, `--memory`): Memory usage percentage
- **Disk** (`-k`, `--disk`): Disk usage percentage for the current working directory's filesystem (`⛁` or `󰋊` with `--nerd-font`)
- **HugePages** (`-H`, `--huge-pages`): Linux HugePages allocation by page size, with per-NUMA-node subscript breakdown on multi-socket systems (e.g. `₀8x2MB,₁8x2MB`)
- **Load Average** (`-l`, `--load`): 1-minute system load average (`λ1.45`)
- **SSH** (`-s`, `--ssh`): Active SSH session indicator with remote client IP and port
- **Network Interfaces** (`-n`, `--netif`): Active network interfaces with assigned addresses
- **Network Namespace** (`-N`, `--netns`): Current Linux network namespace (`ip netns`)

### Virtualization & Containers (`-V`, `--virt`)
- **Fast-path detection**: Native zero-fork checks for Docker (`/.dockerenv`), Podman (`/run/.containerenv`), `$container`, and WSL (`WSL_DISTRO_NAME`, `WSL_INTEROP`, `/proc/sys/kernel/osrelease`), with fallback to `systemd-detect-virt`
- **Hypervisors & Cloud**: KVM, QEMU, VMware, VirtualBox, Hyper-V, Xen, bhyve, Parallels, Apple Virtualization, Amazon EC2, Google Cloud Platform, Azure
- **Containers**: Docker, Podman, LXC, systemd-nspawn, WSL, OpenVZ

### Development & Command Execution
- **Manifest** (`-M`, `--manifest`): Detects project manifest files in the current directory and displays both the toolchain/compiler version (cached per user and environment context in `/tmp`) and package version (`1.99.0 ↦ v0.15.0`). Supports 50+ languages and build systems including Rust (`Cargo.toml`), C/C++ (`CMakeLists.txt`, `meson.build`, `vcpkg.json`, `conanfile.py`, `xmake.lua`), Go (`go.mod`), Zig (`build.zig.zon`), Python (`pyproject.toml`, `setup.cfg`, `Pipfile`), Node.js/Deno/Bun (`package.json`, `deno.json`), Haskell (`*.cabal`, `stack.yaml`), OCaml (`dune-project`, `*.opam`), Java/Kotlin/Scala (`pom.xml`, `build.gradle`, `build.sbt`), Elixir (`mix.exs`), Dart (`pubspec.yaml`), C# (`*.csproj`), Common Lisp (`*.asd`), Lua (`*.rockspec`), and more.
- **Duration** (`-e`, `--duration`): Execution time of the last interactive command (`ns`, `μs`, `ms`, `s`).
- **Exit Code** (`--exit-code <EXIT_CODE>`): Displayed when the last command exits with a non-zero status (`✘ ¹²⁷`), automatically decoding fatal Unix signals (`129..`) into signal names (e.g. `✘ INT`, `✘ KILL`, `✘ SEGV`, `✘ TERM`).
- **Timings** (`--timings`): Prints a per-provider execution latency breakdown for profiling and debugging.

Nerd Fonts
----------

The `--nerd-font` option (enabled automatically by the `nerdy` profile) uses Nerd Font glyphs for OS, language, VCS, and system icons. Make sure a [Nerd Font](https://www.nerdfonts.com/) is configured in your terminal.
