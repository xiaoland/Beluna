#if os(macOS)
import Foundation

struct MoiraRuntimeConfiguration: Sendable {
    var rootDirectoryPath: String
    var receiverBind: String

    static func `default`(
        fileManager: FileManager = .default
    ) -> Self {
        let appSupportDirectory = (try? fileManager.url(
            for: .applicationSupportDirectory,
            in: .userDomainMask,
            appropriateFor: nil,
            create: true
        )) ?? URL(fileURLWithPath: NSTemporaryDirectory(), isDirectory: true)

        return Self(
            rootDirectoryPath: appSupportDirectory
                .appendingPathComponent("Beluna", isDirectory: true)
                .appendingPathComponent("Moira", isDirectory: true)
                .path,
            receiverBind: "127.0.0.1:4317"
        )
    }
}

struct DynamicMoiraRuntimeClient: MoiraRuntimeClient {
    private let library: MoiraRuntimeDynamicLibrary
    private let configuration: MoiraRuntimeConfiguration

    static func makeDefault() throws -> Self {
        Self(
            library: try MoiraRuntimeDynamicLibrary.loadDefault(),
            configuration: .default()
        )
    }

    func loadLoomSnapshot(selection: MoiraLoomSelection) async throws -> MoiraLoomSnapshot {
        try await Task.detached(priority: .utility) {
            try library.loomSnapshot(configuration: configuration, selection: selection)
        }.value
    }

    func wakeCore(request: MoiraCoreWakeRequest) async throws -> MoiraCoreStatus {
        try await Task.detached(priority: .utility) {
            try library.wakeCore(configuration: configuration, request: request)
        }.value
    }

    func stopCore() async throws -> MoiraCoreStatus {
        try await Task.detached(priority: .utility) {
            try library.stopCore(configuration: configuration)
        }.value
    }

    func forceKillCore() async throws -> MoiraCoreStatus {
        try await Task.detached(priority: .utility) {
            try library.forceKillCore(configuration: configuration)
        }.value
    }

    func loadProfileDocument(profileID: String) async throws -> MoiraProfileDocument {
        try await Task.detached(priority: .utility) {
            try library.loadProfileDocument(configuration: configuration, profileID: profileID)
        }.value
    }

    func saveProfileDocument(request: MoiraProfileSaveRequest) async throws -> MoiraProfileDocument {
        try await Task.detached(priority: .utility) {
            try library.saveProfileDocument(configuration: configuration, request: request)
        }.value
    }

    func loadProfileDraft(profileID: String) async throws -> MoiraProfileDraftDocument {
        try await Task.detached(priority: .utility) {
            try library.loadProfileDraft(configuration: configuration, profileID: profileID)
        }.value
    }

    func saveProfileDraft(
        request: MoiraProfileDraftSaveRequest
    ) async throws -> MoiraProfileDraftDocument {
        try await Task.detached(priority: .utility) {
            try library.saveProfileDraft(configuration: configuration, request: request)
        }.value
    }

    func registerKnownLocalBuild(
        registration: MoiraKnownLocalBuildRegistration
    ) async throws -> MoiraLaunchTargetRef {
        try await Task.detached(priority: .utility) {
            try library.registerKnownLocalBuild(
                configuration: configuration,
                registration: registration
            )
        }.value
    }
}

enum MoiraRuntimeDynamicClientError: Error, CustomStringConvertible {
    case libraryMissing([String])
    case libraryLoadFailed([String])
    case symbolMissing(String)
    case statusFailure(String)
    case invalidRequestPayload(String)
    case missingStatusPayload
    case invalidStatusPayload(String)

    var description: String {
        switch self {
        case let .libraryMissing(candidates):
            "Moira FFI dylib missing. Build `moira-ffi`; searched: \(candidates.joined(separator: ", "))"
        case let .libraryLoadFailed(errors):
            "Moira FFI dylib load failed: \(errors.joined(separator: "; "))"
        case let .symbolMissing(symbol):
            "Moira FFI symbol missing: \(symbol)"
        case let .statusFailure(message):
            message
        case let .invalidRequestPayload(message):
            "Moira FFI request could not be encoded: \(message)"
        case .missingStatusPayload:
            "Moira FFI returned an empty status payload"
        case let .invalidStatusPayload(message):
            "Moira FFI returned invalid status JSON: \(message)"
        }
    }
}

#endif
