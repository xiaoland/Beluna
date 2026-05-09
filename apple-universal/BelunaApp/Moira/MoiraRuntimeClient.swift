import Foundation

protocol MoiraRuntimeClient: Sendable {
    func loadLoomSnapshot(selection: MoiraLoomSelection) async throws -> MoiraLoomSnapshot
    func wakeCore(request: MoiraCoreWakeRequest) async throws -> MoiraCoreStatus
    func stopCore() async throws -> MoiraCoreStatus
    func forceKillCore() async throws -> MoiraCoreStatus
    func loadProfileDocument(profileID: String) async throws -> MoiraProfileDocument
    func saveProfileDocument(request: MoiraProfileSaveRequest) async throws -> MoiraProfileDocument
    func loadProfileDraft(profileID: String) async throws -> MoiraProfileDraftDocument
    func saveProfileDraft(
        request: MoiraProfileDraftSaveRequest
    ) async throws -> MoiraProfileDraftDocument
    func registerKnownLocalBuild(
        registration: MoiraKnownLocalBuildRegistration
    ) async throws -> MoiraLaunchTargetRef
}

struct UnavailableMoiraRuntimeClient: MoiraRuntimeClient {
    var reason: String

    func loadLoomSnapshot(selection: MoiraLoomSelection) async throws -> MoiraLoomSnapshot {
        MoiraLoomSnapshot.unavailable(reason: reason)
    }

    func wakeCore(request: MoiraCoreWakeRequest) async throws -> MoiraCoreStatus {
        throw MoiraRuntimeClientError.unavailable(reason)
    }

    func stopCore() async throws -> MoiraCoreStatus {
        throw MoiraRuntimeClientError.unavailable(reason)
    }

    func forceKillCore() async throws -> MoiraCoreStatus {
        throw MoiraRuntimeClientError.unavailable(reason)
    }

    func loadProfileDocument(profileID: String) async throws -> MoiraProfileDocument {
        throw MoiraRuntimeClientError.unavailable(reason)
    }

    func saveProfileDocument(request: MoiraProfileSaveRequest) async throws -> MoiraProfileDocument {
        throw MoiraRuntimeClientError.unavailable(reason)
    }

    func loadProfileDraft(profileID: String) async throws -> MoiraProfileDraftDocument {
        throw MoiraRuntimeClientError.unavailable(reason)
    }

    func saveProfileDraft(
        request: MoiraProfileDraftSaveRequest
    ) async throws -> MoiraProfileDraftDocument {
        throw MoiraRuntimeClientError.unavailable(reason)
    }

    func registerKnownLocalBuild(
        registration: MoiraKnownLocalBuildRegistration
    ) async throws -> MoiraLaunchTargetRef {
        throw MoiraRuntimeClientError.unavailable(reason)
    }
}

struct StaticMoiraRuntimeClient: MoiraRuntimeClient {
    var snapshot: MoiraLoomSnapshot
    var operationStatus: MoiraCoreStatus?
    var profileDocument: MoiraProfileDocument?
    var profileDraft: MoiraProfileDraftDocument?

    init(snapshot: MoiraRuntimeSnapshot) {
        self.snapshot = .statusOnly(snapshot)
        self.operationStatus = snapshot.core
        self.profileDocument = nil
        self.profileDraft = nil
    }

    init(loomSnapshot: MoiraLoomSnapshot) {
        self.snapshot = loomSnapshot
        self.operationStatus = loomSnapshot.status.core
        self.profileDocument = nil
        self.profileDraft = nil
    }

    func loadLoomSnapshot(selection: MoiraLoomSelection) async throws -> MoiraLoomSnapshot {
        snapshot
    }

    func wakeCore(request: MoiraCoreWakeRequest) async throws -> MoiraCoreStatus {
        operationStatus ?? snapshot.status.core
    }

    func stopCore() async throws -> MoiraCoreStatus {
        operationStatus ?? snapshot.status.core
    }

    func forceKillCore() async throws -> MoiraCoreStatus {
        operationStatus ?? snapshot.status.core
    }

    func loadProfileDocument(profileID: String) async throws -> MoiraProfileDocument {
        profileDocument ?? MoiraProfileDocument(
            profileID: profileID,
            profilePath: "",
            contents: ""
        )
    }

    func saveProfileDocument(request: MoiraProfileSaveRequest) async throws -> MoiraProfileDocument {
        MoiraProfileDocument(
            profileID: request.profileID,
            profilePath: "",
            contents: request.contents
        )
    }

    func loadProfileDraft(profileID: String) async throws -> MoiraProfileDraftDocument {
        profileDraft ?? MoiraProfileDraftDocument(
            profileID: profileID,
            profilePath: "",
            coreConfig: "{\n}\n",
            envFiles: [],
            inlineEnvironment: []
        )
    }

    func saveProfileDraft(
        request: MoiraProfileDraftSaveRequest
    ) async throws -> MoiraProfileDraftDocument {
        MoiraProfileDraftDocument(
            profileID: request.profileID,
            profilePath: "",
            coreConfig: request.coreConfig,
            envFiles: request.envFiles,
            inlineEnvironment: request.inlineEnvironment
        )
    }

    func registerKnownLocalBuild(
        registration: MoiraKnownLocalBuildRegistration
    ) async throws -> MoiraLaunchTargetRef {
        MoiraLaunchTargetRef(
            kind: "knownLocalBuild",
            buildID: registration.buildID,
            releaseTag: nil,
            rustTargetTriple: nil
        )
    }
}

enum MoiraRuntimeClientError: Error, CustomStringConvertible {
    case unavailable(String)
    case missingLaunchTarget
    case missingProfile
    case invalidProfileDraft
    case invalidKnownLocalBuildDraft

    var description: String {
        switch self {
        case let .unavailable(reason):
            reason
        case .missingLaunchTarget:
            "Select a launch target before waking Core."
        case .missingProfile:
            "Select a profile before loading it."
        case .invalidProfileDraft:
            "Profile ID and core_config are required; environment entries must have names and file paths."
        case .invalidKnownLocalBuildDraft:
            "Build ID and executable path are required."
        }
    }
}

private extension MoiraLoomSnapshot {
    static func statusOnly(_ status: MoiraRuntimeSnapshot) -> Self {
        Self(
            status: status,
            launchTargets: [],
            profiles: [],
            runs: [],
            selectedRunID: nil,
            ticks: [],
            selectedTick: nil,
            tickDetail: nil,
            updatedAt: nil
        )
    }
}

enum MoiraRuntimeClientFactory {
    static func makeDefault() -> any MoiraRuntimeClient {
        #if os(macOS)
        do {
            return try DynamicMoiraRuntimeClient.makeDefault()
        } catch {
            return UnavailableMoiraRuntimeClient(reason: String(describing: error))
        }
        #else
        return UnavailableMoiraRuntimeClient(reason: "Moira runtime binding is pending for this platform.")
        #endif
    }
}
