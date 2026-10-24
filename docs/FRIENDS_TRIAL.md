# Phase 3 — the one-week return test

This is the test kit, not a completed study. Do not claim retention or Windows safety validation until the observations exist.

## Before anyone uses real folders

1. Make the GitHub repository private before pushing proprietary source.
2. Run the Windows workflow and download its unsigned x64 NSIS installer artifact. The workflow produces an artifact, not a public release. Code signing is not configured; do not tell people to disable Windows security.
3. Complete the generated-fixture checks below on Windows 10/11. Start with three disposable local folders, not your actual Downloads.
4. Take a screenshot using your normal shortcut and confirm its actual folder. Choose it explicitly. Clipboard-only captures do not create a pile item.
5. Use a backup before the first real-folder trial. Keep network, removable, linked and cloud-placeholder folders outside this beta's scope.

## Windows acceptance checklist — pending actual execution

- [ ] Installer installs and launches as a normal user; second launch focuses the existing window.
- [ ] Three folder pickers persist correctly across restarts; overlapping folders are refused.
- [ ] Image preview is correct; PDF shows page one without an installed PDF thumbnail provider. Test corrupt, password-protected and over-limit files: icon fallback must remain usable.
- [ ] K moves to approved Keep; a duplicate name becomes `(1)`, never replaces the original. Test fixed local disks on the same and different volumes.
- [ ] B sends a disposable file to the actual Recycle Bin. Test bin disabled/full, denied permission, locked file and an oversized fixture: app must refuse a permanent-delete path.
- [ ] Z restores the exact binned entry; a newly occupied original name blocks undo.
- [ ] Z restores Keep after app restart; editing the kept file causes undo to refuse.
- [ ] L rotates without changing disk contents; one remaining file stays pending after repeated L. Z reverses Later.
- [ ] C copies a screenshot as an image; paste into Paint to inspect. Clipboard copying does not remove or sort the file.
- [ ] Space opens a document; executable/script types are shown in Explorer rather than launched.
- [ ] External create, rename and remove update the card. Check bulk events, missed notifications and a temporarily unavailable folder.
- [ ] Interrupt Keep and Bin during fixture operations. Reopen: journal recovers uniquely identifiable operations or blocks further moves with an explicit review state. It must never remove either side of an ambiguous copy.
- [ ] No requests leave the installed app during normal use. The web landing is a separate website.
- [ ] No horizontal overflow at the app minimum size; keyboard focus and reduced motion work.

Optional fixture-only Windows native Recycle Bin round trip:

```powershell
cargo test --manifest-path core/Cargo.toml windows_recycle_round_trip -- --ignored --nocapture
```

## Your week

Use it on weekdays when a file pile is actually annoying. Do not manufacture files to make a usage chart look good. Note what happened, including days you did not open it.

| Day | Opened voluntarily? | Pile used | Useful decision or frustration | Error/data-loss incident? |
|---|---|---|---|---|
| 1 | | | | |
| 2 | | | | |
| 3 | | | | |
| 4 | | | | |
| 5 | | | | |
| 6 | | | | |
| 7 | | | | |

## Give it to 3–5 friends

Recruit people who already have messy folders. Give each a short setup walkthrough, then stop reminding them to use it. Ask permission before observing their files; collect no filenames, screenshots or paths by default. No telemetry is included in the app.

At the end of the week, ask:

1. Did you open it a second time before anyone reminded you?
2. What happened just before you opened it?
3. What did you expect Keep, Later, Bin and Undo to do? What surprised you?
4. Did you hesitate because you did not trust it with files?
5. If this disappeared tomorrow, would anything become harder?

| Tester alias | First use | Unprompted second use? | Main friction | Would use again? |
|---|---|---|---|---|
| A | | | | |
| B | | | | |
| C | | | | |
| D (optional) | | | | |
| E (optional) | | | | |

Success is an unprompted second visit. With 3–5 people this is qualitative evidence, not a statistically stable retention rate. Any destructive bug stops the trial immediately. Do not add pricing because people compliment the mascot.

## Review after the week

Write down observed returns, failures, trust concerns and one next improvement. If nobody comes back, investigate the sorting trigger and value before polishing the landing further.

## Maker-post draft — send only after the trial

Title: I made a free Windows app that turns Downloads and Screenshots into a small sorting game

I'm the maker of Feed the Pile, a free, closed-source Windows app. You choose your Downloads and Screenshots folders, then decide Keep, Later or Recycle Bin one file at a time. Screenshots also have Copy, and a tiny pixel creature reacts as you sort.

Everything in the desktop app stays local. No AI, accounts, cloud uploads or automatic deletion. Undo keeps a local journal and refuses to overwrite; files changed elsewhere or emptied from the Recycle Bin may not be recoverable.

[Replace this paragraph with actual Windows versions tested, observed trial results and known limitations. Add the real installer link only after publishing an artifact.]

I'd appreciate feedback from people with messy folders, especially around trust, the first session, and whether this is useful enough to return to.

Check the current subreddit rules and self-promotion policy before posting separately to r/windowsapps and r/software. No posts have been sent.
