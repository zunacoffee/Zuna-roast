# Zuna Roast — changelog

Newest first. One section per release (Mac version number) or iPad/iPhone-only TestFlight build. Technical notes are for whoever works on the code next; open questions and untested items live in the claude.ai project doc `claude/OPEN-ITEMS.md`.

## iPhone build after 1.0.38 — Full Reports on the phone (Nov 3, 2026; iPhone only, `ipad` branch, TestFlight, no Mac release)
- Ty: "I would like to be able to see more of the reporting on my iPhone." More › Reports (`PHV.rprod`) now has the same Production and Inventory reports as the Mac/iPad, read-only, phone layout.
- Phone state `PH.rp/rv/rby/rfrom/rto` saved in `zr2.phReports`; `phRpWith(f)` runs the shared helpers (`rpRange`, `rpProd`, `rpBuckets`, `rpProdDoc`) with the phone's range/grouping in `RP` and restores it, so the Mac/iPad Reports settings on the device aren't touched.
- Range chips (all `RP_PRE`, scrolls sideways) + Custom date inputs. Production: `rpTiles` (2 per row, previous-period line), `rpBarsSvg` chart, By coffee / Coffee + batch size, coffee rows → `PHV.rcof` (totals + roasts with weights, loss, colors, spread, 2·4·6·8 min bean temps) → the phone roast view; bean temp 2/4/6/8 min grid (`.p-marks`). Inventory: `rpInvTiles`, lots (`rpInv`) with Low/New, bar, `rpPace` → `PHV.rlot` (dated entries + running balance; roast entries open the roast).
- Share (`phRpShare`): PDF via `rpPrint(rpProdDoc|rpInvDoc)` or CSV via `download(rpProdCSV|rpInvCSV)` → iOS share sheet.
- MacBook 1728, iMac 2240, 13" iPad and the other iPhone screens unchanged (27 screens compared); only the More › Reports subtitle text changed.

## 1.0.38 — Blend pricing (Nov 3, 2026; Mac; iPad/iPhone get the Mac Pricing screen code but their phone Pricing view does not list blends yet)
- **Pricing › + New blend**: a name and a recipe of green coffees by share (%). Total ≠ 100 % shows red. Blends get their own "Blends" group on the Price list (with "Single origins" above when both exist), recipe line under the name, same Wholesale/Retail rows and typed prices.
- Data: `cfg.pricing.blends = [{id, name, parts:[{key, name, pct}]}]` (settings only — roast files and the data folder unchanged). `key` = `normName(coffee name)`, so a blend follows a coffee to its next lot like typed prices. Blend prices in `cfg.pricing.actual['blend:'+id]`. Selection id `'b:'+id`.
- Cost: blended after roasting — coffee per lb = Σ share × that coffee's own `prCalc().coffee` (its green + freight ÷ own avg weight loss, incl. waste) ÷ total share. `prLotFor(key)` = newest in-stock lot with a price, else newest priced lot. Overhead/labor/bags/margins via the new shared `prBags(coffee,key,o,ctx)` (split out of `prCalc`; `prCalc` returns the same fields as before).
- A part with no green price → the blend shows which coffee is missing instead of a cost. Detail card: recipe editor (`prRecipeHtml`/`prRecipeWire`), cost fold lists each part's share of the cost, "Delete blend" (red confirm; removes its prices too).
- Overview "Do your prices work?" includes blend prices. Wholesale price sheet lists blends as "Blend · Brazil, Colombia" — no shares (recipe stays private).
- Checked with demo data at MacBook 1728×1050 and iMac 2240×1260. With no blends saved, the Price list looks as before (only the new header button).

## 1.0.37 — MacBook fit (Nov 3, 2026; MacBook only)
- Ty has no iMac for now: the MacBook (16" Pro, maybe a 15" Air later) is the roasting Mac. Checked every screen with demo data at 1728×1050 and 1710×1045 (the maximized window); Roast, Today, Profiles, Green coffee, Pricing, Analyze and Settings already fit.
- **History** (`@media (max-width:1900px)`, `html:not(.ipad):not(.iphone)`): detail column `minmax(460px,34%)` (was 38 %), `#rTable` cells 7 px side padding — every column fits (Drop, Dev, $/lb were cut off). Detail buttons wrap on purpose into two rows (`.sp` → full-width break, `#rSheet` margin-left:auto): Roast this again · Open · Make profile, then Roast sheet · CSV · Delete (Delete was hidden).
- **Reports** (`max-width:1800px`): `#rpHead` gap 8, range buttons 8 px side padding — the bar stays on one line (Print / PDF had wrapped alone).
- iMac 2240×1260: all ten screens compared before/after, identical except the clock. iPad/iPhone excluded by the `.ipad/.iphone` scope.

