import Foundation

@MainActor
final class MoiraOperationsViewModel: ObservableObject {
    @Published private(set) var snapshot: MoiraLoomSnapshot
    @Published private(set) var isRefreshing = false
    @Published private(set) var isOperating = false
    @Published private(set) var isTrackingCoreTransition = false
    @Published private(set) var lastErrorText: String?
    @Published private(set) var runtimeErrorText: String?
    @Published private(set) var coreOperationErrorText: String?
    @Published private(set) var targetManagementErrorText: String?
    @Published private(set) var profileManagementErrorText: String?
    @Published private(set) var selectedLaunchTargetID = ""
    @Published private(set) var selectedProfileID = ""

    private let client: any MoiraRuntimeClient
    private let coreTransitionPollIntervalNanoseconds: UInt64
    private let coreTransitionPollAttempts: Int
    private var didInitializeLaunchTargetSelection = false
    private var didInitializeProfileSelection = false

    init(
        client: any MoiraRuntimeClient,
        coreTransitionPollIntervalNanoseconds: UInt64 = 500_000_000,
        coreTransitionPollAttempts: Int = 20
    ) {
        self.client = client
        self.coreTransitionPollIntervalNanoseconds = coreTransitionPollIntervalNanoseconds
        self.coreTransitionPollAttempts = coreTransitionPollAttempts
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
        !isRefreshing && !isOperating && !isTrackingCoreTransition
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

    var canOpenManagementEditor: Bool {
        !isRefreshing && !isOperating
    }

    var canEditSelectedLaunchTarget: Bool {
        canOpenManagementEditor && targetEditorDraftForSelectedLaunchTarget() != nil
    }

    var canEditSelectedProfile: Bool {
        canOpenManagementEditor && selectedProfile != nil
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

    func profileEditorDraftForCreate() -> MoiraProfileEditorDraft {
        MoiraProfileEditorDraft()
    }

    func targetEditorDraftForCreate() -> MoiraTargetEditorDraft {
        MoiraTargetEditorDraft()
    }

    func targetEditorDraftForSelectedLaunchTarget() -> MoiraTargetEditorDraft? {
        selectedLaunchTarget.flatMap { MoiraTargetEditorDraft(target: $0) }
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
            reportError(MoiraRuntimeClientError.missingLaunchTarget.description, on: .coreOperation)
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
        await performCoreOperation(pollStoppingCore: true) {
            try await client.stopCore()
        }
    }

    func forceKillCoreNow() async {
        await performCoreOperation(pollStoppingCore: true) {
            try await client.forceKillCore()
        }
    }

    func loadProfileEditorDraftNow(profileID rawProfileID: String) async -> MoiraProfileEditorDraft? {
        let profileID = rawProfileID.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !profileID.isEmpty else {
            reportError(MoiraRuntimeClientError.missingProfile.description, on: .profileManagement)
            return nil
        }

        guard !isOperating, !isRefreshing else {
            return nil
        }

        isOperating = true
        defer {
            isOperating = false
        }

        do {
            let draft = try await client.loadProfileDraft(profileID: profileID)
            clearError(on: .profileManagement)
            return MoiraProfileEditorDraft(document: draft)
        } catch {
            reportError(String(describing: error), on: .profileManagement)
            return nil
        }
    }

    func saveProfileEditorDraftNow(_ draft: MoiraProfileEditorDraft) async -> Bool {
        guard draft.isValid else {
            reportError(MoiraRuntimeClientError.invalidProfileDraft.description, on: .profileManagement)
            return false
        }

        return await performMoiraOperation(refreshAfter: true, errorSurface: .profileManagement) {
            let savedDraft = try await client.saveProfileDraft(request: draft.saveRequest)
            selectedProfileID = savedDraft.profileID
            didInitializeProfileSelection = true
        }
    }

    func saveTargetEditorDraftNow(_ draft: MoiraTargetEditorDraft) async -> Bool {
        guard draft.isValid else {
            reportError(MoiraRuntimeClientError.invalidKnownLocalBuildDraft.description, on: .targetManagement)
            return false
        }

        return await performMoiraOperation(refreshAfter: true, errorSurface: .targetManagement) {
            let targetRef = try await client.registerKnownLocalBuild(registration: draft.registration)
            selectedLaunchTargetID = targetRef.id
            didInitializeLaunchTargetSelection = true
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
            clearError(on: .runtime)
        } catch {
            reportError(String(describing: error), on: .runtime)
        }
    }

    private func performCoreOperation(
        pollStoppingCore: Bool = false,
        _ operation: () async throws -> MoiraCoreStatus
    ) async {
        guard !isOperating, !isRefreshing else {
            return
        }

        isOperating = true

        do {
            let coreStatus = try await operation()
            snapshot.status.core = coreStatus
            clearError(on: .coreOperation)
        } catch {
            reportError(String(describing: error), on: .coreOperation)
            isOperating = false
            return
        }

        isOperating = false

        if pollStoppingCore {
            await refreshCoreTransitionSnapshot()
            await pollStoppingCoreStatus()
        } else {
            await refreshNow()
        }
    }

    private func pollStoppingCoreStatus() async {
        guard coreTransitionPollAttempts > 0 else {
            return
        }

        isTrackingCoreTransition = true
        defer {
            isTrackingCoreTransition = false
        }

        for _ in 0..<coreTransitionPollAttempts {
            guard snapshot.status.core.isStoppingWithProcess else {
                return
            }

            if coreTransitionPollIntervalNanoseconds > 0 {
                try? await Task.sleep(nanoseconds: coreTransitionPollIntervalNanoseconds)
            }
            await refreshCoreTransitionSnapshot()
        }
    }

    private func refreshCoreTransitionSnapshot() async {
        do {
            var loaded = try await client.loadLoomSnapshot(
                selection: MoiraLoomSelection(runID: selectedRunID, tick: selectedTick)
            )
            loaded.updatedAt = Date()
            snapshot = loaded
            syncSelections()
            clearError(on: .runtime)
        } catch {
            reportError(String(describing: error), on: .runtime)
        }
    }

    @discardableResult
    private func performMoiraOperation(
        refreshAfter: Bool,
        errorSurface: MoiraErrorSurface,
        _ operation: () async throws -> Void
    ) async -> Bool {
        guard !isOperating, !isRefreshing else {
            return false
        }

        isOperating = true
        defer {
            isOperating = false
        }

        do {
            try await operation()
            clearError(on: errorSurface)
            if refreshAfter {
                await refreshNow()
            }
            return true
        } catch {
            reportError(String(describing: error), on: errorSurface)
            return false
        }
    }

    private func reportError(_ text: String, on surface: MoiraErrorSurface) {
        switch surface {
        case .runtime:
            runtimeErrorText = text
        case .coreOperation:
            coreOperationErrorText = text
        case .targetManagement:
            targetManagementErrorText = text
        case .profileManagement:
            profileManagementErrorText = text
        }
        lastErrorText = text
    }

    private func clearError(on surface: MoiraErrorSurface) {
        switch surface {
        case .runtime:
            runtimeErrorText = nil
        case .coreOperation:
            coreOperationErrorText = nil
        case .targetManagement:
            targetManagementErrorText = nil
        case .profileManagement:
            profileManagementErrorText = nil
        }
        lastErrorText = currentErrorText
    }

    private var currentErrorText: String? {
        coreOperationErrorText
            ?? profileManagementErrorText
            ?? targetManagementErrorText
            ?? runtimeErrorText
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

private enum MoiraErrorSurface {
    case runtime
    case coreOperation
    case targetManagement
    case profileManagement
}

private extension MoiraReceiverStatus {
    var canSupportWake: Bool {
        wakeState == "awakening" || wakeState == "listening" || wakeState == "awake"
    }
}

private extension MoiraCoreStatus {
    var isStoppingWithProcess: Bool {
        phase == "stopping" && pid != nil
    }

    var canWake: Bool {
        phase == "idle" || phase == "terminated" || phase == "unavailable"
    }

    var canStop: Bool {
        phase == "running" && pid != nil
    }

    var canForceKill: Bool {
        (phase == "running" || isStoppingWithProcess) && pid != nil
    }
}
