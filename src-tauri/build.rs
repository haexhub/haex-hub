mod generator;

fn main() {
    generator::table_names::generate_table_names();
    generator::rust_types::generate_rust_types();
    tauri_build::build();
}
