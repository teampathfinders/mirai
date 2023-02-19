use std::process::Command;
use chrono::prelude::*;

// Load the current Git commit hash.
fn main() {
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .unwrap();

    let git_hash = String::from_utf8(output.stdout)
        .unwrap()
        .chars()
        .take(7)
        .collect::<String>();

    let build_date = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    println!("cargo:rustc-env=GIT_COMMIT_HASH={git_hash} (build {build_date})");
}
