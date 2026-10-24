# Build Feed the Pile desktop beta, landing demo and trial kit

An empty repository becomes a local Windows sorter for Downloads and Screenshots. Users approve three separate folders, then Keep, Later or Bin one file at a time. Keep uses numbered collision destinations; Bin uses explicit Windows recycle mode and a non-recycle callback guard. SQLite records operations before filesystem changes so undo and recovery persist across restarts. Ambiguous operations retain both copies and require review.

The white Next.js landing includes a clearly labeled sample-file demo, screenshot Copy, creature reactions, scroll reveal, process cards, control strip, free/joke pricing and FAQ. Download buttons explain that a public installer is pending. A one-week personal/friends test kit and unsigned Windows installer workflow are included.

Validation: desktop and landing production builds, nine Linux core tests, full Windows-target Tauri type check, Windows-target core test compilation, and browser checks of the landing sorting/undo/download flows.

Limits: actual Windows runtime checks and the seven-day trial remain pending. Cross-host executable build failed in a compiler dependency archive, so no EXE/installer is represented as working. Read `docs/VERIFICATION.md` and start with disposable folders. Repository must be private before pushing this proprietary source.

Landing design correction: Manrope for headings, DM Sans for body/buttons, pale-blue controls, one consistent SVG icon set, and a real locally bundled Moraine Lake photo. Desktop visual check confirms fonts, photo, pure-white canvas and no horizontal overflow. No GitHub push or deployment was performed for this correction.
