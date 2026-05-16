// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "CloudKitBridge",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .library(
            name: "CloudKitBridge",
            type: .static,
            targets: ["CloudKitBridge"])
    ],
    targets: [
        .target(
            name: "CloudKitBridge",
            path: "Sources/CloudKitBridge",
            publicHeadersPath: "include")
    ]
)
