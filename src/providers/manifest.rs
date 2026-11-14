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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Language {
    Agda,
    Assembly,
    Awk,
    CCpp,
    CSharp,
    Chapel,
    Clojure,
    Cobol,
    CoffeeScript,
    CommonLisp,
    Config,
    Css,
    D,
    Dart,
    Dhall,
    Eiffel,
    Elixir,
    Elm,
    Erlang,
    FSharp,
    Forth,
    Fortran,
    Go,
    Haskell,
    Haxe,
    Html,
    Idris,
    Java,
    JavaScript,
    Julia,
    Kotlin,
    LaTeX,
    Lua,
    Nim,
    Nix,
    Nmap,
    ObjectiveC,
    OCaml,
    Pascal,
    Perl,
    Php,
    PowerShell,
    Prolog,
    Python,
    R,
    Racket,
    Ruby,
    Rust,
    Scala,
    Shell,
    SmallTalk,
    Swift,
    Tcl,
    TypeScript,
    Unison,
    Vala,
    Verilog,
    Vhdl,
    Zig,
}

#[allow(dead_code)]
pub struct LanguageInfo {
    pub icon: &'static str,
    pub color: &'static str,
    pub cterm_color: &'static str,
    pub name: &'static str,
    pub lang: Language,
    pub kind: MatchKind,
}

// A static, perfect hash map of manifest filenames to language information.

