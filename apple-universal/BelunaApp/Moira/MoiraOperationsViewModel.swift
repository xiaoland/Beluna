import Foundation

@MainActor
final class MoiraOperationsViewModel: ObservableObject {
    @Published private(set) var snapshot: MoiraLoomSnapshot
    @Published private(set) var isRefreshing = false
    @Published private(set) var isOperating = false
    @Published private(set) var lastErrorText: String?
    @Published private(set) var selectedLaunchTargetID = ""
    @Published private(set) var selectedProfileID = ""

    private let client: any MoiraRuntimeClient
    private var didInitializeLaunchTargetSelection = false
    private var didInitializeProfileSelection = false

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
        !isRefreshing && !isOperating
    }

    var canWakeCore: Bool {
        guard selectedLaunchTarget != nil else {
            return false
        }

        return !isRefreshing
            && !isOperating
            && snapshot.status.receiver.canSupportWake
            && snapshot.status.core.canWake
    }

    var canStopCore: Bool {
        !isRefreshing && !isOperating && snapshot.status.core.canStop
    }

    var canForceKillCore: Bool {
        !isRefreshing && !isOperating && snapshot.status.core.canForceKill
    }

    var selectedLaunchTarget: MoiraLaunchTargetSummary? {
        snapshot.launchTargets.first { $0.id == selectedLaunchTargetID }
    }

    var selectedProfile: MoiraProfileDocumentSummary? {
        snapshot.profiles.first { $0.id == selectedProfileID }
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
        didInitializeLaunchTargetSelection = true
        selectedLaunchTargetID = id
    }

    func selectProfile(id: String) {
        didInitializeProfileSelection = true
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

    func wakeCore() {
        Task {
            await wakeCoreNow()
        }
    }

    func stopCore() {
        Task {
            await stopCoreNow()
        }
    }

    func forceKillCore() {
        Task {
            await forceKillCoreNow()
        }
    }

    func wakeCoreNow() async {
        guard let target = selectedLaunchTarget else {
            lastErrorText = MoiraRuntimeClientError.missingLaunchTarget.description
            return
        }

        let request = MoiraCoreWakeRequest(
            target: target.target,
            profile: selectedProfile.map { MoiraProfileRef(profileID: $0.profileID) }
        )

        await performCoreOperation {
            try await client.wakeCore(request: request)
        }
    }

    func stopCoreNow() async {
        await performCoreOperation {
            try await client.stopCore()
        }
    }

    func forceKillCoreNow() async {
        await performCoreOperation {
            try await client.forceKillCore()
        }
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

    private func performCoreOperation(_ operation: () async throws -> MoiraCoreStatus) async {
        guard !isOperating, !isRefreshing else {
            return
        }

        isOperating = true
        defer {
            isOperating = false
        }

        do {
            let coreStatus = try await operation()
            snapshot.status.core = coreStatus
            lastErrorText = nil
            await refreshNow()
        } catch {
            lastErrorText = String(describing: error)
        }
    }

    private func syncSelections() {
        if selectedLaunchTargetID.isEmpty,
           !didInitializeLaunchTargetSelection,
           let firstTarget = snapshot.launchTargets.first {
            selectedLaunchTargetID = firstTarget.id
            didInitializeLaunchTargetSelection = true
        } else if !snapshot.launchTargets.contains(where: { $0.id == selectedLaunchTargetID }) {
            selectedLaunchTargetID = snapshot.launchTargets.first?.id ?? ""
        }

        if selectedProfileID.isEmpty,
           !didInitializeProfileSelection,
           let firstProfile = snapshot.profiles.first {
            selectedProfileID = firstProfile.id
            didInitializeProfileSelection = true
        } else if !selectedProfileID.isEmpty,
                  !snapshot.profiles.contains(where: { $0.id == selectedProfileID }) {
            selectedProfileID = ""
        }
    }
}

private extension MoiraReceiverStatus {
    var canSupportWake: Bool {
        wakeState == "listening" || wakeState == "awake"
    }
}

private extension MoiraCoreStatus {
    var canWake: Bool {
        phase == "idle" || phase == "terminated" || phase == "unavailable"
    }

    var canStop: Bool {
        phase == "running" && pid != nil
    }

    var canForceKill: Bool {
        (phase == "running" || phase == "stopping") && pid != nil
    }
}
