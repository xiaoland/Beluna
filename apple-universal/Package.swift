// swift-tools-version: 5.10
import PackageDescription

let package = Package(
    name: "BelunaAppleUniversal",
    platforms: [
        .macOS(.v14)
    ],
    products: [
        .executable(name: "BelunaAppleUniversalApp", targets: ["BelunaAppleUniversalApp"])
    ],
    targets: [
        .executableTarget(
            name: "BelunaAppleUniversalApp"
        ),
        .testTarget(
            name: "BelunaAppleUniversalTests",
            dependencies: ["BelunaAppleUniversalApp"]
        )
    ]
)
