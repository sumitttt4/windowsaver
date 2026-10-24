#[cfg(windows)]
mod recycle;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
pub type Result<T> = std::result::Result<T, String>;
fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub downloads: Option<PathBuf>,
    pub screenshots: Option<PathBuf>,
    pub keep: Option<PathBuf>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub modified: u64,
    pub extension: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Operation {
    pub id: i64,
    pub kind: String,
    pub source: PathBuf,
    pub destination: Option<PathBuf>,
    pub hash: String,
    pub status: String,
    pub recycle_id: Option<String>,
    pub before_ids: Vec<String>,
    pub previous_rank: Option<i64>,
    pub message: String,
}
#[derive(Serialize)]
pub struct Snapshot {
    pub config: Config,
    pub downloads: Vec<Card>,
    pub screenshots: Vec<Card>,
    pub history: Vec<Operation>,
    pub skipped: usize,
    pub warning: Option<String>,
}
#[derive(Clone, Debug)]
pub struct Recycled {
    pub id: String,
    pub original: PathBuf,
}
pub trait Filesystem {
    fn move_file(&self, from: &Path, to: &Path) -> Result<()>;
    fn recycle(&self, from: &Path) -> Result<()>;
    fn recycled(&self) -> Result<Vec<Recycled>>;
    fn restore(&self, id: &str) -> Result<()>;
}
fn same_path(a: &Path, b: &Path) -> bool {
    #[cfg(windows)]
    {
        let a = a.to_string_lossy();
        let b = b.to_string_lossy();
        a.strip_prefix(r"\\?\").unwrap_or(&a) == b.strip_prefix(r"\\?\").unwrap_or(&b)
    }
    #[cfg(not(windows))]
    {
        a == b
    }
}
pub struct Native;
impl Filesystem for Native {
    fn move_file(&self, from: &Path, to: &Path) -> Result<()> {
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            use windows_sys::Win32::Storage::FileSystem::{
                MoveFileExW, MOVEFILE_COPY_ALLOWED, MOVEFILE_WRITE_THROUGH,
            };
            let a: Vec<u16> = from.as_os_str().encode_wide().chain(Some(0)).collect();
            let b: Vec<u16> = to.as_os_str().encode_wide().chain(Some(0)).collect();
            if unsafe {
                MoveFileExW(
                    a.as_ptr(),
                    b.as_ptr(),
                    MOVEFILE_COPY_ALLOWED | MOVEFILE_WRITE_THROUGH,
                )
            } == 0
            {
                return Err(err(std::io::Error::last_os_error()));
            }
            if from.exists() {
                return Err(
                    "Windows retained the source after copying. Both files kept; review recovery."
                        .into(),
                );
            }
            Ok(())
        }
        #[cfg(not(windows))]
        {
            // Only used by fixture tests on Linux; desktop mutation is Windows-only.
            fs::hard_link(from, to).map_err(err)?;
            fs::remove_file(from).map_err(err)
        }
    }
    fn recycle(&self, from: &Path) -> Result<()> {
        #[cfg(windows)]
        {
            recycle::recycle_only(from)
        }
        #[cfg(not(windows))]
        {
            let _ = from;
            Err("Recycle Bin requires Windows".into())
        }
    }
    fn recycled(&self) -> Result<Vec<Recycled>> {
        #[cfg(windows)]
        {
            Ok(trash::os_limited::list()
                .map_err(err)?
                .into_iter()
                .map(|x| Recycled {
                    id: x.id.to_string_lossy().into(),
                    original: x.original_path(),
                })
                .collect())
        }
        #[cfg(not(windows))]
        {
            Err("Recycle Bin requires Windows".into())
        }
    }
    fn restore(&self, id: &str) -> Result<()> {
        #[cfg(windows)]
        {
            let entries: Vec<_> = trash::os_limited::list()
                .map_err(err)?
                .into_iter()
                .filter(|x| x.id.to_string_lossy() == id)
                .collect();
            if entries.len() != 1 {
                return Err("That exact Recycle Bin entry is unavailable. It may have been emptied or restored elsewhere.".into());
            }
            recycle::restore_exact(&entries[0].id, &entries[0].original_path())
        }
        #[cfg(not(windows))]
        {
            let _ = id;
            Err("Recycle Bin requires Windows".into())
        }
    }
}
pub struct Engine<F: Filesystem = Native> {
    db: Connection,
    fs: F,
    pub config: Config,
    recovery_error: Option<String>,
}
fn is_link(path: &Path) -> Result<bool> {
    let meta = fs::symlink_metadata(path).map_err(err)?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        Ok(meta.file_type().is_symlink() || meta.file_attributes() & 0x400 != 0)
    }
    #[cfg(not(windows))]
    {
        Ok(meta.file_type().is_symlink())
    }
}
pub fn validated_folder(path: &Path) -> Result<PathBuf> {
    if !path.is_dir() {
        return Err("Choose an existing folder".into());
    }
    for ancestor in path.ancestors() {
        if is_link(ancestor)? {
            return Err("Linked folders and cloud placeholders are not supported. Choose a regular local folder.".into());
        }
    }
    let canonical = fs::canonicalize(path).map_err(err)?;
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::{GetDriveTypeW, GetVolumePathNameW};
        let raw: Vec<u16> = canonical.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut volume = [0u16; 1024];
        if unsafe { GetVolumePathNameW(raw.as_ptr(), volume.as_mut_ptr(), volume.len() as u32) }
            == 0
            || unsafe { GetDriveTypeW(volume.as_ptr()) } != 3
        {
            return Err("Choose a fixed local drive. Network and removable drives are excluded for safe recycling.".into());
        }
    }
    Ok(canonical)
}
fn hash(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path).map_err(err)?;
    let mut digest = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf).map_err(err)?;
        if n == 0 {
            break;
        }
        digest.update(&buf[..n]);
    }
    Ok(format!("{:x}", digest.finalize()))
}
fn card(path: PathBuf) -> Result<Card> {
    if path.to_str().is_none() {
        return Err("Non-Unicode filename is not supported".into());
    }
    if is_link(&path)? {
        return Err("Linked files are excluded".into());
    }
    let m = fs::metadata(&path).map_err(err)?;
    if !m.is_file() {
        return Err("Only regular files are supported".into());
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if m.file_attributes() & (0x2 | 0x4 | 0x1000 | 0x400000) != 0 {
            return Err("Hidden, system, and offline files are excluded".into());
        }
    }
    let stamp = m
        .modified()
        .map_err(err)?
        .duration_since(UNIX_EPOCH)
        .map_err(err)?;
    let signature = format!("{}:{}:{}", path.display(), m.len(), stamp.as_nanos());
    let name = path
        .file_name()
        .ok_or("Missing file name")?
        .to_string_lossy()
        .into_owned();
    let extension = path
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_lowercase();
    if name.starts_with('.')
        || ["part", "partial", "crdownload", "download", "tmp"].contains(&extension.as_str())
    {
        return Err("Incomplete or hidden file".into());
    }
    if now().saturating_sub(stamp.as_secs()) < 3 {
        return Err("File is still settling; retry shortly".into());
    }
    Ok(Card {
        id: format!("{:x}", Sha256::digest(signature.as_bytes())),
        path,
        name,
        size: m.len(),
        modified: stamp.as_secs(),
        extension,
    })
}
impl<F: Filesystem> Engine<F> {
    pub fn open(path: &Path, filesystem: F) -> Result<Self> {
        let db = Connection::open(path).map_err(err)?;
        db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; CREATE TABLE IF NOT EXISTS settings(key TEXT PRIMARY KEY, value TEXT NOT NULL); CREATE TABLE IF NOT EXISTS operations(id INTEGER PRIMARY KEY AUTOINCREMENT, body TEXT NOT NULL); CREATE TABLE IF NOT EXISTS later(path TEXT PRIMARY KEY, rank INTEGER NOT NULL);").map_err(err)?;
        let config = db
            .query_row("SELECT value FROM settings WHERE key='config'", [], |r| {
                r.get::<_, String>(0)
            })
            .ok()
            .map(|s| serde_json::from_str(&s).map_err(err))
            .transpose()?
            .unwrap_or_default();
        let mut engine = Self {
            db,
            fs: filesystem,
            config,
            recovery_error: None,
        };
        if let Err(e) = engine.recover() {
            engine.recovery_error = Some(e);
        }
        Ok(engine)
    }
    pub fn configure(&mut self, kind: &str, folder: &Path) -> Result<()> {
        let folder = validated_folder(folder)?;
        let mut next = self.config.clone();
        match kind {
            "downloads" => next.downloads = Some(folder),
            "screenshots" => next.screenshots = Some(folder),
            "keep" => next.keep = Some(folder),
            _ => return Err("Unknown folder kind".into()),
        }
        let dirs: Vec<_> = [&next.downloads, &next.screenshots, &next.keep]
            .into_iter()
            .flatten()
            .collect();
        for (i, a) in dirs.iter().enumerate() {
            for b in dirs.iter().skip(i + 1) {
                if a.starts_with(b) || b.starts_with(a) {
                    return Err(
                        "Choose three separate folders; they must not contain one another.".into(),
                    );
                }
            }
        }
        self.db
            .execute(
                "INSERT OR REPLACE INTO settings VALUES ('config',?1)",
                [serde_json::to_string(&next).map_err(err)?],
            )
            .map_err(err)?;
        self.config = next;
        Ok(())
    }
    fn ranks(&self) -> Result<HashMap<PathBuf, i64>> {
        let mut q = self
            .db
            .prepare("SELECT path,rank FROM later")
            .map_err(err)?;
        let rows = q
            .query_map([], |r| {
                Ok((PathBuf::from(r.get::<_, String>(0)?), r.get(1)?))
            })
            .map_err(err)?;
        rows.map(|r| r.map_err(err)).collect()
    }
    pub fn history(&self) -> Result<Vec<Operation>> {
        let mut q = self
            .db
            .prepare("SELECT body FROM operations ORDER BY id DESC")
            .map_err(err)?;
        let rows = q.query_map([], |r| r.get::<_, String>(0)).map_err(err)?;
        rows.map(|r| serde_json::from_str(&r.map_err(err)?).map_err(err))
            .collect()
    }
    fn save(&self, op: &Operation) -> Result<()> {
        self.db
            .execute(
                "UPDATE operations SET body=?1 WHERE id=?2",
                params![serde_json::to_string(op).map_err(err)?, op.id],
            )
            .map_err(err)?;
        Ok(())
    }
    fn insert(&self, mut op: Operation) -> Result<Operation> {
        let tx = self.db.unchecked_transaction().map_err(err)?;
        tx.execute("INSERT INTO operations(body) VALUES ('{}')", [])
            .map_err(err)?;
        op.id = tx.last_insert_rowid();
        tx.execute(
            "UPDATE operations SET body=?1 WHERE id=?2",
            params![serde_json::to_string(&op).map_err(err)?, op.id],
        )
        .map_err(err)?;
        tx.commit().map_err(err)?;
        Ok(op)
    }
    fn list(&self, root: &Option<PathBuf>, skipped: &mut usize) -> Result<Vec<Card>> {
        let Some(root) = root else { return Ok(vec![]) };
        validated_folder(root)?;
        let mut cards = vec![];
        for entry in fs::read_dir(root).map_err(err)? {
            match entry.map_err(err).and_then(|e| card(e.path())) {
                Ok(c) => cards.push(c),
                Err(_) => *skipped += 1,
            }
        }
        let ranks = self.ranks()?;
        cards.sort_by(|a, b| {
            ranks
                .get(&a.path)
                .copied()
                .unwrap_or(0)
                .cmp(&ranks.get(&b.path).copied().unwrap_or(0))
                .then(b.modified.cmp(&a.modified))
                .then(a.name.cmp(&b.name))
        });
        Ok(cards)
    }
    pub fn snapshot(&self) -> Result<Snapshot> {
        let mut skipped = 0;
        let mut warnings = vec![];
        if let Some(e) = &self.recovery_error {
            warnings.push(format!("Recovery: {e}"));
        }
        let downloads = self
            .list(&self.config.downloads, &mut skipped)
            .unwrap_or_else(|e| {
                warnings.push(format!("Downloads: {e}"));
                vec![]
            });
        let screenshots = self
            .list(&self.config.screenshots, &mut skipped)
            .unwrap_or_else(|e| {
                warnings.push(format!("Screenshots: {e}"));
                vec![]
            });
        Ok(Snapshot {
            config: self.config.clone(),
            downloads,
            screenshots,
            history: self.history()?,
            skipped,
            warning: if warnings.is_empty() {
                None
            } else {
                Some(warnings.join(" · "))
            },
        })
    }
    pub fn checked_card(&self, pile: &str, path: &Path, id: &str) -> Result<Card> {
        let root = match pile {
            "downloads" => &self.config.downloads,
            "screenshots" => &self.config.screenshots,
            _ => return Err("Unknown pile".into()),
        }
        .as_ref()
        .ok_or("Choose this pile's folder first")?;
        validated_folder(root)?;
        if path.parent() != Some(root.as_path()) {
            return Err("File is outside the approved folder".into());
        }
        let c = card(path.to_path_buf())?;
        if c.id != id {
            return Err("This file changed. Refresh the pile before deciding.".into());
        }
        Ok(c)
    }
    pub fn decide(&mut self, pile: &str, path: &Path, id: &str, kind: &str) -> Result<Operation> {
        self.recover()?;
        if self.history()?.iter().any(|o| o.status == "review") {
            return Err(
                "An interrupted operation needs review before more files can move. Open History."
                    .into(),
            );
        }
        self.checked_card(pile, path, id)?;
        if !["keep", "later", "bin"].contains(&kind) {
            return Err("Unknown decision".into());
        }
        let destination = if kind == "keep" {
            let root = validated_folder(
                self.config
                    .keep
                    .as_ref()
                    .ok_or("Choose an approved Keep folder first")?,
            )?;
            let name = path.file_name().ok_or("Missing file name")?;
            let mut candidate = root.join(name);
            let stem = path.file_stem().unwrap_or_default().to_string_lossy();
            let ext = path
                .extension()
                .map(|x| format!(".{}", x.to_string_lossy()))
                .unwrap_or_default();
            let mut n = 1;
            while candidate.exists() {
                candidate = root.join(format!("{stem} ({n}){ext}"));
                n += 1;
                if n > 10000 {
                    return Err("Too many filename collisions".into());
                }
            }
            Some(candidate)
        } else {
            None
        };
        let before = if kind == "bin" {
            self.fs.recycled()?.into_iter().map(|r| r.id).collect()
        } else {
            vec![]
        };
        let previous_rank = self.ranks()?.get(path).copied();
        let digest = if kind == "later" {
            String::new()
        } else {
            hash(path)?
        };
        self.checked_card(pile, path, id)?;
        let mut op = self.insert(Operation {
            id: 0,
            kind: kind.into(),
            source: path.into(),
            destination,
            hash: digest,
            status: "applying".into(),
            recycle_id: None,
            before_ids: before,
            previous_rank,
            message: String::new(),
        })?;
        let result = match kind {
            "later" => self
                .db
                .execute(
                    "INSERT OR REPLACE INTO later VALUES (?1,?2)",
                    params![path.to_string_lossy(), op.id],
                )
                .map(|_| ())
                .map_err(err),
            "keep" => self.fs.move_file(path, op.destination.as_ref().unwrap()),
            "bin" => self.fs.recycle(path),
            _ => unreachable!(),
        };
        if let Err(e) = result {
            op.message = e;
            self.save(&op)?;
            self.recover()?;
            return Err(format!(
                "Decision did not finish cleanly. Check History: {}",
                op.message
            ));
        }
        if kind == "later" {
            op.status = "done".into();
            self.save(&op)?;
        } else {
            self.recover()?;
            op = self
                .history()?
                .into_iter()
                .find(|x| x.id == op.id)
                .ok_or("Missing journal entry")?;
            if op.status != "done" {
                return Err(op.message.clone());
            }
        }
        Ok(op)
    }
    pub fn recover(&mut self) -> Result<()> {
        for mut op in self
            .history()?
            .into_iter()
            .filter(|o| ["applying", "undoing", "review"].contains(&o.status.as_str()))
        {
            if op.kind == "later" {
                // Queue ranks are idempotent and never touch files.
                if op.status == "undoing" {
                    self.restore_rank(&op)?;
                    op.status = "undone".into();
                } else {
                    self.db
                        .execute(
                            "INSERT OR REPLACE INTO later VALUES (?1,?2)",
                            params![op.source.to_string_lossy(), op.id],
                        )
                        .map_err(err)?;
                    op.status = "done".into();
                }
            } else if op.kind == "keep" {
                let dest = op
                    .destination
                    .as_ref()
                    .ok_or("Journal missing Keep destination")?;
                let src_ok =
                    op.source.exists() && hash(&op.source).ok().as_deref() == Some(&op.hash);
                let dst_ok = dest.exists() && hash(dest).ok().as_deref() == Some(&op.hash);
                if !op.source.exists() && dst_ok {
                    op.status = "done".into();
                    op.message.clear();
                } else if src_ok && !dest.exists() {
                    op.status = if op.status == "undoing" {
                        "undone"
                    } else {
                        "failed"
                    }
                    .into();
                } else {
                    op.status = "review".into();
                    op.message="Interrupted Keep: both copies exist, content changed, or a file is unavailable. Nothing will be removed automatically. Inspect the recorded paths, then retry recovery.".into();
                }
            } else if op.kind == "bin" {
                let entries = self.fs.recycled()?;
                let matched: Vec<_> = entries
                    .iter()
                    .filter(|r| {
                        same_path(&r.original, &op.source)
                            && match &op.recycle_id {
                                Some(id) => &r.id == id,
                                None => !op.before_ids.contains(&r.id),
                            }
                    })
                    .collect();
                if !op.source.exists() && matched.len() == 1 {
                    op.recycle_id = Some(matched[0].id.clone());
                    op.status = "done".into();
                    op.message.clear();
                } else if op.source.exists() && matched.is_empty() {
                    op.status = if op.status == "undoing"
                        && hash(&op.source).ok().as_deref() == Some(&op.hash)
                    {
                        "undone"
                    } else {
                        "failed"
                    }
                    .into();
                } else {
                    op.status = "review".into();
                    op.message="Recycle recovery is ambiguous. Inspect the exact source path and Recycle Bin; no entry was guessed or purged.".into();
                }
            }
            self.save(&op)?;
        }
        self.recovery_error = None;
        Ok(())
    }
    fn restore_rank(&self, op: &Operation) -> Result<()> {
        match op.previous_rank {
            Some(rank) => {
                self.db
                    .execute(
                        "INSERT OR REPLACE INTO later VALUES (?1,?2)",
                        params![op.source.to_string_lossy(), rank],
                    )
                    .map_err(err)?;
            }
            None => {
                self.db
                    .execute(
                        "DELETE FROM later WHERE path=?1",
                        [op.source.to_string_lossy()],
                    )
                    .map_err(err)?;
            }
        }
        Ok(())
    }
    pub fn undo(&mut self) -> Result<Operation> {
        self.recover()?;
        let mut op = self
            .history()?
            .into_iter()
            .find(|x| x.status == "done")
            .ok_or("Nothing to undo")?;
        if op.kind != "later" {
            if op.source.exists() {
                return Err("Undo paused: the original filename is occupied. Move that file elsewhere, then retry. Nothing was overwritten.".into());
            }
            if let Some(parent) = op.source.parent() {
                validated_folder(parent)?;
            }
        }
        if op.kind == "keep" {
            let dest = op.destination.as_ref().ok_or("Missing destination")?;
            validated_folder(dest.parent().ok_or("Missing destination folder")?)?;
            if is_link(dest)? || hash(dest)? != op.hash {
                return Err(
                    "Kept file changed since the move. Undo refused; inspect History.".into(),
                );
            }
        }
        op.status = "undoing".into();
        self.save(&op)?;
        let result = match op.kind.as_str() {
            "later" => self.restore_rank(&op),
            "keep" => self
                .fs
                .move_file(op.destination.as_ref().unwrap(), &op.source),
            "bin" => self.fs.restore(
                op.recycle_id
                    .as_deref()
                    .ok_or("Missing exact Recycle Bin entry")?,
            ),
            _ => Err("Unknown journal action".into()),
        };
        if let Err(e) = result {
            op.status = "done".into();
            op.message = e.clone();
            self.save(&op)?;
            return Err(e);
        }
        if op.kind != "later" && hash(&op.source)? != op.hash {
            op.status = "review".into();
            op.message="Restored content differs. Inspect the original path; no further file changes were made.".into();
            self.save(&op)?;
            return Err(op.message);
        }
        op.status = "undone".into();
        op.message.clear();
        self.save(&op)?;
        Ok(op)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use tempfile::TempDir;
    struct FakeBin {
        base: PathBuf,
        items: RefCell<HashMap<String, PathBuf>>,
    }
    impl Filesystem for FakeBin {
        fn move_file(&self, a: &Path, b: &Path) -> Result<()> {
            Native.move_file(a, b)
        }
        fn recycle(&self, a: &Path) -> Result<()> {
            let id = format!("fixture-{}", self.items.borrow().len());
            let b = self.base.join(&id);
            Native.move_file(a, &b)?;
            self.items.borrow_mut().insert(id, a.into());
            Ok(())
        }
        fn recycled(&self) -> Result<Vec<Recycled>> {
            Ok(self
                .items
                .borrow()
                .iter()
                .map(|(id, p)| Recycled {
                    id: id.clone(),
                    original: p.clone(),
                })
                .collect())
        }
        fn restore(&self, id: &str) -> Result<()> {
            let source = self.items.borrow().get(id).ok_or("entry missing")?.clone();
            Native.move_file(&self.base.join(id), &source)?;
            self.items.borrow_mut().remove(id);
            Ok(())
        }
    }
    fn fixture() -> (TempDir, Engine<FakeBin>) {
        let dir = tempfile::tempdir().unwrap();
        for name in ["downloads", "screenshots", "keep", "bin"] {
            fs::create_dir(dir.path().join(name)).unwrap();
        }
        let mut e = Engine::open(
            &dir.path().join("journal.db"),
            FakeBin {
                base: dir.path().join("bin"),
                items: RefCell::new(HashMap::new()),
            },
        )
        .unwrap();
        for kind in ["downloads", "screenshots", "keep"] {
            e.configure(kind, &dir.path().join(kind)).unwrap();
        }
        (dir, e)
    }
    fn make(e: &Engine<FakeBin>, name: &str) -> Card {
        let path = e.config.downloads.as_ref().unwrap().join(name);
        fs::write(&path, b"real fixture content").unwrap();
        filetime::set_file_mtime(
            &path,
            filetime::FileTime::from_unix_time(now() as i64 - 10, 0),
        )
        .unwrap();
        card(path).unwrap()
    }
    #[test]
    fn keep_collision_and_undo_are_lossless() {
        let (_dir, mut e) = fixture();
        let c = make(&e, "notes.txt");
        let existing = e.config.keep.as_ref().unwrap().join("notes.txt");
        fs::write(&existing, b"do not overwrite").unwrap();
        let op = e.decide("downloads", &c.path, &c.id, "keep").unwrap();
        assert!(op.destination.as_ref().unwrap().ends_with("notes (1).txt"));
        assert_eq!(fs::read(existing).unwrap(), b"do not overwrite");
        assert!(!c.path.exists());
        e.undo().unwrap();
        assert_eq!(fs::read(c.path).unwrap(), b"real fixture content");
    }
    #[test]
    fn undo_collision_and_external_edit_refuse() {
        let (_dir, mut e) = fixture();
        let c = make(&e, "one.txt");
        let op = e.decide("downloads", &c.path, &c.id, "keep").unwrap();
        fs::write(&c.path, b"new arrival").unwrap();
        assert!(e.undo().unwrap_err().contains("occupied"));
        assert_eq!(fs::read(&c.path).unwrap(), b"new arrival");
        fs::remove_file(&c.path).unwrap();
        fs::write(op.destination.unwrap(), b"edited elsewhere").unwrap();
        assert!(e.undo().unwrap_err().contains("changed"));
    }
    #[test]
    fn later_never_clears_pile_and_is_undoable() {
        let (_dir, mut e) = fixture();
        let a = make(&e, "a.txt");
        let b = make(&e, "b.txt");
        e.decide("downloads", &a.path, &a.id, "later").unwrap();
        let s = e.snapshot().unwrap();
        assert_eq!(s.downloads.len(), 2);
        assert_eq!(s.downloads[0].id, b.id);
        e.undo().unwrap();
        assert!(!e.ranks().unwrap().contains_key(&a.path));
        assert!(a.path.exists());
    }
    #[test]
    fn bin_records_exact_identity_and_undo() {
        let (_dir, mut e) = fixture();
        let c = make(&e, "bin.txt");
        let op = e.decide("downloads", &c.path, &c.id, "bin").unwrap();
        assert_eq!(op.status, "done");
        assert!(op.recycle_id.is_some());
        assert!(!c.path.exists());
        e.undo().unwrap();
        assert_eq!(fs::read(c.path).unwrap(), b"real fixture content");
    }
    #[test]
    fn empty_bin_reports_failure_and_keeps_journal() {
        let (_dir, mut e) = fixture();
        let c = make(&e, "missing.txt");
        e.decide("downloads", &c.path, &c.id, "bin").unwrap();
        e.fs.items.borrow_mut().clear();
        assert!(e.undo().is_err());
        assert_eq!(e.history().unwrap()[0].status, "done");
    }
    #[test]
    fn changed_card_and_unapproved_paths_are_rejected() {
        let (_dir, mut e) = fixture();
        let c = make(&e, "changed.txt");
        fs::write(&c.path, b"new version").unwrap();
        assert!(e.decide("downloads", &c.path, &c.id, "keep").is_err());
        assert!(e.checked_card("screenshots", &c.path, &c.id).is_err());
        assert!(e
            .configure("keep", &e.config.downloads.clone().unwrap())
            .is_err());
    }
    #[test]
    fn interrupted_move_recovers_after_restart() {
        let (dir, e) = fixture();
        let c = make(&e, "restart.txt");
        let dest = e.config.keep.as_ref().unwrap().join("restart.txt");
        let op = e
            .insert(Operation {
                id: 0,
                kind: "keep".into(),
                source: c.path.clone(),
                destination: Some(dest.clone()),
                hash: hash(&c.path).unwrap(),
                status: "applying".into(),
                recycle_id: None,
                before_ids: vec![],
                previous_rank: None,
                message: String::new(),
            })
            .unwrap();
        Native.move_file(&c.path, &dest).unwrap();
        drop(e);
        let mut restored = Engine::open(
            &dir.path().join("journal.db"),
            FakeBin {
                base: dir.path().join("bin"),
                items: RefCell::new(HashMap::new()),
            },
        )
        .unwrap();
        assert_eq!(restored.history().unwrap()[0].id, op.id);
        assert_eq!(restored.history().unwrap()[0].status, "done");
        restored.undo().unwrap();
        assert!(c.path.exists());
    }
    #[test]
    fn interrupted_copy_never_removes_either_copy() {
        let (_dir, mut e) = fixture();
        let c = make(&e, "both.txt");
        let dest = e.config.keep.as_ref().unwrap().join("both.txt");
        fs::copy(&c.path, &dest).unwrap();
        e.insert(Operation {
            id: 0,
            kind: "keep".into(),
            source: c.path.clone(),
            destination: Some(dest.clone()),
            hash: hash(&c.path).unwrap(),
            status: "applying".into(),
            recycle_id: None,
            before_ids: vec![],
            previous_rank: None,
            message: String::new(),
        })
        .unwrap();
        e.recover().unwrap();
        assert_eq!(e.history().unwrap()[0].status, "review");
        assert!(c.path.exists() && dest.exists());
        let b = make(&e, "another.txt");
        assert!(e.decide("downloads", &b.path, &b.id, "keep").is_err());
    }
    #[cfg(unix)]
    #[test]
    fn links_are_excluded() {
        let (_dir, e) = fixture();
        let c = make(&e, "original.txt");
        let link = e.config.downloads.as_ref().unwrap().join("linked.txt");
        std::os::unix::fs::symlink(&c.path, &link).unwrap();
        assert!(card(link).is_err());
    }
    #[cfg(windows)]
    #[test]
    fn shell_and_verbatim_paths_identify_the_same_recycled_file() {
        assert!(same_path(
            Path::new(r"C:\Users\test\Downloads\one.txt"),
            Path::new(r"\\?\C:\Users\test\Downloads\one.txt")
        ));
        assert!(!same_path(
            Path::new(r"C:\one.txt"),
            Path::new(r"C:\two.txt")
        ));
    }
    #[cfg(windows)]
    #[test]
    fn windows_recycle_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let p = fs::canonicalize(dir.path()).unwrap().join("feed-the-pile-generated-fixture.txt");
        fs::write(&p, b"fixture").unwrap();
        let before = Native.recycled().unwrap();
        Native.recycle(&p).unwrap();
        let all = Native.recycled().unwrap();
        eprintln!("RECYCLE_DIAGNOSTIC source={p:?} exists={} entries={all:?}", p.exists());
        let entries: Vec<_> = all
            .into_iter()
            .filter(|x| same_path(&x.original, &p) && !before.iter().any(|b| b.id == x.id))
            .collect();
        assert_eq!(entries.len(), 1);
        Native.restore(&entries[0].id).unwrap();
        assert_eq!(fs::read(p).unwrap(), b"fixture");
    }
    #[cfg(windows)]
    #[test]
    fn windows_native_engine_bin_restart_undo_and_collision() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["downloads", "screenshots", "keep"] {
            fs::create_dir(dir.path().join(name)).unwrap();
        }
        let db = dir.path().join("journal.sqlite3");
        let mut e = Engine::open(&db, Native).unwrap();
        for name in ["downloads", "screenshots", "keep"] {
            e.configure(name, &dir.path().join(name)).unwrap();
        }
        let source = e.config.downloads.as_ref().unwrap().join("native-fixture.txt");
        fs::write(&source, b"native engine fixture").unwrap();
        filetime::set_file_mtime(&source, filetime::FileTime::from_unix_time(now() as i64 - 10, 0)).unwrap();
        let c = e.snapshot().unwrap().downloads.remove(0);
        e.decide("downloads", &c.path, &c.id, "bin").unwrap();
        assert!(!source.exists());
        drop(e);
        let mut e = Engine::open(&db, Native).unwrap();
        // An arrival at the original name must never be overwritten by restore.
        fs::write(&source, b"new arrival").unwrap();
        assert!(e.undo().unwrap_err().contains("occupied"));
        assert_eq!(fs::read(&source).unwrap(), b"new arrival");
        fs::remove_file(&source).unwrap();
        e.undo().unwrap();
        assert_eq!(fs::read(&source).unwrap(), b"native engine fixture");
        // Same-name Keep also uses the real Windows no-clobber move.
        let occupied = e.config.keep.as_ref().unwrap().join("native-fixture.txt");
        fs::write(&occupied, b"already kept").unwrap();
        let c = e.snapshot().unwrap().downloads.remove(0);
        let op = e.decide("downloads", &c.path, &c.id, "keep").unwrap();
        assert!(op.destination.unwrap().ends_with("native-fixture (1).txt"));
        e.undo().unwrap();
        assert_eq!(fs::read(occupied).unwrap(), b"already kept");
        assert_eq!(fs::read(source).unwrap(), b"native engine fixture");
    }

    #[cfg(windows)]
    #[test]
    fn windows_restore_collision_preserves_both_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = fs::canonicalize(dir.path()).unwrap();
        let p = root.join("collision-fixture.txt");
        fs::write(&p, b"recycled original").unwrap();
        Native.recycle(&p).unwrap();
        eprintln!("COLLISION_DIAGNOSTIC source={p:?} exists={} entries={:?}", p.exists(), Native.recycled());
        let item = Native.recycled().unwrap().into_iter().find(|x| same_path(&x.original, &p)).unwrap();
        // Exercise the Shell's collision policy directly, past Engine's preflight.
        fs::write(&p, b"concurrent arrival").unwrap();
        Native.restore(&item.id).unwrap();
        assert_eq!(fs::read(&p).unwrap(), b"concurrent arrival");
        let restored = fs::read_dir(&root).unwrap().filter_map(|x| x.ok()).any(|x| {
            x.path() != p && fs::read(x.path()).ok().as_deref() == Some(b"recycled original".as_slice())
        });
        assert!(restored, "Restore must rename the old file rather than overwrite the new one");
    }

}
