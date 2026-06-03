use std::process::Command;
use std::{env, path::Path};

fn main() {
    println!("cargo:rerun-if-changed=frontend/src");
    println!("cargo:rerun-if-changed=frontend/package.json");
    println!("cargo:rerun-if-changed=frontend/package-lock.json");

    if env::var("CAMGI_SKIP_FRONTEND_BUILD").as_deref() == Ok("1") {
        let js_exists = Path::new("frontend/dist/assets/index.js").exists();
        let css_exists = Path::new("frontend/dist/assets/index.css").exists();
        if js_exists && css_exists {
            return;
        }

        panic!("CAMGI_SKIP_FRONTEND_BUILD=1 was set, but frontend/dist assets are missing");
    }

    // Build frontend
    let status = Command::new("npm")
        .args(&["run", "build"])
        .current_dir("frontend")
        .status()
        .expect("Failed to build frontend");

    if !status.success() {
        panic!("Frontend build failed");
    }
}
