use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=MCP_IMESSAGE_SKIP_UI_BUILD");
    println!("cargo:rerun-if-changed=ui/index.html");
    println!("cargo:rerun-if-changed=ui/package.json");
    println!("cargo:rerun-if-changed=ui/package-lock.json");
    println!("cargo:rerun-if-changed=ui/tsconfig.json");
    println!("cargo:rerun-if-changed=ui/vite.config.ts");
    let manifest_dir = manifest_dir();
    emit_rerun_for_dir(&manifest_dir.join("ui/src"));

    let ui_dir = manifest_dir.join("ui");
    let dist_html = ui_dir.join("dist/index.html");

    if env::var("MCP_IMESSAGE_SKIP_UI_BUILD").ok().as_deref() == Some("1") {
        if !dist_html.exists() {
            panic!(
                "MCP_IMESSAGE_SKIP_UI_BUILD=1 was set, but {} does not exist.",
                dist_html.display()
            );
        }
        return;
    }

    ensure_npm_deps(&ui_dir);
    run_command("npm", &["run", "build"], &ui_dir);

    if !dist_html.exists() {
        panic!(
            "UI build finished without producing {}",
            dist_html.display()
        );
    }
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set"))
}

fn emit_rerun_for_dir(dir: &Path) {
    fn walk(path: &Path) {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path);
                } else if path.is_file() {
                    println!("cargo:rerun-if-changed={}", path.display());
                }
            }
        }
    }

    walk(dir);
}

fn ensure_npm_deps(ui_dir: &Path) {
    let node_modules = ui_dir.join("node_modules");
    let lockfile = ui_dir.join("package-lock.json");
    let installed_lockfile = node_modules.join(".package-lock.json");

    if node_modules.exists() && !is_stale(&lockfile, &installed_lockfile) {
        return;
    }

    run_command("npm", &["ci"], ui_dir);
}

fn is_stale(source: &Path, installed: &Path) -> bool {
    let source = fs::metadata(source).and_then(|meta| meta.modified()).ok();
    let installed = fs::metadata(installed)
        .and_then(|meta| meta.modified())
        .ok();

    match (source, installed) {
        (Some(source), Some(installed)) => source > installed,
        _ => true,
    }
}

fn run_command(program: &str, args: &[&str], cwd: &Path) {
    let status = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .status()
        .unwrap_or_else(|err| {
            panic!(
                "Failed to run `{program} {}` in {}: {err}",
                args.join(" "),
                cwd.display()
            )
        });

    if !status.success() {
        panic!(
            "`{program} {}` failed in {} with status {}",
            args.join(" "),
            cwd.display(),
            status
        );
    }
}
