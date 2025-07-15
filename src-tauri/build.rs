use std::io;
use std::process::Command;

fn main() -> io::Result<()> {
    // get commit
    let output = Command::new("git").args(["rev-parse", "HEAD"]).output()?;
    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={git_hash}");

    tauri_build::build();
    Ok(())
}
