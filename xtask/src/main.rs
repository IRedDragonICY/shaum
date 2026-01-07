//! XTask - Unified build system for Shaum
//!
//! Cross-platform Rust task runner for building WASM, Python bindings, and JSR/NPM packages.
//!
//! # Usage
//! ```sh
//! cargo xtask dist-web      # Build WASM + TypeScript + JSR
//! cargo xtask dist-python   # Build Python wheel
//! cargo xtask dev-web       # Local WASM testing server
//! cargo xtask publish-jsr   # Publish to JSR.io
//! cargo xtask publish-npm   # Publish to NPM
//! ```

use anyhow::{bail, Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    let dry_run = args.iter().any(|a| a == "--dry-run" || a == "-n");

    match args[1].as_str() {
        "dist-web" => dist_web()?,
        "dist-python" => dist_python()?,
        "dev-web" => dev_web()?,
        "build-all" => build_all()?,
        "sync-versions" => sync_versions()?,
        "publish-jsr" => publish_jsr(dry_run)?,
        "publish-npm" => publish_npm(dry_run)?,
        "publish-pypi" => publish_pypi(dry_run)?,
        "publish-crates" => publish_crates(dry_run)?,
        "publish-all" => publish_all(dry_run)?,
        "-h" | "--help" | "help" => print_usage(),
        cmd => {
            eprintln!("❌ Unknown command: {}", cmd);
            print_usage();
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_usage() {
    println!(r#"
🚀 Shaum XTask - Build Automation

USAGE:
    cargo xtask <COMMAND> [OPTIONS]

BUILD COMMANDS:
    dist-web      Build WASM package with TypeScript/JSR
                  Output: dist/web/, pkg/

    dist-python   Build Python wheel via maturin
                  Output: dist/python/

    dev-web       Create test HTML and serve locally
                  Output: dist/dev/

    build-all     Build all targets (Rust, WASM, Python)

    sync-versions Sync version from Cargo.toml to all manifests

PUBLISH COMMANDS:
    publish-crates  Publish all crates to crates.io
    publish-jsr     Publish to JSR.io (Deno/TypeScript)
    publish-npm     Publish to NPM
    publish-pypi    Publish to PyPI (Python)
    publish-all     Publish to all registries

OPTIONS:
    --dry-run, -n   Validate without actually publishing

EXAMPLES:
    cargo xtask dist-web
    cargo xtask publish-jsr --dry-run
    cargo xtask build-all
"#);
}

// =============================================================================
// Helper Functions
// =============================================================================

fn project_root() -> Result<PathBuf> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let root = PathBuf::from(manifest_dir)
        .parent()
        .context("Failed to find project root")?
        .to_path_buf();
    Ok(root)
}

fn run_cmd(cmd: &str, args: &[&str]) -> Result<()> {
    println!("  → {} {}", cmd, args.join(" "));
    
    let status = Command::new(cmd)
        .args(args)
        .current_dir(project_root()?)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("Failed to start: {} {}", cmd, args.join(" ")))?;

    if !status.success() {
        bail!("Command '{}' failed with exit code: {:?}", cmd, status.code());
    }
    Ok(())
}

fn run_cmd_in_dir(dir: &Path, cmd: &str, args: &[&str]) -> Result<()> {
    println!("  → [{}] {} {}", dir.display(), cmd, args.join(" "));
    
    #[cfg(windows)]
    let status = Command::new("cmd")
        .args(["/C", cmd])
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("Failed to start: {} {}", cmd, args.join(" ")))?;
    
    #[cfg(not(windows))]
    let status = Command::new(cmd)
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| format!("Failed to start: {} {}", cmd, args.join(" ")))?;

    if !status.success() {
        bail!("Command '{}' failed with exit code: {:?}", cmd, status.code());
    }
    Ok(())
}

