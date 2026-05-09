import Foundation
import Testing
@testable import BelunaApp

struct MoiraRuntimeBindingTests {

    @MainActor
    @Test func moiraOperationsViewModelLoadsRuntimeSnapshot() async {
        let snapshot = MoiraRuntimeSnapshot(
            lifecycle: .ready,
            resources: [
                MoiraResourceStatus(
                    kind: .otlpReceiver,
                    state: .claimed,
                    label: "Lachesis OTLP receiver",
                    detail: "127.0.0.1:4317"
                ),
            ],
            receiver: MoiraReceiverStatus(
                endpoint: "127.0.0.1:4317",
                wakeState: "listening",
                dbPath: "/tmp/moira.duckdb",
                lastBatchAt: nil,
                lastError: nil,
                rawEventCount: 3,
                wakeCount: 1,
                tickCount: 2
            ),
            core: MoiraCoreStatus(
                phase: "idle",
                targetLabel: nil,
                executablePath: nil,
                workingDir: nil,
                profilePath: nil,
                pid: nil,
                terminalReason: nil
            ),
            updatedAt: nil
        )
        let viewModel = MoiraOperationsViewModel(client: StaticMoiraRuntimeClient(snapshot: snapshot))

        await viewModel.refreshNow()

        #expect(viewModel.runtimeStatusText == "ready")
        #expect(viewModel.receiverStatusText == "listening")
        #expect(viewModel.eventCountText == "3")
        #expect(viewModel.wakeCountText == "1")
        #expect(viewModel.tickCountText == "2")
        #expect(viewModel.lastErrorText == nil)
        #expect(viewModel.snapshot.updatedAt != nil)
    }

    @MainActor
    @Test func moiraOperationsViewModelLoadsLoomSelection() async {
        let loom = MoiraRuntimeBindingFixtures.loomSnapshot()
        let viewModel = MoiraOperationsViewModel(
            client: StaticMoiraRuntimeClient(loomSnapshot: loom)
        )

        await viewModel.refreshNow()

        #expect(viewModel.selectedRunID == "run-1")
        #expect(viewModel.selectedTick == 3)
        #expect(viewModel.rawEvents.map(\.rawEventID) == ["evt-1"])
        #expect(viewModel.selectedLaunchTargetID == "knownLocalBuild:dev-core")
        #expect(viewModel.selectedProfileID == "default")
    }

    @MainActor
    @Test func moiraO11yViewModelLoadsSelectionAndRawInspectorState() async {
        let loom = MoiraRuntimeBindingFixtures.loomSnapshot()
        let viewModel = MoiraO11yViewModel(
            client: StaticMoiraRuntimeClient(loomSnapshot: loom)
        )

        await viewModel.refreshNow()

        #expect(viewModel.runtimeStatusText == "ready")
        #expect(viewModel.receiverStatusText == "listening")
        #expect(viewModel.selectedRunID == "run-1")
        #expect(viewModel.selectedTick == 3)
        #expect(viewModel.selectedTickID == "run-1:3")
        #expect(viewModel.rawEvents.map(\.rawEventID) == ["evt-1"])
        #expect(viewModel.selectedRawEventID == "evt-1")
        #expect(viewModel.selectedRawEvent?.displayTitle == "started")
        #expect(viewModel.errorText == nil)
    }

    @MainActor
    @Test func moiraO11yViewModelRefreshesSelectedWakeAndTick() async {
        let client = RecordingMoiraRuntimeClient(
            snapshots: [
                MoiraRuntimeBindingFixtures.loomSnapshot(),
                MoiraRuntimeBindingFixtures.loomSnapshot(),
                MoiraRuntimeBindingFixtures.loomSnapshot(),
            ],
            operationStatus: MoiraRuntimeBindingFixtures.loomSnapshot().status.core
        )
        let viewModel = MoiraO11yViewModel(client: client)

        await viewModel.refreshNow()
        await viewModel.selectRunNow(id: "run-1")
        await viewModel.selectTickNow(id: "run-1:3")

        #expect(client.loomSelections.map(\.runID) == [nil, "run-1", "run-1"])
        #expect(client.loomSelections.map(\.tick) == [nil, nil, 3])
        #expect(viewModel.selectedRawEventID == "evt-1")
    }

