fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "open_settings",
            "open_ai_suggestions",
            "use_ai_suggestion",
            "set_settings_theme",
            "desktop_capabilities",
            "set_ui_scale",
            "manage_ai_token",
            "open_project_repository",
            "repository_path_action",
            "dispatch",
            "pick_directory",
            "pick_ai_import",
            "pick_authentication_file",
            "job_status",
            "cancel_job",
            "close_snapshot",
            "settings_flush_done",
        ]),
    ))
    .expect("Tauri command permission generation failed");
}
