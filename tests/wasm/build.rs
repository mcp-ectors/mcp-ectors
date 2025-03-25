use std::env;
use std::fs;
use std::process::Command;

fn main() {
    // Define the path where the .wasm file should exist
    let target_dir = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
    let wasm_file_path = format!("{}/wasm32-wasip2/debug/wasm_file.wasm", target_dir);
    
    // Check if the .wasm file exists
    if !fs::metadata(&wasm_file_path).is_ok() {
        println!("No .wasm file found, building for wasm32-wasip2 target...");
        
        // Run cargo build for the wasm32-wasip2 target if the file doesn't exist
        let status = Command::new("cargo")
            .arg("build")
            .arg("--target")
            .arg("wasm32-wasip2")
            .status()
            .expect("Failed to execute cargo build");

        // If the build fails, exit with an error code
        if !status.success() {
            panic!("cargo build failed for wasm32-wasip2 target");
        }
    } else {
        println!(".wasm file found, skipping build.");
    }
}