    @MainActor
    @Test func moiraO11yViewModelKeepsSnapshotAndReportsRefreshError() async {
        let viewModel = MoiraO11yViewModel(client: FailingMoiraRuntimeClient())

        await viewModel.refreshNow()

        #expect(viewModel.runtimeStatusText == "unavailable")
        #expect(viewModel.errorText?.contains("fixtureFailure") == true)
        #expect(viewModel.selectedRawEventID == nil)
    }

    @MainActor
    @Test func coreControlViewModelWakesSelectedLaunchTarget() async {
        let idleSnapshot = MoiraRuntimeBindingFixtures.loomSnapshot()
        let runningStatus = MoiraCoreStatus(
            phase: "running",
            targetLabel: "Dev Core",
            executablePath: "/tmp/beluna",
            workingDir: "/tmp",
            profilePath: "/tmp/default.jsonc",
            pid: 1234,
            terminalReason: nil
        )
        let client = RecordingMoiraRuntimeClient(
            snapshots: [
                idleSnapshot,
                idleSnapshot.withCoreStatus(runningStatus),
            ],
            operationStatus: runningStatus
        )
        let viewModel = MoiraOperationsViewModel(client: client)

        await viewModel.refreshNow()
        await viewModel.wakeCoreNow()

        #expect(client.wakeRequests.count == 1)
        #expect(client.wakeRequests.first?.target == idleSnapshot.launchTargets.first?.target)
        #expect(client.wakeRequests.first?.profile?.profileID == "default")
        #expect(viewModel.coreStatusText == "running")
        #expect(viewModel.snapshot.status.core.pid == 1234)
        #expect(viewModel.lastErrorText == nil)
    }

    @MainActor
    @Test func coreControlViewModelKeepsWakeAvailableDuringReceiverStartup() async {
        let snapshot = MoiraRuntimeBindingFixtures
            .loomSnapshot()
            .withReceiverWakeState("awakening")
        let viewModel = MoiraOperationsViewModel(
            client: StaticMoiraRuntimeClient(loomSnapshot: snapshot)
        )

        await viewModel.refreshNow()

        #expect(viewModel.receiverStatusText == "awakening")
        #expect(viewModel.selectedLaunchTargetID == "knownLocalBuild:dev-core")
        #expect(viewModel.selectedProfileID == "default")
        #expect(viewModel.canWakeCore)
    }

    @MainActor
    @Test func coreControlViewModelRoutesStopAndForceKill() async {
        let runningStatus = MoiraCoreStatus(
            phase: "running",
            targetLabel: "Dev Core",
            executablePath: "/tmp/beluna",
            workingDir: "/tmp",
            profilePath: nil,
            pid: 99,
            terminalReason: nil
        )
        let stoppingStatus = MoiraCoreStatus(
            phase: "stopping",
            targetLabel: "Dev Core",
            executablePath: "/tmp/beluna",
            workingDir: "/tmp",
            profilePath: nil,
            pid: 99,
            terminalReason: nil
        )
        let runningSnapshot = MoiraRuntimeBindingFixtures
            .loomSnapshot()
            .withCoreStatus(runningStatus)
        let client = RecordingMoiraRuntimeClient(
            snapshots: [runningSnapshot, runningSnapshot, runningSnapshot],
            operationStatus: stoppingStatus
        )
        let viewModel = MoiraOperationsViewModel(client: client)

        await viewModel.refreshNow()
        await viewModel.stopCoreNow()
        await viewModel.forceKillCoreNow()

        #expect(client.stopCount == 1)
        #expect(client.forceKillCount == 1)
        #expect(viewModel.lastErrorText == nil)
    }

