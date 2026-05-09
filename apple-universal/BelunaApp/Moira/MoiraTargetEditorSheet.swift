import SwiftUI

struct MoiraTargetEditorSheet: View {
    let title: String
    let allowsIdentityEdit: Bool
    @ObservedObject var viewModel: MoiraOperationsViewModel
    @Environment(\.dismiss) private var dismiss
    @State private var draft: MoiraTargetEditorDraft

    init(
        title: String,
        allowsIdentityEdit: Bool,
        viewModel: MoiraOperationsViewModel,
        initialDraft: MoiraTargetEditorDraft
    ) {
        self.title = title
        self.allowsIdentityEdit = allowsIdentityEdit
        self.viewModel = viewModel
        self._draft = State(initialValue: initialDraft)
    }

    var body: some View {
        VStack(spacing: 0) {
            Form {
                Section(title) {
                    TextField("Build ID", text: $draft.buildID)
                        .textFieldStyle(.roundedBorder)
                        .disabled(!allowsIdentityEdit || viewModel.isOperating)

                    TextField("Executable Path", text: $draft.executablePath)
                        .textFieldStyle(.roundedBorder)

                    TextField("Working Dir", text: $draft.workingDir)
                        .textFieldStyle(.roundedBorder)

                    TextField("Source Dir", text: $draft.sourceDir)
                        .textFieldStyle(.roundedBorder)

                    if let errorText = viewModel.targetManagementErrorText {
                        MoiraManagementErrorText(text: errorText)
                    }
                }
            }
            .formStyle(.grouped)

            Divider()
            sheetActions
        }
        .frame(width: 560, height: 320)
    }

    private var sheetActions: some View {
        HStack(spacing: 10) {
            if viewModel.isOperating {
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
                    if await viewModel.saveTargetEditorDraftNow(draft) {
                        dismiss()
                    }
                }
            }
            .buttonStyle(.borderedProminent)
            .keyboardShortcut(.defaultAction)
            .disabled(!draft.isValid || viewModel.isOperating || viewModel.isRefreshing)
        }
        .padding(16)
    }
}
