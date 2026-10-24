# Verification — 2026-10-24

## Implemented

- Tauri 2 + React + TypeScript + Tailwind desktop source, with approved folder setup and persistence.
- One-card queues, K/L/B/C/Space/Z, image/PDF previews with icon/error fallbacks, native watching with bounded rescan fallback.
- No-clobber Keep, Windows explicit recycle mode and non-recycle callback guard, exact Recycle Bin identities, SQLite durable journal, crash recovery, persistent undo and ambiguous-operation review.
- Single instance, native screenshot image clipboard copying, executable/script open routed to Explorer.
- Shared pixel creature and reactions, actual-empty-pile celebration, reduced-motion support.
- All seven requested landing sections, working sample-file tabs and decisions, sample image Copy, undo, fake-file disclosure and honest pending-download dialog.
- Personal/friends one-week trial kit, observation tables, Windows acceptance matrix and unsent maker-post draft.

## Passed here

- Strict TypeScript + Vite desktop production build, including a locally bundled PDF worker.
- Next.js landing production build + static export.
- Nine Rust core tests on Linux: collision-safe Keep + undo, undo collision + external edits, Later persistence, exact Bin identity + undo with a fixture backend, missing Bin entry, stale/unapproved paths, recovery after restart, interrupted-copy preservation, symlink exclusion.
- Full Tauri Windows GNU target `cargo check`, including native COM recycle guard, image clipboard, watcher and SQLite code.
- Windows-target core test compilation (`cargo check --tests`). This does not execute Windows tests.
- Browser checks on the rendered desktop-width landing: Later rotates without reducing count; Keep reduces count and changes creature; Undo restores count and card; screenshot tab exposes Copy; Bin updates that sample pile; Escape closes the pending-download dialog; keyboard decisions reach a zero-count celebration. A card stacking-layer defect found during visual QA was fixed.
- Private landing deployment completed. Review URL: https://feed-the-pile-sumit.agilemars259.chatgpt.site

## Not passed / not claimed

- Windows runtime checks have not run. This Linux host cannot confirm the actual screenshot location, clipboard paste, Recycle Bin safety behavior, startup compatibility or thumbnail latency on the user's PC.
- A Windows executable release build was attempted. It failed while archiving a Windows-target `tauri-utils` object with `memory map must have a non-zero length`. A targeted clean and single-job rebuild reproduced it. No working EXE or installer is claimed or attached.
- The original Windows GitHub workflow passed on main (`6c9069c`), including native compilation and an unsigned NSIS installer. Follow-up installed-release acceptance and stronger native restore tests are being run in PR #2. Use the exact tested revision, not an older installer.
- Mobile CSS and reduced-motion paths are implemented, but mobile-viewport and assistive-technology browser validation are not claimed.
- The one-week personal trial, 3–5 friends' unprompted returns, public installer publication and Reddit posts have not happened. They require real usage and evidence.
- The owner explicitly authorized public publication. PR #1 was merged; follow-up safety and Windows acceptance changes are in PR #2. Public source is not an open-source license.

## Run next

Follow `RELEASE.md`, then the generated-file acceptance checklist in `FRIENDS_TRIAL.md`. Start with disposable folders and use the actual Windows installer only once it builds. Stop the trial on any destructive bug.

## Landing design correction

White canvas; Manrope headings; DM Sans body/navigation/buttons; light-blue controls; a local SVG icon component; real Moraine Lake photo with source in `landing/ASSETS.md`. Production build passes. Browser verified both font faces load, image loads, white background and pale-blue button values, desktop no horizontal overflow, and Later/Undo behavior. Phone/tablet screenshots were not verified. This revision is in the repository; the earlier hosted preview has not been updated.
