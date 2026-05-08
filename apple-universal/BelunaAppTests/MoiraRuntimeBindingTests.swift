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
    @Test func coreControlViewModelReportsMissingLaunchTarget() async {
        let viewModel = MoiraOperationsViewModel(
            client: StaticMoiraRuntimeClient(loomSnapshot: .unavailable(reason: "fixture"))
        )

        await viewModel.wakeCoreNow()

        #expect(viewModel.lastErrorText == "Select a launch target before waking Core.")
    }

    @MainActor
    @Test func moiraOperationsViewModelKeepsSnapshotAndReportsRefreshError() async {
        let viewModel = MoiraOperationsViewModel(client: FailingMoiraRuntimeClient())

        await viewModel.refreshNow()

        #expect(viewModel.runtimeStatusText == "unavailable")
        #expect(viewModel.lastErrorText?.contains("fixtureFailure") == true)
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
}

private enum MoiraRuntimeClientFixtureError: Error {
    case fixtureFailure
}

private final class RecordingMoiraRuntimeClient: MoiraRuntimeClient, @unchecked Sendable {
    private var snapshots: [MoiraLoomSnapshot]
    private let operationStatus: MoiraCoreStatus
    private(set) var wakeRequests: [MoiraCoreWakeRequest] = []
    private(set) var stopCount = 0
    private(set) var forceKillCount = 0

    init(snapshots: [MoiraLoomSnapshot], operationStatus: MoiraCoreStatus) {
        self.snapshots = snapshots
        self.operationStatus = operationStatus
    }

    func loadLoomSnapshot(selection: MoiraLoomSelection) async throws -> MoiraLoomSnapshot {
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
}

private extension MoiraLoomSnapshot {
    func withCoreStatus(_ coreStatus: MoiraCoreStatus) -> Self {
        var copy = self
        copy.status.core = coreStatus
        return copy
    }
}