## 1.0.36 — Roast sheet on one page (Nov 3, 2026; iMac, 13" iPad, iPhone)
- Ty printed a roast sheet from the iPad and it ran to two pages. Cause: `.ipad td{padding:14px 12px}` (touch-size rows, same specificity as `.paperDoc td`, later in the file) doubled every paper table row; the 2/4/6/8-minute rows from 1.0.35 made the sheet longer on the Mac too.
- `.paperDoc th/td`: padding 3px 6px, font size and `height:auto` with `!important`, so no screen rule can stretch a printed page (also the on-screen sheet preview and the printed reports).
- `.pdSheet` (roast sheet only): gap 9, tighter header/kv/notes, chart 300 → 270 px tall.
- `rsFit(el)` in `rpPrint`: measures the sheet in an off-screen 730 px box; if taller than ~930 px (one Letter page inside the margins, Mac `@page .45in` and the iPad/iPhone PDF alike) sets CSS `zoom` to fit (min 0.7). Reports (production/inventory) still flow onto more pages.
- Also on `main` now: `:root{color-scheme:light!important}` in `@media print` (white PDF margins with the Dark theme), which was on `ipad` since build 47.
- Checked as PDFs in Chromium (Mac and `?ipad=1`), demo roast and a long one (extra notes, two cupping cards): all one page. Not yet tried in the Mac print dialog or iPad share sheet.

