# Beta release runbook

Version 0.1.0 is source-complete for the requested beta scope, not Windows-runtime verified in this Linux workspace.

## Build desktop

Prerequisites: Windows 10/11 x64, Node 22+, Rust stable MSVC, Visual Studio C++ Build Tools and Windows SDK, Edge WebView2 runtime. Official setup: https://v2.tauri.app/start/prerequisites/

```powershell
npm ci
npm run build
cargo test --locked --manifest-path core/Cargo.toml
cargo check --locked --manifest-path src-tauri/Cargo.toml
npm run tauri dev
# After fixture acceptance:
npm run tauri build
```

Installer output: `src-tauri/target/release/bundle/nsis/`. The included GitHub workflow does the same checks and uploads an unsigned installer artifact; the baseline workflow has passed and produced an installer. PR #2 adds installation and runtime acceptance; require those checks to pass on the revision you download. Signing and automatic updates are not configured. No public installer URL is invented.

## Build landing

```powershell
cd landing
npm ci
npm run dev
npm run build
```

Next.js + Tailwind, statically exported to `landing/out`. Before hosting elsewhere, set `metadataBase` in `app/layout.tsx` to the actual deployment origin. Current metadata points to the private review Site. The sample-file hero never reads visitor files. Download buttons show an honest pending-installer dialog. Replace that handler with an exact verified HTTPS installer URL only when the artifact is ready.

## Storage and behavior

Desktop folder config, queue ranks and write-ahead undo records live in `pile.sqlite3` under Tauri's per-user local app-data directory. SQLite uses WAL + FULL synchronous durability. Retain the database if you want persistent undo. The app never clears the system Recycle Bin. Keep uses Windows MoveFileEx without REPLACE_EXISTING, with COPY_ALLOWED + WRITE_THROUGH for cross-volume local disk moves. Ambiguous copies block further mutations and retain both sides for review. Bin requests explicit RECYCLEONDELETE, records the exact newly created Recycle Bin identity and rejects non-recycle pre-delete callbacks. Restore uses a guarded Shell move with RENAMEONCOLLISION, so a concurrent arrival is preserved; an unexpected restore path requires review.

Image decoding is bounded to 40 MB input / 128 MB decoder allocation, with 720×520 previews. PDFs are capped at 30 MB, page one is rendered by a bundled PDF.js worker. Over-limit, corrupt, unsupported and password-protected files fall back to an icon with an explanation; Space opens the system viewer. No remote PDF renderer/CDN is used. Preview cache holds at most 16 thumbnails in memory.

Known limits: normal fixed local drives only; no folders as cards, recursion, linked/reparse items, cloud placeholders, hidden/system files, partial downloads, or non-Unicode paths. Newly modified files settle for three seconds. Windows ACLs, concurrent external changes and OS Recycle Bin behavior still require the real-device acceptance matrix. Undo is not a backup and cannot recover a purged Recycle Bin entry. Clipboard sync, if separately enabled in Windows by the user, is outside the app's control.
