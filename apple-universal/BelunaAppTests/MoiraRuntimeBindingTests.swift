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
    }
#endif

}

private struct FailingMoiraRuntimeClient: MoiraRuntimeClient {
    func loadLoomSnapshot(selection: MoiraLoomSelection) async throws -> MoiraLoomSnapshot {
        throw MoiraRuntimeClientFixtureError.fixtureFailure
    }
}

private enum MoiraRuntimeClientFixtureError: Error {
    case fixtureFailure
}
