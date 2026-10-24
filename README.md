# Feed the Pile

A free, proprietary Windows beta: two piles, one small decision at a time. Working name; availability has not been checked.

- Downloads and Screenshots are folders you choose.
- One card at a time, with local image/PDF previews and an icon fallback.
- **K** Keep → approved folder. **L** Later → end of queue. **B** Bin → Windows Recycle Bin.
- **C** Copy screenshot image, **Space** open, **Z** undo the last decision.
- Munch, the pixel creature, reacts and celebrates actual empty piles.
- No AI, cloud, accounts, telemetry, or automatic deletion.

## Run desktop on Windows

Install the [Tauri Windows prerequisites](https://v2.tauri.app/start/prerequisites/), then:

```powershell
npm ci
npm run tauri dev
```

Start with disposable test folders. Choose three separate regular folders on fixed local drives. Keep never overwrites: collisions receive a numbered filename. Undo is journaled across restarts and blocks when the original path is occupied or the kept file changed. An emptied Recycle Bin cannot be undone.

## Landing

```powershell
cd landing
npm ci
npm run dev
```

The Next.js/Tailwind landing includes a sample-file sorting demo, both tabs, screenshot Copy, mascot reactions, scroll-reveal paragraph, process cards, decision strip, $0 / $0000000 pricing, FAQs and footer. It never touches visitor files. Download buttons disclose that an installer is not published yet.

## Verification and trial

- [Release and build runbook](docs/RELEASE.md)
- [Seven-day personal + friends trial](docs/FRIENDS_TRIAL.md)
- [Verification status](docs/VERIFICATION.md)
- Phase 0 fixture source remains in `spike-core/`; production file engine is in `core/`.
- `.github/workflows/windows-beta.yml` checks and builds an unsigned Windows installer artifact.

The source was prepared in a Linux environment. Windows runtime behavior, screenshots folder location and real friend retention must be tested; they are not fabricated as completed phases. No Reddit posts have been sent.

## Closed source

The linked GitHub repository was public when checked. Do not push this proprietary source until it is private. No open-source license is granted. Third-party dependencies retain their own licenses; review their notices before binary distribution.

Copyright © 2026 Sumit Kumar Sharma. All rights reserved.