    @MainActor
    @Test func coreControlViewModelPollsStoppingCoreUntilTerminated() async {
        let runningStatus = MoiraCoreStatus(
            phase: "running",
            targetLabel: "Dev Core",
            executablePath: "/tmp/beluna",
            workingDir: "/tmp",
            profilePath: nil,
            pid: 99,
            terminalReason: nil
        )
        let stoppingStatus = MoiraCoreStatus(
            phase: "stopping",
            targetLabel: "Dev Core",
            executablePath: "/tmp/beluna",
            workingDir: "/tmp",
            profilePath: nil,
            pid: 99,
            terminalReason: nil
        )
        let terminatedStatus = MoiraCoreStatus(
            phase: "terminated",
            targetLabel: "Dev Core",
            executablePath: "/tmp/beluna",
            workingDir: "/tmp",
            profilePath: nil,
            pid: nil,
            terminalReason: "graceful_stop(signal=15)"
        )
        let client = RecordingMoiraRuntimeClient(
            snapshots: [
                MoiraRuntimeBindingFixtures.loomSnapshot().withCoreStatus(runningStatus),
                MoiraRuntimeBindingFixtures.loomSnapshot().withCoreStatus(stoppingStatus),
                MoiraRuntimeBindingFixtures.loomSnapshot().withCoreStatus(terminatedStatus),
            ],
            operationStatus: stoppingStatus
        )
        let viewModel = MoiraOperationsViewModel(
            client: client,
            coreTransitionPollIntervalNanoseconds: 0,
            coreTransitionPollAttempts: 4
        )

        await viewModel.refreshNow()
        await viewModel.stopCoreNow()

        #expect(client.stopCount == 1)
        #expect(viewModel.coreStatusText == "terminated")
        #expect(viewModel.snapshot.status.core.pid == nil)
        #expect(viewModel.snapshot.status.core.terminalReason == "graceful_stop(signal=15)")
        #expect(viewModel.lastErrorText == nil)
    }

    @MainActor
    @Test func coreControlViewModelKeepsForceKillAvailableWhenGracefulStopKeepsStopping() async {
        let runningStatus = MoiraCoreStatus(
            phase: "running",
            targetLabel: "Dev Core",
            executablePath: "/tmp/beluna",
            workingDir: "/tmp",
            profilePath: nil,
            pid: 99,
            terminalReason: nil
        )
        let stoppingStatus = MoiraCoreStatus(
            phase: "stopping",
            targetLabel: "Dev Core",
            executablePath: "/tmp/beluna",
            workingDir: "/tmp",
            profilePath: nil,
            pid: 99,
            terminalReason: nil
        )
        let client = RecordingMoiraRuntimeClient(
            snapshots: [
                MoiraRuntimeBindingFixtures.loomSnapshot().withCoreStatus(runningStatus),
                MoiraRuntimeBindingFixtures.loomSnapshot().withCoreStatus(stoppingStatus),
                MoiraRuntimeBindingFixtures.loomSnapshot().withCoreStatus(stoppingStatus),
                MoiraRuntimeBindingFixtures.loomSnapshot().withCoreStatus(stoppingStatus),
            ],
            operationStatus: stoppingStatus
        )
        let viewModel = MoiraOperationsViewModel(
            client: client,
            coreTransitionPollIntervalNanoseconds: 0,
            coreTransitionPollAttempts: 2
        )

        await viewModel.refreshNow()
        await viewModel.stopCoreNow()

        #expect(viewModel.coreStatusText == "stopping")
        #expect(viewModel.canForceKillCore)
        #expect(!viewModel.canStopCore)
        #expect(!viewModel.isRefreshing)
        #expect(!viewModel.isTrackingCoreTransition)
    }

    @MainActor
    @Test func coreControlViewModelReportsMissingLaunchTarget() async {
        let viewModel = MoiraOperationsViewModel(
            client: StaticMoiraRuntimeClient(loomSnapshot: .unavailable(reason: "fixture"))
        )

        await viewModel.wakeCoreNow()

        #expect(viewModel.lastErrorText == "Select a launch target before waking Core.")
        #expect(viewModel.coreOperationErrorText == "Select a launch target before waking Core.")
        #expect(viewModel.runtimeErrorText == nil)
        #expect(viewModel.profileManagementErrorText == nil)
    }

