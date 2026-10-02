use std::path::Path;
use std::process::Command;

fn main() {
    embuild::espidf::sysenv::output();

    let sha = std::env::var("GITHUB_SHA")
        .ok()
        .map(|sha| sha.chars().take(7).collect::<String>())
        .or_else(git_short_sha)
        .unwrap_or_else(|| "unknown".to_owned());

    let version = std::env::var("FIRMWARE_VERSION")
        .unwrap_or_else(|_| format!("v{}-{}", env!("CARGO_PKG_VERSION"), sha));

    println!("cargo:rustc-env=FIRMWARE_VERSION={version}");
    println!("cargo:rerun-if-env-changed=FIRMWARE_VERSION");
    println!("cargo:rerun-if-env-changed=GITHUB_SHA");
    println!("cargo:rerun-if-changed=../.git/HEAD");

    compile_eink_api();
}

fn compile_eink_api() {
    let proto_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("proto");
    let api = proto_dir.join("eink").join("v1").join("eink_display.proto");

    println!("cargo:rerun-if-changed={}", api.display());

    prost_build::Config::new()
        .bytes(["."])
        .compile_protos(&[&api], &[&proto_dir])
        .unwrap_or_else(|e| panic!("failed to compile {}: {e}", api.display()));
}

fn git_short_sha() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short=7", "HEAD"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let sha = String::from_utf8(output.stdout).ok()?.trim().to_owned();

    if sha.is_empty() {
        None
    } else {
        Some(sha)
    }
}
