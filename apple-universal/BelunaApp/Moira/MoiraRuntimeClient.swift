import Foundation

protocol MoiraRuntimeClient: Sendable {
    func loadLoomSnapshot(selection: MoiraLoomSelection) async throws -> MoiraLoomSnapshot
}

struct UnavailableMoiraRuntimeClient: MoiraRuntimeClient {
    var reason: String

    func loadLoomSnapshot(selection: MoiraLoomSelection) async throws -> MoiraLoomSnapshot {
        MoiraLoomSnapshot.unavailable(reason: reason)
    }
}

struct StaticMoiraRuntimeClient: MoiraRuntimeClient {
    var snapshot: MoiraLoomSnapshot

    init(snapshot: MoiraRuntimeSnapshot) {
        self.snapshot = .statusOnly(snapshot)
    }

    init(loomSnapshot: MoiraLoomSnapshot) {
        self.snapshot = loomSnapshot
    }

    func loadLoomSnapshot(selection: MoiraLoomSelection) async throws -> MoiraLoomSnapshot {
        snapshot
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