    @MainActor
    @Test func clothoManagementLoadsAndSavesProfileDraft() async {
        let client = RecordingMoiraRuntimeClient(
            snapshots: [
                MoiraRuntimeBindingFixtures.loomSnapshot(),
                MoiraRuntimeBindingFixtures.loomSnapshot().withProfile(
                    MoiraProfileDocumentSummary(
                        profileID: "custom",
                        profilePath: "/tmp/custom.jsonc"
                    )
                ),
            ],
            operationStatus: MoiraRuntimeBindingFixtures.loomSnapshot().status.core
        )
        let viewModel = MoiraOperationsViewModel(client: client)

        await viewModel.refreshNow()
        var draft = await viewModel.loadProfileEditorDraftNow(profileID: "custom")
            ?? MoiraProfileEditorDraft()
        draft.coreConfig = "{\n  \"logging\": { \"dir\": \"./logs\" }\n}\n"
        draft.envFiles.append(MoiraProfileEnvFileDraft(path: "./local.env", required: false))
        draft.inlineEnvironment.append(MoiraProfileInlineEnvironmentDraft(
            key: "OPENAI_API_KEY",
            value: "inline-openai"
        ))
        let didSave = await viewModel.saveProfileEditorDraftNow(draft)

        #expect(didSave)
        #expect(client.loadedProfileIDs == ["custom"])
        #expect(viewModel.selectedProfileID == "custom")
        #expect(draft.loadedProfilePath == "/tmp/custom.jsonc")
        #expect(client.savedProfileDrafts.count == 1)
        #expect(client.savedProfileDrafts.first?.profileID == "custom")
        #expect(client.savedProfileDrafts.first?.envFiles.first?.path == "./local.env")
        #expect(client.savedProfileDrafts.first?.envFiles.first?.required == false)
        #expect(client.savedProfileDrafts.first?.inlineEnvironment.first?.key == "OPENAI_API_KEY")
        #expect(viewModel.lastErrorText == nil)
        #expect(viewModel.profileManagementErrorText == nil)
    }

    @MainActor
    @Test func clothoManagementRegistersKnownLocalBuildAndSelectsTarget() async {
        let client = RecordingMoiraRuntimeClient(
            snapshots: [
                MoiraRuntimeBindingFixtures.loomSnapshot(),
                MoiraRuntimeBindingFixtures.loomSnapshot(),
            ],
            operationStatus: MoiraRuntimeBindingFixtures.loomSnapshot().status.core
        )
        let viewModel = MoiraOperationsViewModel(client: client)

        await viewModel.refreshNow()
        var draft = viewModel.targetEditorDraftForSelectedLaunchTarget()

        #expect(draft?.buildID == "dev-core")
        #expect(draft?.executablePath == "/tmp/beluna")
        #expect(draft?.workingDir == "/tmp")

        draft?.executablePath = "/tmp/beluna-updated"
        draft?.workingDir = "/tmp"
        let didSave = await viewModel.saveTargetEditorDraftNow(draft ?? MoiraTargetEditorDraft())

        #expect(didSave)
        #expect(client.registrations.count == 1)
        #expect(client.registrations.first?.buildID == "dev-core")
        #expect(client.registrations.first?.executablePath == "/tmp/beluna-updated")
        #expect(client.registrations.first?.workingDir == "/tmp")
        #expect(viewModel.selectedLaunchTargetID == "knownLocalBuild:dev-core")
        #expect(viewModel.lastErrorText == nil)
        #expect(viewModel.targetManagementErrorText == nil)
    }

    @MainActor
    @Test func moiraOperationsViewModelKeepsSnapshotAndReportsRefreshError() async {
        let viewModel = MoiraOperationsViewModel(client: FailingMoiraRuntimeClient())

        await viewModel.refreshNow()

        #expect(viewModel.runtimeStatusText == "unavailable")
        #expect(viewModel.lastErrorText?.contains("fixtureFailure") == true)
        #expect(viewModel.runtimeErrorText?.contains("fixtureFailure") == true)
        #expect(viewModel.profileManagementErrorText == nil)
    }

