import SwiftUI

struct MoiraOperationsSection: View {
    @ObservedObject var viewModel: MoiraOperationsViewModel

    var body: some View {
        Section("Moira") {
            runtimeHeader
            receiverStatus
            localObservability
            rawTickRecords
        }
        .task {
            await viewModel.refreshNow()
        }
    }

    private var runtimeHeader: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 8) {
                LabeledContent("Runtime", value: viewModel.runtimeStatusText)

                Spacer()

                Button(action: viewModel.refresh) {
                    Label("Refresh", systemImage: "arrow.clockwise")
                }
                .buttonStyle(.bordered)
                .disabled(!viewModel.canRefresh)
            }

            if let errorText = viewModel.lastErrorText {
                Text(errorText)
                    .font(.caption)
                    .foregroundStyle(.red)
            }

            ForEach(viewModel.snapshot.status.attentionResources) { resource in
                VStack(alignment: .leading, spacing: 4) {
                    Text(resource.label)
                        .font(.caption.weight(.semibold))
                    if let detail = resource.detail {
                        Text(detail)
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                            .textSelection(.enabled)
                    }
                }
            }
        }
    }

    private var receiverStatus: some View {
        VStack(alignment: .leading, spacing: 8) {
            Divider()

            LabeledContent("Receiver", value: viewModel.receiverStatusText)
            LabeledContent("Endpoint") {
                Text(viewModel.snapshot.status.receiver.endpoint)
                    .font(.caption.monospaced())
                    .textSelection(.enabled)
            }

            HStack(spacing: 16) {
                LabeledContent("Raw Events", value: viewModel.eventCountText)
                LabeledContent("Wakes", value: viewModel.wakeCountText)
                LabeledContent("Ticks", value: viewModel.tickCountText)
            }

            if let lastError = viewModel.snapshot.status.receiver.lastError {
                Text(lastError)
                    .font(.caption)
                    .foregroundStyle(.red)
                    .textSelection(.enabled)
            }
        }
    }

    private var localObservability: some View {
        VStack(alignment: .leading, spacing: 8) {
            Divider()

            if viewModel.hasRuns {
                Picker("Wake", selection: runSelection) {
                    ForEach(viewModel.snapshot.runs) { run in
                        Text(runRowTitle(run)).tag(run.id)
                    }
                }
            } else {
                LabeledContent("Wake", value: "none")
            }

            if viewModel.hasTicks {
                Picker("Tick", selection: tickSelection) {
                    ForEach(viewModel.snapshot.ticks) { tick in
                        Text(tickRowTitle(tick)).tag(String(tick.tick))
                    }
                }
            } else {
                LabeledContent("Tick", value: "none")
            }

            if let detail = viewModel.snapshot.tickDetail {
                HStack(spacing: 16) {
                    LabeledContent("Selected Tick", value: "\(detail.summary.tick)")
                    LabeledContent("Events", value: "\(detail.raw.count)")
                    LabeledContent("Cortex", value: detail.summary.cortexHandled ? "yes" : "no")
                }
            }
        }
    }

    private var rawTickRecords: some View {
        VStack(alignment: .leading, spacing: 8) {
            Divider()

            if viewModel.rawEvents.isEmpty {
                LabeledContent("Raw Records", value: "none")
            } else {
                ForEach(Array(viewModel.rawEvents.prefix(24))) { record in
                    MoiraEventRecordRow(record: record)
                }
            }
        }
    }

    private var runSelection: Binding<String> {
        Binding(
            get: { viewModel.selectedRunBindingValue },
            set: { viewModel.selectRun(id: $0) }
        )
    }

    private var tickSelection: Binding<String> {
        Binding(
            get: { viewModel.selectedTickBindingValue },
            set: { viewModel.selectTick(value: $0) }
        )
    }

    private func runRowTitle(_ run: MoiraRunSummary) -> String {
        let prefix = String(run.runID.prefix(12))
        if let latestTick = run.latestTick {
            return "\(prefix)  tick \(latestTick)"
        }
        return prefix
    }

    private func tickRowTitle(_ tick: MoiraTickSummary) -> String {
        "tick \(tick.tick)  events \(tick.eventCount)"
    }
}
