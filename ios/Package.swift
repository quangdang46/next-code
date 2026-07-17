// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "NextCodeKit",
    platforms: [
        .iOS(.v17),
        .macOS(.v14),
    ],
    products: [
        .library(name: "NextCodeKit", targets: ["NextCodeKit"])
    ],
    targets: [
        .target(
            name: "NextCodeKit",
            swiftSettings: [.enableUpcomingFeature("StrictConcurrency")]
        ),
        .testTarget(
            name: "NextCodeKitTests",
            dependencies: ["NextCodeKit"]
        ),
    ]
)
