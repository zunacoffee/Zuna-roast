# Zuna Roast — notes for whoever works on this next

This repository is **the** current version of Zuna Roast (Probat P12 III roasting software for Ty / Zuna Coffee).
Older HTML files in the claude.ai project (zuna_roast.html, _p12, _focus, _console, _33, _touch) are history only — do not build from them. The touch version was discarded on purpose.

## What lives where
- `app/index.html` — the whole app screen (single file). Started from Ty's build 33.
- `app/native.js` — Mac-app bridge: data folder, roaster WebSocket, keep-awake, Save dialogs, updates. Inert in a browser.
- `src-tauri/` — Tauri 2 Mac shell. `files.rs` = atomic writes inside the data folder; `roaster.rs` = native WebSocket (tries browser-style → lowercase path → plain handshakes; the P12 returned 400 to the plain one).
- Data is NOT here: it lives in `Desktop/ZUNA COFFEE/Zuna Roast` (iCloud Desktop) — Roasts/, Profiles/, Green coffee/, Cupping/, Settings/.
- Several devices share that folder (iMac at the roaster, MacBook, iPad). Since 1.0.22 the app watches it (`fs_stamp` fingerprint every 8 s, on focus; never mid-roast or while typing),
  merges inventory/cupping/planned profiles/today's plan/settings item-by-item (3-way, `BASE` = last seen), re-checks the next roast number against the folder at save,
  and keeps one device with Control via `Settings/control.json` (heartbeat 15 s; a newer claim wins and the other device drops to watching). Test: two app copies on one folder.

## Shipping a change
1. Edit, then `npm run check` (JS syntax). Test in a browser with the built-in Simulator.
2. Bump the version in **three** places: `package.json`, `src-tauri/Cargo.toml`, and the `zuna-roast` entry in `src-tauri/Cargo.lock`.
3. Commit, push to `main`, then run the **Release Mac app** workflow (Actions tab or `gh workflow run release.yml --ref main`).
   Tag pushes are blocked from Claude sessions; the workflow creates the `vX.Y.Z` tag itself.
4. Installed apps offer the update when they open (never mid-roast). Check `releases/latest/download/latest.json`.

## Rules Ty cares about
- Never change the data folder layout or file names; new roast fields must be optional so old roasts still open.
- Pressure is stored in Pa; the Pa/mbar setting only changes display.
- Professional, uncluttered UI; Zuna logo is the app icon (`assets/zuna-icon-reverse.svg`, from LOGO FINALS2 → ICON).
- Update signing key = repo secret `TAURI_SIGNING_PRIVATE_KEY` (Ty holds the backup). The app is ad-hoc signed (`signingIdentity: "-"`).
