# Historical Phase 0 baseline

This records the original spike before the production sorter was built. Current implementation and checks are in [VERIFICATION.md](VERIFICATION.md). Pending Windows runtime measurements below remain unverified.

# Capability spike report — 2026-10-24

## Current verdict: Windows gate remains open

Prepared: Tauri 2 diagnostic bridge, React/TypeScript harness, Windows Known Folder lookup, Shell image/PDF thumbnail probe, filesystem watcher fixture, no-clobber same-volume move/undo fixture, exact-entry recycle/restore fixture with collision and hash checks.

These are experiments, not verified production features. No v1 sorter or landing has been built.

| Question | Approach | Evidence needed | Status |
|---|---|---|---|
| Fast image thumbnails through Tauri? | Windows Shell thumbnail → PNG → IPC → React image decode/paint | Visual correctness; cold/warm timings, large files, UI responsiveness | Pending Windows run |
| PDF page-one thumbnails? | Shell thumbnail provider with THUMBNAILONLY; explicit failure if absent | PDF installed-provider and provider-missing cases; compare page one | Pending Windows run |
| Recycle Bin and undo? | trash crate 5.2.9, exact newly-added fixture ID, collision test, SHA-256 round trip | Windows local disk, denied/locked file, unavailable bin, external bin emptying | Fixture pending; production safety unproven |
| Keep without overwrite? | Same-volume hard-link create-if-absent then unlink | Fixture collision and restoration; Windows run | Cross-volume deliberately unsupported |
| Folder watching? | notify native watcher receives actual fixture event | Windows bursts, rename, removal, disconnection, rescan recovery | Windows pending |
| Actual screenshot folder? | SHGetKnownFolderPath + read-only existence checks | User takes a real screenshot and confirms location | Unknown until confirmed on user's PC |

## Environment and automated results

Build host: Linux. Initially no Rust toolchain or PowerShell. This host cannot validate Windows Shell, clipboard, Recycle Bin or the user's folders.

Frontend build: PASS — TypeScript strict check and Vite production build.
Core fixture test: PASS on Linux — one test covering collision preservation, Keep/undo byte integrity, and a native watcher event.
Windows-target core type check: PASS (`x86_64-pc-windows-gnu`), including trash 5.2.9 restore API usage. This is not a Windows runtime test.
Full Tauri Windows-target check: blocked at resource generation because this Linux host lacks `x86_64-w64-mingw32-windres`; no full-app compile pass is claimed. Added the required icon resource discovered during this check. Windows runtime: not run here.

## Acceptance criteria for the next run

- Image and PDF first thumbnail appear without blocking interaction. Record a practical target of under 500 ms warm and under 1500 ms cold on the target PC; these are targets, not achieved measurements. Exclude time spent choosing files.
- Missing PDF provider gives a file-icon fallback in v1, never a stuck spinner. Decide whether to ship a bundled PDF renderer only after testing target machines.
- Recycle and exact-entry undo preserve bytes; collisions refuse without overwriting. Test on disposable files only, including failures.
- Notification is a rescan hint, not the source of truth; watcher loss must trigger a bounded rescan in v1.
- Screenshot folder is explicitly confirmed. Existing Known Folder path alone is insufficient.
- No source pushed until remote visibility is private.

## Remaining risks

PowerShell startup/compilation is likely too expensive per card and corporate policy may block it. This spike measures the bridge; production should use native worker extraction. Shell PDF thumbnails depend on installed providers and may be stale. Same-volume hard-link moves have a two-step interruption window; a durable journal is required before user-file operations. This harness deliberately keeps those operations away from real piles.

## References

- https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ishellitemimagefactory-getimage
- https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid
- https://docs.rs/trash/5.2.9/trash/os_limited/index.html
- https://v2.tauri.app/start/prerequisites/
