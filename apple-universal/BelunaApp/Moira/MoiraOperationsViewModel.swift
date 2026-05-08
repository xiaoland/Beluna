import Foundation

@MainActor
final class MoiraOperationsViewModel: ObservableObject {
    @Published private(set) var snapshot: MoiraLoomSnapshot
    @Published private(set) var isRefreshing = false
    @Published private(set) var lastErrorText: String?
    @Published private(set) var selectedLaunchTargetID = ""
    @Published private(set) var selectedProfileID = ""

    private let client: any MoiraRuntimeClient

    init(client: any MoiraRuntimeClient) {
        self.client = client
        self.snapshot = .unavailable(reason: "Moira runtime status is waiting for first load.")
    }

    var runtimeStatusText: String {
        snapshot.status.lifecycle.rawValue
    }

    var receiverStatusText: String {
        snapshot.status.receiver.wakeState
    }

    var coreStatusText: String {
        snapshot.status.core.phase
    }

    var eventCountText: String {
        "\(snapshot.status.receiver.rawEventCount)"
    }

    var wakeCountText: String {
        "\(snapshot.status.receiver.wakeCount)"
    }

    var tickCountText: String {
        "\(snapshot.status.receiver.tickCount)"
    }

    var canRefresh: Bool {
        !isRefreshing
    }

    var selectedRunID: String? {
        snapshot.selectedRunID
    }

    var selectedTick: UInt64? {
        snapshot.selectedTick
    }

    var selectedRunBindingValue: String {
        snapshot.selectedRunID ?? ""
    }

    var selectedTickBindingValue: String {
        snapshot.selectedTick.map(String.init) ?? ""
    }

    var rawEvents: [MoiraEventRecord] {
        snapshot.tickDetail?.raw ?? []
    }

    var hasLaunchTargets: Bool {
        !snapshot.launchTargets.isEmpty
    }

    var hasProfiles: Bool {
        !snapshot.profiles.isEmpty
    }

    var hasRuns: Bool {
        !snapshot.runs.isEmpty
    }

    var hasTicks: Bool {
        !snapshot.ticks.isEmpty
    }

    func refresh() {
        Task {
            await refreshNow()
        }
    }

    func selectLaunchTarget(id: String) {
        selectedLaunchTargetID = id
    }

    func selectProfile(id: String) {
        selectedProfileID = id
    }

    func selectRun(id: String) {
        let selected = id.isEmpty ? nil : id
        Task {
            await refreshNow(selection: MoiraLoomSelection(runID: selected, tick: nil))
        }
    }

    func selectTick(value: String) {
        let selectedTick = UInt64(value)
        Task {
            await refreshNow(selection: MoiraLoomSelection(runID: selectedRunID, tick: selectedTick))
        }
    }

    func refreshNow() async {
        await refreshNow(selection: MoiraLoomSelection(runID: selectedRunID, tick: selectedTick))
    }

    private func refreshNow(selection: MoiraLoomSelection) async {
        guard !isRefreshing else {
            return
        }

        isRefreshing = true
        defer {
            isRefreshing = false
        }

        do {
            var loaded = try await client.loadLoomSnapshot(selection: selection)
            loaded.updatedAt = Date()
            snapshot = loaded
            syncSelections()
            lastErrorText = nil
        } catch {
            lastErrorText = String(describing: error)
        }
    }

    private func syncSelections() {
        if selectedLaunchTargetID.isEmpty, let firstTarget = snapshot.launchTargets.first {
            selectedLaunchTargetID = firstTarget.id
        } else if !snapshot.launchTargets.contains(where: { $0.id == selectedLaunchTargetID }) {
            selectedLaunchTargetID = snapshot.launchTargets.first?.id ?? ""
        }

        if selectedProfileID.isEmpty, let firstProfile = snapshot.profiles.first {
            selectedProfileID = firstProfile.id
        } else if !snapshot.profiles.contains(where: { $0.id == selectedProfileID }) {
            selectedProfileID = snapshot.profiles.first?.id ?? ""
        }
    }
}
