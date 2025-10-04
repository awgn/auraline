use crate::{chunk::Chunk, options::Options};
use phf::phf_map;
use smallvec::SmallVec;
use smol_str::{format_smolstr, SmolStr, SmolStrBuilder};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Confidence {
    /// Generic or ambiguous indicator (e.g., Makefile, package.yaml).
    /// Requires content inspection to be sure.
    Low,
    /// Strong indicator, but used across a close-knit ecosystem (e.g., package.json for JS/TS).
    Medium,
    /// Unique, unambiguous indicator (e.g., Cargo.toml, go.mod).
    High,
}

#[allow(dead_code)]
pub struct LanguageInfo {
    pub icon: &'static str,
    pub color: &'static str,
    pub cterm_color: &'static str,
    pub name: &'static str,
    pub confidence: Confidence,
}

// A static, perfect hash map of manifest filenames to language information.

static MANIFEST_MAP: phf::Map<&'static str, LanguageInfo> = phf_map! {
    // Agda
    ".agda" => LanguageInfo { icon: "A", color: "#F1D352", cterm_color: "221", name: "Agda", confidence: Confidence::High },
    ".lagda" => LanguageInfo { icon: "A", color: "#F1D352", cterm_color: "221", name: "Agda", confidence: Confidence::High },

    // Assembly
    ".asm" => LanguageInfo { icon: "", color: "#6E8493", cterm_color: "102", name: "Assembly", confidence: Confidence::Low },

    // Awk
    ".awk" => LanguageInfo { icon: "", color: "#E5E5E5", cterm_color: "254", name: "Awk", confidence: Confidence::High },
    ".mawk" => LanguageInfo { icon: "", color: "#E5E5E5", cterm_color: "254", name: "Awk", confidence: Confidence::High },
    ".gawk" => LanguageInfo { icon: "", color: "#E5E5E5", cterm_color: "254", name: "Awk", confidence: Confidence::High },

    // C & C++
    "Makefile" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", confidence: Confidence::Low },
    "Makefile.am" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", confidence: Confidence::Low },
    "CMakeLists.txt" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", confidence: Confidence::Medium },
    "meson.build" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", confidence: Confidence::Medium },
    "autogen.sh" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", confidence: Confidence::Medium },
    "configure.ac" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", confidence: Confidence::Medium },
    ".c" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", confidence: Confidence::High },
    ".h" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", confidence: Confidence::High },
    ".inc" => LanguageInfo { icon: "", color: "#B3B3B3", cterm_color: "249", name: "C/C++ Header", confidence: Confidence::Low },
    ".cpp" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", confidence: Confidence::High },
    ".hpp" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C/C++", confidence: Confidence::High },
    ".cxx" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C++", confidence: Confidence::High },
    ".cc" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "C++", confidence: Confidence::High },

    // C#
    ".csproj" => LanguageInfo { icon: "", color: "#5a29a4", cterm_color: "99", name: "C#", confidence: Confidence::High },
    ".cs" => LanguageInfo { icon: "", color: "#5a29a4", cterm_color: "99", name: "C#", confidence: Confidence::High },

    // Chapel
    ".chpl" => LanguageInfo { icon: "C", color: "#8dc63f", cterm_color: "113", name: "Chapel", confidence: Confidence::High },

    // Clojure
    ".clj" => LanguageInfo { icon: "", color: "#82b131", cterm_color: "106", name: "Clojure", confidence: Confidence::High },
    ".cljs" => LanguageInfo { icon: "", color: "#82b131", cterm_color: "106", name: "ClojureScript", confidence: Confidence::High },
    ".cljc" => LanguageInfo { icon: "", color: "#82b131", cterm_color: "106", name: "Clojure", confidence: Confidence::High },
    ".edn" => LanguageInfo { icon: "", color: "#82b131", cterm_color: "106", name: "EDN", confidence: Confidence::Medium },

    // COBOL
    ".cbl" => LanguageInfo { icon: "C", color: "#005ca5", cterm_color: "25", name: "COBOL", confidence: Confidence::Low },

    // CoffeeScript
    "Cakefile" => LanguageInfo { icon: "", color: "#244776", cterm_color: "24", name: "CoffeeScript", confidence: Confidence::High },
    ".coffee" => LanguageInfo { icon: "", color: "#244776", cterm_color: "24", name: "CoffeeScript", confidence: Confidence::High },

    // Common Lisp
    ".asd" => LanguageInfo { icon: "", color: "#b31a23", cterm_color: "124", name: "Common Lisp", confidence: Confidence::High },
    ".lisp" => LanguageInfo { icon: "", color: "#b31a23", cterm_color: "124", name: "Lisp", confidence: Confidence::High },
    ".cl" => LanguageInfo { icon: "", color: "#b31a23", cterm_color: "124", name: "Common Lisp", confidence: Confidence::High },

    // Config Files
    ".config" => LanguageInfo { icon: "", color: "#A0A0A0", cterm_color: "247", name: "Config", confidence: Confidence::Low },
    ".conf" => LanguageInfo { icon: "", color: "#A0A0A0", cterm_color: "247", name: "Config", confidence: Confidence::Low },
    ".cfg" => LanguageInfo { icon: "", color: "#A0A0A0", cterm_color: "247", name: "Config", confidence: Confidence::Low },
    ".ini" => LanguageInfo { icon: "", color: "#A0A0A0", cterm_color: "247", name: "INI", confidence: Confidence::Medium },
    ".toml" => LanguageInfo { icon: "", color: "#A0A0A0", cterm_color: "247", name: "TOML", confidence: Confidence::Medium },
    ".yaml" => LanguageInfo { icon: "󰰴", color: "#A0A0A0", cterm_color: "247", name: "YAML", confidence: Confidence::Medium },
    ".yml" => LanguageInfo { icon: "󰰴", color: "#A0A0A0", cterm_color: "247", name: "YAML", confidence: Confidence::Medium },
    ".json" => LanguageInfo { icon: "", color: "#E5E5E5", cterm_color: "254", name: "JSON", confidence: Confidence::Medium },

    // CSS
    ".css" => LanguageInfo { icon: "", color: "#563d7c", cterm_color: "98", name: "CSS", confidence: Confidence::High },

    // D
    "dub.json" => LanguageInfo { icon: "D", color: "#ba595e", cterm_color: "131", name: "D", confidence: Confidence::High },
    "dub.sdl" => LanguageInfo { icon: "D", color: "#ba595e", cterm_color: "131", name: "D", confidence: Confidence::High },
    ".d" => LanguageInfo { icon: "D", color: "#ba595e", cterm_color: "131", name: "D", confidence: Confidence::High },

    // Dart
    "pubspec.yaml" => LanguageInfo { icon: "", color: "#0175c2", cterm_color: "32", name: "Dart", confidence: Confidence::High },
    ".dart" => LanguageInfo { icon: "", color: "#0175c2", cterm_color: "32", name: "Dart", confidence: Confidence::High },

    // Dhall
    ".dhall" => LanguageInfo { icon: "D", color: "#174B59", cterm_color: "23", name: "Dhall", confidence: Confidence::High },

    // Eiffel
    ".ecf" => LanguageInfo { icon: "E", color: "#0C8B95", cterm_color: "30", name: "Eiffel", confidence: Confidence::High },

    // Elixir
    "mix.exs" => LanguageInfo { icon: "", color: "#4b275f", cterm_color: "54", name: "Elixir", confidence: Confidence::High },
    ".ex" => LanguageInfo { icon: "", color: "#4b275f", cterm_color: "54", name: "Elixir", confidence: Confidence::High },
    ".exs" => LanguageInfo { icon: "", color: "#4b275f", cterm_color: "54", name: "Elixir", confidence: Confidence::High },

    // Elm
    "elm.json" => LanguageInfo { icon: "", color: "#60b5cc", cterm_color: "74", name: "Elm", confidence: Confidence::High },

    // Erlang
    ".erl" => LanguageInfo { icon: "", color: "#b83998", cterm_color: "133", name: "Erlang", confidence: Confidence::High },
    ".hrl" => LanguageInfo { icon: "", color: "#b83998", cterm_color: "133", name: "Erlang", confidence: Confidence::Medium },

    // F#
    ".fsproj" => LanguageInfo { icon: "", color: "#378bba", cterm_color: "38", name: "F#", confidence: Confidence::High },
    ".fs" => LanguageInfo { icon: "", color: "#378bba", cterm_color: "38", name: "F#", confidence: Confidence::High },
    ".fsx" => LanguageInfo { icon: "", color: "#378bba", cterm_color: "38", name: "F#", confidence: Confidence::High },

    // Forth
    ".fth" => LanguageInfo { icon: "F", color: "#341708", cterm_color: "52", name: "Forth", confidence: Confidence::Medium },

    // Fortran
    ".f90" => LanguageInfo { icon: "󱈚", color: "#734f96", cterm_color: "98", name: "Fortran", confidence: Confidence::Medium },
    ".f95" => LanguageInfo { icon: "󱈚", color: "#734f96", cterm_color: "98", name: "Fortran", confidence: Confidence::Medium },
    ".f" => LanguageInfo { icon: "󱈚", color: "#734f96", cterm_color: "98", name: "Fortran", confidence: Confidence::Low },

    // Go
    "go.mod" => LanguageInfo { icon: "", color: "#00add8", cterm_color: "38", name: "Go", confidence: Confidence::High },
    ".go" => LanguageInfo { icon: "", color: "#00add8", cterm_color: "38", name: "Go", confidence: Confidence::High },

    // Hack
    ".hhconfig" => LanguageInfo { icon: "H", color: "#89afcf", cterm_color: "110", name: "Hack", confidence: Confidence::High },

    // Haskell
    ".cabal" => LanguageInfo { icon: "", color: "#5e5086", cterm_color: "61", name: "Haskell", confidence: Confidence::High },
    "stack.yaml" => LanguageInfo { icon: "", color: "#5e5086", cterm_color: "61", name: "Haskell", confidence: Confidence::High },
    "Setup.hs" => LanguageInfo { icon: "", color: "#5e5086", cterm_color: "61", name: "Haskell", confidence: Confidence::High },
    ".hs" => LanguageInfo { icon: "", color: "#5e5086", cterm_color: "61", name: "Haskell", confidence: Confidence::High },
    ".lhs" => LanguageInfo { icon: "", color: "#5e5086", cterm_color: "61", name: "Haskell", confidence: Confidence::High },

    // Haxe
    "haxelib.json" => LanguageInfo { icon: "H", color: "#DF7900", cterm_color: "208", name: "Haxe", confidence: Confidence::High },
    ".hxml" => LanguageInfo { icon: "H", color: "#DF7900", cterm_color: "208", name: "Haxe", confidence: Confidence::High },

    // HTML
    ".html" => LanguageInfo { icon: "", color: "#e34c26", cterm_color: "196", name: "HTML", confidence: Confidence::High },
    ".htm" => LanguageInfo { icon: "", color: "#e34c26", cterm_color: "196", name: "HTML", confidence: Confidence::High },

    // Idris
    ".idr" => LanguageInfo { icon: "I", color: "#1F232D", cterm_color: "235", name: "Idris", confidence: Confidence::High },
    ".lidr" => LanguageInfo { icon: "I", color: "#1F232D", cterm_color: "235", name: "Idris", confidence: Confidence::High },

    // Java & JVM
    "pom.xml" => LanguageInfo { icon: "", color: "#cc0000", cterm_color: "160", name: "Java/JVM", confidence: Confidence::Medium },
    "build.gradle" => LanguageInfo { icon: "", color: "#cc0000", cterm_color: "160", name: "Java/JVM", confidence: Confidence::Medium },
    ".java" => LanguageInfo { icon: "", color: "#cc0000", cterm_color: "160", name: "Java", confidence: Confidence::High },

    // JavaScript / TypeScript / Node
    "package.json" => LanguageInfo { icon: "", color: "#f0db4f", cterm_color: "220", name: "JavaScript/Node", confidence: Confidence::Medium },
    "tsconfig.json" => LanguageInfo { icon: "", color: "#3178c6", cterm_color: "39", name: "TypeScript", confidence: Confidence::High },

    // Julia
    "Project.toml" => LanguageInfo { icon: "", color: "#a270ba", cterm_color: "140", name: "Julia", confidence: Confidence::High },

    // Kotlin
    "build.gradle.kts" => LanguageInfo { icon: "󱈙", color: "#7f52ff", cterm_color: "99", name: "Kotlin", confidence: Confidence::Medium },
    ".kt" => LanguageInfo { icon: "󱈙", color: "#7f52ff", cterm_color: "99", name: "Kotlin", confidence: Confidence::High },
    ".kts" => LanguageInfo { icon: "󱈙", color: "#7f52ff", cterm_color: "99", name: "Kotlin", confidence: Confidence::High },

    // LaTeX
    ".tex" => LanguageInfo { icon: "", color: "#008080", cterm_color: "30", name: "LaTeX", confidence: Confidence::Medium },
    ".latex" => LanguageInfo { icon: "", color: "#008080", cterm_color: "30", name: "LaTeX", confidence: Confidence::Medium },

    // Lua
    ".rockspec" => LanguageInfo { icon: "", color: "#2c2d72", cterm_color: "18", name: "Lua", confidence: Confidence::High },
    ".lua" => LanguageInfo { icon: "", color: "#2c2d72", cterm_color: "18", name: "Lua", confidence: Confidence::Medium },

    // Nim
    ".nimble" => LanguageInfo { icon: "", color: "#ffc200", cterm_color: "220", name: "Nim", confidence: Confidence::High },
    ".nim" => LanguageInfo { icon: "", color: "#ffc200", cterm_color: "220", name: "Nim", confidence: Confidence::High },

    // Nix
    "flake.nix" => LanguageInfo { icon: "", color: "#7E76D4", cterm_color: "104", name: "Nix", confidence: Confidence::High },

    // Nmap
    ".nse" => LanguageInfo { icon: "N", color: "#404040", cterm_color: "238", name: "Nmap Script", confidence: Confidence::High },

    // Objective-C
    "project.pbxproj" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "Objective-C", confidence: Confidence::Medium },
    ".m" => LanguageInfo { icon: "", color: "#6792c9", cterm_color: "67", name: "Objective-C", confidence: Confidence::Medium },

    // OCaml
    "dune-project" => LanguageInfo { icon: "", color: "#ec6813", cterm_color: "166", name: "OCaml", confidence: Confidence::High },
    "opam" => LanguageInfo { icon: "", color: "#ec6813", cterm_color: "166", name: "OCaml", confidence: Confidence::High },
    ".ml" => LanguageInfo { icon: "", color: "#ec6813", cterm_color: "166", name: "OCaml", confidence: Confidence::High },
    ".mli" => LanguageInfo { icon: "", color: "#ec6813", cterm_color: "166", name: "OCaml", confidence: Confidence::High },

    // Pascal / Delphi
    ".dpr" => LanguageInfo { icon: "P", color: "#e32b2f", cterm_color: "196", name: "Delphi/Pascal", confidence: Confidence::High },
    ".lpr" => LanguageInfo { icon: "P", color: "#EEDD34", cterm_color: "184", name: "Lazarus/Pascal", confidence: Confidence::High },

    // Perl
    ".pl" => LanguageInfo { icon: "", color: "#39457e", cterm_color: "61", name: "Perl", confidence: Confidence::Medium },
    ".pm" => LanguageInfo { icon: "", color: "#39457e", cterm_color: "61", name: "Perl", confidence: Confidence::High },

    // PHP
    "composer.json" => LanguageInfo { icon: "", color: "#8892be", cterm_color: "103", name: "PHP", confidence: Confidence::High },
    ".php" => LanguageInfo { icon: "", color: "#8892be", cterm_color: "103", name: "PHP", confidence: Confidence::High },

    // PowerShell
    ".psd1" => LanguageInfo { icon: "󰨊", color: "#012456", cterm_color: "18", name: "PowerShell", confidence: Confidence::High },

    // Prolog
    ".pro" => LanguageInfo { icon: "P", color: "#880000", cterm_color: "88", name: "Prolog", confidence: Confidence::Low },

    // Python
    "pyproject.toml" => LanguageInfo { icon: "", color: "#f6c43b", cterm_color: "203", name: "Python", confidence: Confidence::High },
    "Pipfile" => LanguageInfo { icon: "", color: "#f6c43b", cterm_color: "203", name: "Python", confidence: Confidence::High },
    "requirements.txt" => LanguageInfo { icon: "", color: "#f6c43b", cterm_color: "203", name: "Python", confidence: Confidence::Medium },
    ".py" => LanguageInfo { icon: "", color: "#f6c43b", cterm_color: "203", name: "Python", confidence: Confidence::High },
    ".pyx" => LanguageInfo { icon: "", color: "#f6c43b", cterm_color: "203", name: "Cython", confidence: Confidence::High },

    // R
    ".r" => LanguageInfo { icon: "󰟔", color: "#276dc2", cterm_color: "33", name: "R", confidence: Confidence::High },

    // Racket
    "info.rkt" => LanguageInfo { icon: "R", color: "#394FAC", cterm_color: "61", name: "Racket", confidence: Confidence::High },

    // Ruby
    "Gemfile" => LanguageInfo { icon: "", color: "#cc342d", cterm_color: "160", name: "Ruby", confidence: Confidence::High },
    ".gemspec" => LanguageInfo { icon: "", color: "#cc342d", cterm_color: "160", name: "Ruby", confidence: Confidence::High },
    ".rb" => LanguageInfo { icon: "", color: "#cc342d", cterm_color: "160", name: "Ruby", confidence: Confidence::High },

    // Rust
    "Cargo.toml" => LanguageInfo { icon: "", color: "#dea584", cterm_color: "173", name: "Rust", confidence: Confidence::High },
    ".rs" => LanguageInfo { icon: "", color: "#dea584", cterm_color: "173", name: "Rust", confidence: Confidence::High },

    // Scala
    "build.sbt" => LanguageInfo { icon: "", color: "#c22d40", cterm_color: "160", name: "Scala", confidence: Confidence::High },

    // Shell
    ".bashrc" => LanguageInfo { icon: "", color: "#89E051", cterm_color: "113", name: "Shell", confidence: Confidence::Medium },
    ".zshrc" => LanguageInfo { icon: "", color: "#89E051", cterm_color: "113", name: "Shell", confidence: Confidence::Medium },
    ".sh" => LanguageInfo { icon: "", color: "#89E051", cterm_color: "113", name: "Shell", confidence: Confidence::Low },
    ".bash" => LanguageInfo { icon: "", color: "#89E051", cterm_color: "113", name: "Bash", confidence: Confidence::High },
    ".csh" => LanguageInfo { icon: "", color: "#89E051", cterm_color: "113", name: "Csh", confidence: Confidence::High },
    ".tcsh" => LanguageInfo { icon: "", color: "#89E051", cterm_color: "113", name: "Tcsh", confidence: Confidence::High },
    ".fish" => LanguageInfo { icon: "", color: "#89E051", cterm_color: "113", name: "Fish", confidence: Confidence::High },
    ".ksh" => LanguageInfo { icon: "", color: "#89E051", cterm_color: "113", name: "Ksh", confidence: Confidence::High },
    ".zsh" => LanguageInfo { icon: "", color: "#89E051", cterm_color: "113", name: "Zsh", confidence: Confidence::High },

    // SmallTalk
    ".st" => LanguageInfo { icon: "S", color: "#596700", cterm_color: "58", name: "SmallTalk", confidence: Confidence::High },

    // Swift
    "Package.swift" => LanguageInfo { icon: "", color: "#ffac45", cterm_color: "215", name: "Swift", confidence: Confidence::High },

    // Tcl
    ".tcl" => LanguageInfo { icon: "T", color: "#1D529D", cterm_color: "25", name: "Tcl", confidence: Confidence::Medium },

    // Unison
    ".u" => LanguageInfo { icon: "U", color: "#FF7A62", cterm_color: "209", name: "Unison", confidence: Confidence::High },

    // Vala
    ".vala" => LanguageInfo { icon: "", color: "#7239B3", cterm_color: "98", name: "Vala", confidence: Confidence::Medium },

    // Verilog
    ".v" => LanguageInfo { icon: "V", color: "#000080", cterm_color: "18", name: "Verilog", confidence: Confidence::Low },
    ".vh" => LanguageInfo { icon: "V", color: "#000080", cterm_color: "18", name: "Verilog", confidence: Confidence::Low },
    ".sv" => LanguageInfo { icon: "V", color: "#000080", cterm_color: "18", name: "Verilog", confidence: Confidence::Low },

    // VHDL
    ".vhd" => LanguageInfo { icon: "V", color: "#AD0000", cterm_color: "88", name: "VHDL", confidence: Confidence::Low },
    ".vhdl" => LanguageInfo { icon: "V", color: "#AD0000", cterm_color: "88", name: "VHDL", confidence: Confidence::Low },

    // Zig
    "build.zig" => LanguageInfo { icon: "", color: "#f7a41d", cterm_color: "214", name: "Zig", confidence: Confidence::High },
    ".zig" => LanguageInfo { icon: "", color: "#f7a41d", cterm_color: "214", name: "Zig", confidence: Confidence::High },
};

