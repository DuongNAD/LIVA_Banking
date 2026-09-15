fn main() {
    // `tauri.conf.json` khai `"cuda-redist": "./"` trong `bundle.resources` để
    // ba DLL cuBLAS (752 MB) đáp xuống **cạnh .exe** — trên Windows
    // `resource_dir()` chính là thư mục chứa binary, và DLL load-time-linked
    // bắt buộc phải nằm ở đó chứ không phải trong thư mục con.
    //
    // Thư mục phải TỒN TẠI kể cả khi không dựng bản CUDA. Đọc `tauri-utils`:
    // nguồn là **thư mục rỗng** thì nó bỏ qua và đi tiếp (`resources.rs`, nhánh
    // `path.is_dir()`), nhưng nguồn **không tồn tại** thì thành lỗi bundle —
    // và một mẫu có `*` còn tệ hơn: glob không khớp gì là `GlobPathNotFound`,
    // lỗi cứng. Vì vậy ở đây dùng thư mục chứ không dùng glob, và tạo sẵn nó.
    //
    // Hệ quả: bản CPU (kể cả `release.yml`, vốn chạy `npx tauri build` không
    // kèm `--features cuda`) dựng được như cũ mà không cần biết gì về CUDA.
    // Đổ DLL vào đây bằng `node scripts/stage-cuda-redist.mjs`.
    let _ = std::fs::create_dir_all(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("cuda-redist"),
    );

    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "toggle_ghost_mode",
            "set_eco_mode",
            "update_interactive_zones",
            "open_dashboard",
            "open_setup",
            "vault_secret_present",
            "store_vault_secret",
            "delete_vault_secret",
            "issue_websocket_session",
            "native_ipc_call",
            "native_ipc_call_stream",
            "banking_get_overview",
            "statement_ingest_file",
            "banking_run_reconciliation",
            "banking_get_reconciliation_matrix",
            "reconciliation_resolve_hitl",
            "banking_get_compliance_status",
        ]),
    ))
    .expect("failed to build Tauri application manifest")
}
