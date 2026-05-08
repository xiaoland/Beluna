import Foundation

@MainActor
final class MoiraOperationsViewModel: ObservableObject {
    @Published private(set) var snapshot: MoiraRuntimeSnapshot
    @Published private(set) var isRefreshing = false
    @Published private(set) var lastErrorText: String?

    private let client: any MoiraRuntimeClient

    init(client: any MoiraRuntimeClient) {
        self.client = client
        self.snapshot = .unavailable(reason: "Moira runtime status is waiting for first load.")
    }

    var runtimeStatusText: String {
        snapshot.lifecycle.rawValue
    }

    var receiverStatusText: String {
        snapshot.receiver.wakeState
    }

    var coreStatusText: String {
        snapshot.core.phase
    }

    var eventCountText: String {
        "\(snapshot.receiver.rawEventCount)"
    }

    var wakeCountText: String {
        "\(snapshot.receiver.wakeCount)"
    }

    var tickCountText: String {
        "\(snapshot.receiver.tickCount)"
    }

    var canRefresh: Bool {
        !isRefreshing
    }

    func refresh() {
        Task {
            await refreshNow()
        }
    }

    func refreshNow() async {
        guard !isRefreshing else {
            return
        }

        isRefreshing = true
        defer {
            isRefreshing = false
        }

        do {
            var loaded = try await client.loadSnapshot()
            loaded.updatedAt = Date()
            snapshot = loaded
            lastErrorText = nil
        } catch {
            lastErrorText = String(describing: error)
        }
    }
}
