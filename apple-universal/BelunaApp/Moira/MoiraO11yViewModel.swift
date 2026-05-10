import Foundation

enum MoiraO11yDetailMode: String, CaseIterable, Identifiable {
    case gantt = "Gantt"
    case raw = "Raw"

    var id: String {
        rawValue
    }
}

@MainActor
final class MoiraO11yViewModel: ObservableObject {
    @Published private(set) var snapshot: MoiraLoomSnapshot
    @Published private(set) var isRefreshing = false
    @Published private(set) var errorText: String?
    @Published private(set) var selectedRawEventID: String?
    @Published private(set) var detailMode: MoiraO11yDetailMode = .gantt

    private let client: any MoiraRuntimeClient

    init(client: any MoiraRuntimeClient) {
        self.client = client
        self.snapshot = .unavailable(reason: "Moira O11y is waiting for first load.")
    }

    var runtimeStatusText: String {
        snapshot.status.lifecycle.rawValue
    }

    var receiverStatusText: String {
        snapshot.status.receiver.wakeState
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

    var selectedRunID: String? {
        snapshot.selectedRunID
    }

    var selectedTick: UInt64? {
        snapshot.selectedTick
    }

    var selectedTickID: String? {
        selectedTickSummary?.id
    }

    var selectedTickSummary: MoiraTickSummary? {
        if let detail = snapshot.tickDetail {
            return detail.summary
        }

        guard let selectedTick else {
            return nil
        }

        return snapshot.ticks.first { tick in
            tick.tick == selectedTick
        }
    }

    var rawEvents: [MoiraEventRecord] {
        snapshot.tickDetail?.raw ?? []
    }

    var ganttSnapshot: MoiraTickGanttSnapshot {
        MoiraTickGanttSnapshot.make(records: rawEvents)
    }

    var selectedRawEvent: MoiraEventRecord? {
        guard let selectedRawEventID else {
            return rawEvents.first
        }

        return rawEvents.first { record in
            record.rawEventID == selectedRawEventID
        } ?? rawEvents.first
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
        await refreshNow(selection: MoiraLoomSelection(runID: selectedRunID, tick: selectedTick))
    }

    func selectRun(id: String?) {
        Task {
            await selectRunNow(id: id)
        }
    }

    func selectRunNow(id: String?) async {
        selectedRawEventID = nil
        await refreshNow(selection: MoiraLoomSelection(runID: normalized(id), tick: nil))
    }

    func selectTick(id: String?) {
        Task {
            await selectTickNow(id: id)
        }
    }

    func selectTickNow(id: String?) async {
        selectedRawEventID = nil
        let tick = id.flatMap { selectedID in
            snapshot.ticks.first { summary in
                summary.id == selectedID
            }?.tick ?? UInt64(selectedID)
        }
        await refreshNow(selection: MoiraLoomSelection(runID: selectedRunID, tick: tick))
    }

    func selectRawEvent(id: String?) {
        selectedRawEventID = id
    }

    func selectDetailMode(_ mode: MoiraO11yDetailMode) {
        detailMode = mode
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
            syncRawEventSelection()
            errorText = nil
        } catch {
            errorText = String(describing: error)
        }
    }

    private func syncRawEventSelection() {
        if let selectedRawEventID,
           rawEvents.contains(where: { record in record.rawEventID == selectedRawEventID }) {
            return
        }

        selectedRawEventID = rawEvents.first?.rawEventID
    }

    private func normalized(_ value: String?) -> String? {
        let trimmed = value?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        return trimmed.isEmpty ? nil : trimmed
    }
}