static MANIFEST_MAP: phf::Map<&'static str, LanguageInfo> = phf_map! {
    // Agda
    ".agda" => LanguageInfo { icon: "A", color: "#F1D352", cterm_color: "221", name: "Agda", lang: Language::Agda, kind: MatchKind::Indicator },
    ".lagda" => LanguageInfo { icon: "A", color: "#F1D352", cterm_color: "221", name: "Agda", lang: Language::Agda, kind: MatchKind::Indicator },

    // Assembly
    ".asm" => LanguageInfo { icon: "", color: "#6E8493", cterm_color: "102", name: "Assembly", lang: Language::Assembly, kind: MatchKind::Generic },

    // Awk
    ".awk" => LanguageInfo { icon: "", color: "#E5E5E5", cterm_color: "254", name: "Awk", lang: Language::Awk, kind: MatchKind::Indicator },
    ".mawk" => LanguageInfo { icon: "", color: "#E5E5E5", cterm_color: "254", name: "Awk", lang: Language::Awk, kind: MatchKind::Indicator },
    ".gawk" => LanguageInfo { icon: "", color: "#E5E5E5", cterm_color: "254", name: "Awk", lang: Language::Awk, kind: MatchKind::Indicator },

    // C & C++
    "Makefile" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", lang: Language::CCpp, kind: MatchKind::Generic },
    "Makefile.am" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", lang: Language::CCpp, kind: MatchKind::Generic },
    "CMakeLists.txt" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", lang: Language::CCpp, kind: MatchKind::ProjectManifest },
    "meson.build" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", lang: Language::CCpp, kind: MatchKind::Generic },
    "autogen.sh" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", lang: Language::CCpp, kind: MatchKind::Generic },
    "configure.ac" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", lang: Language::CCpp, kind: MatchKind::Generic },
    ".c" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", lang: Language::CCpp, kind: MatchKind::Indicator },
    ".h" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", lang: Language::CCpp, kind: MatchKind::Indicator },
    ".inc" => LanguageInfo { icon: "", color: "#B3B3B3", cterm_color: "249", name: "C/C++ Header", lang: Language::CCpp, kind: MatchKind::Generic },
    ".cpp" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", lang: Language::CCpp, kind: MatchKind::Indicator },
    ".hpp" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", lang: Language::CCpp, kind: MatchKind::Indicator },
    ".cxx" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C++", lang: Language::CCpp, kind: MatchKind::Indicator },
    ".cc" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C++", lang: Language::CCpp, kind: MatchKind::Indicator },

    // C#
    ".csproj" => LanguageInfo { icon: "", color: "#5a29a4", cterm_color: "99", name: "C#", lang: Language::CSharp, kind: MatchKind::ProjectManifest},
    ".cs" => LanguageInfo { icon: "", color: "#5a29a4", cterm_color: "99", name: "C#", lang: Language::CSharp, kind: MatchKind::Indicator },

    // Chapel
    ".chpl" => LanguageInfo { icon: "C", color: "#8dc63f", cterm_color: "113", name: "Chapel", lang: Language::Chapel, kind: MatchKind::Indicator },

    // Clojure
    ".clj" => LanguageInfo { icon: "", color: "#82b131", cterm_color: "106", name: "Clojure", lang: Language::Clojure, kind: MatchKind::Indicator },
    ".cljs" => LanguageInfo { icon: "", color: "#82b131", cterm_color: "106", name: "ClojureScript", lang: Language::Clojure, kind: MatchKind::Indicator },
    ".cljc" => LanguageInfo { icon: "", color: "#82b131", cterm_color: "106", name: "Clojure", lang: Language::Clojure, kind: MatchKind::Indicator },
    ".edn" => LanguageInfo { icon: "", color: "#82b131", cterm_color: "106", name: "EDN", lang: Language::Clojure, kind: MatchKind::Generic },

    // COBOL
    ".cbl" => LanguageInfo { icon: "C", color: "#005ca5", cterm_color: "25", name: "COBOL", lang: Language::Cobol, kind: MatchKind::Indicator},

    // CoffeeScript
    "Cakefile" => LanguageInfo { icon: "", color: "#244776", cterm_color: "24", name: "CoffeeScript", lang: Language::CoffeeScript, kind: MatchKind::Indicator },
    ".coffee" => LanguageInfo { icon: "", color: "#244776", cterm_color: "24", name: "CoffeeScript", lang: Language::CoffeeScript, kind: MatchKind::Indicator },

    // Common Lisp
    ".asd" => LanguageInfo { icon: "", color: "#b31a23", cterm_color: "124", name: "Common Lisp", lang: Language::CommonLisp, kind: MatchKind::ProjectManifest},
    ".lisp" => LanguageInfo { icon: "", color: "#b31a23", cterm_color: "124", name: "Lisp", lang: Language::CommonLisp, kind: MatchKind::Indicator },
    ".cl" => LanguageInfo { icon: "", color: "#b31a23", cterm_color: "124", name: "Common Lisp", lang: Language::CommonLisp, kind: MatchKind::Indicator },

    // Config Files
    ".config" => LanguageInfo { icon: "", color: "#A0A0A0", cterm_color: "247", name: "Config", lang: Language::Config, kind: MatchKind::Generic },
    ".conf" => LanguageInfo { icon: "", color: "#A0A0A0", cterm_color: "247", name: "Config", lang: Language::Config, kind: MatchKind::Generic },
    ".cfg" => LanguageInfo { icon: "", color: "#A0A0A0", cterm_color: "247", name: "Config", lang: Language::Config, kind: MatchKind::Generic },
    ".ini" => LanguageInfo { icon: "", color: "#A0A0A0", cterm_color: "247", name: "INI", lang: Language::Config, kind: MatchKind::Generic },
    ".toml" => LanguageInfo { icon: "", color: "#A0A0A0", cterm_color: "247", name: "TOML", lang: Language::Config, kind: MatchKind::Generic },
    ".yaml" => LanguageInfo { icon: "󰰴", color: "#A0A0A0", cterm_color: "247", name: "YAML", lang: Language::Config, kind: MatchKind::Generic },
    ".yml" => LanguageInfo { icon: "󰰴", color: "#A0A0A0", cterm_color: "247", name: "YAML", lang: Language::Config, kind: MatchKind::Generic },
    ".json" => LanguageInfo { icon: "", color: "#E5E5E5", cterm_color: "254", name: "JSON", lang: Language::Config, kind: MatchKind::Generic },

    // CSS
    ".css" => LanguageInfo { icon: "", color: "#563d7c", cterm_color: "98", name: "CSS", lang: Language::Css, kind: MatchKind::Indicator },

    // D
    "dub.json" => LanguageInfo { icon: "D", color: "#ba595e", cterm_color: "131", name: "D", lang: Language::D, kind: MatchKind::ProjectManifest},
    "dub.sdl" => LanguageInfo { icon: "D", color: "#ba595e", cterm_color: "131", name: "D", lang: Language::D, kind: MatchKind::ProjectManifest},
    ".d" => LanguageInfo { icon: "D", color: "#ba595e", cterm_color: "131", name: "D", lang: Language::D, kind: MatchKind::Indicator },

    // Dart
    "pubspec.yaml" => LanguageInfo { icon: "", color: "#0175c2", cterm_color: "32", name: "Dart", lang: Language::Dart, kind: MatchKind::ProjectManifest},
    ".dart" => LanguageInfo { icon: "", color: "#0175c2", cterm_color: "32", name: "Dart", lang: Language::Dart, kind: MatchKind::Indicator },

    // Dhall
    ".dhall" => LanguageInfo { icon: "D", color: "#174B59", cterm_color: "23", name: "Dhall", lang: Language::Dhall, kind: MatchKind::Indicator },

    // Eiffel
    ".ecf" => LanguageInfo { icon: "E", color: "#0C8B95", cterm_color: "30", name: "Eiffel", lang: Language::Eiffel, kind: MatchKind::ProjectManifest},

    // Elixir
    "mix.exs" => LanguageInfo { icon: "", color: "#4b275f", cterm_color: "54", name: "Elixir", lang: Language::Elixir, kind: MatchKind::ProjectManifest},
    ".ex" => LanguageInfo { icon: "", color: "#4b275f", cterm_color: "54", name: "Elixir", lang: Language::Elixir, kind: MatchKind::Indicator },
    ".exs" => LanguageInfo { icon: "", color: "#4b275f", cterm_color: "54", name: "Elixir", lang: Language::Elixir, kind: MatchKind::Indicator },

    // Elm
    "elm.json" => LanguageInfo { icon: "", color: "#60b5cc", cterm_color: "74", name: "Elm", lang: Language::Elm, kind: MatchKind::ProjectManifest},

    // Erlang
    ".erl" => LanguageInfo { icon: "", color: "#b83998", cterm_color: "133", name: "Erlang", lang: Language::Erlang, kind: MatchKind::Indicator },
    ".hrl" => LanguageInfo { icon: "", color: "#b83998", cterm_color: "133", name: "Erlang", lang: Language::Erlang, kind: MatchKind::Indicator},

    // F#
    ".fsproj" => LanguageInfo { icon: "", color: "#378bba", cterm_color: "38", name: "F#", lang: Language::FSharp, kind: MatchKind::ProjectManifest},
    ".fs" => LanguageInfo { icon: "", color: "#378bba", cterm_color: "38", name: "F#", lang: Language::FSharp, kind: MatchKind::Indicator },
    ".fsx" => LanguageInfo { icon: "", color: "#378bba", cterm_color: "38", name: "F#", lang: Language::FSharp, kind: MatchKind::Indicator },

    // Forth
    ".fth" => LanguageInfo { icon: "F", color: "#341708", cterm_color: "52", name: "Forth", lang: Language::Forth, kind: MatchKind::Generic },

    // Fortran
    ".f90" => LanguageInfo { icon: "󱈚", color: "#734f96", cterm_color: "98", name: "Fortran", lang: Language::Fortran, kind: MatchKind::Indicator},
    ".f95" => LanguageInfo { icon: "󱈚", color: "#734f96", cterm_color: "98", name: "Fortran", lang: Language::Fortran, kind: MatchKind::Indicator},
    ".f" => LanguageInfo { icon: "󱈚", color: "#734f96", cterm_color: "98", name: "Fortran", lang: Language::Fortran, kind: MatchKind::Indicator},

    // Go
    "go.mod" => LanguageInfo { icon: "", color: "#00add8", cterm_color: "38", name: "Go", lang: Language::Go, kind: MatchKind::ProjectManifest},
    ".go" => LanguageInfo { icon: "", color: "#00add8", cterm_color: "38", name: "Go", lang: Language::Go, kind: MatchKind::Indicator },

    // Haskell
    ".cabal" => LanguageInfo { icon: "", color: "#5e5086", cterm_color: "61", name: "Haskell", lang: Language::Haskell, kind: MatchKind::ProjectManifest},
    "stack.yaml" => LanguageInfo { icon: "", color: "#5e5086", cterm_color: "61", name: "Haskell", lang: Language::Haskell, kind: MatchKind::ProjectManifest},
    "Setup.hs" => LanguageInfo { icon: "", color: "#5e5086", cterm_color: "61", name: "Haskell", lang: Language::Haskell, kind: MatchKind::Indicator },
    ".hs" => LanguageInfo { icon: "", color: "#5e5086", cterm_color: "61", name: "Haskell", lang: Language::Haskell, kind: MatchKind::Indicator },
    ".lhs" => LanguageInfo { icon: "", color: "#5e5086", cterm_color: "61", name: "Haskell", lang: Language::Haskell, kind: MatchKind::Indicator },

    // Haxe
    "haxelib.json" => LanguageInfo { icon: "H", color: "#DF7900", cterm_color: "208", name: "Haxe", lang: Language::Haxe, kind: MatchKind::ProjectManifest},
    ".hxml" => LanguageInfo { icon: "H", color: "#DF7900", cterm_color: "208", name: "Haxe", lang: Language::Haxe, kind: MatchKind::Indicator },

    // HTML
    ".html" => LanguageInfo { icon: "", color: "#e34c26", cterm_color: "196", name: "HTML", lang: Language::Html, kind: MatchKind::Indicator },
    ".htm" => LanguageInfo { icon: "", color: "#e34c26", cterm_color: "196", name: "HTML", lang: Language::Html, kind: MatchKind::Indicator },

    // Idris
    ".idr" => LanguageInfo { icon: "I", color: "#1F232D", cterm_color: "235", name: "Idris", lang: Language::Idris, kind: MatchKind::Indicator },
    ".lidr" => LanguageInfo { icon: "I", color: "#1F232D", cterm_color: "235", name: "Idris", lang: Language::Idris, kind: MatchKind::Indicator },

    // Java & JVM
    "pom.xml" => LanguageInfo { icon: "", color: "#cc0000", cterm_color: "160", name: "Java/JVM", lang: Language::Java, kind: MatchKind::ProjectManifest},
    "build.gradle" => LanguageInfo { icon: "", color: "#cc0000", cterm_color: "160", name: "Java/JVM", lang: Language::Java, kind: MatchKind::ProjectManifest },
    ".java" => LanguageInfo { icon: "", color: "#cc0000", cterm_color: "160", name: "Java", lang: Language::Java, kind: MatchKind::Indicator },

    // JavaScript / TypeScript / Node
    "package.json" => LanguageInfo { icon: "", color: "#f0db4f", cterm_color: "220", name: "JavaScript/Node", lang: Language::JavaScript, kind: MatchKind::ProjectManifest},
    "tsconfig.json" => LanguageInfo { icon: "", color: "#3178c6", cterm_color: "39", name: "TypeScript", lang: Language::TypeScript, kind: MatchKind::ProjectManifest},

    // Julia
    "Project.toml" => LanguageInfo { icon: "", color: "#a270ba", cterm_color: "140", name: "Julia", lang: Language::Julia, kind: MatchKind::ProjectManifest},

    // Kotlin
    "build.gradle.kts" => LanguageInfo { icon: "󱈙", color: "#7f52ff", cterm_color: "99", name: "Kotlin", lang: Language::Kotlin, kind: MatchKind::ProjectManifest},
    ".kt" => LanguageInfo { icon: "󱈙", color: "#7f52ff", cterm_color: "99", name: "Kotlin", lang: Language::Kotlin, kind: MatchKind::Indicator },
    ".kts" => LanguageInfo { icon: "󱈙", color: "#7f52ff", cterm_color: "99", name: "Kotlin", lang: Language::Kotlin, kind: MatchKind::Indicator },

    // LaTeX
    ".tex" => LanguageInfo { icon: "", color: "#008080", cterm_color: "30", name: "LaTeX", lang: Language::LaTeX, kind: MatchKind::Indicator},
    ".latex" => LanguageInfo { icon: "", color: "#008080", cterm_color: "30", name: "LaTeX", lang: Language::LaTeX, kind: MatchKind::Indicator},

    // Lua
    ".rockspec" => LanguageInfo { icon: "", color: "#2c2d72", cterm_color: "18", name: "Lua", lang: Language::Lua, kind: MatchKind::ProjectManifest},
    ".lua" => LanguageInfo { icon: "", color: "#2c2d72", cterm_color: "18", name: "Lua", lang: Language::Lua, kind: MatchKind::Indicator},

    // Nim
    ".nimble" => LanguageInfo { icon: "", color: "#ffc200", cterm_color: "220", name: "Nim", lang: Language::Nim, kind: MatchKind::ProjectManifest},
    ".nim" => LanguageInfo { icon: "", color: "#ffc200", cterm_color: "220", name: "Nim", lang: Language::Nim, kind: MatchKind::Indicator },

    // Nix
    "flake.nix" => LanguageInfo { icon: "", color: "#7E76D4", cterm_color: "104", name: "Nix", lang: Language::Nix, kind: MatchKind::ProjectManifest},

    // Nmap
    ".nse" => LanguageInfo { icon: "N", color: "#404040", cterm_color: "238", name: "Nmap Script", lang: Language::Nmap, kind: MatchKind::Indicator },

    // Objective-C
    "project.pbxproj" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "Objective-C", lang: Language::ObjectiveC, kind: MatchKind::Generic },
    ".m" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "Objective-C", lang: Language::ObjectiveC, kind: MatchKind::Generic },

    // OCaml
    "dune-project" => LanguageInfo { icon: "", color: "#ec6813", cterm_color: "166", name: "OCaml", lang: Language::OCaml, kind: MatchKind::ProjectManifest},
    "opam" => LanguageInfo { icon: "", color: "#ec6813", cterm_color: "166", name: "OCaml", lang: Language::OCaml, kind: MatchKind::ProjectManifest},
    ".opam" => LanguageInfo { icon: "", color: "#ec6813", cterm_color: "166", name: "OCaml", lang: Language::OCaml, kind: MatchKind::ProjectManifest},
    ".ml" => LanguageInfo { icon: "", color: "#ec6813", cterm_color: "166", name: "OCaml", lang: Language::OCaml, kind: MatchKind::Indicator },
    ".mli" => LanguageInfo { icon: "", color: "#ec6813", cterm_color: "166", name: "OCaml", lang: Language::OCaml, kind: MatchKind::Indicator },

    // Pascal / Delphi
    ".dpr" => LanguageInfo { icon: "P", color: "#e32b2f", cterm_color: "196", name: "Delphi/Pascal", lang: Language::Pascal, kind: MatchKind::Indicator },
    ".lpr" => LanguageInfo { icon: "P", color: "#EEDD34", cterm_color: "184", name: "Lazarus/Pascal", lang: Language::Pascal, kind: MatchKind::Indicator },

    // Perl
    ".pl" => LanguageInfo { icon: "", color: "#39457e", cterm_color: "61", name: "Perl", lang: Language::Perl, kind: MatchKind::Indicator},
    ".pm" => LanguageInfo { icon: "", color: "#39457e", cterm_color: "61", name: "Perl", lang: Language::Perl, kind: MatchKind::Indicator },

    // PHP
    "composer.json" => LanguageInfo { icon: "", color: "#8892be", cterm_color: "103", name: "PHP", lang: Language::Php, kind: MatchKind::ProjectManifest},
    ".php" => LanguageInfo { icon: "", color: "#8892be", cterm_color: "103", name: "PHP", lang: Language::Php, kind: MatchKind::Indicator },

    // PowerShell
    ".psd1" => LanguageInfo { icon: "󰨊", color: "#012456", cterm_color: "18", name: "PowerShell", lang: Language::PowerShell, kind: MatchKind::Indicator },

    // Prolog
    ".pro" => LanguageInfo { icon: "P", color: "#880000", cterm_color: "88", name: "Prolog", lang: Language::Prolog, kind: MatchKind::Generic },

    // Python
    "pyproject.toml" => LanguageInfo { icon: "", color: "#f6c43b", cterm_color: "203", name: "Python", lang: Language::Python, kind: MatchKind::ProjectManifest},
    "setup.cfg" => LanguageInfo { icon: "", color: "#f6c43b", cterm_color: "203", name: "Python", lang: Language::Python, kind: MatchKind::ProjectManifest},
    "Pipfile" => LanguageInfo { icon: "", color: "#f6c43b", cterm_color: "203", name: "Python", lang: Language::Python, kind: MatchKind::ProjectManifest},
    ".py" => LanguageInfo { icon: "", color: "#f6c43b", cterm_color: "203", name: "Python", lang: Language::Python, kind: MatchKind::Indicator },
    ".pyx" => LanguageInfo { icon: "", color: "#f6c43b", cterm_color: "203", name: "Cython", lang: Language::Python, kind: MatchKind::Indicator },

    // R
    ".r" => LanguageInfo { icon: "󰟔", color: "#276dc2", cterm_color: "33", name: "R", lang: Language::R, kind: MatchKind::Indicator },

    // Racket
    "info.rkt" => LanguageInfo { icon: "R", color: "#394FAC", cterm_color: "61", name: "Racket", lang: Language::Racket, kind: MatchKind::ProjectManifest},

    // Ruby
    "Gemfile" => LanguageInfo { icon: "", color: "#cc342d", cterm_color: "160", name: "Ruby", lang: Language::Ruby, kind: MatchKind::ProjectManifest},
    ".gemspec" => LanguageInfo { icon: "", color: "#cc342d", cterm_color: "160", name: "Ruby", lang: Language::Ruby, kind: MatchKind::ProjectManifest},
    ".rb" => LanguageInfo { icon: "", color: "#cc342d", cterm_color: "160", name: "Ruby", lang: Language::Ruby, kind: MatchKind::Indicator },

    // Rust
    "Cargo.toml" => LanguageInfo { icon: "", color: "#dea584", cterm_color: "173", name: "Rust", lang: Language::Rust, kind: MatchKind::ProjectManifest},
    ".rs" => LanguageInfo { icon: "", color: "#dea584", cterm_color: "173", name: "Rust", lang: Language::Rust, kind: MatchKind::Indicator },

    // Scala
    "build.sbt" => LanguageInfo { icon: "", color: "#c22d40", cterm_color: "160", name: "Scala", lang: Language::Scala, kind: MatchKind::ProjectManifest},

    // Shell
    ".sh" => LanguageInfo { icon: "", color: "#89E051", cterm_color: "113", name: "Shell", lang: Language::Shell, kind: MatchKind::Indicator},
    ".bash" => LanguageInfo { icon: "", color: "#89E051", cterm_color: "113", name: "Bash", lang: Language::Shell, kind: MatchKind::Indicator },
    ".csh" => LanguageInfo { icon: "", color: "#89E051", cterm_color: "113", name: "Csh", lang: Language::Shell, kind: MatchKind::Indicator },
    ".tcsh" => LanguageInfo { icon: "", color: "#89E051", cterm_color: "113", name: "Tcsh", lang: Language::Shell, kind: MatchKind::Indicator },
    ".fish" => LanguageInfo { icon: "", color: "#89E051", cterm_color: "113", name: "Fish", lang: Language::Shell, kind: MatchKind::Indicator },
    ".ksh" => LanguageInfo { icon: "", color: "#89E051", cterm_color: "113", name: "Ksh", lang: Language::Shell, kind: MatchKind::Indicator },
    ".zsh" => LanguageInfo { icon: "", color: "#89E051", cterm_color: "113", name: "Zsh", lang: Language::Shell, kind: MatchKind::Indicator },

    // SmallTalk
    ".st" => LanguageInfo { icon: "S", color: "#596700", cterm_color: "58", name: "SmallTalk", lang: Language::SmallTalk, kind: MatchKind::Indicator },

    // Swift
    "Package.swift" => LanguageInfo { icon: "", color: "#ffac45", cterm_color: "215", name: "Swift", lang: Language::Swift, kind: MatchKind::ProjectManifest},

    // Tcl
    ".tcl" => LanguageInfo { icon: "T", color: "#1D529D", cterm_color: "25", name: "Tcl", lang: Language::Tcl, kind: MatchKind::Indicator},

    // Unison
    ".u" => LanguageInfo { icon: "U", color: "#FF7A62", cterm_color: "209", name: "Unison", lang: Language::Unison, kind: MatchKind::Indicator },

    // Vala
    ".vala" => LanguageInfo { icon: "", color: "#7239B3", cterm_color: "98", name: "Vala", lang: Language::Vala, kind: MatchKind::Indicator},

    // Verilog
    ".v" => LanguageInfo { icon: "V", color: "#000080", cterm_color: "18", name: "Verilog", lang: Language::Verilog, kind: MatchKind::Generic },
    ".vh" => LanguageInfo { icon: "V", color: "#000080", cterm_color: "18", name: "Verilog", lang: Language::Verilog, kind: MatchKind::Generic },
    ".sv" => LanguageInfo { icon: "V", color: "#000080", cterm_color: "18", name: "Verilog", lang: Language::Verilog, kind: MatchKind::Generic},

    // VHDL
    ".vhd" => LanguageInfo { icon: "V", color: "#AD0000", cterm_color: "88", name: "VHDL", lang: Language::Vhdl, kind: MatchKind::Indicator},
    ".vhdl" => LanguageInfo { icon: "V", color: "#AD0000", cterm_color: "88", name: "VHDL", lang: Language::Vhdl, kind: MatchKind::Indicator},

    // Zig
    "build.zig.zon" => LanguageInfo { icon: "", color: "#f7a41d", cterm_color: "214", name: "Zig", lang: Language::Zig, kind: MatchKind::ProjectManifest},
    "build.zig" => LanguageInfo { icon: "", color: "#f7a41d", cterm_color: "214", name: "Zig", lang: Language::Zig, kind: MatchKind::ProjectManifest},
    ".zig" => LanguageInfo { icon: "", color: "#f7a41d", cterm_color: "214", name: "Zig", lang: Language::Zig, kind: MatchKind::Indicator },
};