fn command_exists(cmd: &str) -> bool {
    #[cfg(windows)]
    {
        Command::new("where")
            .arg(cmd)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        Command::new("which")
            .arg(cmd)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

fn copy_file(src: &Path, dst: &Path) -> Result<()> {
    if src.exists() {
        if let Some(parent) = dst.parent() {
            ensure_dir(parent)?;
        }
        fs::copy(src, dst)?;
        println!("  ✅ Copied {} → {}", src.file_name().unwrap_or_default().to_string_lossy(), dst.display());
    }
    Ok(())
}

fn read_cargo_version() -> Result<String> {
    let root = project_root()?;
    let cargo_path = root.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_path)?;
    
    for line in content.lines() {
        if line.trim().starts_with("version") && line.contains("=") {
            if let Some(version) = line.split('"').nth(1) {
                return Ok(version.to_string());
            }
        }
    }
    bail!("Could not find version in Cargo.toml")
}

// =============================================================================
// Task: sync-versions
// =============================================================================

fn sync_versions() -> Result<()> {
    let root = project_root()?;
    let version = read_cargo_version()?;
    
    println!("🔄 Syncing version {} to all manifests...", version);
    
    // Update jsr.json
    update_json_version(&root.join("jsr-config/jsr.json"), &version)?;
    update_json_version(&root.join("pkg/jsr.json"), &version)?;
    update_json_version(&root.join("pkg/package.json"), &version)?;
    
    // Update pyproject.toml
    update_pyproject_version(&root.join("bindings/shaum_py/pyproject.toml"), &version)?;
    
    println!("✅ Version sync complete!");
    Ok(())
}

fn update_json_version(path: &Path, version: &str) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    
    let content = fs::read_to_string(path)?;
    let updated = content
        .lines()
        .map(|line| {
            if line.trim().starts_with("\"version\"") {
                format!("    \"version\": \"{}\",", version)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    
    fs::write(path, updated)?;
    println!("  ✅ Updated {}", path.file_name().unwrap_or_default().to_string_lossy());
    Ok(())
}

fn update_pyproject_version(path: &Path, version: &str) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }
    
    let content = fs::read_to_string(path)?;
    let updated = content
        .lines()
        .map(|line| {
            if line.trim().starts_with("version =") {
                format!("version = \"{}\"", version)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    
    fs::write(path, updated)?;
    println!("  ✅ Updated pyproject.toml");
    Ok(())
}

// =============================================================================
// Task: dist-web (WASM + JSR + NPM)
// =============================================================================

fn dist_web() -> Result<()> {
    println!("\n🕸️  Building WASM Package for Web/JSR/NPM...\n");
    
    let root = project_root()?;
    let wasm_dir = root.join("bindings").join("shaum_wasm");
    let dist_web = root.join("dist").join("web");
    let pkg_dir = root.join("pkg");
    
    // Check for wasm-pack
    if !command_exists("wasm-pack") {
        println!("  ⚠️ wasm-pack not found. Installing...");
        run_cmd("cargo", &["install", "wasm-pack"])?;
    }
    
    // Build WASM targeting web browsers → dist/web
    println!("  🏗️  Building WASM (web target)...");
    run_cmd_in_dir(&wasm_dir, "wasm-pack", &[
        "build",
        "--target", "web",
        "--out-dir", dist_web.to_string_lossy().as_ref(),
        "--out-name", "shaum",
    ])?;
    
    // Also build for nodejs → pkg (for NPM/JSR)
    println!("  🏗️  Building WASM (nodejs target for NPM/JSR)...");
    run_cmd_in_dir(&wasm_dir, "wasm-pack", &[
        "build",
        "--target", "nodejs",
        "--out-dir", pkg_dir.to_string_lossy().as_ref(),
        "--out-name", "shaum",
    ])?;
    
    // Sync JSR config files
    sync_jsr_files(&root)?;
    
    // Patch package.json name for NPM
    let pkg_json = pkg_dir.join("package.json");
    if pkg_json.exists() {
        let content = fs::read_to_string(&pkg_json)?;
        let new_content = content.replace("\"name\": \"shaum-wasm\"", "\"name\": \"@islamic/shaum\"");
        fs::write(&pkg_json, new_content)?;
        println!("  ✅ Patched package.json: name = @islamic/shaum");
    }
    
    println!("\n✅ WASM build complete!");
    println!("   Web: dist/web/");
    println!("   NPM/JSR: pkg/");
    
    Ok(())
}

fn sync_jsr_files(root: &Path) -> Result<()> {
    println!("  📦 Syncing JSR/NPM metadata...");
    
    let pkg_dir = root.join("pkg");
    let jsr_config = root.join("jsr-config");
    
    // Copy JSR config files
    copy_file(&jsr_config.join("jsr.json"), &pkg_dir.join("jsr.json"))?;
    copy_file(&jsr_config.join("mod.ts"), &pkg_dir.join("mod.ts"))?;
    copy_file(&jsr_config.join("types.ts"), &pkg_dir.join("types.ts"))?;
    
    // Copy documentation
    copy_file(&root.join("README.md"), &pkg_dir.join("README.md"))?;
    copy_file(&root.join("LICENSE"), &pkg_dir.join("LICENSE"))?;
    
    println!("  ✅ JSR/NPM files synced!");
    Ok(())
}

// =============================================================================
// Task: dist-python
// =============================================================================

fn dist_python() -> Result<()> {
    println!("\n🐍 Building Python Package...\n");
    
    let root = project_root()?;
    let py_dir = root.join("bindings").join("shaum_py");
    let dist_python = root.join("dist").join("python");
    
    // Check for maturin
    if !command_exists("maturin") {
        println!("  ⚠️ maturin not found. Installing...");
        run_cmd("pip", &["install", "maturin"])?;
    }
    
    // Ensure dist directory exists
    ensure_dir(&dist_python)?;
    
    // Build Python wheel
    println!("  🏗️  Running maturin build...");
    run_cmd_in_dir(&py_dir, "maturin", &[
        "build",
        "--release",
        "--out", dist_python.to_string_lossy().as_ref(),
    ])?;
    
    println!("\n✅ Python build complete!");
    println!("   Output: dist/python/");
    println!("   Install: pip install dist/python/shaum-*.whl");
    
    Ok(())
}

// =============================================================================
// Task: publish-jsr
// =============================================================================

fn publish_jsr(dry_run: bool) -> Result<()> {
    println!("\n📦 Publishing to JSR.io...\n");
    
    let root = project_root()?;
    let pkg_dir = root.join("pkg");
    
    // Ensure pkg is up to date
    if !pkg_dir.join("shaum_bg.wasm").exists() {
        println!("  ⚠️ WASM not built. Building first...");
        dist_web()?;
    }
    
    // Check for deno or npx jsr
    let mut args = vec!["publish", "--allow-slow-types"];
    if dry_run {
        args.push("--dry-run");
        args.push("--allow-dirty");
        println!("  🔍 Dry run mode - validating only...");
    }
    
    if command_exists("deno") {
        run_cmd_in_dir(&pkg_dir, "deno", &args)?;
    } else {
        println!("  ℹ️  Using npx for JSR publish...");
        let mut npx_args = vec!["jsr"];
        npx_args.extend(args.clone());
        run_cmd_in_dir(&pkg_dir, "npx", &npx_args)?;
    }
    
    if dry_run {
        println!("\n✅ JSR validation complete!");
    } else {
        println!("\n✅ Published to JSR.io!");
    }
    
    Ok(())
}

// =============================================================================
// Task: publish-npm
// =============================================================================

fn publish_npm(dry_run: bool) -> Result<()> {
    println!("\n📦 Publishing to NPM...\n");
    
    let root = project_root()?;
    let pkg_dir = root.join("pkg");
    
    // Ensure pkg is up to date
    if !pkg_dir.join("shaum_bg.wasm").exists() {
        println!("  ⚠️ WASM not built. Building first...");
        dist_web()?;
    }
    
    let mut args = vec!["publish", "--access", "public"];
    if dry_run {
        args.push("--dry-run");
        println!("  🔍 Dry run mode - validating only...");
    }
    
    run_cmd_in_dir(&pkg_dir, "npm", &args)?;
    
    if dry_run {
        println!("\n✅ NPM validation complete!");
    } else {
        println!("\n✅ Published to NPM!");
    }
    
    Ok(())
}

// =============================================================================
// Task: publish-pypi
// =============================================================================

fn publish_pypi(dry_run: bool) -> Result<()> {
    println!("\n🐍 Publishing to PyPI...\n");
    
    let root = project_root()?;
    let py_dir = root.join("bindings").join("shaum_py");
    
    // Copy README.md to bindings/shaum_py so maturin can find it
    copy_file(&root.join("README.md"), &py_dir.join("README.md"))?;
    
    // Determine command: "maturin" or "python -m maturin"
    let cmd_args = if command_exists("maturin") {
        vec!["maturin"]
    } else {
        // Try to update/install
        println!("  ⚠️  'maturin' command not found in PATH. Checking pip...");
        run_cmd("pip", &["install", "maturin"])?;
        
        // Check if it's runnable via python module
        let status = std::process::Command::new("python")
             .args(&["-m", "maturin", "--version"])
             .stdout(std::process::Stdio::null())
             .stderr(std::process::Stdio::null())
             .status();
             
        if status.map(|s| s.success()).unwrap_or(false) {
            println!("  ℹ️  Using 'python -m maturin'");
            vec!["python", "-m", "maturin"]
        } else {
            // Warn but try 'maturin' anyway (maybe path updated?)
            println!("  ⚠️  Could not verify 'maturin'. Assuming it's in PATH or will fail.");
            vec!["maturin"]
        }
    };
    
    let (prog, base_args) = (cmd_args[0], &cmd_args[1..]);
    let mut args = base_args.to_vec();
    
    if dry_run {
        println!("  🔍 Dry run mode - building wheel only...");
        args.extend_from_slice(&["build", "--release"]);
        run_cmd_in_dir(&py_dir, prog, &args)?;
        println!("\n✅ Python wheel built successfully!");
    } else {
        args.push("publish");
        run_cmd_in_dir(&py_dir, prog, &args)?;
        println!("\n✅ Published to PyPI!");
    }
    
    Ok(())
}

// =============================================================================
// Task: publish-crates
// =============================================================================

/// Workspace crates in dependency order (leaves first, facade last)
const WORKSPACE_CRATES: &[&str] = &[
    "shaum-types",      // No internal deps
    "shaum-calendar",   // Depends on shaum-types
    "shaum-astronomy",  // Depends on shaum-types
    "shaum-rules",      // Depends on shaum-types, shaum-calendar, shaum-astronomy
    "shaum-network",    // Depends on shaum-types
    "shaum-core",       // Facade - depends on all above
];

fn publish_crates(dry_run: bool) -> Result<()> {
    println!("\n📦 Publishing crates to crates.io...\n");
    
    let root = project_root()?;
    
    for crate_name in WORKSPACE_CRATES {
        println!("  📦 Publishing {}...", crate_name);
        
        // Determine crate directory
        let crate_dir = if *crate_name == "shaum-core" {
            root.join("crates").join("shaum_core")
        } else {
            root.join("crates").join(crate_name)
        };
        
        let mut args = vec!["publish"];
        if dry_run {
            args.push("--dry-run");
        }
        
        // Run and capture output to handle "already exists" gracefully
        let output = std::process::Command::new("cargo")
            .args(&args)
            .current_dir(&crate_dir)
            .output()
            .with_context(|| format!("Failed to run cargo publish for {}", crate_name))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        if output.status.success() {
            println!("  ✅ {} published!", crate_name);
        } else if stderr.contains("already exists") {
            println!("  ⏭️  {} already published, skipping...", crate_name);
            continue;
        } else {
            // Print the error and fail
            eprintln!("{}", stdout);
            eprintln!("{}", stderr);
            bail!("Failed to publish {}", crate_name);
        }
        
        // Sleep between publishes to avoid rate limiting
        if !dry_run {
            println!("  ⏳ Waiting 30s for crates.io index update...");
            std::thread::sleep(std::time::Duration::from_secs(30));
        }
    }
    
    if dry_run {
        println!("\n✅ All crates validated!");
    } else {
        println!("\n✅ All crates published to crates.io!");
    }
    
    Ok(())
}

// =============================================================================
// Task: publish-all
// =============================================================================

fn publish_all(dry_run: bool) -> Result<()> {
    println!("\n🚀 Publishing to all registries...\n");
    
    // Crates.io first (other platforms may depend on it)
    publish_crates(dry_run)?;
    
    // Web platforms
    publish_jsr(dry_run)?;
    publish_npm(dry_run)?;
    
    // Python
    publish_pypi(dry_run)?;
    
    println!("\n✅ All publishing complete!");
    Ok(())
}

// =============================================================================
// Task: dev-web
// =============================================================================

fn dev_web() -> Result<()> {
    println!("\n🔧 Setting up local WASM development environment...\n");
    
    let root = project_root()?;
    let dist_dev = root.join("dist").join("dev");
    
    // First build WASM
    dist_web()?;
    
    // Create dev directory
    ensure_dir(&dist_dev)?;
    
    // Copy WASM files to dev
    let wasm_dist = root.join("dist").join("web");
    for entry in fs::read_dir(&wasm_dist)? {
        let entry = entry?;
        let src = entry.path();
        let dst = dist_dev.join(entry.file_name());
        if src.is_file() {
            fs::copy(&src, &dst)?;
        }
    }
    
    // Create test HTML file
    let html_content = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Shaum - WASM Test</title>
    <style>
        body { font-family: system-ui, sans-serif; max-width: 800px; margin: 2rem auto; padding: 0 1rem; }
        h1 { color: #1a5f2a; }
        input { padding: 0.5rem; font-size: 1rem; margin-right: 0.5rem; }
        button { padding: 0.5rem 1rem; font-size: 1rem; background: #1a5f2a; color: white; border: none; cursor: pointer; }
        button:hover { background: #145020; }
        #result { margin-top: 1rem; padding: 1rem; background: #f5f5f5; border-radius: 4px; white-space: pre-wrap; }
        .status-Wajib { color: #c00; font-weight: bold; }
        .status-Sunnah, .status-SunnahMuakkadah { color: #060; }
        .status-Haram { color: #900; font-weight: bold; }
        .status-Makruh { color: #960; }
    </style>
</head>
<body>
    <h1>🌙 Shaum - Fasting Status Analyzer</h1>
    <p>Enter a Gregorian date to check its Islamic fasting ruling.</p>
    
    <div>
        <input type="date" id="dateInput" value="">
        <button onclick="analyzeDate()">Analyze</button>
    </div>
    
    <div id="result">Enter a date and click Analyze</div>

    <script type="module">
        import init, { Shaum, analyze } from './shaum.js';
        
        window.analyzeDate = async function() {
            await init();
            
            const dateInput = document.getElementById('dateInput').value;
            const resultDiv = document.getElementById('result');
            
            if (!dateInput) {
                resultDiv.textContent = 'Please select a date';
                return;
            }
            
            try {
                const shaum = new Shaum(dateInput);
                const result = shaum.analyze();
                const explanation = shaum.explain();
                const hijri = shaum.hijri_date();
                
                resultDiv.innerHTML = `
<strong>Date:</strong> ${dateInput}
<strong>Hijri:</strong> ${hijri}
<strong>Status:</strong> <span class="status-${result.primaryStatus}">${result.primaryStatus}</span>
<strong>Reasons:</strong> ${result.reasons.join(', ') || 'None'}

<strong>Explanation:</strong>
${explanation}
                `.trim();
            } catch (e) {
                resultDiv.textContent = `Error: ${e}`;
            }
        };
        
        // Set default date to today
        document.getElementById('dateInput').valueAsDate = new Date();
    </script>
</body>
</html>"#;
    
    fs::write(dist_dev.join("index.html"), html_content)?;
    
    println!("\n✅ Development environment ready!");
    println!("   Output: dist/dev/");
    println!("\n   To test, run a local server:");
    println!("   python -m http.server 8080 -d dist/dev");
    println!("   Then open: http://localhost:8080");
    
    Ok(())
}

// =============================================================================
// Task: build-all
// =============================================================================

fn build_all() -> Result<()> {
    println!("\n🚀 Building All Targets...\n");
    
    // 0. Sync versions
    sync_versions()?;
    
    // 1. Rust core
    println!("\n🦀 Building Rust (Release)...");
    run_cmd("cargo", &["build", "--release", "-p", "shaum-core"])?;
    
    // 2. WASM + JSR
    dist_web()?;
    
    // 3. Python
    dist_python()?;
    
    println!("\n✅✅✅ ALL BUILDS COMPLETE! ✅✅✅");
    println!(" - Rust: target/release");
    println!(" - WASM/Web: dist/web/");
    println!(" - NPM/JSR: pkg/");
    println!(" - Python: dist/python/");
    
    Ok(())
}
