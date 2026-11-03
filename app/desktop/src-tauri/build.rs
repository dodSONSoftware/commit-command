fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "desktop_capabilities",
            "open_project_repository",
            "dispatch",
            "pick_directory",
            "pick_authentication_file",
            "job_status",
            "cancel_job",
            "review_data",
            "review_confirm",
            "review_cancel",
            "close_snapshot",
        ]),
    ))
    .expect("Tauri command permission generation failed");
}