/// Heuristic used to locate the version string inside a manifest file's content.
#[derive(Clone, Copy)]
enum VersionStrategy {
    /// Search for `key = "..."` only inside specific TOML table sections (in priority order).
    /// An empty section name `""` matches keys before the first `[table]` header.
    TomlSectionQuoted(&'static [&'static str], &'static str),

    /// Search for `key = value` (unquoted) inside specific INI sections.
    IniSectionUnquoted(&'static [&'static str], &'static str),

    /// Search for a key, then a separator, then extract the quoted value.
    /// Ignores whitespace between key, separator, and value.
    /// e.g. `KeyValueQuoted("\"version\"", ":")` for JSON.
    KeyValueQuoted(&'static str, &'static str),

    /// Find the first line whose start matches `prefix`, then extract the first `"…"` on it.
    LinePrefixQuoted(&'static str),

    /// Find the first line whose start matches `prefix`, take the bare value after it
    /// (leading whitespace stripped) up to the first whitespace, `)`, or `#`.
    /// e.g. `"version:"` in .cabal / pubspec.yaml, `"(version "` in dune-project.
    LinePrefixUnquoted(&'static str),

    /// Extract Maven `<project><version>` at depth 1, ignoring `<parent>` / `<dependencies>`,
    /// falling back to `<parent><version>` only when the project inherits its version.
    PomXmlVersion,

    /// Search for `VERSION` inside a CMake project() declaration.
    CMakeVersion,

    /// Search for `<Version>` or `<PackageVersion>` in MSBuild projects.
    MSBuildVersion,

    /// Search for `.version` in Ruby gemspecs.
    RubyGemspec,
}

/// Map a `Language` (with an optional key for disambiguation) to the appropriate
/// extraction strategy.  Returns `None` for manifests that carry no self-declared
/// version (go.mod, stack.yaml, Gemfile, Pipfile, …).
fn project_version_strategy(lang: Language, key: &str) -> Option<VersionStrategy> {
    use Language::*;
    match (lang, key) {
        (Rust, _) => Some(VersionStrategy::TomlSectionQuoted(
            &["package", "workspace.package"],
            "version",
        )),
        (Julia, _) => Some(VersionStrategy::TomlSectionQuoted(&[""], "version")),
        (Nim, _) => Some(VersionStrategy::LinePrefixQuoted("version")),
        (Python, "pyproject.toml") => Some(VersionStrategy::TomlSectionQuoted(
            &["project", "tool.poetry", "tool.flit.metadata"],
            "version",
        )),
        (Python, "setup.cfg") => Some(VersionStrategy::IniSectionUnquoted(
            &["metadata"],
            "version",
        )),
        (Lua, ".rockspec") => Some(VersionStrategy::LinePrefixQuoted("version")),
        (CCpp, "CMakeLists.txt") => Some(VersionStrategy::CMakeVersion),
        (D, "dub.sdl") => Some(VersionStrategy::LinePrefixQuoted("version")),
        (D, "dub.json") => Some(VersionStrategy::KeyValueQuoted("\"version\"", ":")),
        (OCaml, "opam") | (OCaml, ".opam") => Some(VersionStrategy::LinePrefixQuoted("version")),
        (OCaml, "dune-project") => Some(VersionStrategy::LinePrefixUnquoted("(version ")),
        (Elixir, _) => Some(VersionStrategy::KeyValueQuoted("version", ":")),
        (JavaScript, _) | (Php, _) | (Haxe, _) | (Elm, _) => {
            Some(VersionStrategy::KeyValueQuoted("\"version\"", ":"))
        }
        (Ruby, ".gemspec") => Some(VersionStrategy::RubyGemspec),
        (Scala, _) => Some(VersionStrategy::KeyValueQuoted("version", ":=")),
        (CommonLisp, _) => Some(VersionStrategy::LinePrefixQuoted(":version")),
        (Racket, _) => Some(VersionStrategy::LinePrefixQuoted("(define version")),
        (Haskell, ".cabal") | (Dart, _) => Some(VersionStrategy::LinePrefixUnquoted("version:")),
        (Java, "pom.xml") => Some(VersionStrategy::PomXmlVersion),
        (Java, "build.gradle") => Some(VersionStrategy::KeyValueQuoted("version", "=")),
        (Kotlin, "build.gradle.kts") => Some(VersionStrategy::KeyValueQuoted("version", "=")),
        (CSharp, _) | (FSharp, _) => Some(VersionStrategy::MSBuildVersion),
        (Zig, "build.zig.zon") => Some(VersionStrategy::KeyValueQuoted(".version", "=")),
        // Everything else (Go, Swift, Nix, Gemfile, Pipfile, …)
        // has no self-declared version we can reliably extract.
        _ => None,
    }
}

/// Extract the first non-empty string enclosed in `"` or `'` from `s`.
#[inline]
fn extract_first_quoted(s: &str) -> Option<SmolStr> {
    let q_open = s.find(['"', '\''])?;
    let quote_char = s.as_bytes()[q_open] as char;
    let rest = &s[q_open + 1..];
    let q_end = rest.find(quote_char)?;
    let v = &rest[..q_end];
    (!v.is_empty()).then(|| SmolStr::new(v))
}

fn parse_section_header(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('[') {
        return None;
    }
    let without_comment = trimmed.split('#').next()?.trim_end();
    if let Some(inner) = without_comment
        .strip_prefix("[[")
        .and_then(|s| s.strip_suffix("]]"))
    {
        return Some(inner.trim());
    }
    without_comment
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .map(str::trim)
}

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

fn extract_pom_version(content: &str) -> Option<SmolStr> {
    let mut rest = content;
    let mut depth: usize = 0;
    let mut in_parent = false;
    let mut parent_version: Option<SmolStr> = None;

    while let Some(lt) = rest.find('<') {
        rest = &rest[lt..];

        if let Some(after_comment) = rest.strip_prefix("<!--") {
            let end = after_comment.find("-->")?;
            rest = &after_comment[end + 3..];
            continue;
        }
        if rest.starts_with("<?") || rest.starts_with("<!") {
            let gt = rest.find('>')?;
            rest = &rest[gt + 1..];
            continue;
        }

        let gt = rest.find('>')?;
        let tag_body = rest[1..gt].trim();
        rest = &rest[gt + 1..];

        if let Some(closing) = tag_body.strip_prefix('/') {
            let closing_name = closing.split_whitespace().next().unwrap_or("");
            depth = depth.saturating_sub(1);
            if depth == 1 && closing_name == "parent" {
                in_parent = false;
            }
            continue;
        }

        if tag_body.ends_with('/') {
            continue;
        }

        let tag_name = tag_body.split_whitespace().next().unwrap_or("");
        if depth == 1 && tag_name == "version" {
            let close_idx = rest.find("</version>")?;
            let v = rest[..close_idx].trim();
            rest = &rest[close_idx + "</version>".len()..];
            if !v.is_empty() && !v.starts_with("${") {
                return Some(SmolStr::new(v));
            }
            continue;
        }

        if depth == 1 && tag_name == "parent" {
            in_parent = true;
            depth += 1;
            continue;
        }

        if depth == 2 && in_parent && tag_name == "version" {
            let close_idx = rest.find("</version>")?;
            let v = rest[..close_idx].trim();
            rest = &rest[close_idx + "</version>".len()..];
            if !v.is_empty() && !v.starts_with("${") && parent_version.is_none() {
                parent_version = Some(SmolStr::new(v));
            }
            continue;
        }

        depth += 1;
    }

    parent_version
}

/// Apply `strategy` to the raw text of a manifest file and return the version string,
/// or `None` if the pattern is not found or the matched value is empty.
fn extract_version(strategy: VersionStrategy, content: &str) -> Option<SmolStr> {
    match strategy {
        VersionStrategy::TomlSectionQuoted(sections, key) => {
            let mut current_section = "";
            let mut in_ml_single = false;
            let mut in_ml_double = false;
            let mut fallback: Option<(usize, SmolStr)> = None;

            for line in content.lines() {
                if in_ml_single {
                    if count_occurrences(line, "'''") % 2 == 1 {
                        in_ml_single = false;
                    }
                    continue;
                }
                if in_ml_double {
                    if count_occurrences(line, "\"\"\"") % 2 == 1 {
                        in_ml_double = false;
                    }
                    continue;
                }

                let trimmed = line.trim_start();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }

                if let Some(sec) = parse_section_header(trimmed) {
                    current_section = sec;
                    continue;
                }

                if let Some(prio) = sections.iter().position(|&s| s == current_section) {
                    if let Some(rest) = trimmed.strip_prefix(key) {
                        let rest = rest.trim_start();
                        if let Some(rest) = rest.strip_prefix('=') {
                            if let Some(v) = extract_first_quoted(rest.trim_start()) {
                                if prio == 0 {
                                    return Some(v);
                                }
                                if fallback.as_ref().is_none_or(|(best_p, _)| prio < *best_p) {
                                    fallback = Some((prio, v));
                                }
                            }
                        }
                    }
                }

                if count_occurrences(line, "'''") % 2 == 1 {
                    in_ml_single = true;
                } else if count_occurrences(line, "\"\"\"") % 2 == 1 {
                    in_ml_double = true;
                }
            }

            fallback.map(|(_, v)| v)
        }

        VersionStrategy::IniSectionUnquoted(sections, key) => {
            let mut current_section = "";
            for line in content.lines() {
                let trimmed = line.trim_start();
                if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
                    continue;
                }
                if let Some(sec) = parse_section_header(trimmed) {
                    current_section = sec;
                    continue;
                }
                if sections.contains(&current_section) {
                    if let Some(rest) = trimmed.strip_prefix(key) {
                        let rest = rest.trim_start();
                        if let Some(rest) = rest.strip_prefix('=') {
                            let rest = rest.trim_start();
                            let end = rest
                                .find(|c: char| c.is_whitespace() || c == ')' || c == '#')
                                .unwrap_or(rest.len());
                            let v = &rest[..end];
                            if !v.is_empty() {
                                return Some(SmolStr::new(v));
                            }
                        }
                    }
                }
            }
            None
        }

        VersionStrategy::KeyValueQuoted(key, sep) => {
            for line in content.lines() {
                let line = line.trim_start();
                if let Some(rest) = line.strip_prefix(key) {
                    let rest = rest.trim_start();
                    if let Some(rest) = rest.strip_prefix(sep) {
                        if let Some(v) = extract_first_quoted(rest.trim_start()) {
                            return Some(v);
                        }
                    }
                }
            }
            None
        }

        VersionStrategy::LinePrefixQuoted(prefix) => {
            for line in content.lines() {
                let line = line.trim_start();
                if !line.starts_with(prefix) {
                    continue;
                }
                if let Some(v) = extract_first_quoted(line) {
                    return Some(v);
                }
            }
            None
        }

        VersionStrategy::LinePrefixUnquoted(prefix) => {
            for line in content.lines() {
                let line = line.trim_start();
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

        VersionStrategy::PomXmlVersion => extract_pom_version(content),

        VersionStrategy::CMakeVersion => {
            let project_idx = content
                .find("project(")
                .or_else(|| content.find("project ("))?;
            let rest = &content[project_idx..];
            let end_idx = rest.find(')')?;
            let project_decl = &rest[..end_idx];
            let version_idx = project_decl.find("VERSION")?;
            let mut parts = project_decl[version_idx + "VERSION".len()..].split_whitespace();
            let v = parts.next()?;
            Some(SmolStr::new(v))
        }

        VersionStrategy::MSBuildVersion => {
            for tag in &["<Version>", "<PackageVersion>"] {
                if let Some(start) = content.find(tag) {
                    let start = start + tag.len();
                    let rest = &content[start..];
                    if let Some(end) = rest.find('<') {
                        let v = rest[..end].trim();
                        if !v.is_empty() {
                            return Some(SmolStr::new(v));
                        }
                    }
                }
            }
            None
        }

        VersionStrategy::RubyGemspec => {
            for line in content.lines() {
                let line = line.trim_start();
                if let Some(idx) = line.find(".version") {
                    let rest = line[idx + ".version".len()..].trim_start();
                    if let Some(rest) = rest.strip_prefix('=') {
                        if let Some(v) = extract_first_quoted(rest.trim_start()) {
                            return Some(v);
                        }
                    }
                }
            }
            None
        }
    }
}

/// Read `path` from disk and attempt to extract the version using the strategy
/// determined by the `Language` and the lookup `key`.
fn version_from_file(path: &std::path::Path, lang: Language, key: &str) -> Option<SmolStr> {
    let strategy = project_version_strategy(lang, key)?;
    let content = std::fs::read_to_string(path).ok()?;
    extract_version(strategy, &content)
}

const CACHE_TTL_SECS: u64 = 3600; // 1 hour (auto-invalidated on binary/shim/env/toolchain change)

fn get_cache_path() -> &'static std::path::Path {
    static CACHE_PATH: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();
    CACHE_PATH.get_or_init(|| {
        #[cfg(unix)]
        let id = unsafe { libc::getuid() }.to_string();
        #[cfg(not(unix))]
        let id = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "default".to_string());
        std::env::temp_dir().join(format!("auraline-manifest-cache-{id}.txt"))
    })
}

