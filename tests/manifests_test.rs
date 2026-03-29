use std::process::Command;
use std::path::PathBuf;

fn get_auraline_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_auraline"))
}

fn test_manifest_extraction(dir_name: &str, expected_version: &str) {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("manifests")
        
        .join(dir_name);

    let output = Command::new(get_auraline_bin())
        .current_dir(&manifest_dir)
        .arg("prompt")
        .arg("--manifest")
        .arg("--nerd-font")
        .output()
        .expect("Failed to execute auraline");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected_str = format!("v{}", expected_version);
    
    assert!(
        stdout.contains(&expected_str),
        "Expected '{}' in output for '{}', but got:\n{}",
        expected_str,
        dir_name,
        stdout
    );
}

#[test] fn test_rust() { test_manifest_extraction("rust", "1.0.75"); }
#[test] fn test_javascript() { test_manifest_extraction("javascript", "4.18.2"); }
#[test] fn test_python_toml() { test_manifest_extraction("python_toml", "2.0.1"); }
#[test] fn test_python_cfg() { test_manifest_extraction("python_cfg", "0.27.0"); }
#[test] fn test_cmake() { test_manifest_extraction("cmake", "1.7.16"); }
#[test] fn test_java_pom() { test_manifest_extraction("java_pom", "4.13.2"); }
#[test] fn test_java_gradle() { test_manifest_extraction("java_gradle", "1.57.2"); }
#[test] fn test_kotlin_gradle() { test_manifest_extraction("kotlin_gradle", "0.11.0"); }
#[test] fn test_dart() { test_manifest_extraction("dart", "1.6.1-wip"); }
#[test] fn test_haskell() { test_manifest_extraction("haskell", "3.1.5"); }
#[test] fn test_elixir() { test_manifest_extraction("elixir", "2.1.0"); }
#[test] fn test_ocaml_dune() { test_manifest_extraction("ocaml_dune", "5.9.0"); }
#[test] fn test_ocaml_opam() { test_manifest_extraction("ocaml_opam", "5.7.0"); }
#[test] fn test_csharp() { test_manifest_extraction("csharp", "1.53.0-beta.1"); }
#[test] fn test_lua() { test_manifest_extraction("lua", "3.9.2-1"); }
#[test] fn test_common_lisp() { test_manifest_extraction("common_lisp", "2.1.1"); }
#[test] fn test_zig() { test_manifest_extraction("zig", "0.13.0"); }
