use crate::{chunk::Chunk, options::Options};
use phf::phf_map;
use smallvec::SmallVec;
use smol_str::{format_smolstr, SmolStr, SmolStrBuilder};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum MatchKind {
    /// Generic or ambiguous: tooling, config formats, build systems shared across
    /// many ecosystems. Does not reliably identify the primary language.
    /// e.g. `Makefile`, `CMakeLists.txt`, `.toml`, `.yaml`, `.sh`
    Generic,
    /// Source file extension or ecosystem-specific file that reliably identifies
    /// the language, but is not a formal project declaration.
    /// e.g. `.rs`, `.py`, `.go`, `requirements.txt`
    Indicator,
    /// Canonical, unambiguous project manifest. Its presence formally declares
    /// which language owns this directory.
    /// e.g. `Cargo.toml`, `go.mod`, `mix.exs`
    ProjectManifest,
}

#[allow(dead_code)]
pub struct LanguageInfo {
    pub icon: &'static str,
    pub color: &'static str,
    pub cterm_color: &'static str,
    pub name: &'static str,
    pub kind: MatchKind,
}

// A static, perfect hash map of manifest filenames to language information.

static MANIFEST_MAP: phf::Map<&'static str, LanguageInfo> = phf_map! {
    // Agda
    ".agda" => LanguageInfo { icon: "A", color: "#F1D352", cterm_color: "221", name: "Agda", kind: MatchKind::Indicator },
    ".lagda" => LanguageInfo { icon: "A", color: "#F1D352", cterm_color: "221", name: "Agda", kind: MatchKind::Indicator },

    // Assembly
    ".asm" => LanguageInfo { icon: "", color: "#6E8493", cterm_color: "102", name: "Assembly", kind: MatchKind::Generic },

    // Awk
    ".awk" => LanguageInfo { icon: "", color: "#E5E5E5", cterm_color: "254", name: "Awk", kind: MatchKind::Indicator },
    ".mawk" => LanguageInfo { icon: "", color: "#E5E5E5", cterm_color: "254", name: "Awk", kind: MatchKind::Indicator },
    ".gawk" => LanguageInfo { icon: "", color: "#E5E5E5", cterm_color: "254", name: "Awk", kind: MatchKind::Indicator },

    // C & C++
    "Makefile" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", kind: MatchKind::Generic },
    "Makefile.am" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", kind: MatchKind::Generic },
    "CMakeLists.txt" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", kind: MatchKind::Generic },
    "meson.build" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", kind: MatchKind::Generic },
    "autogen.sh" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", kind: MatchKind::Generic },
    "configure.ac" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", kind: MatchKind::Generic },
    ".c" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", kind: MatchKind::Indicator },
    ".h" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", kind: MatchKind::Indicator },
    ".inc" => LanguageInfo { icon: "", color: "#B3B3B3", cterm_color: "249", name: "C/C++ Header", kind: MatchKind::Generic },
    ".cpp" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", kind: MatchKind::Indicator },
    ".hpp" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", kind: MatchKind::Indicator },
    ".cxx" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C++", kind: MatchKind::Indicator },
    ".cc" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C++", kind: MatchKind::Indicator },

    // C#
    ".csproj" => LanguageInfo { icon: "", color: "#5a29a4", cterm_color: "99", name: "C#", kind: MatchKind::ProjectManifest},
    ".cs" => LanguageInfo { icon: "", color: "#5a29a4", cterm_color: "99", name: "C#", kind: MatchKind::Indicator },

    // Chapel
    ".chpl" => LanguageInfo { icon: "C", color: "#8dc63f", cterm_color: "113", name: "Chapel", kind: MatchKind::Indicator },

    // Clojure
    ".clj" => LanguageInfo { icon: "", color: "#82b131", cterm_color: "106", name: "Clojure", kind: MatchKind::Indicator },
    ".cljs" => LanguageInfo { icon: "", color: "#82b131", cterm_color: "106", name: "ClojureScript", kind: MatchKind::Indicator },
    ".cljc" => LanguageInfo { icon: "", color: "#82b131", cterm_color: "106", name: "Clojure", kind: MatchKind::Indicator },
    ".edn" => LanguageInfo { icon: "", color: "#82b131", cterm_color: "106", name: "EDN", kind: MatchKind::Generic },

    // COBOL
    ".cbl" => LanguageInfo { icon: "C", color: "#005ca5", cterm_color: "25", name: "COBOL", kind: MatchKind::Indicator},

    // CoffeeScript
    "Cakefile" => LanguageInfo { icon: "", color: "#244776", cterm_color: "24", name: "CoffeeScript", kind: MatchKind::Indicator },
    ".coffee" => LanguageInfo { icon: "", color: "#244776", cterm_color: "24", name: "CoffeeScript", kind: MatchKind::Indicator },

    // Common Lisp
    ".asd" => LanguageInfo { icon: "", color: "#b31a23", cterm_color: "124", name: "Common Lisp", kind: MatchKind::ProjectManifest},
    ".lisp" => LanguageInfo { icon: "", color: "#b31a23", cterm_color: "124", name: "Lisp", kind: MatchKind::Indicator },
    ".cl" => LanguageInfo { icon: "", color: "#b31a23", cterm_color: "124", name: "Common Lisp", kind: MatchKind::Indicator },

    // Config Files
    ".config" => LanguageInfo { icon: "", color: "#A0A0A0", cterm_color: "247", name: "Config", kind: MatchKind::Generic },
    ".conf" => LanguageInfo { icon: "", color: "#A0A0A0", cterm_color: "247", name: "Config", kind: MatchKind::Generic },
    ".cfg" => LanguageInfo { icon: "", color: "#A0A0A0", cterm_color: "247", name: "Config", kind: MatchKind::Generic },
    ".ini" => LanguageInfo { icon: "", color: "#A0A0A0", cterm_color: "247", name: "INI", kind: MatchKind::Generic },
    ".toml" => LanguageInfo { icon: "", color: "#A0A0A0", cterm_color: "247", name: "TOML", kind: MatchKind::Generic },
    ".yaml" => LanguageInfo { icon: "󰰴", color: "#A0A0A0", cterm_color: "247", name: "YAML", kind: MatchKind::Generic },
    ".yml" => LanguageInfo { icon: "󰰴", color: "#A0A0A0", cterm_color: "247", name: "YAML", kind: MatchKind::Generic },
    ".json" => LanguageInfo { icon: "", color: "#E5E5E5", cterm_color: "254", name: "JSON", kind: MatchKind::Generic },

    // CSS
    ".css" => LanguageInfo { icon: "", color: "#563d7c", cterm_color: "98", name: "CSS", kind: MatchKind::Indicator },

    // D
    "dub.json" => LanguageInfo { icon: "D", color: "#ba595e", cterm_color: "131", name: "D", kind: MatchKind::ProjectManifest},
    "dub.sdl" => LanguageInfo { icon: "D", color: "#ba595e", cterm_color: "131", name: "D", kind: MatchKind::ProjectManifest},
    ".d" => LanguageInfo { icon: "D", color: "#ba595e", cterm_color: "131", name: "D", kind: MatchKind::Indicator },

    // Dart
    "pubspec.yaml" => LanguageInfo { icon: "", color: "#0175c2", cterm_color: "32", name: "Dart", kind: MatchKind::ProjectManifest},
    ".dart" => LanguageInfo { icon: "", color: "#0175c2", cterm_color: "32", name: "Dart", kind: MatchKind::Indicator },

    // Dhall
    ".dhall" => LanguageInfo { icon: "D", color: "#174B59", cterm_color: "23", name: "Dhall", kind: MatchKind::Indicator },

    // Eiffel
    ".ecf" => LanguageInfo { icon: "E", color: "#0C8B95", cterm_color: "30", name: "Eiffel", kind: MatchKind::ProjectManifest},

    // Elixir
    "mix.exs" => LanguageInfo { icon: "", color: "#4b275f", cterm_color: "54", name: "Elixir", kind: MatchKind::ProjectManifest},
    ".ex" => LanguageInfo { icon: "", color: "#4b275f", cterm_color: "54", name: "Elixir", kind: MatchKind::Indicator },
    ".exs" => LanguageInfo { icon: "", color: "#4b275f", cterm_color: "54", name: "Elixir", kind: MatchKind::Indicator },

    // Elm
    "elm.json" => LanguageInfo { icon: "", color: "#60b5cc", cterm_color: "74", name: "Elm", kind: MatchKind::ProjectManifest},

    // Erlang
    ".erl" => LanguageInfo { icon: "", color: "#b83998", cterm_color: "133", name: "Erlang", kind: MatchKind::Indicator },
    ".hrl" => LanguageInfo { icon: "", color: "#b83998", cterm_color: "133", name: "Erlang", kind: MatchKind::Indicator},

    // F#
    ".fsproj" => LanguageInfo { icon: "", color: "#378bba", cterm_color: "38", name: "F#", kind: MatchKind::ProjectManifest},
    ".fs" => LanguageInfo { icon: "", color: "#378bba", cterm_color: "38", name: "F#", kind: MatchKind::Indicator },
    ".fsx" => LanguageInfo { icon: "", color: "#378bba", cterm_color: "38", name: "F#", kind: MatchKind::Indicator },

    // Forth
    ".fth" => LanguageInfo { icon: "F", color: "#341708", cterm_color: "52", name: "Forth", kind: MatchKind::Generic },

    // Fortran
    ".f90" => LanguageInfo { icon: "󱈚", color: "#734f96", cterm_color: "98", name: "Fortran", kind: MatchKind::Indicator},
    ".f95" => LanguageInfo { icon: "󱈚", color: "#734f96", cterm_color: "98", name: "Fortran", kind: MatchKind::Indicator},
    ".f" => LanguageInfo { icon: "󱈚", color: "#734f96", cterm_color: "98", name: "Fortran", kind: MatchKind::Indicator},

    // Go
    "go.mod" => LanguageInfo { icon: "", color: "#00add8", cterm_color: "38", name: "Go", kind: MatchKind::ProjectManifest},
    ".go" => LanguageInfo { icon: "", color: "#00add8", cterm_color: "38", name: "Go", kind: MatchKind::Indicator },

    // Haskell
    ".cabal" => LanguageInfo { icon: "", color: "#5e5086", cterm_color: "61", name: "Haskell", kind: MatchKind::ProjectManifest},
    "stack.yaml" => LanguageInfo { icon: "", color: "#5e5086", cterm_color: "61", name: "Haskell", kind: MatchKind::ProjectManifest},
    "Setup.hs" => LanguageInfo { icon: "", color: "#5e5086", cterm_color: "61", name: "Haskell", kind: MatchKind::Indicator },
    ".hs" => LanguageInfo { icon: "", color: "#5e5086", cterm_color: "61", name: "Haskell", kind: MatchKind::Indicator },
    ".lhs" => LanguageInfo { icon: "", color: "#5e5086", cterm_color: "61", name: "Haskell", kind: MatchKind::Indicator },

    // Haxe
    "haxelib.json" => LanguageInfo { icon: "H", color: "#DF7900", cterm_color: "208", name: "Haxe", kind: MatchKind::ProjectManifest},
    ".hxml" => LanguageInfo { icon: "H", color: "#DF7900", cterm_color: "208", name: "Haxe", kind: MatchKind::Indicator },

    // HTML
    ".html" => LanguageInfo { icon: "", color: "#e34c26", cterm_color: "196", name: "HTML", kind: MatchKind::Indicator },
    ".htm" => LanguageInfo { icon: "", color: "#e34c26", cterm_color: "196", name: "HTML", kind: MatchKind::Indicator },

    // Idris
    ".idr" => LanguageInfo { icon: "I", color: "#1F232D", cterm_color: "235", name: "Idris", kind: MatchKind::Indicator },
    ".lidr" => LanguageInfo { icon: "I", color: "#1F232D", cterm_color: "235", name: "Idris", kind: MatchKind::Indicator },

    // Java & JVM
    "pom.xml" => LanguageInfo { icon: "", color: "#cc0000", cterm_color: "160", name: "Java/JVM", kind: MatchKind::ProjectManifest},
    "build.gradle" => LanguageInfo { icon: "", color: "#cc0000", cterm_color: "160", name: "Java/JVM", kind: MatchKind::Generic },
    ".java" => LanguageInfo { icon: "", color: "#cc0000", cterm_color: "160", name: "Java", kind: MatchKind::Indicator },

    // JavaScript / TypeScript / Node
    "package.json" => LanguageInfo { icon: "", color: "#f0db4f", cterm_color: "220", name: "JavaScript/Node", kind: MatchKind::ProjectManifest},
    "tsconfig.json" => LanguageInfo { icon: "", color: "#3178c6", cterm_color: "39", name: "TypeScript", kind: MatchKind::ProjectManifest},

    // Julia
    "Project.toml" => LanguageInfo { icon: "", color: "#a270ba", cterm_color: "140", name: "Julia", kind: MatchKind::ProjectManifest},

    // Kotlin
    "build.gradle.kts" => LanguageInfo { icon: "󱈙", color: "#7f52ff", cterm_color: "99", name: "Kotlin", kind: MatchKind::Indicator},
    ".kt" => LanguageInfo { icon: "󱈙", color: "#7f52ff", cterm_color: "99", name: "Kotlin", kind: MatchKind::Indicator },
    ".kts" => LanguageInfo { icon: "󱈙", color: "#7f52ff", cterm_color: "99", name: "Kotlin", kind: MatchKind::Indicator },

    // LaTeX
    ".tex" => LanguageInfo { icon: "", color: "#008080", cterm_color: "30", name: "LaTeX", kind: MatchKind::Indicator},
    ".latex" => LanguageInfo { icon: "", color: "#008080", cterm_color: "30", name: "LaTeX", kind: MatchKind::Indicator},

    // Lua
    ".rockspec" => LanguageInfo { icon: "", color: "#2c2d72", cterm_color: "18", name: "Lua", kind: MatchKind::ProjectManifest},
    ".lua" => LanguageInfo { icon: "", color: "#2c2d72", cterm_color: "18", name: "Lua", kind: MatchKind::Indicator},

    // Nim
    ".nimble" => LanguageInfo { icon: "", color: "#ffc200", cterm_color: "220", name: "Nim", kind: MatchKind::ProjectManifest},
    ".nim" => LanguageInfo { icon: "", color: "#ffc200", cterm_color: "220", name: "Nim", kind: MatchKind::Indicator },

    // Nix
    "flake.nix" => LanguageInfo { icon: "", color: "#7E76D4", cterm_color: "104", name: "Nix", kind: MatchKind::ProjectManifest},

    // Nmap
    ".nse" => LanguageInfo { icon: "N", color: "#404040", cterm_color: "238", name: "Nmap Script", kind: MatchKind::Indicator },

    // Objective-C
    "project.pbxproj" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "Objective-C", kind: MatchKind::Generic },
    ".m" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "Objective-C", kind: MatchKind::Generic },

    // OCaml
    "dune-project" => LanguageInfo { icon: "", color: "#ec6813", cterm_color: "166", name: "OCaml", kind: MatchKind::ProjectManifest},
    "opam" => LanguageInfo { icon: "", color: "#ec6813", cterm_color: "166", name: "OCaml", kind: MatchKind::ProjectManifest},
    ".ml" => LanguageInfo { icon: "", color: "#ec6813", cterm_color: "166", name: "OCaml", kind: MatchKind::Indicator },
    ".mli" => LanguageInfo { icon: "", color: "#ec6813", cterm_color: "166", name: "OCaml", kind: MatchKind::Indicator },

    // Pascal / Delphi
    ".dpr" => LanguageInfo { icon: "P", color: "#e32b2f", cterm_color: "196", name: "Delphi/Pascal", kind: MatchKind::Indicator },
    ".lpr" => LanguageInfo { icon: "P", color: "#EEDD34", cterm_color: "184", name: "Lazarus/Pascal", kind: MatchKind::Indicator },

    // Perl
    ".pl" => LanguageInfo { icon: "", color: "#39457e", cterm_color: "61", name: "Perl", kind: MatchKind::Indicator},
    ".pm" => LanguageInfo { icon: "", color: "#39457e", cterm_color: "61", name: "Perl", kind: MatchKind::Indicator },

    // PHP
    "composer.json" => LanguageInfo { icon: "", color: "#8892be", cterm_color: "103", name: "PHP", kind: MatchKind::ProjectManifest},
    ".php" => LanguageInfo { icon: "", color: "#8892be", cterm_color: "103", name: "PHP", kind: MatchKind::Indicator },

    // PowerShell
    ".psd1" => LanguageInfo { icon: "󰨊", color: "#012456", cterm_color: "18", name: "PowerShell", kind: MatchKind::Indicator },

    // Prolog
    ".pro" => LanguageInfo { icon: "P", color: "#880000", cterm_color: "88", name: "Prolog", kind: MatchKind::Generic },

    // Python
    "pyproject.toml" => LanguageInfo { icon: "", color: "#f6c43b", cterm_color: "203", name: "Python", kind: MatchKind::ProjectManifest},
    "Pipfile" => LanguageInfo { icon: "", color: "#f6c43b", cterm_color: "203", name: "Python", kind: MatchKind::ProjectManifest},
    ".py" => LanguageInfo { icon: "", color: "#f6c43b", cterm_color: "203", name: "Python", kind: MatchKind::Indicator },
    ".pyx" => LanguageInfo { icon: "", color: "#f6c43b", cterm_color: "203", name: "Cython", kind: MatchKind::Indicator },

    // R
    ".r" => LanguageInfo { icon: "󰟔", color: "#276dc2", cterm_color: "33", name: "R", kind: MatchKind::Indicator },

    // Racket
    "info.rkt" => LanguageInfo { icon: "R", color: "#394FAC", cterm_color: "61", name: "Racket", kind: MatchKind::ProjectManifest},

    // Ruby
    "Gemfile" => LanguageInfo { icon: "", color: "#cc342d", cterm_color: "160", name: "Ruby", kind: MatchKind::ProjectManifest},
    ".gemspec" => LanguageInfo { icon: "", color: "#cc342d", cterm_color: "160", name: "Ruby", kind: MatchKind::ProjectManifest},
    ".rb" => LanguageInfo { icon: "", color: "#cc342d", cterm_color: "160", name: "Ruby", kind: MatchKind::Indicator },

    // Rust
    "Cargo.toml" => LanguageInfo { icon: "", color: "#dea584", cterm_color: "173", name: "Rust", kind: MatchKind::ProjectManifest},
    ".rs" => LanguageInfo { icon: "", color: "#dea584", cterm_color: "173", name: "Rust", kind: MatchKind::Indicator },

    // Scala
    "build.sbt" => LanguageInfo { icon: "", color: "#c22d40", cterm_color: "160", name: "Scala", kind: MatchKind::ProjectManifest},

    // Shell
    ".sh" => LanguageInfo { icon: "", color: "#89E051", cterm_color: "113", name: "Shell", kind: MatchKind::Indicator},
    ".bash" => LanguageInfo { icon: "", color: "#89E051", cterm_color: "113", name: "Bash", kind: MatchKind::Indicator },
    ".csh" => LanguageInfo { icon: "", color: "#89E051", cterm_color: "113", name: "Csh", kind: MatchKind::Indicator },
    ".tcsh" => LanguageInfo { icon: "", color: "#89E051", cterm_color: "113", name: "Tcsh", kind: MatchKind::Indicator },
    ".fish" => LanguageInfo { icon: "", color: "#89E051", cterm_color: "113", name: "Fish", kind: MatchKind::Indicator },
    ".ksh" => LanguageInfo { icon: "", color: "#89E051", cterm_color: "113", name: "Ksh", kind: MatchKind::Indicator },
    ".zsh" => LanguageInfo { icon: "", color: "#89E051", cterm_color: "113", name: "Zsh", kind: MatchKind::Indicator },

    // SmallTalk
    ".st" => LanguageInfo { icon: "S", color: "#596700", cterm_color: "58", name: "SmallTalk", kind: MatchKind::Indicator },

    // Swift
    "Package.swift" => LanguageInfo { icon: "", color: "#ffac45", cterm_color: "215", name: "Swift", kind: MatchKind::ProjectManifest},

    // Tcl
    ".tcl" => LanguageInfo { icon: "T", color: "#1D529D", cterm_color: "25", name: "Tcl", kind: MatchKind::Indicator},

    // Unison
    ".u" => LanguageInfo { icon: "U", color: "#FF7A62", cterm_color: "209", name: "Unison", kind: MatchKind::Indicator },

    // Vala
    ".vala" => LanguageInfo { icon: "", color: "#7239B3", cterm_color: "98", name: "Vala", kind: MatchKind::Indicator},

    // Verilog
    ".v" => LanguageInfo { icon: "V", color: "#000080", cterm_color: "18", name: "Verilog", kind: MatchKind::Generic },
    ".vh" => LanguageInfo { icon: "V", color: "#000080", cterm_color: "18", name: "Verilog", kind: MatchKind::Generic },
    ".sv" => LanguageInfo { icon: "V", color: "#000080", cterm_color: "18", name: "Verilog", kind: MatchKind::Generic},

    // VHDL
    ".vhd" => LanguageInfo { icon: "V", color: "#AD0000", cterm_color: "88", name: "VHDL", kind: MatchKind::Indicator},
    ".vhdl" => LanguageInfo { icon: "V", color: "#AD0000", cterm_color: "88", name: "VHDL", kind: MatchKind::Indicator},

    // Zig
    "build.zig" => LanguageInfo { icon: "", color: "#f7a41d", cterm_color: "214", name: "Zig", kind: MatchKind::ProjectManifest},
    ".zig" => LanguageInfo { icon: "", color: "#f7a41d", cterm_color: "214", name: "Zig", kind: MatchKind::Indicator },
};