fn tool_binary_name(lang: Language) -> Option<&'static str> {
    match lang {
        Language::Rust => Some("rustc"),
        Language::Go => Some("go"),
        Language::Python => Some("python"),
        Language::JavaScript | Language::TypeScript => Some("node"),
        Language::Java => Some("javac"),
        Language::Ruby => Some("ruby"),
        Language::Php => Some("php"),
        Language::Dart => Some("dart"),
        Language::Julia => Some("julia"),
        Language::Zig => Some("zig"),
        Language::Nim => Some("nim"),
        Language::Elixir => Some("elixir"),
        Language::CSharp | Language::FSharp => Some("dotnet"),
        Language::CCpp => Some("cmake"),
        Language::Haskell => Some("ghc"),
        Language::OCaml => Some("ocamlc"),
        Language::Kotlin => Some("kotlin"),
        Language::Scala => Some("scala"),
        Language::Lua => Some("lua"),
        Language::D => Some("dmd"),
        _ => None,
    }
}

fn hash_file_stamp(path: &std::path::Path, hasher: &mut impl std::hash::Hasher) -> bool {
    use std::hash::Hash;
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    path.hash(hasher);
    meta.len().hash(hasher);
    if let Ok(mtime) = meta.modified() {
        if let Ok(dur) = mtime.duration_since(std::time::UNIX_EPOCH) {
            dur.as_nanos().hash(hasher);
        }
    }
    true
}

