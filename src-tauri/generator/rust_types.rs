// src-tauri/src/build/rust_types.rs
use std::fs;
use std::path::Path;

pub fn generate_rust_types() {
    // Prüfe ob die generierte Datei vom TypeScript-Script existiert
    let generated_path = Path::new("src/database/generated.rs");

    if !generated_path.exists() {
        eprintln!("⚠️  Warning: src/database/generated.rs not found!");
        eprintln!("   Run 'pnpm generate:rust-types' first.");

        // Erstelle eine leere Datei als Fallback
        fs::write(
            generated_path,
            "// Run 'pnpm generate:rust-types' to generate this file\n",
        )
        .ok();
    }

    println!("cargo:rerun-if-changed=src/database/generated.rs");
    println!("cargo:rerun-if-changed=src/database/schemas/crdt.ts");
    println!("cargo:rerun-if-changed=src/database/schemas/haex.ts");
}
