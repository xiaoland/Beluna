// swift-tools-version: 5.10
import PackageDescription

let package = Package(
    name: "BelunaAppleUniversal",
    platforms: [
        .macOS(.v14)
    ],
    products: [
        .executable(name: "BelunaApp", targets: ["BelunaApp"])
    ],
    targets: [
        .executableTarget(
            name: "BelunaApp"
        ),
        .testTarget(
            name: "BelunaAppleUniversalTests",
            dependencies: ["BelunaApp"]
        )
    ]
)
