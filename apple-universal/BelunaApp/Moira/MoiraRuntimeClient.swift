import Foundation

protocol MoiraRuntimeClient: Sendable {
    func loadLoomSnapshot(selection: MoiraLoomSelection) async throws -> MoiraLoomSnapshot
    func wakeCore(request: MoiraCoreWakeRequest) async throws -> MoiraCoreStatus
    func stopCore() async throws -> MoiraCoreStatus
    func forceKillCore() async throws -> MoiraCoreStatus
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
}

struct StaticMoiraRuntimeClient: MoiraRuntimeClient {
    var snapshot: MoiraLoomSnapshot
    var operationStatus: MoiraCoreStatus?

    init(snapshot: MoiraRuntimeSnapshot) {
        self.snapshot = .statusOnly(snapshot)
        self.operationStatus = snapshot.core
    }

    init(loomSnapshot: MoiraLoomSnapshot) {
        self.snapshot = loomSnapshot
        self.operationStatus = loomSnapshot.status.core
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
}

enum MoiraRuntimeClientError: Error, CustomStringConvertible {
    case unavailable(String)
    case missingLaunchTarget

    var description: String {
        switch self {
        case let .unavailable(reason):
            reason
        case .missingLaunchTarget:
            "Select a launch target before waking Core."
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
