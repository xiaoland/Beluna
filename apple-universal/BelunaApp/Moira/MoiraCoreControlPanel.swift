import SwiftUI

struct MoiraCoreControlPanel: View {
    @ObservedObject var viewModel: MoiraOperationsViewModel
    @State private var showingForceKillConfirmation = false

    var body: some View {
        Form {
            runtimeSection
            MoiraLaunchContextSection(viewModel: viewModel)
            operationSection
        }
        .formStyle(.grouped)
        .padding(16)
        .frame(minWidth: 620, minHeight: 680)
        .task {
            await viewModel.refreshNow()
        }
        .confirmationDialog(
            "Force kill Core?",
            isPresented: $showingForceKillConfirmation,
            titleVisibility: .visible
        ) {
            Button("Force Kill", role: .destructive) {
                viewModel.forceKillCore()
            }
        }
    }

    private var runtimeSection: some View {
        Section("Moira Runtime") {
            HStack(spacing: 8) {
                LabeledContent("Runtime", value: viewModel.runtimeStatusText)
                Spacer()
                Button(action: viewModel.refresh) {
                    Label("Refresh", systemImage: "arrow.clockwise")
                }
                .buttonStyle(.bordered)
                .disabled(!viewModel.canRefresh)
            }

            LabeledContent("Receiver", value: viewModel.receiverStatusText)
            LabeledContent("Endpoint") {
                Text(viewModel.snapshot.status.receiver.endpoint)
                    .font(.caption.monospaced())
                    .textSelection(.enabled)
            }

            if let errorText = viewModel.runtimeErrorText {
                Text(errorText)
                    .font(.caption)
                    .foregroundStyle(.red)
                    .textSelection(.enabled)
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

    private var operationSection: some View {
        Section("Operations") {
            coreOperationStatus

            Divider()

            HStack(spacing: 10) {
                Button(action: viewModel.wakeCore) {
                    Label("Wake", systemImage: "play.fill")
                }
                .buttonStyle(.borderedProminent)
                .disabled(!viewModel.canWakeCore)

                Button(action: viewModel.stopCore) {
                    Label("Stop", systemImage: "stop.fill")
                }
                .buttonStyle(.bordered)
                .disabled(!viewModel.canStopCore)

                Button(role: .destructive) {
                    showingForceKillConfirmation = true
                } label: {
                    Label("Force Kill", systemImage: "xmark.octagon.fill")
                }
                .buttonStyle(.bordered)
                .disabled(!viewModel.canForceKillCore)

                if viewModel.isOperating || viewModel.isTrackingCoreTransition {
                    ProgressView()
                        .controlSize(.small)
                }
            }

            if let errorText = viewModel.coreOperationErrorText {
                Text(errorText)
                    .font(.caption)
                    .foregroundStyle(.red)
                    .textSelection(.enabled)
            }
        }
    }

    private var coreOperationStatus: some View {
        Group {
            LabeledContent("Core Phase", value: viewModel.coreStatusText)

            if let pid = viewModel.snapshot.status.core.pid {
                LabeledContent("PID", value: "\(pid)")
            }

            if let terminalReason = viewModel.snapshot.status.core.terminalReason {
                LabeledContent("Terminal") {
                    Text(terminalReason)
                        .font(.caption.monospaced())
                        .textSelection(.enabled)
                }
            }

            if let executablePath = viewModel.snapshot.status.core.executablePath {
                pathText("Executable", executablePath)
            }
            if let workingDir = viewModel.snapshot.status.core.workingDir {
                pathText("Working Dir", workingDir)
            }
            if let profilePath = viewModel.snapshot.status.core.profilePath {
                pathText("Profile", profilePath)
            }
        }
    }

    private func pathText(_ label: String, _ path: String) -> some View {
        LabeledContent(label) {
            Text(path)
                .font(.caption.monospaced())
                .lineLimit(2)
                .textSelection(.enabled)
        }
    }
}
