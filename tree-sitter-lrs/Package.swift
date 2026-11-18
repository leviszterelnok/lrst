// swift-tools-version:5.3
import PackageDescription

let package = Package(
    name: "TreeSitterLrs",
    products: [
        .library(name: "TreeSitterLrs", targets: ["TreeSitterLrs"]),
    ],
    dependencies: [
        .package(url: "https://github.com/ChimeHQ/SwiftTreeSitter", from: "0.8.0"),
    ],
    targets: [
        .target(
            name: "TreeSitterLrs",
            dependencies: [],
            path: ".",
            sources: [
                "src/parser.c",
                // NOTE: if your language has an external scanner, add it here.
            ],
            resources: [
                .copy("queries")
            ],
            publicHeadersPath: "bindings/swift",
            cSettings: [.headerSearchPath("src")]
        ),
        .testTarget(
            name: "TreeSitterLrsTests",
            dependencies: [
                "SwiftTreeSitter",
                "TreeSitterLrs",
            ],
            path: "bindings/swift/TreeSitterLrsTests"
        )
    ],
    cLanguageStandard: .c11
)
