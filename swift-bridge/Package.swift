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
            name: "CloudKitObjCBridge",
            path: "Sources/CloudKitObjCBridge",
            publicHeadersPath: "include"),
        .target(
            name: "CloudKitBridge",
            dependencies: ["CloudKitObjCBridge"],
            path: "Sources/CloudKitBridge")
    ]
)