    @MainActor
    @Test func profileManagementReportsLoadErrorsInProfileScope() async {
        let viewModel = MoiraOperationsViewModel(client: FailingMoiraRuntimeClient())

        let draft = await viewModel.loadProfileEditorDraftNow(profileID: "dev")

        #expect(draft == nil)
        #expect(viewModel.lastErrorText?.contains("fixtureFailure") == true)
        #expect(viewModel.profileManagementErrorText?.contains("fixtureFailure") == true)
        #expect(viewModel.runtimeErrorText == nil)
        #expect(viewModel.coreOperationErrorText == nil)
    }

    @Test func decodesMoiraRuntimeStatusJSON() throws {
        let json = """
        {
          "lifecycle": "ready",
          "resources": [
            {
              "kind": "telemetryStore",
              "state": "claimed",
              "label": "Lachesis telemetry store",
              "detail": "/tmp/moira.duckdb"
            },
            {
              "kind": "otlpReceiver",
              "state": "conflict",
              "label": "Lachesis OTLP receiver",
              "detail": "Address already in use"
            }
          ],
          "receiver": {
            "endpoint": "127.0.0.1:4317",
            "wakeState": "faulted",
            "dbPath": "/tmp/moira.duckdb",
            "lastBatchAt": null,
            "lastError": "Address already in use",
            "rawEventCount": 7,
            "wakeCount": 2,
            "tickCount": 3
          },
          "core": {
            "phase": "idle"
          }
        }
        """

        let snapshot = try JSONDecoder().decode(
            MoiraRuntimeSnapshot.self,
            from: Data(json.utf8)
        )

        #expect(snapshot.lifecycle == .ready)
        #expect(snapshot.receiver.rawEventCount == 7)
        #expect(snapshot.core.phase == "idle")
        #expect(snapshot.attentionResources.map(\.state) == [.conflict])
    }

    @Test func decodesMoiraLoomSnapshotJSON() throws {
        let snapshot = try JSONDecoder().decode(
            MoiraLoomSnapshot.self,
            from: Data(MoiraRuntimeBindingFixtures.loomJSON.utf8)
        )

        #expect(snapshot.status.lifecycle == .ready)
        #expect(snapshot.launchTargets.first?.label == "Dev Core")
        #expect(snapshot.profiles.first?.profileID == "default")
        #expect(snapshot.runs.first?.runID == "run-1")
        #expect(snapshot.selectedRunID == "run-1")
        #expect(snapshot.selectedTick == 3)
        #expect(snapshot.tickDetail?.raw.first?.eventName == "started")
    }

    @Test func formatsMoiraJSONForRawInspector() {
        let value = JSONValue.object([
            "z": .number(3),
            "a": .object([
                "message": .string("hello"),
            ]),
        ])

        let pretty = MoiraJSONFormatter.prettyString(value)
        let compact = MoiraJSONFormatter.compactString(value)

        #expect(pretty.contains("\"a\""))
        #expect(pretty.contains("\"message\" : \"hello\""))
        #expect(compact == "{a: {message: hello}, z: 3.0}")
    }