## 1.0.35 — Bean temp at 2, 4, 6 and 8 minutes (Nov 2, 2026; iMac, 13" iPad, iPhone)
- Ty wanted the **exact** bean temp at 2:00, 4:00, 6:00 and 8:00 for every roast — no averages. `btAt(r,sec)`: the recorded reading nearest CHARGE+sec (within 1.5 s; `sampleAt`), none past Drop or with no Charge. `RP_MARKS=[120,240,360,480]`.
- **Reports › Production**: new card "Bean temp at 2, 4, 6 and 8 minutes" listing every roast in the range (newest first: #, date, coffee, profile, batch, 2:00–8:00), click → roast sheet (`rpMarksTable`). Coffee drill-down roast list got the four columns; its on-screen Cost column was dropped to fit (still in print/CSV) and the "Roast sheet" button column removed (row click opens it). Printed production report includes the table; CSV per-roast rows get `bt_2min_F`…`bt_8min_F` (unit follows the setting).
- **Events timeline** (`evTimeline(r,bg)`, `evDiff`): events + minute marks in time order. Roast sheet Events table shows them (marks in italics; "vs background" = time diff for events, ±° for marks). History detail has a new **Events** line (marks grey).
- iPad: same through the merge. iPhone (ipad branch only): roast view boxes under the chart = events + marks in time order, 4 per row (`phEvents(r,null,true)`, `.p-ev4`, marks `.mk`); Live view unchanged.
- Layout: `.rpCol>.rpTable:first-child{flex:none}`, `.rpMarksCard` fills the rest of the column and scrolls.

## 1.0.34 — Switch lights follow the P12 touchscreen (Nov 2, 2026; iMac, 13" iPad, iPhone)
- Mac 1.0.34 from `main` (via `preview/switch-state-follow`). `main` merged into `ipad` the same day: `onPush` handles switch pushes before `phPush`; the `IPHONE` guards on `nudge`/`sendSwitch`/`sendEventToRoaster` kept.
- Found by listening to the real P12 (read-only: `getCurrentRoastingStep` + `getControlParams`) while Ty switched the burner on the touchscreen. The P12 reports switches as **numbers**, not true/false: `burnerState` −1 off (also before drives are on), 0 in between, 2 burning, 1 seen for ~1 s while shutting off; `drivesState`/`coolingFanState`/`stirrerState`/`standby` 0/1. The burner can only be switched on with the drives on.
- `swState(v)`: boolean as is, number → `v>0`. Used by `pollSwitches` (every 3 s) and the standby lookup in `sendSwitch`. Before, the `typeof … === 'boolean'` check skipped every value, so no switch light ever followed the machine.
- `onSwitchPush`: the P12 pushes `{"data":{"burnerState":2},"pushMessage":"burnerState"}` (same for each switch) the moment it changes, including from its own touchscreen; `onPush` now updates the light from it (ignored for 2.5 s after the app's own switch, like the poll). Also seen: `afterBurnerState`, `setBurnerCapacity` (`burnercapacity`), `addAlert`/`setAlert`, `exhaustFan` pushes — not used.
- `Link.onMessage`: the P12 sometimes puts several JSON messages in one frame, one per line (12 times in ~3.5 min, including the burner on/off pushes). These failed as "Non-JSON message" and were lost; now split and handled one by one.
- Simulator sends numbers and switch pushes like the real P12.
- Tested on the P12 (Control off): Drives and Burner lights changed within ~1 s of the touchscreen. Burner OFF can show ON for ~1 s (state 1). Untested: whether the P12 accepts true/false in `setControlParams` for switches (the app still sends booleans).
## iPhone build after 1.0.33 — Report PDFs no longer blank (Nov 2, 2026; iPhone, `ipad` branch, TestFlight build 47, no Mac release)
- Ty: "the reports are blank when pdf is selected on the iPhone." The phone rule `html.iphone body>*:not(#phApp)…{display:none!important}` (specificity 3 ids) beat the print rule `#printArea{display:block!important}`, so More › Reports › PDF and History › Share roast sheet (PDF) printed an empty page. Added `:not(#printArea)` to the phone rule.
- Print styles: `:root{color-scheme:light!important}` so the paper margins stay white with the Dark theme (`applyTheme` sets `colorScheme` dark inline; the @page margins took the dark canvas). Print only — no screen changes (iMac 2240, 13" iPad, iPhone compared). **This line still needs to go to `main` with the next Mac release** (the Mac's Save as PDF has the same dark-margin issue with the Dark theme).

## iPad build after 1.0.33 — Full swipe between tabs (Nov 1, 2026; 13" iPad only, `ipad` branch, TestFlight, no Mac release)
- Ty wanted "full swipe functionality". `ipadInit` swipe handler: one-finger quick sideways swipe (≥90 px, <700 ms, mostly horizontal) moves through **every** `#nav` screen in order, including the ones under More (Roast → Today → Profiles → History → Green → Pricing → Cupping → Analyze → Reports → Settings); stops at the ends (no wrap).
- Works almost anywhere, **including during a roast**. Not when the finger starts in input/textarea/select, the profile builder chart `#bCv`, a modal, or when Roast details is open; a table that can still scroll sideways in that direction scrolls instead (`hScrolls`). On other charts (`cvMain` etc.) only a quick long flick (<400 ms, ≥140 px) switches; a slow drag keeps the readout.
- `swHint` = centered pill with the screen name + dots for ~1 s. `renderRecBar` (called from `tick`) = blue "● Roasting · m:ss · Bean temp … — Back to roast ›" bar at the bottom on any other screen while recording; tap → Roast. iPad only (not iPhone, never iMac).

## 1.0.33 — Reports (Nov 1, 2026; iMac, 13" iPad, iPhone)
- Mac 1.0.33 from `main` (Mac parts only; `IPAD&&!IPC` replaced by `false`, no iOS plugin changes). `main` merged into `ipad` (iPad/iPhone parts kept from `preview/reports`) and uploaded to TestFlight the same day.
- New **Reports** tab (iMac/MacBook nav between Analyze and Settings; 13" iPad under More; `scr-reports`, `renderReports`, state `RP` + `zr2.reports`). Read-only — never writes to the data folder.
- Range bar: This week · Last week · This month · Last month · This quarter · Last quarter · This year · Last year · Custom (date inputs). `rpRange()` → [from,to) local days, stops at today ("… to date"), previous period = same span of the period before.
- **Production** (`rpProd`): per coffee (`meta.coffee||meta.name`, same as Yearly use) or coffee + batch size (`rpBatches`, 12 % rule). Roasts, green in, roasted out, avg loss (weighted), avg whole/ground/spread, green cost (`priceOf` current lot price, as Yearly use), per roasted lb; tiles vs previous period; green-by-day/week/month SVG bars; click a coffee → its roasts with Roast sheet links. Same roast filter as Yearly use (has samples), so totals match.
- **Inventory** (`rpInv`, `rpLotEvents`): per lot on hand at start, came in (starting stock in range + positive adj, with cost), roasted, adjusted, on hand at end (matches `invLeft`), dated ledger; Low = `LOW` 30 lb.
- **Roast sheet** (`roastSheetHtml`, `rsChart` SVG): logo/name from Settings › Business, chart with background + burner/air strip, events vs background, burner/air changes, weights, colors + spread band, cost (incl. green per 12 oz), cupping card notes (`rsCupping`), comments + timed notes. History detail has a **Roast sheet** button (`openSheet(id,back)`).
- **Print / PDF**: report HTML goes into `#printArea` (shown only in `@media print`, `@page letter .45in`). Mac: `ZR_NATIVE.printPage` → Rust `print_page` → `WebviewWindow::print()`. iPad/iPhone: zuna-ios `share_pdf` (`UIPrintPageRenderer` + `viewPrintFormatter` → PDF → share sheet). Browser: `window.print()`.
- **CSV**: `rpProdCSV` (summary + per-roast rows), `rpInvCSV` (summary + ledger).
- iPhone: More › Reports (`PHV.rprod`, read-only production summary + PDF), roast view **Share roast sheet (PDF)** (`#phRsPdf` — not `#phSheet`, that id is the phone's bottom sheet).
- Fix: the cupping card's phone segment style (1.0.26) had leaked onto every `.p-seg` (dark text on dark in History filter, Green, Pricing, cupping Body/Acidity). Scoped to `.p-seg.cs`.
- MacBook ≤1800 px: report grid stacks; nav button padding 10 px under 1600 px so the brand isn't cut off with 10 tabs.

## 1.0.32 — Every screen on the design rules (Oct 31, 2026; iMac + 13" iPad)
- Today: status as text (✓ Roasted #, ▶ Loaded, Planned), accent edge + no Load on the loaded batch, quiet ↑↓ and red ✕, "Green lb per batch", need list "25.0 · 205.0 lb ✓" / "Short 18.0 lb".
- Number format helpers (display only): `fRoasted` (2 dp), `fGreen` (as typed), `fAg` / `fColors` (1 dp), `DASH` grey dash (`.miss`). Used in History, Profiles (list, setup, roasted-with) and the History detail. `.pstar` "★ Profile"; `.dsub` batch text by the profile name.
- History detail: "No comments." only when there are no notes; note times grey; adjustment values in text color.
- Global: all `font-size:calc(N px*ts)` ≤14.5 replaced by `--fs-cap/sm/body` (181 places, iMac and iPad CSS + inline templates); ≥15 px left alone (roast keys, iPad pads). `.card/.modal` radius 12, `.page/.split/.detail/.scont` spacing tokens, header/detail/dialog buttons `min-height:var(--h-btn)`.
- Cupping marks `.mt.on`, `.cseg .on`, iPad `.pbig .on` = accent. `ask(..., danger=true)` uses `.dangerFill` (red). Analyze change labels lane `H-B+30`. Pricing price columns 16% each.
- 13" iPad fits after the size change: Analyze cell padding 8 and detail 600 px; cupping detail gap 12, chips 44; Green coffee detail 440 px.

## 1.0.31 — Design rules + Roast screen on them (Oct 31, 2026; iMac + iPad)
- Step 1 = design rules (published as the "Zuna Roast Design Rules" artifact; the rules themselves are in `CLAUDE.md`); step 2 = move one screen at a time onto them. Done in 1.0.31–1.0.32.
- Ty confirmed all colors must stay changeable in Settings; **Good / Warning / Danger** were added to Appearance › General (`good`/`warn`/`danger` → `--good/--warn/--danger`, same defaults).
- Roast screen: iMac readouts `clamp(44px,3.4vw,76px)` (was 56 max), setpoints 48 (was 40); 13" iPad bean temp 74, time/RoR 50, setpoints 52 (`.ipad:not(.ipc)`); countdown "Background next **51** · in **0:18**" at 14 px; selected preset = accent; background strip labels only where they differ from this roast; strip top label unclipped; "Next from background".

## 1.0.30 — Spread column on the Profiles list (Oct 31, 2026; iMac + iPad)
- Profiles list has the Spread column like History: after Color W / G, `spTxt(spreadOf(r.meta))`, same coloring.

## iPad build after 1.0.29 — History & Profiles list → detail (Oct 31, 2026; 13" iPad only, `ipad` branch, no Mac release)
- **Full-width list** (the iMac's table, `thead` shown, rows ~15 px padding, empty Cup column hidden); **tap a row → the detail opens full screen** with "‹ History" / "‹ Profiles" (`#rBack`/`#pBack`, `ipDetSet(k,on)`, classes `html.ipDet-roasts` / `html.ipDet-profiles`). Never on the iMac, the iPhone or the compact iPad (`:not(.ipc)`; the compact iPad keeps its cards via `IPAD&&IPC`).
- Detail uses a 2-column grid (History: details | comments; Profiles: plan | setup, comments | reminders), chart 270 px, action buttons sticky at the bottom. A planned profile's detail (`data-kind` ≠ `p`) keeps its normal layout.
- Ty did NOT like a tabbed Profile detail (Overview / Burner & air / Reminders / Roasts) — mockup left unused on `preview/profile-tabs`. Don't propose tabs again.

## 1.0.29 — UI polish (Oct 31, 2026; iMac + 13" iPad)
- **Roast chart (all)**: gutters sized to the axis numbers (`fitPad`, was fixed `PAD.l 66 / PAD.r 52`); "°F" and "°F/min" inside the top corners of the plot; strip lane names ("Burner %", "Air %", "Pressure Pa") inside their lanes, right labels without units (`stripPm`, `stripPmLabel`).
- **History (iMac)**: no Age column (age is in the detail header), ★ (`.pstar`) instead of the PROFILE pill; Cup column hidden (`.cupt.nocup`, iMac only) unless `cfg.cupScoring` or a roast has a score. Same for Profiles' Cup column, the History "Cupping score" row and the iPad profile card's "Cup —".
- **Roast detail**: adjustments one per line (`.adjl`). **Profile detail**: add-reminder form behind "+ Add reminder" (`details.remdet`); no "not cupped" on each roasted-with row.
- **Green coffee**: Arrived shown as "Aug 12, 2026" (`fmtDay`); date-only strings parsed as local days in `fmtDate` (the lot history showed the starting stock a day early). Stored values unchanged.
- `cupSt(null)` no longer colors an empty score red. Toast 15 px, above the background bar. Settings forms max 1280 px on wide iMac screens.
- **iPad only**: legend row above the roast chart off by default — Settings › Chart scales › "Show the line legend above the chart" (`cfg.scales.ipLegend`, `html.ipLeg`); Drying/Maillard/Dev header chips only when the phase bar is off; right column 364 px on the 13"; Today row buttons 44 pt (13" only); list cards one line with ellipsis (`table-layout:fixed`), right-hand values never clipped; Analyze fits (Cup/Taste visible, tiles 3 across, detail 640 px); cupping pad: 9 flavor families in one row per stage (13").
- The review's other suggestions were built in 1.0.31–1.0.32 (bigger readouts, larger countdown, strip labels, decimals, Analyze labels, Pricing spacing, cupping selected color, red destructive confirm). Still open: see `claude/OPEN-ITEMS.md`.

## 1.0.28 — Phase bar under the roast chart
- Drying / Maillard / Development bar under the chart, with the background's phases below it and differences in each label; colors in Appearance, on/off in Chart scales (`cfg.scales.phaseBar`, `phaseBarBkg`).

## 1.0.27 — Turning point marked sooner
- `detectTP` used to wait for a 1.5 °C (~2.7°F) rise and 8 s after the low; now `cfg.tpRiseC` (default 1°F, stored °C) and `cfg.tpSecs` (default 3), plus the previous reading must also be above the low (one jittery reading can't mark it). Settings → Chart scales.
- TP time is still the lowest bean temp; only how soon it's confirmed changed. Simulator: mark appeared 13–16 s after the low before, 7–12 s now.

## 1.0.26 — Cupping card like Ty's paper card (all devices)
- Ty cups on a **descriptive paper card**, not 100-point scores: 9 flavor families (floral, fruity, herbal, nutty, caramel, chocolate, resin, spice, carbon) checked under F = fragrance (dry), A = aroma (break), FL = flavor, AT = aftertaste; body and acidity heavy/medium/light; balance and overall Y/N; notes; WB/GB.
- Saved on each cupping sample as optional `card {fam:{floral:['F','FL'],…}, body, acid, bal, ov}`; notes stay in `sample.comments`. "Anything off?" writes the roast's `taste.tags` (same as Analyze's taste buttons).
- **iMac/MacBook**: session = all samples side by side (`renderCupCard`). **iPad** (`html.ipad`): one sample at a time, big buttons in cupping order (`renderCupPad`, `cupPadI`). **iPhone**: More › Cupping uses the same card one sample at a time (`phCupCard`).
- Header filled from the roast and green inventory; green coffee got optional **Screen size** (`screen`) and **Water activity** (`wa`) fields (moisture already existed).
- Analyze reads the card (`cardFaults` in `cupFaults`): carbon → roasty (2 if in the cup), resin/herbal in the cup → roasty/underdeveloped, light body → thin, heavy acidity → sharp, balance + overall Yes with no faults → good. **"Cupping cards over time"** section on Analyze per coffee AND batch size.
- SCA 100-point form is optional: `cfg.cupScoring` (Settings › Appearance › Cupping), off by default; when on, sessions get a Card / 100-point switch. Score-based analyzer faults only count when it's on. The phone no longer writes `r.cup` unless the 100-point view is in use.
- CSV export adds `card_*` columns.

## iPhone version (Oct 28, 2026 — merged into `ipad`; TestFlight 1.0.25 build 19)
- Detection: `html.iphone` on an iPhone, or `?iphone=1` in a browser (`?iphone=0` forces off); never on an iPad or the iMac. Viewport gets `viewport-fit=cover` only on the phone. iPhone is portrait in `Info.ios.plist`; iPad stays landscape (`~ipad`).
- Tab bar: **Today · History · Live · Green · More** (More = Cupping, Pricing, Reports, data folder). Analyze, profile builder and Settings stay on the iMac/iPad.
- Today: batch plan, done/next, the batch roasting now on the iMac, and roasts from the last 14 days missing roasted weight or colors ("needs details"). History: search, All / Profiles ★ / This week; a roast shows chart with background, burner/air strip, event times, weights, colors + spread, cost, cup score, burner/air changes, comments. Roast details: roasted weight, Agtron whole/ground, comments (edits the saved roast file via `commitRoast`).
- Live (view only): reads the iMac's `Settings/roast-in-progress.json` from iCloud (refreshCloud every ~4 s; can lag), plus an optional read-only roaster connection on the roastery Wi-Fi (Connect button; disconnects when leaving Live or backgrounding). Background = `profileByName` of the roast's name/batch; shows planned FC.
- Green: tiles, low-stock warning, Add pounds sheet (+60 kg / +69 kg / +50 / +100 lb, optional price → weighted average, same `adj` entries as the iMac), + Coffee. Cupping: sessions, new session, the cupping card, autosaves. Pricing: per coffee 12 oz / 2 lb / 5 lb, wholesale or retail, cost breakdown, margins, edit prices (`cfg.pricing.actual`), "Use" suggested price.
- Safety: `startRoast`, `setControl(true)`, `sendControl`, `nudge`, `sendSwitch`, `sendEventToRoaster` refuse on the phone; roaster pushes go to `phPush`. The roast-recovery banner is skipped on the phone.
- Build 19: Live before Charge uses the same layout (next batch from Today, 0:00, RoR, its background curve + faded burner/air plan, planned TP/DE/FC/Drop); notes name the roasting device from `control.json` (`phWho`). More › **Appearance on this phone** (`zr2.phTheme`: app / white / dark / auto = match iPhone), phone only — `applyTheme` uses `phThemeColors` on the phone.

## 1.0.25 — Follow background (burner/air replay by time)
- **Follow background** button on the SETPOINTS heading (`cfg.followBg`, stays on until turned off). While Control is on, a background is loaded and the roast is between Charge and Drop, the app sends the background's burner and air changes (`S.bgAdj`, from its samples; planned profiles work too) at the same time after Charge (`followTick`, every 250 ms). Burner and air only — never drum or switches.
- Countdown "Auto next 85 · in 0:08" under each setpoint, bold accent in the last 5 s. Changes more than 5 s overdue are skipped, not sent late.
- Any manual burner/air change pauses following for that control ("Following paused — you took over" + Resume; resume skips missed changes). Auto changes go to the roast notes "(auto, from background)", the Connection log, and `controlLog` entries get `auto:true` (optional field).

## 1.0.24 — Start fresh
- Settings → Saving & storage → **Start fresh…** (asks twice). Backs everything up to `Settings/start-fresh-backup-<date>.json`, then clears roasts + profiles, planned profiles, green, cupping and today's plan on every device (roast ids go into `settings.deletedRoasts`; lists are written empty and merge as deletions). Settings stay.
- Numbering restarts at #1 on every device: `settings.numResetAt`; each device ignores its remembered last number (`zr2.lastNum` / `zr2.lastNumAt`) if it's older. History → Import restores the backup (ids in deletedRoasts get new ids).

## 1.0.23 — Bean temp steady rise (less RoR) and colored color spread
- Ty wants to see whether **bean temp rose steadily** and catch any place it **fell**; he asked to back off the RoR analysis.
- Analyze: `btSteady` finds **dips** (bean temp ≥ `anBtDip`, default 1°F, below its last high, 5-sample smoothed) and **flat spots** (rose < half that over `anBtFlat`, default 30 s) from TP+15 s to **5 s before drop**. Each is tied to the burner cut / air opening in the 90 s before it. Red/amber shading on the chart, "Bean temp" list column and tile.
- RoR crash/flick/stall warnings and the `earlier_cut` rule removed; RoR line on the Analyze chart hidden unless **Show RoR** (`zr2.anRor`). `soften_cut` and `more_early_heat` key off bean temp dips/flats. The BB 6-minute RoR rule stays.
- Roast screen: bean temp box flashes amber "FALLING −x°" (2 beeps) if it drops after the turning point (`checkBtFall`, setting `anBtLive`).
- Color spread colored everywhere by editable bands `spBands` ({max,label,col}, last = "over"); Settings → Appearance → Color spread (`spColorsOn`). Separate from the Analyze spread target `anSpread`.

## 1.0.22 — Shared folder (several devices)
- Watches the folder (`fs_stamp`), merges inventory/cupping/planned/plan/settings item-by-item, re-checks roast numbers at save, and only one device has Control (`Settings/control.json`). Deleting roast files in Finder doesn't stick (devices write them back) — use History › Delete or Start fresh.

## 1.0.21 — Auto-off and Control fixes
- Auto-off for cooling/stirrer: the countdown after Drop is only cancelled by Undo on Drop (not by Esc, a new roast, saving, or a connection blip; it fires when the link is back). Esc with Roast details open closes it instead of starting a roast; a 2nd Esc within 4 s of Stop is ignored. Connection log notes auto-off start/cancel.
- The "Control is off — Turn on" box by the setpoints was removed (top-bar Control is the one switch); setpoint/switch buttons fade (`.locked`) and say "Control is off — turn it on at the top". Presets are max 4 per row (`.pre` grid).

## 1.0.20 — Spread rounding
- `spreadOf` rounds to 1 decimal (68.6 − 57.4 showed 11.199999999999996).

## 1.0.19 — Brewed Behavior roasting targets (Ty's consultant's manual)
- Ty does **not** use DTR / development %. Development is a time after first crack. Don't reintroduce percentages.
- Rules come from the Brewed Behavior roasting manual (ch. 8–14): turning point ~1:30; color change 3:30–5:00 at 300°F ± 10; **6-minute rule** (at 6:00 RoR 15°F/min = 1° every 4 s; 20 = 1°/3 s too fast; 12 = 1°/5 s too slow); development 2–4 min; total 8–12 min; color spread target 5 (up to 10 acceptable above Agtron 45); weight loss vs same green at the same color = inner development; bigger batch → hotter charge; optional hard-bean S-curve (90% → 20% at 6:00 → 60% at 12:00).
- **Ty's P12 first cracks at 396°F** (the manual's roasters: 378). Settings default `bbFcC` = 396°F. Default new plan: TP 1:30/195, color change 4:00/310, FC 9:30/396, drop 11:55/413.
- Builder: drop time and temp typed directly; development shown as time; "Brewed Behavior targets" / "Hard bean S-curve" starting shapes; 6:00-rule band on the chart. RoR curve stays a smooth easing decay — doesn't have to be perfectly straight; warn on stalls/crashes/spikes.
- Checks (`bbChecks`, shared by builder and Analyze) and Analyze suggestions use the BB targets. Settings → Analyze rules holds them all (`bb*` keys, stored °C / °C per min / seconds).

## 1.0.18 — Profile builder (plan a profile before the first roast)
- **Profiles** page → **+ New profile**: from scratch, from another profile (e.g. 13 lb → 25 lb, with charge/burner batch-change guesses), or edit an existing one (real profiles also get **Edit as plan**).
- Milestones: charge, turning point, dry end, first crack, drop — plus **any number of extra milestones**. **Drag the dots** on the chart: left/right = time, up/down = bean temp (Charge only up/down). Neighbours can't be passed; at least ~2°F/min rise is kept on each side.
- Curve model: RoR 0 at TP, ramps 30 s, then a fitted smooth decay (a + b·e^(−t/τ)) through dry end/FC/drop; extra milestones bend it with the smallest smooth correction. Every milestone is hit exactly.
- Burner & air plan with steps at events, extra milestones or times. **Suggest burner** / **Match burner to the curve**: uses a reference roast of the same coffee and batch size, aligned by stage, adjusted by learned burner→RoR sensitivity (RoR ≈ α + β·burner(t−45 s) + γ·BT, ridge-fitted per coffee AND batch). Never uses other coffees.
- Planned profiles: badge "Planned", no roast number, no inventory use, not in History/Analyze. Saved in `Profiles/planned-profiles.json` and in the full backup. Become the Roast-screen background for that coffee name and batch size; retired automatically when a real roast is saved as that profile.

## 1.0.17 — Analyze screen (roast curve analyzer)
- **Analyze** tab: pick a roast, see curve check, tap taste feedback (roasty, underdeveloped, baked, thin, etc. — a little / some / a lot), get burner, air and drop recommendations tied to a time or event, plus a projected target curve. **Build next profile** saves the changes as reminders and the target curve (green dotted on the Roast screen).
- Learning is **per coffee AND per batch size** (Ty roasts 13 lb or 25 lb; batches within 12% count as the same). It only learns from changes that were actually made, checked against the next roast's recorded burner/air. Never learn across coffees.
- Profiles are kept **per name and batch size**; the background picks the one matching the entered weight, e.g. `Profiles/Ethiopia Guji — Filter (13 lb).json`.
- New optional roast fields only: `taste`, `plan`, `target`, `bgId`.

## 1.0.16 — Strip labels, History tab
- Burner/air strip labels: a quick follow-up change shows only the settled value; labels that would be clipped drop below the line. The **Roasts** tab is now **History** (folder still `Roasts/`).

## 1.0.15 — RoR lines
- Two RoR lines across the chart (Chart scales → RoR lines: on/off, value in °/min, shading strength, colors `rorLine/rorZone`, `rorLine2/rorZone2`). Fix: the Pricing pill class was renamed `.ppill` (it had overridden `.pt` panel titles).

## 1.0.14 — Pricing
- Pricing screen (Price list / Overview / Costs & settings): true cost per bag from green cost ÷ yield + overhead + labor + packaging + waste; 12 oz / 2 lb / 5 lb, wholesale and retail; gross margin vs after-overhead. Rent $1,200/month, landlord pays electric; Moxie33 left out. Ty prices from his own costs — no market-price comparison.

## 1.0.13 — Green coffee colors and flags
- Country flag beside Origin; process tags colored by editable keyword rows (`procColors`); Yearly use "By origin" card; editable Green coffee page colors. Settings → Appearance grouped by screen.

## 1.0.12 — Cupping grade colors
- Cupping scores colored by grade everywhere; bands, names and colors editable (`cupBands`, `cupColorsOn`).

## 1.0.10 — Faster back-to-back roasts
- New roast button after a saved roast, profile name fills coffee/weight, optional shading under each chart line.

## 1.0.0–1.0.9 — Mac app (Oct 2026)
- Ty's build 33 HTML turned into a Tauri Mac app with GitHub releases and auto-update (never mid-roast). Roaster WebSocket runs natively in Rust.
- Fixes along the way: ad-hoc signing for the "damaged app" warning; Pa/mbar pressure setting; auto-cooling no longer triggers early (roast clock must not run backward, plus time/temp/rising checks); WebSocket handshake tries several styles because the P12 returned 400 to the plain one; Profiles page as a list like History; Delete roast with one confirm (inventory never changes — an adjustment entry instead); Control-on color in Settings; Simulator moved to Settings › Connection; event buttons stay outlined; next-event highlight is a setting (default off).