fn hash_ancestor_override(
    start: &std::path::Path,
    candidates: &[&str],
    hasher: &mut impl std::hash::Hasher,
) {
    let mut cur = Some(start);
    while let Some(dir) = cur {
        for name in candidates {
            let p = dir.join(name);
            if hash_file_stamp(&p, hasher) {
                return;
            }
        }
        cur = dir.parent();
    }
}

/// Computes a toolchain fingerprint key for `lang` based on the resolved compiler
/// binary + mtime, active version-manager environment variables, and any local
/// toolchain override file (e.g. `rust-toolchain.toml`, `.nvmrc`, `.tool-versions`).
/// Repositories sharing the same toolchain reuse a single cached entry across directories.
fn cache_key_for_lang(lang: Language) -> SmolStr {
    let cwd = std::env::current_dir().ok();
    cache_key_for_lang_in(lang, cwd.as_deref())
}

fn cache_key_for_lang_in(lang: Language, cwd: Option<&std::path::Path>) -> SmolStr {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();

    if let Some(bin_name) = tool_binary_name(lang) {
        let resolved = crate::cmd::resolve_in_path(bin_name).or_else(|| {
            (lang == Language::Python).then(|| crate::cmd::resolve_in_path("python3"))?
        });
        if let Some(bin_path) = resolved {
            hash_file_stamp(&bin_path, &mut hasher);
        }
    }

    for var in ["ASDF_DIR", "MISE_SHELL", "PROTO_HOME"] {
        if let Some(v) = std::env::var_os(var) {
            var.hash(&mut hasher);
            v.hash(&mut hasher);
        }
    }

    match lang {
        Language::Rust => {
            if let Some(tc) = std::env::var_os("RUSTUP_TOOLCHAIN") {
                tc.hash(&mut hasher);
            }
            let rustup_settings = std::env::var_os("RUSTUP_HOME")
                .map(std::path::PathBuf::from)
                .or_else(|| {
                    std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".rustup"))
                })
                .map(|p| p.join("settings.toml"));
            if let Some(settings) = rustup_settings {
                hash_file_stamp(&settings, &mut hasher);
            }
            if let Some(dir) = cwd {
                hash_ancestor_override(
                    dir,
                    &["rust-toolchain.toml", "rust-toolchain"],
                    &mut hasher,
                );
            }
        }
        Language::Python => {
            for var in ["VIRTUAL_ENV", "CONDA_PREFIX", "PYENV_VERSION"] {
                if let Some(v) = std::env::var_os(var) {
                    var.hash(&mut hasher);
                    v.hash(&mut hasher);
                }
            }
            if let Some(dir) = cwd {
                hash_ancestor_override(dir, &[".python-version"], &mut hasher);
            }
        }
        Language::JavaScript | Language::TypeScript => {
            for var in [
                "NVM_BIN",
                "FNM_MULTISHELL_PATH",
                "VOLTA_HOME",
                "NODE_VERSION",
            ] {
                if let Some(v) = std::env::var_os(var) {
                    var.hash(&mut hasher);
                    v.hash(&mut hasher);
                }
            }
            if let Some(dir) = cwd {
                hash_ancestor_override(dir, &[".nvmrc", ".node-version"], &mut hasher);
            }
        }
        Language::Ruby => {
            for var in ["RBENV_VERSION", "GEM_HOME"] {
                if let Some(v) = std::env::var_os(var) {
                    var.hash(&mut hasher);
                    v.hash(&mut hasher);
                }
            }
            if let Some(dir) = cwd {
                hash_ancestor_override(dir, &[".ruby-version"], &mut hasher);
            }
        }
        Language::Go => {
            for var in ["GOROOT", "GOTOOLCHAIN"] {
                if let Some(v) = std::env::var_os(var) {
                    var.hash(&mut hasher);
                    v.hash(&mut hasher);
                }
            }
        }
        Language::Java | Language::Kotlin | Language::Scala => {
            if let Some(v) = std::env::var_os("JAVA_HOME") {
                v.hash(&mut hasher);
            }
        }
        Language::Elixir => {
            if let Some(v) = std::env::var_os("MIX_HOME") {
                v.hash(&mut hasher);
            }
        }
        _ => {}
    }

    if let Some(dir) = cwd {
        for f in [".tool-versions", "mise.toml", ".mise.toml"] {
            hash_file_stamp(&dir.join(f), &mut hasher);
        }
    }

    format_smolstr!("{:?}@{:x}", lang, hasher.finish())
}

