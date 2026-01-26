use chrono::Utc;
use std::env;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Atomically writes content to a file by first writing to a temp file, then renaming.
/// This prevents readers from seeing a partially-written or empty file.
fn atomic_write(path: &Path, content: &str) -> std::io::Result<()> {
    // Generate a temp filename in the same directory
    let temp_name = format!(
        ".version_js_tmp_{}.js",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let temp_path = path.parent().unwrap_or(Path::new(".")).join(&temp_name);

    // Write content to temp file
    fs::write(&temp_path, content)?;

    // Atomically rename temp file to target (on Unix, rename is atomic)
    fs::rename(&temp_path, path)?;

    Ok(())
}

fn main() {
    // Generate current timestamp as the build version in ISO 8601 format
    let version = Utc::now().to_rfc3339();

    // 1. Write version.js for Service Worker (in public/)
    let public_dir = Path::new("public");
    if !public_dir.exists() {
        panic!("public/ directory does not exist. Please create it before building.");
    }

    let version_js_path = public_dir.join("version.js");
    // Use globalThis assignment instead of const to prevent dx's minifier from
    // stripping it as "dead code". globalThis creates a side effect that survives minification.
    let version_js_content = format!(
        "// Generated file, DO NOT EDIT.\n\
         globalThis.__TODO_MVP_BUILD_VERSION__ = \"{}\";\n",
        version
    );
    atomic_write(&version_js_path, &version_js_content).expect("Failed to write public/version.js");

    // println!(
    //     "cargo:warning=Generated public/version.js with version: {}",
    //     version
    // );

    // 2. Write build_version.rs for Rust (in OUT_DIR)
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let build_version_rs_path = Path::new(&out_dir).join("build_version.rs");
    let build_version_rs_content = format!("pub const BUILD_VERSION: &str = \"{}\";\n", version);
    atomic_write(&build_version_rs_path, &build_version_rs_content)
        .expect("Failed to write build_version.rs");

    // println!(
    //     "cargo:warning=Generated build_version.rs with version: {}",
    //     version
    // );

    // 3. Constraint: Only re-run if build.rs itself changes
    // This prevents infinite build loops in `dx serve`
    println!("cargo:rerun-if-changed=build.rs");
}