/// Heuristic used to locate the version string inside a manifest file's content.
#[derive(Clone, Copy)]
enum VersionStrategy {
    /// Scan the whole content for `needle`; the version is the `"…"` immediately after it.
    /// Safe to use when the needle is unique enough not to match dependency specs.
    /// e.g. `"version: \""` in mix.exs, `"\"version\": \""` in JSON files.
    QuotedAfter(&'static str),

    /// Find the first line whose start matches `prefix`, then extract the first `"…"` on it.
    /// Use this when the same key appears inside dependency inline-tables on other lines.
    /// e.g. `"version"` in Cargo.toml avoids matching `serde = { version = "1.0" }`.
    LineFirstQuoted(&'static str),

    /// Find the first line whose start matches `prefix`, take the bare value after it
    /// (leading whitespace stripped) up to the first whitespace, `)`, or `#`.
    /// e.g. `"version:"` in .cabal / pubspec.yaml, `"(version "` in dune-project.
    LineUnquoted(&'static str),

    /// Find `open` anywhere in the content; extract the text between `open` and `close`.
    /// e.g. `("<version>", "</version>")` for pom.xml.
    XmlBetween(&'static str, &'static str),
}

/// Map a MANIFEST_MAP lookup key (filename or `.ext`) to the appropriate extraction strategy.
/// Returns `None` for manifests that carry no self-declared version (go.mod, Gemfile, …).
fn version_strategy(key: &str) -> Option<VersionStrategy> {
    Some(match key {
        // ── TOML-like: line-start guard avoids matching dependency version specs ──────────
        "Cargo.toml" | "Project.toml" | "pyproject.toml" | ".nimble" | ".rockspec" | "dub.sdl"
        | "opam" => VersionStrategy::LineFirstQuoted("version"),

        // ── Elixir Mix: `version: "1.0.0"` — the `: ` makes it unique in .exs ────────────
        "mix.exs" => VersionStrategy::QuotedAfter("version: \""),

        // ── JSON-based manifests ──────────────────────────────────────────────────────────
        "package.json" | "composer.json" | "haxelib.json" | "dub.json" | "elm.json" => {
            VersionStrategy::QuotedAfter("\"version\": \"")
        }

        // ── Ruby gemspec: `.version = "…"` ───────────────────────────────────────────────
        ".gemspec" => VersionStrategy::QuotedAfter(".version = \""),

        // ── Scala SBT: `version := "…"` ──────────────────────────────────────────────────
        "build.sbt" => VersionStrategy::QuotedAfter("version := \""),

        // ── Common Lisp ASDF: `:version "…"` in defsystem ───────────────────────────────
        ".asd" => VersionStrategy::QuotedAfter(":version \""),

        // ── Racket info.rkt: `(define version "…")` ──────────────────────────────────────
        "info.rkt" => VersionStrategy::QuotedAfter("(define version \""),

        // ── Unquoted / different delimiters ──────────────────────────────────────────────
        // .cabal:       `version:             0.1.0.0`
        // pubspec.yaml: `version: 1.0.0+1`
        ".cabal" | "pubspec.yaml" => VersionStrategy::LineUnquoted("version:"),
        // dune-project: `(version 3.0)`
        "dune-project" => VersionStrategy::LineUnquoted("(version "),

        // ── XML ───────────────────────────────────────────────────────────────────────────
        "pom.xml" => VersionStrategy::XmlBetween("<version>", "</version>"),
        ".csproj" | ".fsproj" => VersionStrategy::XmlBetween("<Version>", "</Version>"),

        // Everything else (go.mod, Package.swift, flake.nix, Gemfile, …) has no
        // self-declared version we can reliably extract.
        _ => return None,
    })
}

/// Apply `strategy` to the raw text of a manifest file and return the version string,
/// or `None` if the pattern is not found or the matched value is empty.
fn extract_version(strategy: VersionStrategy, content: &str) -> Option<SmolStr> {
    match strategy {
        VersionStrategy::QuotedAfter(needle) => {
            let start = content.find(needle)? + needle.len();
            let rest = &content[start..];
            let end = rest.find('"')?;
            let v = &rest[..end];
            (!v.is_empty()).then(|| SmolStr::new(v))
        }

        VersionStrategy::LineFirstQuoted(prefix) => {
            for line in content.lines() {
                if !line.starts_with(prefix) {
                    continue;
                }
                // Find the first quoted string on this line.
                let q_open = line.find('"')?;
                let rest = &line[q_open + 1..];
                let q_end = rest.find('"')?;
                let v = &rest[..q_end];
                if !v.is_empty() {
                    return Some(SmolStr::new(v));
                }
            }
            None
        }

        VersionStrategy::LineUnquoted(prefix) => {
            for line in content.lines() {
                if !line.starts_with(prefix) {
                    continue;
                }
                let rest = line[prefix.len()..].trim_start();
                // Stop at whitespace, closing paren, or comment marker.
                let end = rest
                    .find(|c: char| c.is_whitespace() || c == ')' || c == '#')
                    .unwrap_or(rest.len());
                let v = &rest[..end];
                if !v.is_empty() {
                    return Some(SmolStr::new(v));
                }
            }
            None
        }

        VersionStrategy::XmlBetween(open, close) => {
            let start = content.find(open)? + open.len();
            let rest = &content[start..];
            let end = rest.find(close)?;
            let v = rest[..end].trim();
            (!v.is_empty()).then(|| SmolStr::new(v))
        }
    }
}

/// Read `path` from disk and attempt to extract the version using the strategy
/// associated with `key` (the MANIFEST_MAP lookup key for that file).
async fn version_from_file(path: &std::path::Path, key: &str) -> Option<SmolStr> {
    let strategy = version_strategy(key)?;
    let content = tokio::fs::read_to_string(path).await.ok()?;
    extract_version(strategy, &content)
}

pub async fn show(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.nerd_font || !opts.manifest {
        return None;
    }

    let mut entries   = tokio::fs::read_dir(".").await.ok()?;
    let mut languages: SmallVec<[&LanguageInfo; 8]> = SmallVec::new();
    // All ProjectManifest entries found during the scan, in filesystem order.
    // We will try each in sequence and return the first that yields a version.
    let mut manifests: SmallVec<[(std::path::PathBuf, SmolStr); 4]> = SmallVec::new();

    while let Some(entry) = entries.next_entry().await.ok()? {
        let file_name = entry.file_name();
        // Skip non-UTF-8 names (continue instead of aborting the whole scan).
        let Some(name) = file_name.to_str() else {
            continue;
        };
        // Skip hidden / dot-files.
        if name.starts_with('.') {
            continue;
        }

        if let Some(lang) = MANIFEST_MAP.get(name) {
            if lang.kind == MatchKind::ProjectManifest {
                manifests.push((entry.path(), SmolStr::new(name)));
            }
            languages.push(lang);
        } else if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
            let key = format_smolstr!(".{ext}");
            if let Some(lang) = MANIFEST_MAP.get(key.as_str()) {
                if lang.kind == MatchKind::ProjectManifest {
                    manifests.push((entry.path(), key));
                }
                languages.push(lang);
            }
        }
    }

    languages.sort_by(|a, b| {
        let ord = b.kind.cmp(&a.kind);
        if ord == std::cmp::Ordering::Equal {
            a.icon.cmp(b.icon)
        } else {
            ord
        }
    });

    let top_kind = languages.first()?.kind;
    languages.retain(|lang| lang.kind == top_kind);

    // Concatenate deduplicated icons into a SmolStr.
    let mut builder = SmolStrBuilder::new();
    let mut cur_icon = "";
    for lang in &languages {
        if lang.icon != cur_icon {
            builder.push_str(lang.icon);
            cur_icon = lang.icon;
        }
    }
    let icons = builder.finish();

    // Attempt version extraction only when the top tier is ProjectManifest.
    // Try each collected manifest in filesystem order; stop at the first that
    // yields a non-None version (version_from_file returns None immediately for
    // manifests with no strategy, so we avoid unnecessary file reads).
    let version = if top_kind == MatchKind::ProjectManifest {
        let mut found = None;
        for (path, key) in &manifests {
            if let Some(v) = version_from_file(path, key).await {
                found = Some(v);
                break;
            }
        }
        found
    } else {
        None
    };

    Some(match version {
        Some(v) => Chunk::new(icons, format_smolstr!("v{v}")),
        None => Chunk::icon(icons),
    })
}
