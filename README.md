# Zuna Roast

Roasting software for the **Probat P12 III**, made for Zuna Coffee. It runs as a Mac app.

## Install

1. Open the newest **[Release](https://github.com/zunacoffee/Zuna-roast/releases/latest)**, then download the file that ends in `.dmg`.
2. Open the `.dmg` and drag **Zuna Roast** into **Applications**.
3. The first time you open it, macOS blocks it because it isn't from the App Store. Go to **System Settings → Privacy & Security**, scroll down, and click **Open Anyway**. You only do this once.
4. When macOS asks whether Zuna Roast can find devices on your **local network**, click **Allow**. Without this, it can't reach the roaster.

## Updates

When a new version exists, Zuna Roast asks **when you open it**: *"Update Zuna Roast to version …?"* Click **Update now**. The app restarts on the new version.

- It never updates in the middle of a roast.
- Your roasts, profiles, green coffee, cupping and settings are never touched by an update.
- Every past version stays under **Releases**. To go back, download an older `.dmg` and drag it into Applications again.

You can also check by hand in **Settings → Saving & storage → Check for updates**.

## Where your data lives

Everything is saved as ordinary files in **`Documents/Zuna Roast`**:

```
Zuna Roast/
  Roasts/2026/10 October/R0042_2026-10-12_1445_….json + .csv
  Profiles/        one file per profile
  Green coffee/    inventory.json + inventory.csv
  Cupping/         cupping.json + cupping.csv
  Settings/        settings.json · today-plan.json · roast-in-progress.json
```

- Turn on **iCloud → Desktop & Documents Folders** so the folder is backed up and readable from your phone or another Mac.
- **New Mac?** Install the app and let iCloud bring the folder down, or copy the folder over. Everything is there.
- **Settings → Saving & storage** shows the folder, has **Show in Finder**, and lets you pick a different folder.
- **Export everything** saves one backup file. **Roasts → Import** restores it: roasts, green coffee, cupping and (if you say yes) settings, without making duplicates.

**Moving from the browser version:** in the old version, go to Settings → Saving & storage and press **Export everything**. Then, in the Mac app, press **Roasts → Import** and pick that file.

## How it's put together (for whoever maintains it)

| Path | What it is |
|---|---|
| `app/index.html` | The whole app screen: roast, profiles, roasts, green coffee, cupping, settings. Still opens in a browser for quick previews. |
| `app/native.js` | Mac-app bridge: data folder, roaster WebSocket, keep-awake, Save dialogs, update check. Does nothing in a browser. |
| `src-tauri/` | The Mac app shell ([Tauri 2](https://tauri.app)). `src/files.rs`: safe, atomic file writes inside the data folder. `src/roaster.rs`: the WebSocket link to the P12. |
| `.github/workflows/release.yml` | Builds the `.dmg` on GitHub's Macs and publishes a Release when a `v*` tag is pushed. |
| `.github/workflows/check.yml` | On every change, checks that the screen code parses and the Mac app compiles. |

**To ship a new version:** change `"version"` in `package.json` (for example `1.0.1`), commit, then push a tag with the same number (`git tag v1.0.1 && git push --tags`). GitHub builds it, and every installed app offers the update the next time it opens.

**Update signing:** updates are signed with a private key stored as the repository secret `TAURI_SIGNING_PRIVATE_KEY`, and the matching public key is in `src-tauri/tauri.conf.json`. Keep a backup of the private key. Without it, installed copies can't be updated automatically, and you'd have to install new versions by hand.

**Roast data never goes in this repository.** `.gitignore` keeps roast files and keys out.