fn read_cached_version(key: &str) -> Option<SmolStr> {
    let cache_path = get_cache_path();
    let content = std::fs::read_to_string(cache_path).ok()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();

    for line in content.lines() {
        let mut parts = line.splitn(3, ':');
        let name = parts.next()?;
        let version = parts.next()?;
        let timestamp_str = parts.next()?;
        if name == key {
            let timestamp: u64 = timestamp_str.parse().ok()?;
            if now >= timestamp && now - timestamp < CACHE_TTL_SECS {
                return Some(SmolStr::new(version));
            }
        }
    }
    None
}

fn write_cached_version(key: &str, version: &str) -> Option<()> {
    let cache_path = get_cache_path();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();

    let mut lines = Vec::new();

    if let Ok(content) = std::fs::read_to_string(cache_path) {
        for line in content.lines() {
            let mut parts = line.splitn(3, ':');
            let (Some(name), Some(_ver), Some(ts_str)) = (parts.next(), parts.next(), parts.next())
            else {
                continue;
            };
            if name != key {
                if let Ok(ts) = ts_str.parse::<u64>() {
                    if now >= ts && now - ts < CACHE_TTL_SECS {
                        lines.push(line.to_string());
                    }
                }
            }
        }
    }

    lines.push(format!("{}:{}:{}", key, version, now));

    let temp_path = cache_path.with_extension(format!("tmp.{}", std::process::id()));
    std::fs::write(&temp_path, lines.join("\n")).ok()?;
    if std::fs::rename(&temp_path, cache_path).is_err() {
        let _ = std::fs::remove_file(&temp_path);
        return None;
    }

    Some(())
}

