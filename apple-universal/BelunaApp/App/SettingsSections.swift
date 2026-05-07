import SwiftUI

struct ConnectionSettingsSection: View {
    @ObservedObject var viewModel: ChatViewModel

    var body: some View {
        Section("Connection") {
            TextField(SocketPathSettings.defaultSocketPath, text: $viewModel.socketPathDraft)
                .textFieldStyle(.roundedBorder)
                .font(.body.monospaced())

            HStack(spacing: 8) {
                Button("Apply Socket Path", action: viewModel.applySocketPathDraft)
                    .buttonStyle(.borderedProminent)
                    .disabled(!viewModel.canApplySocketPath)

                Button(viewModel.connectButtonTitle, action: viewModel.toggleConnection)
                    .buttonStyle(.bordered)

                Button(viewModel.retryButtonTitle, action: viewModel.retryConnection)
                    .buttonStyle(.bordered)
                    .disabled(!viewModel.canRetry)
            }
        }
    }
}

struct ChatRetentionSettingsSection: View {
    @ObservedObject var viewModel: ChatViewModel

    var body: some View {
        Section("Chat") {
            VStack(alignment: .leading, spacing: 8) {
                Text("Chat Message Buffer Capacity")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)

                TextField("1000", text: $viewModel.messageCapacityDraft)
                    .textFieldStyle(.roundedBorder)
                    .font(.body.monospaced())

                Button("Apply Message Capacity", action: viewModel.applyMessageCapacityDraft)
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
                    Button("Clear Local History", action: viewModel.clearLocalSenseActHistory)
                        .buttonStyle(.borderedProminent)
                        .disabled(!viewModel.canClearLocalSenseActHistory)

                    Text("Persisted: \(viewModel.persistedSenseActMessageCount)")
                        .font(.caption.monospaced())
                        .foregroundStyle(.secondary)
                }
            }
        }
    }
}

struct RuntimeStatusSection: View {
    @ObservedObject var viewModel: ChatViewModel

    var body: some View {
        Section("Status") {
            LabeledContent("Connection", value: viewModel.connectionState.rawValue)
            LabeledContent("Beluna", value: viewModel.belunaState.rawValue)

            LabeledContent("Active Socket Path") {
                Text(viewModel.socketPath)
                    .font(.caption.monospaced())
            }

            LabeledContent("Message Capacity", value: "\(viewModel.messageCapacity)")
            LabeledContent(
                "Visible / Buffered",
                value: "\(viewModel.visibleMessageCount) / \(viewModel.bufferedMessageCount)"
            )
            LabeledContent("Persisted Sense/Act", value: "\(viewModel.persistedSenseActMessageCount)")
        }
    }
}

struct MoiraOperationsSection: View {
    var body: some View {
        Section("Moira") {
            LabeledContent("Runtime", value: "Pending")
            LabeledContent("Receiver", value: "Pending")
            LabeledContent("Local Observability", value: "Pending")
        }
    }
}
