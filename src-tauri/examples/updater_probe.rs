#![allow(clippy::expect_used)]
//! Manual probe: `cargo run --example updater_probe` downloads and verifies
//! the latest release against the real manifest and public key.
use tauri_plugin_updater::UpdaterExt;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let updater = handle
                    .updater_builder()
                    .version_comparator(|_current, _remote| true)
                    .build()
                    .expect("build updater");
                match updater.check().await {
                    Ok(Some(u)) => {
                        eprintln!("[probe] available {} -> {}", u.current_version, u.version);
                        let mut total = 0usize;
                        match u
                            .download(
                                |chunk, len| {
                                    total += chunk;
                                    if let Some(len) = len {
                                        if total % (5 << 20) < chunk {
                                            eprintln!("[probe] {}/{}", total, len);
                                        }
                                    }
                                },
                                || eprintln!("[probe] download finished"),
                            )
                            .await
                        {
                            Ok(bytes) => eprintln!(
                                "[probe] downloaded + signature verified: {} bytes",
                                bytes.len()
                            ),
                            Err(e) => eprintln!("[probe] download/verify FAILED: {e}"),
                        }
                    }
                    Ok(None) => eprintln!("[probe] no update"),
                    Err(e) => eprintln!("[probe] check failed: {e}"),
                }
                handle.exit(0);
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("run");
}
