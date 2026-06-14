use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use tokio::sync::mpsc;

use crate::events::AppEvent;

pub fn open_in_editor(body: &str, editor_cmd: &str, tx: mpsc::Sender<AppEvent>) -> Result<()> {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let temp_path = PathBuf::from(format!("/tmp/farfetch_body_{ts}.json"));

    std::fs::write(&temp_path, body)?;

    let editor = editor_cmd.to_string();
    let path = temp_path.clone();

    tokio::task::spawn_blocking(move || {
        // Launch the editor. GUI editors (Zed, VS Code) don't block; terminal
        // editors (vim, nano) will block — both are fine in a spawn_blocking thread.
        let mut child = match std::process::Command::new(&editor).arg(&path).spawn() {
            Ok(c) => c,
            Err(_) => {
                let _ = std::fs::remove_file(&path);
                let _ = tx.blocking_send(AppEvent::EditorClosed);
                return;
            }
        };

        use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
        let (ntx, nrx) = std::sync::mpsc::channel();
        let mut watcher = match RecommendedWatcher::new(ntx, Config::default()) {
            Ok(w) => w,
            Err(_) => {
                let _ = std::fs::remove_file(&path);
                let _ = tx.blocking_send(AppEvent::EditorClosed);
                return;
            }
        };
        let _ = watcher.watch(&path, RecursiveMode::NonRecursive);

        loop {
            match nrx.recv_timeout(std::time::Duration::from_millis(200)) {
                Ok(Ok(e)) if matches!(e.kind, notify::EventKind::Modify(_)) => {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let _ = tx.blocking_send(AppEvent::FileChanged(content));
                        let _ = std::fs::remove_file(&path);
                        return;
                    }
                }
                Ok(Err(_)) => break,
                Err(_) => {
                    // Timeout: check if the editor process has exited without saving.
                    if let Ok(Some(_)) = child.try_wait() {
                        break;
                    }
                }
                _ => {}
            }
        }

        let _ = std::fs::remove_file(&path);
        let _ = tx.blocking_send(AppEvent::EditorClosed);
    });

    Ok(())
}
