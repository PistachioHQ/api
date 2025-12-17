//! Build script for generating OpenAPI client code.
//!
//! Using openapi-generator instead of progenitor due to OpenAPI 3.1 support.
//! See: <https://github.com/oxidecomputer/progenitor/issues/762>

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    // Generate public API client
    generate_openapi_client(
        &manifest_dir.join("../../openapi/bundled.yml"),
        &manifest_dir.join("src/generated"),
        "crate::generated",
    );

    // Generate admin API client
    generate_openapi_client(
        &manifest_dir.join("../../openapi/admin-bundled.yml"),
        &manifest_dir.join("src/generated_admin"),
        "crate::generated_admin",
    );
}

fn generate_openapi_client(openapi_spec: &Path, generated_dir: &Path, crate_prefix: &str) {
    println!("cargo:rerun-if-changed={}", openapi_spec.display());

    let temp_dir = tempfile::tempdir().unwrap();

    let status = Command::new("openapi-generator")
        .args([
            "generate",
            "-i",
            openapi_spec.to_str().unwrap(),
            "-g",
            "rust",
            "--package-name",
            "gen",
            "-o",
            temp_dir.path().to_str().unwrap(),
            "--additional-properties=supportAsync=true",
            "--global-property=apiTests=false,modelTests=false",
        ])
        .status()
        .expect("Failed to execute openapi-generator");

    if !status.success() {
        panic!(
            "Failed to generate OpenAPI client for {}",
            openapi_spec.display()
        );
    }

    let apis_out = generated_dir.join("apis");
    let models_out = generated_dir.join("models");
    fs::create_dir_all(&apis_out).unwrap();
    fs::create_dir_all(&models_out).unwrap();

    let models_replacement = format!("{}::models", crate_prefix);
    let models_dir = temp_dir.path().join("src/models");
    for entry in fs::read_dir(models_dir).unwrap() {
        let entry = entry.unwrap();
        let content = fs::read_to_string(entry.path())
            .unwrap()
            .replace("crate::models", &models_replacement);

        fs::write(models_out.join(entry.file_name()), content).unwrap();
    }

    let crate_replacement = format!("{}::", crate_prefix);
    let apis_dir = temp_dir.path().join("src/apis");
    for entry in fs::read_dir(apis_dir).unwrap() {
        let entry = entry.unwrap();
        let content = fs::read_to_string(entry.path())
            .unwrap()
            .replace("crate::", &crate_replacement);

        fs::write(apis_out.join(entry.file_name()), content).unwrap();
    }

    let lib_content = fs::read_to_string(temp_dir.path().join("src/lib.rs")).unwrap();
    fs::write(generated_dir.join("mod.rs"), lib_content).unwrap();

    // Format all generated Rust files with rustfmt
    format_generated_files(generated_dir);
}

fn format_generated_files(dir: &Path) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            format_generated_files(&path);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let output = Command::new("rustfmt")
                .args(["--edition", "2024"])
                .arg(&path)
                .output();
            match output {
                Ok(out) => {
                    if !out.status.success() {
                        println!(
                            "cargo:warning=rustfmt failed for {}: {}",
                            path.display(),
                            String::from_utf8_lossy(&out.stderr)
                        );
                    }
                }
                Err(e) => {
                    println!(
                        "cargo:warning=Failed to run rustfmt on {}: {}",
                        path.display(),
                        e
                    );
                }
            }
        }
    }
}
