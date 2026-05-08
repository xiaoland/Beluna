import Foundation

protocol MoiraRuntimeClient: Sendable {
    func loadSnapshot() async throws -> MoiraRuntimeSnapshot
}

struct UnavailableMoiraRuntimeClient: MoiraRuntimeClient {
    var reason: String

    func loadSnapshot() async throws -> MoiraRuntimeSnapshot {
        MoiraRuntimeSnapshot.unavailable(reason: reason)
    }
}

struct StaticMoiraRuntimeClient: MoiraRuntimeClient {
    var snapshot: MoiraRuntimeSnapshot

    func loadSnapshot() async throws -> MoiraRuntimeSnapshot {
        snapshot
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
