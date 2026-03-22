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

                    Button(viewModel.retryButtonTitle) {
                        viewModel.retryConnection()
                    }
                    .buttonStyle(.bordered)
                    .disabled(!viewModel.canRetry)
                }
            }

            Section("Chat") {
                VStack(alignment: .leading, spacing: 8) {
                    Text("Chat Message Buffer Capacity")
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(.secondary)

                    TextField("1000", text: $viewModel.messageCapacityDraft)
                        .textFieldStyle(.roundedBorder)
                        .font(.body.monospaced())

                    Button("Apply Message Capacity") {
                        viewModel.applyMessageCapacityDraft()
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(!viewModel.canApplyMessageCapacity)
                }

                VStack(alignment: .leading, spacing: 8) {
                    Text("Local Sense/Act History")
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(.secondary)

                    Text("Sent senses and received acts are persisted locally and restored on next launch.")
                        .font(.caption)
                        .foregroundStyle(.secondary)

                    HStack(spacing: 8) {
                        Button("Clear Local History") {
                            viewModel.clearLocalSenseActHistory()
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(!viewModel.canClearLocalSenseActHistory)

                        Text("Persisted: \(viewModel.persistedSenseActMessageCount)")
                            .font(.caption.monospaced())
                            .foregroundStyle(.secondary)
                    }
                }
            }

            Section("Status") {
                LabeledContent("Connection", value: viewModel.connectionState.rawValue)
                LabeledContent("Beluna", value: viewModel.belunaState.rawValue)

                LabeledContent("Active Socket Path") {
                    Text(viewModel.socketPath)
                        .font(.caption.monospaced())
                }

                LabeledContent("Message Capacity", value: "\(viewModel.messageCapacity)")
                LabeledContent("Visible / Buffered", value: "\(viewModel.visibleMessageCount) / \(viewModel.bufferedMessageCount)")
                LabeledContent("Persisted Sense/Act", value: "\(viewModel.persistedSenseActMessageCount)")
            }
        }
        .formStyle(.grouped)
        .padding(16)
        .frame(minWidth: 560, minHeight: 360)
    }
}
