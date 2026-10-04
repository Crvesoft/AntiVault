fn main() {
    // `frontendDist` is embedded into the binary by `tauri::generate_context!()`, but
    // `tauri_build` only declares `tauri.conf.json` and `capabilities/` as inputs.
    // Without these directives a rebuilt frontend is never re-embedded, so the release
    // binary keeps serving the previous UI until some unrelated Rust file changes.
    println!("cargo:rerun-if-changed=../dist");
    println!("cargo:rerun-if-changed=../dist/index.html");
    println!("cargo:rerun-if-changed=../dist/assets");

    tauri_build::build()
}
