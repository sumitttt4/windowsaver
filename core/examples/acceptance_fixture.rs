//! CI-only fixture setup. Never invoked by the desktop application.
use pile_core::{Engine, Native};
use std::{fs, path::PathBuf};
fn main() {
    let root = PathBuf::from(std::env::args_os().nth(1).expect("fixture root"));
    let db = PathBuf::from(std::env::args_os().nth(2).expect("isolated journal path"));
    assert!(!db.exists(), "Refuse to replace an existing app journal");
    fs::create_dir_all(db.parent().unwrap()).unwrap();
    let mut engine = Engine::open(&db, Native).unwrap();
    for name in ["downloads", "screenshots", "keep"] {
        let folder = root.join(name);
        fs::create_dir_all(&folder).unwrap();
        engine.configure(name, &folder).unwrap();
    }
    println!("{}", serde_json::to_string(&engine.config).unwrap());
}
