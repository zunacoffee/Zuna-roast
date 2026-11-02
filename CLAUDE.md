# Zuna Roast — notes for whoever works on this next

This repository is **the** current version of Zuna Roast (Probat P12 III roasting software for Ty / Zuna Coffee).
Old prototype HTML files are in `archive/` as history only — do not build from them. The touch version was discarded on purpose.
Release history with technical notes: `CHANGELOG.md`. Open questions and untested items: claude.ai project doc `claude/OPEN-ITEMS.md`.

## What lives where
- `app/index.html` — the whole app screen (single file). Started from Ty's build 33.
- `app/native.js` — Mac-app bridge: data folder, roaster WebSocket, keep-awake, Save dialogs, updates. Inert in a browser.
- `src-tauri/` — Tauri 2 Mac shell. `files.rs` = atomic writes inside the data folder; `roaster.rs` = native WebSocket (tries browser-style → lowercase path → plain handshakes; the P12 returned 400 to the plain one).
- Data is NOT here: it lives in `Desktop/ZUNA COFFEE/Zuna Roast` (iCloud Desktop) — Roasts/, Profiles/, Green coffee/, Cupping/, Settings/.
- Several devices share that folder (iMac at the roaster, MacBook, 13" iPad, iPhone). Since 1.0.22 the app watches it (`fs_stamp` fingerprint every 8 s, on focus; never mid-roast or while typing),
  merges inventory/cupping/planned profiles/today's plan/settings item-by-item (3-way, `BASE` = last seen), re-checks the next roast number against the folder at save,
  and keeps one device with Control via `Settings/control.json` (heartbeat 15 s; a newer claim wins and the other device drops to watching). Test: two app copies on one folder.

## Branches
- `main` = the Mac app only. Never put iPad/iPhone code (`IPAD`, `PH`, the `zuna-ios` plugin) on `main`.
- `ipad` = the iPad and iPhone version (same `app/index.html` plus the iOS shell). Its copy of this file adds the iPad/iPhone notes.
- `preview/...` = work waiting for Ty's "go." A change that touches the iPad/iPhone layouts is built on a preview branch made from `ipad`; `main` gets only the Mac parts.
- Only one chat releases at a time.

## Shipping a change
0. Describe the plan to Ty and show a mockup or preview with demo data. Build on a `preview/...` branch. Release only when Ty says "go."
1. Edit, then `npm run check` (JS syntax). Test in a browser with the built-in Simulator.
2. Bump the version in **three** places: `package.json`, `src-tauri/Cargo.toml`, and the `zuna-roast` entry in `src-tauri/Cargo.lock`.
3. Commit, push to `main`, then run the **Release Mac app** workflow (Actions tab or `gh workflow run release.yml --ref main`).
   Tag pushes are blocked from Claude sessions; the workflow creates the `vX.Y.Z` tag itself.
4. Installed apps offer the update when they open (never mid-roast). Check `releases/latest/download/latest.json`.
5. Merge `main` into `ipad` (keep both sides where `ipad` added phone/iPad hooks, e.g. `onPush` → `phPush`, `ipCard` in renderRoasts/renderGreen), compare screenshots, push, run **Release iPad app**.
6. Add the release to `CHANGELOG.md`, update the "Latest release" line in the project doc `claude/CURRENT-VERSION.md`, and update `claude/OPEN-ITEMS.md`.

## Screenshots for review
Headless Chrome (Playwright) on `app/index.html` at iMac 2240×1260, 13" iPad 1366×1024 (`?ipad=1`), iPhone 393×852 (`?iphone=1`). Load demo data through History › Import (a full backup JSON) and run the built-in simulator at 10× for a live-roast shot. Compare before and after for every device the shared file affects. Don't check the 10.5"/11" compact iPad (`ipc`) — Ty stopped using it.

## Rules Ty cares about
- Never change the data folder layout or file names; new roast fields must be optional so old roasts still open.
- Pressure is stored in Pa; the Pa/mbar setting only changes display.
- Professional, clean, uncluttered UI. All colors stay changeable in Settings. Zuna logo is the app icon (`assets/zuna-icon-reverse.svg`, from LOGO FINALS2 → ICON).
- Update signing key = repo secret `TAURI_SIGNING_PRIVATE_KEY` (Ty holds the backup). The app is ad-hoc signed (`signingIdentity: "-"`).
- Development is a time after first crack — never DTR / development %. Targets follow the Brewed Behavior manual; Ty's P12 first cracks at 396°F.
- Learning (Analyze, burner suggestions) is per coffee AND batch size (13 lb / 25 lb). Never across coffees.
- Things Ty decided against: market-price comparison in Pricing, a tabbed Profile detail, the touch version.

## Design rules (since 1.0.31 — keep all new UI on them)
- CSS tokens in `:root`. Text `--fs-cap 12 / --fs-sm 13 / --fs-body 14 / --fs-title 18 / --fs-page 22` (all × `--ts`); spacing `--sp-1..6` = 4/8/12/16/24/32; corners `--r-sm 6 / --r-md 8 / --r-lg 12 / --r-pill`; control heights `--h-btn 36 / --h-btn-sm 28` (iPad 44/40). No new `calc(Npx*var(--ts,1))` sizes.
- Buttons: Primary (accent fill, one per panel), Secondary (outline), Quiet (accent text), Destructive (red text; red fill only to confirm, `ask(..., danger=true)`).
- Selected: "turned on" = accent fill; view switch = raised white segment; open row = accentSoft + accent edge.
- Numbers: roasted weight 2 decimals, Agtron 1, temps 1, times m:ss, missing = grey dash (`DASH`), tabular digits. Helpers `fRoasted`, `fGreen`, `fAg`, `fColors`.
- Color: accent = actions/selection; `--good/--warn/--danger` = status; chart colors only on charts.
