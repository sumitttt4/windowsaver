use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

// Only our own disposable fixtures are movable. There is no user-file delete command.
fn no_clobber_move(from: &Path, to: &Path) -> Result<(), String> {
    // Same-volume hard link gives atomic create-if-absent semantics. No rename overwrite race.
    // This spike intentionally does not implement cross-volume moves.
    fs::hard_link(from, to).map_err(|e| e.to_string())?;
    fs::remove_file(from).map_err(|e| format!("Both copies may remain; source unlink failed: {e}"))
}

pub fn fixture_checks() -> Result<Vec<String>, String> {
    use notify::{RecursiveMode, Watcher};
    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let source = dir.path().join("keep-fixture.txt");
    let dest = dir.path().join("approved-fixture.txt");
    fs::write(&source, b"original").map_err(|e| e.to_string())?;
    fs::write(&dest, b"occupied").map_err(|e| e.to_string())?;
    if no_clobber_move(&source, &dest).is_ok()
        || fs::read(&source).unwrap_or_default() != b"original"
        || fs::read(&dest).unwrap_or_default() != b"occupied"
    {
        return Err("Collision test failed".into());
    }
    let free = dir.path().join("free-fixture.txt");
    no_clobber_move(&source, &free)?;
    no_clobber_move(&free, &source)?;
    if fs::read(&source).map_err(|e| e.to_string())? != b"original" {
        return Err("Keep undo integrity failed".into());
    }
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |event| {
        let _ = tx.send(event);
    })
    .map_err(|e| e.to_string())?;
    watcher
        .watch(dir.path(), RecursiveMode::NonRecursive)
        .map_err(|e| e.to_string())?;
    let watched = dir.path().join("watch-fixture.txt");
    fs::write(&watched, b"watch").map_err(|e| e.to_string())?;
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let event = rx
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .map_err(|e| e.to_string())?;
        let event = event.map_err(|e| e.to_string())?;
        if event.paths.iter().any(|p| p == &watched) {
            break;
        }
        if Instant::now() >= deadline {
            return Err("Watcher timed out".into());
        }
    }
    Ok(vec![
        "PASS: collision leaves both files unchanged".into(),
        "PASS: Keep and undo preserve fixture bytes (same volume only)".into(),
        "PASS: native watcher received fixture event".into(),
    ])
}

#[cfg(windows)]
pub fn recycle_check() -> Result<Vec<String>, String> {
    use sha2::{Digest, Sha256};
    use trash::os_limited::{list, restore_all};
    // Persist sandbox: if recovery fails the original directory remains available.
    let dir: std::path::PathBuf = tempfile::tempdir().map_err(|e| e.to_string())?.keep();
    let path = dir.join("feed-the-pile-recovery-fixture.txt");
    let content = b"Feed the Pile disposable recovery fixture. No user file.";
    fs::write(&path, content).map_err(|e| e.to_string())?;
    let before = list().map_err(|e| e.to_string())?;
    trash::delete(&path)
        .map_err(|e| format!("Recycle failed; fixture at {}: {e}", path.display()))?;
    if path.exists() {
        return Err("Recycle operation did not remove fixture".into());
    }
    let matches: Vec<_> = list()
        .map_err(|e| e.to_string())?
        .into_iter()
        .filter(|item| item.original_path() == path && !before.iter().any(|old| old.id == item.id))
        .collect();
    if matches.len() != 1 {
        return Err(format!(
            "Could not identify exact fixture entry. Restore manually in Recycle Bin: {}",
            path.display()
        ));
    }
    // Validate collision rejection against the exact recorded entry.
    fs::write(&path, b"collision sentinel").map_err(|e| e.to_string())?;
    let collision = restore_all(matches.clone());
    if collision.is_ok() || fs::read(&path).unwrap_or_default() != b"collision sentinel" {
        return Err(format!(
            "FAIL: restore collision contract violated. Inspect {} and Recycle Bin",
            path.display()
        ));
    }
    fs::remove_file(&path).map_err(|e| e.to_string())?; // Only our newly-created sentinel.
    restore_all(matches)
        .map_err(|e| format!("Undo failed; fixture remains in Recycle Bin: {e}"))?;
    let restored = fs::read(&path).map_err(|e| e.to_string())?;
    if Sha256::digest(&restored) != Sha256::digest(content) {
        return Err("Restored fixture hash mismatch".into());
    }
    Ok(vec![
        "PASS: disposable fixture reached Recycle Bin".into(),
        "PASS: undo collision refused without overwrite".into(),
        "PASS: exact entry restored, SHA-256 unchanged".into(),
        format!("Restored fixture retained at {}", path.display()),
    ])
}

#[cfg(test)]
mod tests {
    #[test]
    fn collision_integrity_and_watch() {
        super::fixture_checks().unwrap();
    }
}