    @Test func encodesMoiraCoreWakeRequestJSON() throws {
        let request = MoiraCoreWakeRequest(
            target: MoiraLaunchTargetRef(
                kind: "knownLocalBuild",
                buildID: "dev-core",
                releaseTag: nil,
                rustTargetTriple: nil
            ),
            profile: MoiraProfileRef(profileID: "default")
        )

        let data = try JSONEncoder().encode(request)
        let object = try JSONSerialization.jsonObject(with: data) as? [String: Any]
        let target = object?["target"] as? [String: Any]
        let profile = object?["profile"] as? [String: Any]

        #expect(target?["kind"] as? String == "knownLocalBuild")
        #expect(target?["buildId"] as? String == "dev-core")
        #expect(profile?["profileId"] as? String == "default")
    }

#if os(macOS)
    @Test func dynamicClientLoadsBundledMoiraFFI() throws {
        let fileManager = FileManager.default
        let frameworksURL = Bundle.main.privateFrameworksURL

        #expect(frameworksURL != nil)
        guard let frameworksURL else {
            return
        }

        #expect(fileManager.fileExists(
            atPath: frameworksURL.appendingPathComponent("libmoira_ffi.dylib").path
        ))
        #expect(fileManager.fileExists(
            atPath: frameworksURL.appendingPathComponent("libduckdb.dylib").path
        ))

        let rootURL = fileManager.temporaryDirectory
            .appendingPathComponent("BelunaMoiraRuntimeBinding-\(UUID().uuidString)", isDirectory: true)
        try fileManager.createDirectory(at: rootURL, withIntermediateDirectories: true)
        defer {
            try? fileManager.removeItem(at: rootURL)
        }

        let library = try MoiraRuntimeDynamicLibrary.loadBundled()
        defer {
            _ = try? library.shutdownResources()
        }

        let snapshot = try library.loomSnapshot(
            configuration: MoiraRuntimeConfiguration(
                rootDirectoryPath: rootURL.path,
                receiverBind: "127.0.0.1:0"
            ),
            selection: .none
        )

        #expect(snapshot.status.lifecycle == .ready)
        #expect(snapshot.status.resources.contains { $0.kind == .otlpReceiver })
        #expect(snapshot.status.receiver.dbPath.hasPrefix(rootURL.path))
        #expect(snapshot.runs.isEmpty)
        #expect(snapshot.tickDetail == nil)

        var stopError: String?
        do {
            _ = try library.stopCore(
                configuration: MoiraRuntimeConfiguration(
                    rootDirectoryPath: rootURL.path,
                    receiverBind: "127.0.0.1:0"
                )
            )
        } catch {
            stopError = String(describing: error)
        }
        #expect(stopError?.contains("no supervised Core") == true)
    }
#endif

}

private struct FailingMoiraRuntimeClient: MoiraRuntimeClient {
    func loadLoomSnapshot(selection: MoiraLoomSelection) async throws -> MoiraLoomSnapshot {
        throw MoiraRuntimeClientFixtureError.fixtureFailure
    }

    func wakeCore(request: MoiraCoreWakeRequest) async throws -> MoiraCoreStatus {
        throw MoiraRuntimeClientFixtureError.fixtureFailure
    }

    func stopCore() async throws -> MoiraCoreStatus {
        throw MoiraRuntimeClientFixtureError.fixtureFailure
    }

    func forceKillCore() async throws -> MoiraCoreStatus {
        throw MoiraRuntimeClientFixtureError.fixtureFailure
    }

    func loadProfileDocument(profileID: String) async throws -> MoiraProfileDocument {
        throw MoiraRuntimeClientFixtureError.fixtureFailure
    }

    func saveProfileDocument(request: MoiraProfileSaveRequest) async throws -> MoiraProfileDocument {
        throw MoiraRuntimeClientFixtureError.fixtureFailure
    }

    func loadProfileDraft(profileID: String) async throws -> MoiraProfileDraftDocument {
        throw MoiraRuntimeClientFixtureError.fixtureFailure
    }

    func saveProfileDraft(
        request: MoiraProfileDraftSaveRequest
    ) async throws -> MoiraProfileDraftDocument {
        throw MoiraRuntimeClientFixtureError.fixtureFailure
    }

    func registerKnownLocalBuild(
        registration: MoiraKnownLocalBuildRegistration
    ) async throws -> MoiraLaunchTargetRef {
        throw MoiraRuntimeClientFixtureError.fixtureFailure
    }
}

private enum MoiraRuntimeClientFixtureError: Error {
    case fixtureFailure
}

