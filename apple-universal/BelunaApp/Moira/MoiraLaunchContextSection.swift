import SwiftUI

struct MoiraLaunchContextSection: View {
    @ObservedObject var viewModel: MoiraOperationsViewModel
    @State private var activeEditor: MoiraManagementEditor?

    var body: some View {
        Section("Launch Context") {
            launchTargetPicker
            targetManagementActions

            if let target = viewModel.selectedLaunchTarget {
                launchTargetDetail(target)
            }

            profilePicker
            profileManagementActions

            if let profile = viewModel.selectedProfile {
                pathText("Profile Path", profile.profilePath)
            }

            if let errorText = viewModel.targetManagementErrorText {
                MoiraManagementErrorText(text: errorText)
            }

            if let errorText = viewModel.profileManagementErrorText {
                MoiraManagementErrorText(text: errorText)
            }
        }
        .sheet(item: $activeEditor) { editor in
            editorSheet(editor)
        }
    }

    private var launchTargetPicker: some View {
        Group {
            if viewModel.hasLaunchTargets {
                Picker("Launch Target", selection: launchTargetSelection) {
                    ForEach(viewModel.snapshot.launchTargets) { target in
                        Text(target.label).tag(target.id)
                    }
                }
            } else {
                LabeledContent("Launch Target", value: "none")
            }
        }
    }

    private var profilePicker: some View {
        Group {
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
        }
    }

    private var targetManagementActions: some View {
        HStack(spacing: 10) {
            Button {
                activeEditor = .createTarget
            } label: {
                Label("Create Target", systemImage: "plus.circle")
            }
            .buttonStyle(.bordered)
            .disabled(!viewModel.canOpenManagementEditor)

            Button {
                activeEditor = .editTarget
            } label: {
                Label("Edit Target", systemImage: "square.and.pencil")
            }
            .buttonStyle(.bordered)
            .disabled(!viewModel.canEditSelectedLaunchTarget)
        }
    }

    private var profileManagementActions: some View {
        HStack(spacing: 10) {
            Button {
                activeEditor = .createProfile
            } label: {
                Label("Create Profile", systemImage: "plus.circle")
            }
            .buttonStyle(.bordered)
            .disabled(!viewModel.canOpenManagementEditor)

            Button {
                if let profile = viewModel.selectedProfile {
                    activeEditor = .editProfile(profile)
                }
            } label: {
                Label("Edit Profile", systemImage: "square.and.pencil")
            }
            .buttonStyle(.bordered)
            .disabled(!viewModel.canEditSelectedProfile)
        }
    }

    @ViewBuilder
    private func editorSheet(_ editor: MoiraManagementEditor) -> some View {
        switch editor {
        case .createTarget:
            MoiraTargetEditorSheet(
                title: "Create Target",
                allowsIdentityEdit: true,
                viewModel: viewModel,
                initialDraft: viewModel.targetEditorDraftForCreate()
            )
        case .editTarget:
            if let draft = viewModel.targetEditorDraftForSelectedLaunchTarget() {
                MoiraTargetEditorSheet(
                    title: "Edit Target",
                    allowsIdentityEdit: false,
                    viewModel: viewModel,
                    initialDraft: draft
                )
            } else {
                MoiraUnavailableEditorSheet(title: "Edit Target")
            }
        case .createProfile:
            MoiraProfileEditorSheet(
                title: "Create Profile",
                allowsIdentityEdit: true,
                profileIDToLoad: nil,
                viewModel: viewModel,
                initialDraft: viewModel.profileEditorDraftForCreate()
            )
        case let .editProfile(profile):
            MoiraProfileEditorSheet(
                title: "Edit Profile",
                allowsIdentityEdit: false,
                profileIDToLoad: profile.profileID,
                viewModel: viewModel,
                initialDraft: MoiraProfileEditorDraft(profileID: profile.profileID)
            )
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

private enum MoiraManagementEditor: Identifiable {
    case createTarget
    case editTarget
    case createProfile
    case editProfile(MoiraProfileDocumentSummary)

    var id: String {
        switch self {
        case .createTarget:
            "create-target"
        case .editTarget:
            "edit-target"
        case .createProfile:
            "create-profile"
        case let .editProfile(profile):
            "edit-profile-\(profile.id)"
        }
    }
}

private struct MoiraUnavailableEditorSheet: View {
    @Environment(\.dismiss) private var dismiss
    let title: String

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text(title)
                .font(.headline)

            Text("Selected item cannot be edited from this panel.")
                .foregroundStyle(.secondary)

            HStack {
                Spacer()
                Button("Cancel") {
                    dismiss()
                }
                .keyboardShortcut(.cancelAction)
            }
        }
        .padding(20)
        .frame(width: 360)
    }
}
