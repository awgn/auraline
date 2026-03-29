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
    "CMakeLists.txt" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", lang: Language::CCpp, kind: MatchKind::Generic },
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
    "build.gradle" => LanguageInfo { icon: "", color: "#cc0000", cterm_color: "160", name: "Java/JVM", lang: Language::Java, kind: MatchKind::Generic },
    ".java" => LanguageInfo { icon: "", color: "#cc0000", cterm_color: "160", name: "Java", lang: Language::Java, kind: MatchKind::Indicator },

    // JavaScript / TypeScript / Node
    "package.json" => LanguageInfo { icon: "", color: "#f0db4f", cterm_color: "220", name: "JavaScript/Node", lang: Language::JavaScript, kind: MatchKind::ProjectManifest},
    "tsconfig.json" => LanguageInfo { icon: "", color: "#3178c6", cterm_color: "39", name: "TypeScript", lang: Language::TypeScript, kind: MatchKind::ProjectManifest},

    // Julia
    "Project.toml" => LanguageInfo { icon: "", color: "#a270ba", cterm_color: "140", name: "Julia", lang: Language::Julia, kind: MatchKind::ProjectManifest},

    // Kotlin
    "build.gradle.kts" => LanguageInfo { icon: "󱈙", color: "#7f52ff", cterm_color: "99", name: "Kotlin", lang: Language::Kotlin, kind: MatchKind::Indicator},
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
    "build.zig" => LanguageInfo { icon: "", color: "#f7a41d", cterm_color: "214", name: "Zig", lang: Language::Zig, kind: MatchKind::ProjectManifest},
    ".zig" => LanguageInfo { icon: "", color: "#f7a41d", cterm_color: "214", name: "Zig", lang: Language::Zig, kind: MatchKind::Indicator },
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

/// Map a `Language` (with an optional key for disambiguation) to the appropriate
/// extraction strategy.  Returns `None` for manifests that carry no self-declared
/// version (go.mod, stack.yaml, Gemfile, Pipfile, …).
fn version_strategy(lang: Language, key: &str) -> Option<VersionStrategy> {
    use Language::*;
    Some(match (lang, key) {
        // ── TOML-like: line-start guard avoids matching dependency version specs ──────────
        (Rust, _) | (Julia, _) | (Nim, _) => VersionStrategy::LineFirstQuoted("version"),
        (Python, "pyproject.toml") => VersionStrategy::LineFirstQuoted("version"),
        (Lua, ".rockspec") => VersionStrategy::LineFirstQuoted("version"),

        // ── D: two manifest formats ──────────────────────────────────────────────────────
        (D, "dub.sdl") => VersionStrategy::LineFirstQuoted("version"),
        (D, "dub.json") => VersionStrategy::QuotedAfter("\"version\": \""),

        // ── OCaml: two manifest formats ──────────────────────────────────────────────────
        (OCaml, "opam") => VersionStrategy::LineFirstQuoted("version"),
        (OCaml, "dune-project") => VersionStrategy::LineUnquoted("(version "),

        // ── Elixir Mix: `version: "1.0.0"` — the `: ` makes it unique in .exs ───────────
        (Elixir, _) => VersionStrategy::QuotedAfter("version: \""),

        // ── JSON-based manifests ─────────────────────────────────────────────────────────
        (JavaScript, _) | (Php, _) | (Haxe, _) | (Elm, _) => {
            VersionStrategy::QuotedAfter("\"version\": \"")
        }

        // ── Ruby gemspec: `.version = "…"` ───────────────────────────────────────────────
        (Ruby, ".gemspec") => VersionStrategy::QuotedAfter(".version = \""),

        // ── Scala SBT: `version := "…"` ─────────────────────────────────────────────────
        (Scala, _) => VersionStrategy::QuotedAfter("version := \""),

        // ── Common Lisp ASDF: `:version "…"` in defsystem ───────────────────────────────
        (CommonLisp, _) => VersionStrategy::QuotedAfter(":version \""),

        // ── Racket info.rkt: `(define version "…")` ─────────────────────────────────────
        (Racket, _) => VersionStrategy::QuotedAfter("(define version \""),

        // ── Unquoted key-value ───────────────────────────────────────────────────────────
        // .cabal:       `version:             0.1.0.0`
        // pubspec.yaml: `version: 1.0.0+1`
        (Haskell, ".cabal") | (Dart, _) => VersionStrategy::LineUnquoted("version:"),

        // ── XML ──────────────────────────────────────────────────────────────────────────
        (Java, _) => VersionStrategy::XmlBetween("<version>", "</version>"),
        (CSharp, _) | (FSharp, _) => VersionStrategy::XmlBetween("<Version>", "</Version>"),

        // Everything else (Go, Swift, Nix, Haskell/stack.yaml, Gemfile, Pipfile, …)
        // has no self-declared version we can reliably extract.
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
/// determined by the `Language` and the lookup `key`.
async fn version_from_file(path: &std::path::Path, lang: Language, key: &str) -> Option<SmolStr> {
    let strategy = version_strategy(lang, key)?;
    let content = tokio::fs::read_to_string(path).await.ok()?;
    extract_version(strategy, &content)
}

pub async fn show(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.nerd_font || !opts.manifest {
        return None;
    }

    let mut entries = tokio::fs::read_dir(".").await.ok()?;
    let mut languages: SmallVec<[&LanguageInfo; 8]> = SmallVec::new();
    // All ProjectManifest entries found during the scan, in filesystem order.
    // We will try each in sequence and return the first that yields a version.
    let mut manifests: SmallVec<[(std::path::PathBuf, Language, SmolStr); 4]> = SmallVec::new();

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
                manifests.push((entry.path(), lang.lang, SmolStr::new(name)));
            }
            languages.push(lang);
        } else if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
            let key = format_smolstr!(".{ext}");
            if let Some(lang) = MANIFEST_MAP.get(key.as_str()) {
                if lang.kind == MatchKind::ProjectManifest {
                    manifests.push((entry.path(), lang.lang, key));
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
        for (path, lang, key) in &manifests {
            if let Some(v) = version_from_file(path, *lang, key).await {
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