private final class RecordingMoiraRuntimeClient: MoiraRuntimeClient, @unchecked Sendable {
    private var snapshots: [MoiraLoomSnapshot]
    private let operationStatus: MoiraCoreStatus
    private let profileDocument: MoiraProfileDocument?
    private let profileDraft: MoiraProfileDraftDocument?
    private(set) var wakeRequests: [MoiraCoreWakeRequest] = []
    private(set) var loadedProfileIDs: [String] = []
    private(set) var savedProfiles: [MoiraProfileSaveRequest] = []
    private(set) var savedProfileDrafts: [MoiraProfileDraftSaveRequest] = []
    private(set) var registrations: [MoiraKnownLocalBuildRegistration] = []
    private(set) var loomSelections: [MoiraLoomSelection] = []
    private(set) var stopCount = 0
    private(set) var forceKillCount = 0

    init(
        snapshots: [MoiraLoomSnapshot],
        operationStatus: MoiraCoreStatus,
        profileDocument: MoiraProfileDocument? = nil,
        profileDraft: MoiraProfileDraftDocument? = nil
    ) {
        self.snapshots = snapshots
        self.operationStatus = operationStatus
        self.profileDocument = profileDocument
        self.profileDraft = profileDraft
    }

    func loadLoomSnapshot(selection: MoiraLoomSelection) async throws -> MoiraLoomSnapshot {
        loomSelections.append(selection)
        guard !snapshots.isEmpty else {
            return MoiraRuntimeBindingFixtures.loomSnapshot()
        }
        return snapshots.removeFirst()
    }

    func wakeCore(request: MoiraCoreWakeRequest) async throws -> MoiraCoreStatus {
        wakeRequests.append(request)
        return operationStatus
    }

    func stopCore() async throws -> MoiraCoreStatus {
        stopCount += 1
        return operationStatus
    }

    func forceKillCore() async throws -> MoiraCoreStatus {
        forceKillCount += 1
        return operationStatus
    }

    func loadProfileDocument(profileID: String) async throws -> MoiraProfileDocument {
        loadedProfileIDs.append(profileID)
        return profileDocument ?? MoiraProfileDocument(
            profileID: profileID,
            profilePath: "/tmp/\(profileID).jsonc",
            contents: ""
        )
    }

    func saveProfileDocument(request: MoiraProfileSaveRequest) async throws -> MoiraProfileDocument {
        savedProfiles.append(request)
        return MoiraProfileDocument(
            profileID: request.profileID,
            profilePath: "/tmp/\(request.profileID).jsonc",
            contents: request.contents
        )
    }

    func loadProfileDraft(profileID: String) async throws -> MoiraProfileDraftDocument {
        loadedProfileIDs.append(profileID)
        return profileDraft ?? MoiraProfileDraftDocument(
            profileID: profileID,
            profilePath: "/tmp/\(profileID).jsonc",
            coreConfig: "{\n}\n",
            envFiles: [],
            inlineEnvironment: []
        )
    }

    func saveProfileDraft(
        request: MoiraProfileDraftSaveRequest
    ) async throws -> MoiraProfileDraftDocument {
        savedProfileDrafts.append(request)
        return MoiraProfileDraftDocument(
            profileID: request.profileID,
            profilePath: "/tmp/\(request.profileID).jsonc",
            coreConfig: request.coreConfig,
            envFiles: request.envFiles,
            inlineEnvironment: request.inlineEnvironment
        )
    }

    func registerKnownLocalBuild(
        registration: MoiraKnownLocalBuildRegistration
    ) async throws -> MoiraLaunchTargetRef {
        registrations.append(registration)
        return MoiraLaunchTargetRef(
            kind: "knownLocalBuild",
            buildID: registration.buildID,
            releaseTag: nil,
            rustTargetTriple: nil
        )
    }
}

private extension MoiraLoomSnapshot {
    func withCoreStatus(_ coreStatus: MoiraCoreStatus) -> Self {
        var copy = self
        copy.status.core = coreStatus
        return copy
    }

    func withReceiverWakeState(_ wakeState: String) -> Self {
        var copy = self
        copy.status.receiver.wakeState = wakeState
        return copy
    }

    func withProfile(_ profile: MoiraProfileDocumentSummary) -> Self {
        var copy = self
        copy.profiles.append(profile)
        return copy
    }
}
