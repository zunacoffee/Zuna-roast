// swift-tools-version:5.3
import PackageDescription

let package = Package(
    name: "tauri-plugin-zuna-ios",
    platforms: [
        .iOS(.v14)
    ],
    products: [
        .library(
            name: "tauri-plugin-zuna-ios",
            type: .static,
            targets: ["tauri-plugin-zuna-ios"])
    ],
    dependencies: [
        .package(name: "Tauri", path: "../.tauri/tauri-api")
    ],
    targets: [
        .target(
            name: "tauri-plugin-zuna-ios",
            dependencies: [
                .byName(name: "Tauri")
            ],
            path: "Sources")
    ]
)
