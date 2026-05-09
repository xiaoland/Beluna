import SwiftUI

struct MoiraProfileEditorSheet: View {
    let title: String
    let allowsIdentityEdit: Bool
    let profileIDToLoad: String?
    @ObservedObject var viewModel: MoiraOperationsViewModel
    @Environment(\.dismiss) private var dismiss
    @State private var draft: MoiraProfileEditorDraft
    @State private var didLoad = false
    @State private var isLoading = false

    private let coreConfigEditorHeight: CGFloat = 180

    init(
        title: String,
        allowsIdentityEdit: Bool,
        profileIDToLoad: String?,
        viewModel: MoiraOperationsViewModel,
        initialDraft: MoiraProfileEditorDraft
    ) {
        self.title = title
        self.allowsIdentityEdit = allowsIdentityEdit
        self.profileIDToLoad = profileIDToLoad
        self.viewModel = viewModel
        self._draft = State(initialValue: initialDraft)
    }

    var body: some View {
        VStack(spacing: 0) {
            Form {
                Section(title) {
                    if isLoading {
                        ProgressView()
                            .controlSize(.small)
                    }

                    TextField("Profile ID", text: $draft.profileID)
                        .textFieldStyle(.roundedBorder)
                        .disabled(!allowsIdentityEdit || isLoading || viewModel.isOperating)

                    if let loadedProfilePath = draft.loadedProfilePath {
                        pathText("Loaded Path", loadedProfilePath)
                    }

                    Text("Core Config")
                        .font(.caption.weight(.semibold))

                    TextEditor(text: $draft.coreConfig)
                        .font(.body.monospaced())
                        .frame(height: coreConfigEditorHeight)
                        .disabled(isLoading)

                    MoiraProfileEnvironmentFilesEditor(
                        envFiles: $draft.envFiles,
                        isDisabled: isLoading
                    )

                    MoiraProfileInlineEnvironmentEditor(
                        inlineEnvironment: $draft.inlineEnvironment,
                        isDisabled: isLoading
                    )

                    if let errorText = viewModel.profileManagementErrorText {
                        MoiraManagementErrorText(text: errorText)
                    }
                }
            }
            .formStyle(.grouped)

            Divider()
            sheetActions
        }
        .frame(width: 640, height: 600)
        .task {
            await loadInitialDraftIfNeeded()
        }
    }

    private var sheetActions: some View {
        HStack(spacing: 10) {
            if isLoading || viewModel.isOperating {
                ProgressView()
                    .controlSize(.small)
            }

            Spacer()

            Button("Cancel") {
                dismiss()
            }
            .keyboardShortcut(.cancelAction)

            Button("Save") {
                Task {
                    if await viewModel.saveProfileEditorDraftNow(draft) {
                        dismiss()
                    }
                }
            }
            .buttonStyle(.borderedProminent)
            .keyboardShortcut(.defaultAction)
            .disabled(!draft.isValid || isLoading || viewModel.isOperating || viewModel.isRefreshing)
        }
        .padding(16)
    }

    private func loadInitialDraftIfNeeded() async {
        guard !didLoad else {
            return
        }
        didLoad = true

        guard let profileIDToLoad else {
            return
        }

        isLoading = true
        defer {
            isLoading = false
        }

        if let loadedDraft = await viewModel.loadProfileEditorDraftNow(profileID: profileIDToLoad) {
            draft = loadedDraft
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
