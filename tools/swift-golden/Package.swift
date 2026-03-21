// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "SwiftGolden",
    platforms: [.macOS(.v14)],
    dependencies: [
        .package(url: "https://github.com/pointfreeco/swift-snapshot-testing", from: "1.17.0"),
    ],
    targets: [
        .target(
            name: "SwiftGolden",
            path: "Sources/SwiftGolden"
        ),
        .testTarget(
            name: "SwiftGoldenTests",
            dependencies: [
                "SwiftGolden",
                .product(name: "SnapshotTesting", package: "swift-snapshot-testing"),
            ],
            path: "Tests/SwiftGoldenTests"
        ),
    ]
)
