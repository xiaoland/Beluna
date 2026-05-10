import SwiftUI

struct MoiraO11yPanel: View {
    @ObservedObject var viewModel: MoiraO11yViewModel

    var body: some View {
        NavigationSplitView {
            wakeSidebar
        } content: {
            tickSidebar
        } detail: {
            tickDetailPane
        }
        .navigationTitle("O11y / Lachesis")
        .toolbar {
            ToolbarItemGroup {
                if viewModel.isRefreshing {
                    ProgressView()
                        .controlSize(.small)
                }

                Button(action: viewModel.refresh) {
                    Label("Refresh", systemImage: "arrow.clockwise")
                }
                .disabled(!viewModel.canRefresh)
            }
        }
        .frame(minWidth: 920, minHeight: 620)
        .task {
            await viewModel.refreshNow()
        }
    }

    private var wakeSidebar: some View {
        VStack(spacing: 0) {
            o11yStatusHeader
            Divider()

            if viewModel.snapshot.runs.isEmpty {
                MoiraO11yEmptyPane(
                    title: "Wake List Empty",
                    systemImage: "tray",
                    detail: "Refresh after Core emits OTLP logs."
                )
            } else {
                List(selection: runSelection) {
                    ForEach(viewModel.snapshot.runs) { run in
                        MoiraWakeRow(run: run)
                            .tag(run.id as String?)
                    }
                }
                .listStyle(.sidebar)
            }
        }
        .navigationSplitViewColumnWidth(min: 240, ideal: 280)
    }

    private var tickSidebar: some View {
        Group {
            if viewModel.snapshot.ticks.isEmpty {
                MoiraO11yEmptyPane(
                    title: "Tick List Empty",
                    systemImage: "list.bullet.rectangle",
                    detail: "Select a wake with tick records."
                )
            } else {
                List(selection: tickSelection) {
                    ForEach(viewModel.snapshot.ticks) { tick in
                        MoiraTickRow(tick: tick)
                            .tag(tick.id as String?)
                    }
                }
                .listStyle(.inset)
            }
        }
        .navigationSplitViewColumnWidth(min: 260, ideal: 320)
    }

    private var tickDetailPane: some View {
        Group {
            if let tickDetail = viewModel.snapshot.tickDetail {
                VStack(spacing: 0) {
                    tickHeader(tickDetail.summary)
                    Divider()
                    detailModePicker
                    Divider()
                    selectedTickDetail
                }
            } else {
                MoiraO11yEmptyPane(
                    title: "Selected Tick Empty",
                    systemImage: "waveform.path.ecg.rectangle",
                    detail: "Select a wake and tick to inspect source records."
                )
            }
        }
    }

    private var o11yStatusHeader: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(alignment: .firstTextBaseline, spacing: 12) {
                Text("Lachesis")
                    .font(.headline)

                Spacer()

                Text(viewModel.receiverStatusText)
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)
            }

            LazyVGrid(
                columns: [
                    GridItem(.flexible(minimum: 72), alignment: .leading),
                    GridItem(.flexible(minimum: 72), alignment: .leading),
                ],
                alignment: .leading,
                spacing: 8
            ) {
                MoiraO11yMetricLabel(title: "Runtime", value: viewModel.runtimeStatusText)
                MoiraO11yMetricLabel(title: "Events", value: viewModel.eventCountText)
                MoiraO11yMetricLabel(title: "Wakes", value: viewModel.wakeCountText)
                MoiraO11yMetricLabel(title: "Ticks", value: viewModel.tickCountText)
            }

            if let errorText = viewModel.errorText {
                Text(errorText)
                    .font(.caption)
                    .foregroundStyle(.red)
                    .textSelection(.enabled)
            }
        }
        .padding(14)
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    private var detailModePicker: some View {
        HStack {
            Picker("Tick Detail", selection: detailModeSelection) {
                ForEach(MoiraO11yDetailMode.allCases) { mode in
                    Text(mode.rawValue).tag(mode)
                }
            }
            .pickerStyle(.segmented)
            .frame(width: 180)

            Spacer()
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 8)
    }

    private var selectedTickDetail: some View {
        Group {
            switch viewModel.detailMode {
            case .gantt:
                MoiraTickGanttView(
                    snapshot: viewModel.ganttSnapshot,
                    selectedRawEventID: rawEventSelection
                )
            case .raw:
                rawEventSplit
            }
        }
    }

    private var rawEventSplit: some View {
        HStack(spacing: 0) {
            if viewModel.rawEvents.isEmpty {
                MoiraO11yEmptyPane(
                    title: "Raw Records Empty",
                    systemImage: "doc.text",
                    detail: "The selected tick has no stored raw records."
                )
                .frame(minWidth: 320, idealWidth: 360, maxWidth: 420)
            } else {
                List(selection: rawEventSelection) {
                    ForEach(viewModel.rawEvents) { record in
                        MoiraRawEventRow(record: record)
                            .tag(record.id as String?)
                    }
                }
                .listStyle(.inset)
                .frame(minWidth: 320, idealWidth: 360, maxWidth: 440)
            }

            Divider()

            MoiraRawEventInspector(record: viewModel.selectedRawEvent)
                .frame(minWidth: 380, maxWidth: .infinity, maxHeight: .infinity)
        }
    }

    private var runSelection: Binding<String?> {
        Binding(
            get: { viewModel.selectedRunID },
            set: { viewModel.selectRun(id: $0) }
        )
    }

    private var tickSelection: Binding<String?> {
        Binding(
            get: { viewModel.selectedTickID },
            set: { viewModel.selectTick(id: $0) }
        )
    }

    private var detailModeSelection: Binding<MoiraO11yDetailMode> {
        Binding(
            get: { viewModel.detailMode },
            set: { viewModel.selectDetailMode($0) }
        )
    }

    private var rawEventSelection: Binding<String?> {
        Binding(
            get: { viewModel.selectedRawEventID },
            set: { viewModel.selectRawEvent(id: $0) }
        )
    }

    private func tickHeader(_ summary: MoiraTickSummary) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(alignment: .firstTextBaseline, spacing: 12) {
                Text("Tick \(summary.tick)")
                    .font(.headline)

                if summary.cortexHandled {
                    Text("Cortex handled")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                Spacer()
            }

            HStack(spacing: 18) {
                MoiraO11yMetricLabel(title: "Events", value: "\(summary.eventCount)")
                MoiraO11yMetricLabel(title: "Warnings", value: "\(summary.warningCount)")
                MoiraO11yMetricLabel(title: "Errors", value: "\(summary.errorCount)")
                MoiraO11yMetricLabel(title: "Raw", value: "\(viewModel.rawEvents.count)")
            }

            HStack(spacing: 14) {
                MoiraO11yMetadataLine(title: "First Seen", value: summary.firstSeenAt, monospaced: true)
                MoiraO11yMetadataLine(title: "Last Seen", value: summary.lastSeenAt, monospaced: true)
            }

            if let traceID = summary.traceID {
                MoiraO11yMetadataLine(title: "Trace", value: traceID, monospaced: true)
            }
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}
