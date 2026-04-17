// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "macos-recorder",
    platforms: [
        .macOS(.v15),
    ],
    products: [
        .executable(
            name: "EpidoteRecorderCLI",
            targets: ["EpidoteRecorderCLI"]
        ),
    ],
    targets: [
        .executableTarget(
            name: "EpidoteRecorderCLI",
            linkerSettings: [
                .linkedFramework("AVFoundation"),
                .linkedFramework("CoreGraphics"),
                .linkedFramework("ScreenCaptureKit"),
            ]
        ),
    ]
)