pub async fn show(opts: &Options) -> Option<Chunk<SmolStr>> {
    if !opts.nerd_font || !opts.manifest {
        return None;
    }

    // retrive the list of files in the current directory
    let mut entries = tokio::fs::read_dir(".").await.ok()?;
    let mut languages = SmallVec::<[_; 8]>::new();

    while let Some(entry) = entries.next_entry().await.ok()? {
        // skip hidden files
        if entry.file_name().to_str()?.starts_with('.') {
            continue;
        }

        let lang = MANIFEST_MAP.get(entry.file_name().to_str()?);
        if let Some(lang) = lang {
            languages.push(lang);
        } else if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
            if let Some(lang) = MANIFEST_MAP.get(&format_smolstr!(".{ext}")) {
                languages.push(lang);
            }
        }
    }

    languages.sort_by(|a, b| {
        let ord = b.confidence.cmp(&a.confidence);
        if ord == std::cmp::Ordering::Equal {
            a.icon.cmp(b.icon)
        } else {
            ord
        }
    });

    // return the languages icons with highest confidence...
    let top_confidence = languages.first()?.confidence;
    languages.retain(|lang| lang.confidence == top_confidence);

    // flatten to a single string
    let mut builder = SmolStrBuilder::new();
    let mut cur_icon = "";
    for lang in &languages {
        if lang.icon != cur_icon {
            builder.push_str(lang.icon);
            cur_icon = lang.icon;
        }
    }
    Some(Chunk::info(builder.finish()))
}
