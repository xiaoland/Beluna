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

            Section("Observability") {
                VStack(alignment: .leading, spacing: 8) {
                    Text("Metrics Endpoint")
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(.secondary)

                    TextField("http://127.0.0.1:9464/metrics", text: $viewModel.metricsEndpointDraft)
                        .textFieldStyle(.roundedBorder)
                        .font(.body.monospaced())

                    HStack(spacing: 8) {
                        Button("Apply Metrics Endpoint") {
                            viewModel.applyMetricsEndpointDraft()
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(!viewModel.canApplyMetricsEndpoint)

                        Button("Refresh Metrics") {
                            viewModel.refreshMetrics()
                        }
                        .buttonStyle(.bordered)
                        .disabled(viewModel.isMetricsRefreshing)
                    }
                }

                VStack(alignment: .leading, spacing: 8) {
                    Text("Log Directory")
                        .font(.caption.weight(.semibold))
                        .foregroundStyle(.secondary)

                    TextField("~/logs/core", text: $viewModel.logDirectoryPathDraft)
                        .textFieldStyle(.roundedBorder)
                        .font(.body.monospaced())

                    Button("Apply Log Directory") {
                        viewModel.applyLogDirectoryPathDraft()
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(!viewModel.canApplyLogDirectoryPath)
                }
            }

            Section("Status") {
                LabeledContent("Connection", value: viewModel.connectionState.rawValue)
                LabeledContent("Beluna", value: viewModel.belunaState.rawValue)

                LabeledContent("Active Socket Path") {
                    Text(viewModel.socketPath)
                        .font(.caption.monospaced())
                }

                LabeledContent("Metrics Endpoint") {
                    Text(viewModel.metricsEndpoint)
                        .font(.caption.monospaced())
                }

                LabeledContent("Log Directory") {
                    Text(viewModel.logDirectoryPath)
                        .font(.caption.monospaced())
                }

                LabeledContent("Metrics Status") {
                    Text(viewModel.metricsStatusText)
                        .font(.caption)
                }

                LabeledContent("Log Status") {
                    Text(viewModel.logStatusText)
                        .font(.caption)
                }
            }
        }
        .formStyle(.grouped)
        .padding(16)
        .frame(minWidth: 560, minHeight: 360)
    }
}