async fn build_system_version(lang: Language) -> Option<SmolStr> {
    let key = cache_key_for_lang(lang);
    if let Some(cached) = read_cached_version(&key) {
        return Some(cached);
    }

    let version = build_system_version_uncached(lang).await?;
    write_cached_version(&key, &version);
    Some(version)
}

async fn build_system_version_uncached(lang: Language) -> Option<SmolStr> {
    use crate::cmd::CMD;
    match lang {
        Language::Rust => {
            let out = CMD.exec("rustc", ["--version"]).await?;
            let v = out.split_whitespace().nth(1)?;
            Some(SmolStr::new(v))
        }
        Language::Go => {
            let out = CMD.exec("go", ["version"]).await?;
            let v = out.split_whitespace().nth(2)?;
            Some(SmolStr::new(v.strip_prefix("go").unwrap_or(v)))
        }
        Language::Python => {
            let out = match CMD.exec("python", ["--version"]).await {
                Some(out) => out,
                None => CMD.exec("python3", ["--version"]).await?,
            };
            let v = out.split_whitespace().nth(1)?;
            Some(SmolStr::new(v))
        }
        Language::JavaScript | Language::TypeScript => {
            let out = CMD.exec("node", ["--version"]).await?;
            Some(SmolStr::new(out.strip_prefix('v').unwrap_or(&out)))
        }
        Language::Java => {
            let out = CMD.exec("javac", ["--version"]).await?;
            let v = out.split_whitespace().nth(1)?;
            Some(SmolStr::new(v))
        }
        Language::Ruby => {
            let out = CMD.exec("ruby", ["--version"]).await?;
            let v = out.split_whitespace().nth(1)?;
            Some(SmolStr::new(v))
        }
        Language::Php => {
            let out = CMD.exec("php", ["--version"]).await?;
            let v = out.split_whitespace().nth(1)?;
            Some(SmolStr::new(v))
        }
        Language::Dart => {
            let out = CMD.exec("dart", ["--version"]).await?;
            let v = out.split_whitespace().nth(3)?;
            Some(SmolStr::new(v))
        }
        Language::Julia => {
            let out = CMD.exec("julia", ["--version"]).await?;
            let v = out.split_whitespace().nth(2)?;
            Some(SmolStr::new(v))
        }
        Language::Zig => {
            let out = CMD.exec("zig", ["version"]).await?;
            Some(out)
        }
        Language::Nim => {
            let out = CMD.exec("nim", ["--version"]).await?;
            let v = out.split_whitespace().nth(3)?;
            Some(SmolStr::new(v))
        }
        Language::Elixir => {
            let out = CMD.exec("elixir", ["--version"]).await?;
            let v = out.split_whitespace().last()?;
            Some(SmolStr::new(v))
        }
        Language::CSharp | Language::FSharp => {
            let out = CMD.exec("dotnet", ["--version"]).await?;
            Some(SmolStr::new(out))
        }
        Language::CCpp => {
            let out = CMD.exec("cmake", ["--version"]).await?;
            let v = out.split_whitespace().nth(2)?;
            Some(SmolStr::new(v))
        }
        Language::Haskell => {
            let out = CMD.exec("ghc", ["--numeric-version"]).await?;
            Some(SmolStr::new(out))
        }
        Language::OCaml => {
            let out = CMD.exec("ocamlc", ["--version"]).await?;
            Some(SmolStr::new(out))
        }
        Language::Kotlin => {
            let out = CMD.exec("kotlin", ["-version"]).await?;
            let v = out.split_whitespace().nth(2)?;
            Some(SmolStr::new(v.strip_suffix("-jre").unwrap_or(v)))
        }
        Language::Scala => {
            let out = CMD.exec("scala", ["-version"]).await?;
            let v = out.split_whitespace().nth(4)?;
            Some(SmolStr::new(v))
        }
        Language::Lua => {
            let out = CMD.exec("lua", ["-v"]).await?;
            let v = out.split_whitespace().nth(1)?;
            Some(SmolStr::new(v))
        }
        Language::D => {
            let out = CMD.exec("dmd", ["--version"]).await?;
            let v = out.split_whitespace().nth(3)?;
            Some(SmolStr::new(v))
        }
        _ => None,
    }
}

