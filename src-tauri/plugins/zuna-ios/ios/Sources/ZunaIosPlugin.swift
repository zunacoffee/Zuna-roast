import Foundation
import SwiftRs
import Tauri
import UIKit
import UniformTypeIdentifiers
import WebKit

struct AwakeArgs: Decodable { let on: Bool }
struct ShareArgs: Decodable { let name: String; let text: String }
struct ChromeArgs: Decodable { let color: String; let dark: Bool }

/// iPad-only helpers for Zuna Roast (see src/lib.rs for what each one is for).
class ZunaIosPlugin: Plugin, UIDocumentPickerDelegate {
    private let bookmarkKey = "zunaRoastFolderBookmark"
    private var pendingPick: Invoke?
    private var folder: URL?
    private weak var webView: WKWebView?

    override func load(webview: WKWebView) {
        webView = webview
    }

    // MARK: roast folder (iCloud Drive › Desktop › ZUNA COFFEE › Zuna Roast)

    private func remember(_ url: URL) throws {
        let data = try url.bookmarkData(options: [], includingResourceValuesForKeys: nil, relativeTo: nil)
        UserDefaults.standard.set(data, forKey: bookmarkKey)
    }

    private func use(_ url: URL) {
        if let old = folder, old != url { old.stopAccessingSecurityScopedResource() }
        _ = url.startAccessingSecurityScopedResource()
        folder = url
    }

    @objc public func pickFolder(_ invoke: Invoke) {
        DispatchQueue.main.async {
            guard let vc = self.manager.viewController else { invoke.reject("The app window is not ready"); return }
            let picker = UIDocumentPickerViewController(forOpeningContentTypes: [UTType.folder])
            picker.delegate = self
            picker.allowsMultipleSelection = false
            picker.directoryURL = self.folder
            self.pendingPick = invoke
            vc.present(picker, animated: true)
        }
    }

    public func documentPicker(_ controller: UIDocumentPickerViewController, didPickDocumentsAt urls: [URL]) {
        guard let invoke = pendingPick else { return }
        pendingPick = nil
        guard let url = urls.first else { invoke.resolve(["path": ""]); return }
        use(url)
        do {
            try remember(url)
            invoke.resolve(["path": url.path])
        } catch {
            invoke.reject("Could not remember that folder: \(error.localizedDescription)")
        }
    }

    public func documentPickerWasCancelled(_ controller: UIDocumentPickerViewController) {
        pendingPick?.resolve(["path": ""])
        pendingPick = nil
    }

    @objc public func restoreFolder(_ invoke: Invoke) {
        guard let data = UserDefaults.standard.data(forKey: bookmarkKey) else {
            invoke.resolve(["path": ""])
            return
        }
        do {
            var stale = false
            let url = try URL(resolvingBookmarkData: data, options: [], relativeTo: nil, bookmarkDataIsStale: &stale)
            use(url)
            if stale { try? remember(url) }
            invoke.resolve(["path": url.path])
        } catch {
            invoke.resolve(["path": "", "error": error.localizedDescription])
        }
    }

    /// Ask iCloud to bring down every roast file that is still only in the cloud,
    /// and wait (up to ~25 s) for them to arrive.
    @objc public func downloadFolder(_ invoke: Invoke) {
        guard let root = folder else { invoke.resolve(["pending": 0]); return }
        DispatchQueue.global(qos: .userInitiated).async {
            let fm = FileManager.default
            let keys: [URLResourceKey] = [.isDirectoryKey, .isUbiquitousItemKey, .ubiquitousItemDownloadingStatusKey]
            func notHere() -> [URL] {
                var out: [URL] = []
                guard let en = fm.enumerator(at: root, includingPropertiesForKeys: keys, options: []) else { return out }
                for case let u as URL in en {
                    let name = u.lastPathComponent
                    if name.hasPrefix(".") && name.hasSuffix(".icloud") {
                        // Older iPadOS shows cloud-only files as ".Name.json.icloud"
                        let real = String(name.dropFirst().dropLast(".icloud".count))
                        out.append(u.deletingLastPathComponent().appendingPathComponent(real))
                        continue
                    }
                    guard let v = try? u.resourceValues(forKeys: Set(keys)) else { continue }
                    if v.isDirectory == true || v.isUbiquitousItem != true { continue }
                    if let st = v.ubiquitousItemDownloadingStatus, st != .current { out.append(u) }
                }
                return out
            }
            var left = notHere()
            for u in left { try? fm.startDownloadingUbiquitousItem(at: u) }
            let deadline = Date().addingTimeInterval(25)
            while !left.isEmpty && Date() < deadline {
                Thread.sleep(forTimeInterval: 0.5)
                left = notHere()
            }
            invoke.resolve(["pending": left.count])
        }
    }

    // MARK: keep the screen on during a roast

    @objc public func keepAwake(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(AwakeArgs.self)
        DispatchQueue.main.async {
            UIApplication.shared.isIdleTimerDisabled = args.on
            invoke.resolve(["on": args.on])
        }
    }

    // MARK: screen edges and status bar follow the app theme

    private func color(_ hex: String) -> UIColor {
        var h = hex.trimmingCharacters(in: .whitespaces).replacingOccurrences(of: "#", with: "")
        if h.count == 3 { h = h.map { "\($0)\($0)" }.joined() }
        guard h.count == 6, let n = UInt32(h, radix: 16) else { return .systemBackground }
        return UIColor(red: CGFloat((n >> 16) & 255) / 255, green: CGFloat((n >> 8) & 255) / 255, blue: CGFloat(n & 255) / 255, alpha: 1)
    }

    @objc public func setChrome(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(ChromeArgs.self)
        let c = color(args.color)
        DispatchQueue.main.async {
            let vc = self.manager.viewController
            vc?.view.backgroundColor = c
            vc?.view.window?.backgroundColor = c
            if let wv = self.webView {
                wv.isOpaque = false
                wv.backgroundColor = c
                wv.scrollView.backgroundColor = c
                if #available(iOS 15.0, *) { wv.underPageBackgroundColor = c }
            }
            // Info.ios.plist sets UIViewControllerBasedStatusBarAppearance = false, so this applies app-wide.
            UIApplication.shared.statusBarStyle = args.dark ? .lightContent : .darkContent
            vc?.setNeedsStatusBarAppearanceUpdate()
            invoke.resolve(["ok": true])
        }
    }

    // MARK: exports go out through the share sheet

    @objc public func shareFile(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(ShareArgs.self)
        let safe = args.name.replacingOccurrences(of: "/", with: "-")
        let url = FileManager.default.temporaryDirectory.appendingPathComponent(safe)
        try args.text.write(to: url, atomically: true, encoding: .utf8)
        DispatchQueue.main.async {
            guard let vc = self.manager.viewController else { invoke.reject("The app window is not ready"); return }
            let sheet = UIActivityViewController(activityItems: [url], applicationActivities: nil)
            if let pop = sheet.popoverPresentationController {
                pop.sourceView = vc.view
                pop.sourceRect = CGRect(x: vc.view.bounds.midX, y: 70, width: 1, height: 1)
                pop.permittedArrowDirections = [.up]
            }
            sheet.completionWithItemsHandler = { _, done, _, _ in invoke.resolve(["done": done]) }
            vc.present(sheet, animated: true)
        }
    }
}

@_cdecl("init_plugin_zuna_ios")
func initPlugin() -> Plugin {
    return ZunaIosPlugin()
}
