// build.rs — RustDB C++ bridge build script
//
// HOW THIS WORKS:
//   Cargo runs this script before compilation. It:
//     1. Scans cpp/src/ for every .cpp file and adds it to the C++ compilation unit
//     2. Runs cxx codegen on every bridge module declared in BRIDGE_MODULES
//     3. Links the resulting static library into the Rust binary
//
// TO ADD A NEW C++ FILE:
//   - Drop your .cpp into cpp/src/  →  it is compiled automatically
//   - Drop your .h  into cpp/include/rustdb/  →  it is on the include path automatically
//   - If it needs a Rust↔C++ bridge, add its module name to BRIDGE_MODULES below
//   - Define the bridge in rust-bridge/src/<module>.rs  (see buffer_pool.rs as template)
//
// NOTHING ELSE in Cargo.toml or lib.rs needs to change.

fn main() {
    // -----------------------------------------------------------------------
    // 1. Bridge modules — add the name of each rust-bridge/src/<name>.rs file
    //    that contains a #[cxx::bridge] block.
    // -----------------------------------------------------------------------
    const BRIDGE_MODULES: &[&str] = &[
        "buffer_pool",
        // "wal_writer",      ← uncomment / add new ones here
        // "disk_manager",
    ];

    // -----------------------------------------------------------------------
    // 2. Collect all .cpp source files from cpp/src/ automatically.
    //    You never need to list them individually.
    // -----------------------------------------------------------------------
    let cpp_sources = collect_cpp_sources("cpp/src");

    if cpp_sources.is_empty() {
        // No C++ files yet — emit a dummy compilation to keep the build valid
        // during the scaffold-only phase. Remove once real .cpp files exist.
        println!("cargo:warning=No C++ source files found in cpp/src/ — skipping C++ compilation.");
        return;
    }

    // -----------------------------------------------------------------------
    // 3. Run cxx codegen for every bridge module and build the C++ static lib.
    // -----------------------------------------------------------------------
    let mut build = cxx_build::bridges(
        BRIDGE_MODULES
            .iter()
            .map(|m| format!("rust-bridge/src/{m}.rs")),
    );

    // Compiler settings
    build
        .cpp(true)
        .std("c++17")
        .include("cpp/include")        // your headers live here
        .flag_if_supported("-Wall")
        .flag_if_supported("-Wextra")
        .flag_if_supported("-O2");

    for src in &cpp_sources {
        build.file(src);
    }

    build.compile("rustdb_cpp"); // produces librustdb_cpp.a

    // -----------------------------------------------------------------------
    // 4. Tell Cargo when to re-run this script.
    // -----------------------------------------------------------------------
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=cpp/src");
    println!("cargo:rerun-if-changed=cpp/include");
    println!("cargo:rerun-if-changed=rust-bridge/src");
}

// ---------------------------------------------------------------------------
// Helper: recursively collect all .cpp files under a directory
// ---------------------------------------------------------------------------
fn collect_cpp_sources(dir: &str) -> Vec<std::path::PathBuf> {
    let mut sources = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return sources;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Recurse into subdirectories (e.g. cpp/src/storage/)
            if let Some(sub) = path.to_str() {
                sources.extend(collect_cpp_sources(sub));
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("cpp") {
            sources.push(path);
        }
    }
    sources
} 