pub async fn show(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.manifest {
        return None;
    }

    let entries = std::fs::read_dir(".").ok()?;
    let mut top_kind: Option<MatchKind> = None;
    let mut top_icons: SmallVec<[&'static str; 8]> = SmallVec::new();
    // All ProjectManifest entries found during the scan, in filesystem order.
    // We will try each in sequence and return the first that yields a version.
    let mut manifests: SmallVec<[(std::path::PathBuf, Language, SmolStr); 4]> = SmallVec::new();

    for entry_res in entries {
        let Ok(entry) = entry_res else {
            continue;
        };
        let file_name = entry.file_name();
        // Skip non-UTF-8 names (continue instead of aborting the whole scan).
        let Some(name) = file_name.to_str() else {
            continue;
        };
        // Skip hidden / dot-files.
        if name.starts_with('.') {
            continue;
        }

        let matched = if let Some(lang) = MANIFEST_MAP.get(name) {
            Some((lang, name))
        } else if let Some(dot_idx) = name.rfind('.') {
            let ext_key = &name[dot_idx..];
            MANIFEST_MAP.get(ext_key).map(|lang| (lang, ext_key))
        } else {
            None
        };

        if let Some((lang, key)) = matched {
            if lang.kind == MatchKind::ProjectManifest {
                manifests.push((entry.path(), lang.lang, SmolStr::new(key)));
            }

            let label = opts.select_str(lang.name, lang.icon);

            match top_kind {
                None => {
                    top_kind = Some(lang.kind);
                    top_icons.push(label);
                }
                Some(cur_top) if lang.kind > cur_top => {
                    top_kind = Some(lang.kind);
                    top_icons.clear();
                    top_icons.push(label);
                }
                Some(cur_top) if lang.kind == cur_top && !top_icons.contains(&label) => {
                    top_icons.push(label);
                }
                _ => {}
            }
        }
    }

    let top_kind = top_kind?;
    top_icons.sort_unstable();

    // Concatenate deduplicated icons/names into a SmolStr.
    let mut builder = SmolStrBuilder::new();
    for (idx, icon) in top_icons.iter().enumerate() {
        if idx > 0 && !opts.nerd_font {
            builder.push('/');
        }
        builder.push_str(icon);
    }
    let icons = builder.finish();

    // Attempt version extraction only when the top tier is ProjectManifest.
    let version = if top_kind == MatchKind::ProjectManifest && !manifests.is_empty() {
        let primary_lang = manifests[0].1;

        // Overlap tool subprocess (if cache miss) with synchronous local manifest parsing.
        let (tool_version, found) = tokio::join!(build_system_version(primary_lang), async {
            manifests
                .iter()
                .find_map(|(path, lang, key)| version_from_file(path, *lang, key))
        });

        match (tool_version, found) {
            (Some(tv), Some(pv)) => Some(format_smolstr!("{} ↦ v{}", tv, pv)),
            (Some(tv), None) => Some(tv),
            (None, Some(pv)) => Some(format_smolstr!("v{}", pv)),
            (None, None) => None,
        }
    } else {
        None
    };

    Some(match version {
        Some(ver) => Chunk::new(icons, ver),
        None => Chunk::icon(icons),
    })
}

#[cfg(test)]
mod tests {
    use super::{extract_version, VersionStrategy};

    #[test]
    fn test_extract_version_utf8_before_quote() {
        // Non-ASCII characters between key/prefix and the opening quote must not
        // skew byte-vs-char indexing or panic.
        let content = "version = /* versione 🚀 */ \"1.2.3\"\n";
        assert_eq!(
            extract_version(VersionStrategy::LinePrefixQuoted("version"), content).as_deref(),
            Some("1.2.3")
        );
        assert_eq!(
            extract_version(VersionStrategy::KeyValueQuoted("version", "="), content).as_deref(),
            Some("1.2.3")
        );
    }

    #[test]
    fn test_cargo_toml_ignores_dependency_tables_and_supports_workspace() {
        let strat =
            VersionStrategy::TomlSectionQuoted(&["package", "workspace.package"], "version");

        let dep_first = r#"
[dependencies.tokio]
version = "1.54.0"

[package]
name = "demo"
version = "0.4.2"
"#;
        assert_eq!(extract_version(strat, dep_first).as_deref(), Some("0.4.2"));

        let inherited = r#"
[workspace.package]
version = "2.1.0"

[package]
name = "member"
version.workspace = true

[dependencies.serde]
version = "1.0.0"
"#;
        assert_eq!(extract_version(strat, inherited).as_deref(), Some("2.1.0"));

        let virtual_ws = r#"
[workspace]
members = ["a"]

[workspace.dependencies.foo]
version = "9.9.9"
"#;
        assert_eq!(extract_version(strat, virtual_ws), None);
    }

    #[test]
    fn test_pom_xml_skips_parent_version_unless_inherited() {
        let with_own = r#"
<project>
    <parent>
        <groupId>org.springframework.boot</groupId>
        <artifactId>spring-boot-starter-parent</artifactId>
        <version>3.2.4</version>
    </parent>
    <artifactId>my-service</artifactId>
    <version>0.8.1</version>
    <dependencies>
        <dependency><version>1.0.0</version></dependency>
    </dependencies>
</project>
"#;
        assert_eq!(
            extract_version(VersionStrategy::PomXmlVersion, with_own).as_deref(),
            Some("0.8.1")
        );

        let inherited_only = r#"
<project>
    <parent>
        <groupId>com.acme</groupId>
        <artifactId>acme-parent</artifactId>
        <version>4.0.0</version>
    </parent>
    <artifactId>acme-child</artifactId>
</project>
"#;
        assert_eq!(
            extract_version(VersionStrategy::PomXmlVersion, inherited_only).as_deref(),
            Some("4.0.0")
        );
    }

    #[test]
    fn test_cache_key_shared_across_dirs_unless_toolchain_override_exists() {
        use super::{cache_key_for_lang_in, Language};

        let base =
            std::env::temp_dir().join(format!("auraline-manifest-key-test-{}", std::process::id()));
        let repo_a = base.join("repo_a");
        let repo_b = base.join("repo_b");
        let repo_pinned = base.join("repo_pinned");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&repo_a).unwrap();
        std::fs::create_dir_all(&repo_b).unwrap();
        std::fs::create_dir_all(&repo_pinned).unwrap();
        std::fs::write(
            repo_pinned.join("rust-toolchain.toml"),
            "[toolchain]\nchannel = \"nightly\"\n",
        )
        .unwrap();

        let key_a = cache_key_for_lang_in(Language::Rust, Some(&repo_a));
        let key_b = cache_key_for_lang_in(Language::Rust, Some(&repo_b));
        let key_pinned = cache_key_for_lang_in(Language::Rust, Some(&repo_pinned));

        assert_eq!(key_a, key_b);
        assert_ne!(key_a, key_pinned);

        let _ = std::fs::remove_dir_all(&base);
    }
}
