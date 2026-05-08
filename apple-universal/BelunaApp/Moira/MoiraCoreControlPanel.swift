import SwiftUI

struct MoiraCoreControlPanel: View {
    @ObservedObject var viewModel: MoiraOperationsViewModel
    @State private var showingForceKillConfirmation = false

    var body: some View {
        Form {
            runtimeSection
            coreSection
            launchSection
            operationSection
        }
        .formStyle(.grouped)
        .padding(16)
        .frame(minWidth: 560, minHeight: 460)
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

            if let errorText = viewModel.lastErrorText {
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

    private var coreSection: some View {
        Section("Core") {
            LabeledContent("Phase", value: viewModel.coreStatusText)

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

    private var launchSection: some View {
        Section("Launch Context") {
            if viewModel.hasLaunchTargets {
                Picker("Launch Target", selection: launchTargetSelection) {
                    ForEach(viewModel.snapshot.launchTargets) { target in
                        Text(target.label).tag(target.id)
                    }
                }
            } else {
                LabeledContent("Launch Target", value: "none")
            }

            if viewModel.hasProfiles {
                Picker("Profile", selection: profileSelection) {
                    Text("None").tag("")
                    ForEach(viewModel.snapshot.profiles) { profile in
                        Text(profile.profileID).tag(profile.id)
                    }
                }
            } else {
                LabeledContent("Profile", value: "none")
            }

            if let target = viewModel.selectedLaunchTarget {
                launchTargetDetail(target)
            }

            if let profile = viewModel.selectedProfile {
                pathText("Profile Path", profile.profilePath)
            }
        }
    }

    private var operationSection: some View {
        Section("Operations") {
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

                if viewModel.isOperating {
                    ProgressView()
                        .controlSize(.small)
                }
            }
        }
    }

    private var launchTargetSelection: Binding<String> {
        Binding(
            get: { viewModel.selectedLaunchTargetID },
            set: { viewModel.selectLaunchTarget(id: $0) }
        )
    }

    private var profileSelection: Binding<String> {
        Binding(
            get: { viewModel.selectedProfileID },
            set: { viewModel.selectProfile(id: $0) }
        )
    }

    private func launchTargetDetail(_ target: MoiraLaunchTargetSummary) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            LabeledContent("Readiness", value: target.readiness)
            LabeledContent("Provenance", value: target.provenance)

            if let issue = target.issue {
                Text(issue)
                    .font(.caption)
                    .foregroundStyle(.red)
                    .textSelection(.enabled)
            }

            if let executablePath = target.executablePath {
                pathText("Executable", executablePath)
            }
            if let workingDir = target.workingDir {
                pathText("Working Dir", workingDir)
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
