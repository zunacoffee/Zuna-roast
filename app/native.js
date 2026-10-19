/* =====================================================================
   Zuna Roast — Mac app bridge
   Loaded before the main app script. In a normal browser it does nothing.
   Inside the Mac app it gives the roast code the same tools it uses in
   Chrome, but backed by the app itself:
     • a data folder (default ~/Documents/Zuna Roast) shaped like Chrome's
       folder handles, so the existing save/sync code works unchanged
     • a WebSocket stand-in that runs the roaster link natively
     • keep-awake while recording or while Control is on
     • Save dialogs for exports, Show in Finder, and the update check
   ===================================================================== */
(function () {
  'use strict';
  const T = window.__TAURI__;
  if (!T || !T.core) { window.ZR_NATIVE = null; return; }
  const invoke = T.core.invoke;
  const N = { version: '', folder: '' };
  window.ZR_NATIVE = N;
  /* iPad app (Tauri on iPadOS). The Mac app's web view never reports touch, so this is false there. */
  const IOS = /iPad|iPhone/.test(navigator.userAgent) || (navigator.maxTouchPoints > 1 && /Macintosh/.test(navigator.userAgent));
  N.ios = IOS;
  const ios = (cmd, args) => invoke('plugin:zuna-ios|' + cmd, args || {});
  /* "…/Mobile Documents/com~apple~CloudDocs/Desktop/ZUNA COFFEE" → "iCloud Drive › Desktop › ZUNA COFFEE" */
  const nice = p => { const s = String(p || ''), i = s.indexOf('com~apple~CloudDocs/');
    return i >= 0 ? 'iCloud Drive › ' + s.slice(i + 20).split('/').filter(Boolean).join(' › ') : s; };
  N.folderPicked = false;

  /* ---------------- data folder ---------------- */
  const join = (a, b) => (a ? a + '/' + b : b);
  const lastPart = p => String(p).split('/').filter(Boolean).pop() || p;
  function notFound(msg) { const e = new Error(msg || 'Not found'); e.name = 'NotFoundError'; return e; }
  const wrapErr = e => (String(e).startsWith('NotFound') ? notFound(String(e)) : new Error(String(e)));

  class FileH {
    constructor(rel, name) { this.kind = 'file'; this.rel = rel; this.name = name; }
    async getFile() {
      const rel = this.rel;
      return { text: async () => { try { return await invoke('fs_read', { rel }); } catch (e) { throw wrapErr(e); } } };
    }
    async createWritable() {
      const rel = this.rel; let buf = '';
      return {
        write: async t => { buf += typeof t === 'string' ? t : String(t); },
        close: async () => { try { await invoke('fs_write', { rel, text: buf }); } catch (e) { throw wrapErr(e); } }
      };
    }
  }
  class DirH {
    constructor(rel, name) { this.kind = 'directory'; this.rel = rel; this.name = name; }
    async getDirectoryHandle(name, o = {}) {
      const rel = join(this.rel, name);
      if (o.create) await invoke('fs_mkdir', { rel });
      else if (!(await invoke('fs_is_dir', { rel }))) throw notFound(rel);
      return new DirH(rel, name);
    }
    async getFileHandle(name) { return new FileH(join(this.rel, name), name); }
    async removeEntry(name) { await invoke('fs_remove', { rel: join(this.rel, name) }); }
    async *entries() {
      let list;
      try { list = await invoke('fs_list', { rel: this.rel || '.' }); } catch (e) { throw wrapErr(e); }
      for (const e of list) {
        const rel = join(this.rel, e.name);
        yield [e.name, e.dir ? new DirH(rel, e.name) : new FileH(rel, e.name)];
      }
    }
    async queryPermission() { return 'granted'; }
    async requestPermission() { return 'granted'; }
  }

  async function iosOpen(path) {
    const r = await invoke('set_data_folder', { path: String(path) });
    N.folder = nice(r); N.folderPicked = true;
    try {
      const t = setTimeout(() => { if (window.toast) toast('Getting your roasts from iCloud…'); }, 800);
      const d = await ios('download_folder'); clearTimeout(t);
      if (d && d.pending && window.toast) toast(`${d.pending} file(s) are still downloading from iCloud — they'll show up shortly`);
    } catch (e) {}
    return new DirH('', lastPart(r));
  }
  N.root = async () => {
    if (IOS) {
      try { const r = await ios('restore_folder'); if (r && r.path) return await iosOpen(r.path); } catch (e) {}
    }
    const p = await invoke('data_folder'); N.folder = IOS ? nice(p) : p; return new DirH('', lastPart(p));
  };
  N.chooseFolder = async () => {
    if (IOS) {
      const r = await ios('pick_folder');
      if (!r || !r.path) return null;
      return await iosOpen(r.path);
    }
    const p = await T.dialog.open({ directory: true, multiple: false, title: 'Choose the folder for your roast data' });
    if (!p) return null;
    const r = await invoke('set_data_folder', { path: String(p) });
    N.folder = r; return new DirH('', lastPart(r));
  };
  N.reveal = () => IOS ? (window.toast && toast('Open the Files app › ' + N.folder)) : invoke('reveal_data_folder').catch(e => window.toast && toast('Could not open Finder: ' + e));
  N.saveExport = async (name, text) => {
    if (IOS) { const r = await ios('share_file', { name: String(name), text: String(text) }); return !!(r && r.done); }
    const p = await T.dialog.save({ defaultPath: name, title: 'Save export' });
    if (!p) return false;
    await invoke('save_export', { path: String(p), text: String(text) });
    return true;
  };

  /* ---------------- roaster WebSocket ---------------- */
  let seq = 0; const socks = new Map();
  class NativeWebSocket {
    constructor(url) {
      this.url = String(url); this.readyState = 0; this.id = ++seq;
      this.onopen = this.onmessage = this.onerror = this.onclose = null;
      socks.set(this.id, this);
      invoke('ws_open', { id: this.id, url: this.url }).catch(e => { this._error(String(e)); this._closed(); });
    }
    send(text) {
      if (this.readyState !== 1) throw new Error('WebSocket is not open');
      invoke('ws_send', { id: this.id, text: String(text) }).catch(() => {});
    }
    close() {
      if (this.readyState >= 2) return;
      this.readyState = 2;
      invoke('ws_close', { id: this.id }).catch(() => {});
    }
    _error(msg) { try { this.onerror && this.onerror({ type: 'error', message: msg }); } catch (e) {} }
    _closed() {
      if (this.readyState === 3) return;
      this.readyState = 3; socks.delete(this.id);
      try { this.onclose && this.onclose({ type: 'close' }); } catch (e) {}
    }
  }
  NativeWebSocket.CONNECTING = 0; NativeWebSocket.OPEN = 1; NativeWebSocket.CLOSING = 2; NativeWebSocket.CLOSED = 3;
  T.event.listen('roaster-ws', ev => {
    const { id, kind, data } = ev.payload || {}; const s = socks.get(id); if (!s) return;
    if (kind === 'open') {
      if (s.readyState === 2) { invoke('ws_close', { id }).catch(() => {}); return; }
      s.readyState = 1; try { s.onopen && s.onopen({ type: 'open' }); } catch (e) {}
    } else if (kind === 'message') {
      try { s.onmessage && s.onmessage({ type: 'message', data }); } catch (e) { console.error(e); }
    } else if (kind === 'info') {
      if (window.log) try { log('sys', 'Roaster link: ' + data); } catch (e) {}
    } else if (kind === 'error') {
      if (window.log) try { log('err', 'Roaster link: ' + data); } catch (e) {}
      s._error(data);
    } else if (kind === 'close') {
      s._closed();
    }
  });
  window.WebSocket = NativeWebSocket;

  /* ---------------- keep the Mac awake ---------------- */
  const awakeFor = new Set(); let awakeOn = false;
  N.awake = (why, on) => {
    on ? awakeFor.add(why) : awakeFor.delete(why);
    const want = awakeFor.size > 0;
    if (want !== awakeOn) { awakeOn = want; (IOS ? ios('keep_awake', { on: want }) : invoke('keep_awake', { on: want })).catch(() => {}); }
  };
  try {
    Object.defineProperty(navigator, 'wakeLock', {
      configurable: true,
      value: {
        request: async () => {
          N.awake('control', true);
          const l = [];
          return { released: false, addEventListener(t, f) { l.push(f); },
            async release() { if (this.released) return; this.released = true; N.awake('control', false); l.forEach(f => { try { f(); } catch (e) {} }); } };
        }
      }
    });
  } catch (e) {}
  const recording = () => { try { return typeof S !== 'undefined' && !!S.recording; } catch (e) { return false; } };
  setInterval(() => N.awake('recording', recording()), 3000);

  /* ---------------- updates (only offered when the app opens, never mid-roast) ---------------- */
  N.checkUpdate = async manual => {
    const say = m => { if (window.toast) toast(m); };
    if (!T.updater) { if (manual) say('Updates are not available in this build'); return; }
    if (recording()) { if (manual) say('Finish the roast first, then check for updates'); return; }
    let up;
    try { up = await T.updater.check(); } catch (e) { if (manual) say('Could not check for updates — are you online?'); return; }
    if (!up) { if (manual) say(`You have the latest version (${N.version})`); return; }
    if (recording()) return;
    const notes = up.body ? String(up.body).trim() + '\n\n' : '';
    const yes = await ask(`Update Zuna Roast to version ${up.version}?`,
      `${notes}You have ${N.version}. The app restarts after updating. Your roasts, profiles and settings are not touched.`, 'Update now');
    if (!yes) return;
    say('Downloading the update…');
    try { await up.downloadAndInstall(); say('Update installed — restarting'); await T.process.relaunch(); }
    catch (e) { say('The update did not finish: ' + e); }
  };

  /* ---------------- closing the window mid-roast ---------------- */
  function guardClose() {
    try {
      const w = T.window.getCurrentWindow();
      w.onCloseRequested(async ev => {
        let unsaved = false;
        try { unsaved = recording() || (S.dirty && S.roast.samples.length && !S.viewingId); } catch (e) {}
        if (!unsaved) return;
        ev.preventDefault();
        const yes = await ask(recording() ? 'A roast is recording' : 'This roast is not saved',
          recording() ? 'Quitting stops the recording. It is kept for recovery the next time you open the app.' : 'Quit anyway? The unsaved roast is lost.',
          'Quit anyway', true);
        if (yes) { try { if (recording() && window.saveDraft) saveDraft(); } catch (e) {} setTimeout(() => w.destroy(), 300); }
      });
    } catch (e) {}
  }

  window.addEventListener('DOMContentLoaded', async () => {
    try { N.version = await T.app.getVersion(); } catch (e) {}
    if (IOS) {
      /* First time on the iPad: point it at the shared iCloud roast folder */
      setTimeout(async () => {
        if (N.folderPicked || !window.ask) return;
        const yes = await ask('Choose your roast folder',
          'Pick iCloud Drive › Desktop › ZUNA COFFEE so this iPad uses the same roasts, profiles, green coffee and settings as the iMac. You only do this once.',
          'Choose folder');
        if (yes && window.chooseFolder) chooseFolder();
      }, 1200);
      return;
    }
    guardClose();
    setTimeout(() => N.checkUpdate(false), 5000);
  });
})();
