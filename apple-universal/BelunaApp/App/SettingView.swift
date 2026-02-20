import SwiftUI

struct SettingView: View {
    @ObservedObject var viewModel: ChatViewModel

    var body: some View {
        Form {
            Section("Connection") {
                TextField("/tmp/beluna.sock", text: $viewModel.socketPathDraft)
                    .textFieldStyle(.roundedBorder)
                    .font(.body.monospaced())

                HStack(spacing: 8) {
                    Button("Apply Socket Path") {
                        viewModel.applySocketPathDraft()
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(!viewModel.canApplySocketPath)

                    Button(viewModel.connectButtonTitle) {
                        viewModel.toggleConnection()
                    }
                    .buttonStyle(.bordered)

                    Button("Retry") {
                        viewModel.retryConnection()
                    }
                    .buttonStyle(.bordered)
                    .disabled(!viewModel.canRetry)
                }
            }

            Section("Status") {
                LabeledContent("Connection", value: viewModel.connectionState.rawValue)
                LabeledContent("Beluna", value: viewModel.belunaState.rawValue)
                LabeledContent("Active Socket Path") {
                    Text(viewModel.socketPath)
                        .font(.caption.monospaced())
                }
            }
        }
        .formStyle(.grouped)
        .padding(16)
        .frame(minWidth: 520, minHeight: 260)
    }
}
