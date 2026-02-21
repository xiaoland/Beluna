import SwiftUI

private enum ObservabilityPane: String, CaseIterable, Identifiable {
    case metrics = "Metrics"
    case logs = "Logs"

    var id: String { rawValue }
}

struct ObservabilityView: View {
    @ObservedObject var viewModel: ObservabilityViewModel
    @State private var selectedPane: ObservabilityPane = .metrics

    private let byteCountFormatter: ByteCountFormatter = {
        let formatter = ByteCountFormatter()
        formatter.countStyle = .file
        return formatter
    }()

    private static let timestampFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.dateFormat = "yyyy-MM-dd HH:mm:ss.SSS"
        return formatter
    }()

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            content
        }
        .frame(minWidth: 900, minHeight: 560)
        .onAppear {
            viewModel.startIfNeeded()
        }
    }

    private var header: some View {
        VStack(spacing: 10) {
            HStack(alignment: .center, spacing: 10) {
                Text("Observability")
                    .font(.title3.bold())

                Spacer(minLength: 12)

                Picker("Pane", selection: $selectedPane) {
                    ForEach(ObservabilityPane.allCases) { pane in
                        Text(pane.rawValue).tag(pane)
                    }
                }
                .pickerStyle(.segmented)
                .frame(width: 220)
            }

            if selectedPane == .metrics {
                metricsHeaderControls
            } else {
                logsHeaderControls
            }
        }
        .padding(12)
    }

    private var metricsHeaderControls: some View {
        VStack(spacing: 8) {
            HStack(alignment: .center, spacing: 10) {
                TextField("http://127.0.0.1:9464/metrics", text: $viewModel.metricsEndpointDraft)
                    .textFieldStyle(.roundedBorder)
                    .font(.body.monospaced())

                Button("Apply") {
                    viewModel.applyMetricsEndpointDraft()
                }
                .buttonStyle(.borderedProminent)
                .disabled(!viewModel.canApplyMetricsEndpoint)

                Button("Refresh") {
                    viewModel.refreshMetrics()
                }
                .buttonStyle(.bordered)
                .disabled(viewModel.isMetricsRefreshing)
            }

            HStack(spacing: 8) {
                Text(viewModel.metricsStatusText)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
                    .frame(maxWidth: .infinity, alignment: .leading)

                if let refreshedAt = viewModel.metricsLastRefreshedAt {
                    Text("Updated \(refreshedAt, style: .time)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
        }
    }

    private var logsHeaderControls: some View {
        VStack(spacing: 8) {
            HStack(alignment: .center, spacing: 10) {
                TextField("Beluna Core log directory", text: $viewModel.logDirectoryPathDraft)
                    .textFieldStyle(.roundedBorder)
                    .font(.body.monospaced())

                #if os(macOS)
                Button("Choose Folder") {
                    viewModel.chooseLogDirectory()
                }
                .buttonStyle(.bordered)
                #endif

                Button("Apply") {
                    viewModel.applyLogDirectoryPathDraft()
                }
                .buttonStyle(.borderedProminent)
                .disabled(!viewModel.canApplyLogDirectoryPath)

                Button("Refresh") {
                    viewModel.refresh()
                }
                .buttonStyle(.bordered)
                .disabled(viewModel.isRefreshing)
            }

            HStack(spacing: 8) {
                Text(viewModel.statusText)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
                    .frame(maxWidth: .infinity, alignment: .leading)

                if let refreshedAt = viewModel.lastRefreshedAt {
                    Text("Updated \(refreshedAt, style: .time)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
        }
    }

    private var content: some View {
        Group {
            if selectedPane == .metrics {
                metricsContent
            } else {
                logsContent
            }
        }
    }

    private var metricsContent: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 12) {
                HStack(spacing: 12) {
                    metricCard(
                        title: "Cortex Cycle ID",
                        value: formatMetricValue(viewModel.metricsCycleID)
                    )
                    metricCard(
                        title: "Act Catalog Count",
                        value: formatMetricValue(viewModel.metricsActDescriptorCatalogCount)
                    )
                }

                Divider()

                Text("Raw Metrics Excerpt")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)

                Text(
                    viewModel.metricsRawExcerpt.isEmpty
                        ? "No metrics payload captured yet."
                        : viewModel.metricsRawExcerpt
                )
                .font(.system(.caption, design: .monospaced))
                .textSelection(.enabled)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(10)
                .background(Color.primary.opacity(0.03))
                .clipShape(RoundedRectangle(cornerRadius: 8))
            }
            .padding(12)
            .frame(maxWidth: .infinity, alignment: .leading)
        }
    }

    private var logsContent: some View {
        NavigationSplitView {
            List(
                selection: Binding(
                    get: { viewModel.selectedFilePath },
                    set: { viewModel.selectFile(path: $0) }
                )
            ) {
                ForEach(viewModel.files) { file in
                    HStack(alignment: .center, spacing: 10) {
                        Image(systemName: "doc.text")
                            .foregroundStyle(.secondary)

                        VStack(alignment: .leading, spacing: 2) {
                            Text(file.name)
                                .font(.body.monospaced())
                                .lineLimit(1)
                            HStack(spacing: 6) {
                                Text(byteCountFormatter.string(fromByteCount: file.sizeBytes))
                                if let modifiedAt = file.modifiedAt {
                                    Text("•")
                                    Text(modifiedAt, style: .time)
                                }
                            }
                            .font(.caption)
                            .foregroundStyle(.secondary)
                        }
                    }
                    .tag(file.path)
                }
            }
            .navigationTitle("Log Files")
        } detail: {
            if viewModel.selectedFilePath == nil {
                ContentUnavailableView(
                    "No Log File Selected",
                    systemImage: "doc.text.magnifyingglass",
                    description: Text("Choose a file from the left panel.")
                )
            } else {
                VStack(spacing: 0) {
                    HStack(spacing: 10) {
                        Text(viewModel.selectedFileName)
                            .font(.headline.monospaced())
                            .lineLimit(1)
                            .truncationMode(.middle)

                        Spacer(minLength: 10)

                        if viewModel.selectedFileSizeBytes > 0 {
                            Text(byteCountFormatter.string(fromByteCount: viewModel.selectedFileSizeBytes))
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }

                        if let modifiedAt = viewModel.selectedFileModifiedAt {
                            Text(modifiedAt, style: .time)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                    }
                    .padding(.horizontal, 12)
                    .padding(.vertical, 8)
                    Divider()

                    if viewModel.selectedFileEntries.isEmpty {
                        VStack(spacing: 8) {
                            ContentUnavailableView(
                                "No Structured Log Rows",
                                systemImage: "tablecells",
                                description: Text("The selected file has no parseable JSON log lines.")
                            )
                            ScrollView {
                                Text(viewModel.selectedFileContent)
                                    .font(.system(.caption, design: .monospaced))
                                    .textSelection(.enabled)
                                    .frame(maxWidth: .infinity, alignment: .leading)
                                    .padding(12)
                            }
                        }
                    } else {
                        Table(viewModel.selectedFileEntries, selection: $viewModel.selectedLogEntryID) {
                            TableColumn("Time") { entry in
                                Text(formattedTimestamp(for: entry))
                                    .font(.system(.caption, design: .monospaced))
                            }
                            .width(min: 190, ideal: 220, max: 280)

                            TableColumn("Level") { entry in
                                Text(entry.level)
                                    .font(.system(.caption, design: .monospaced))
                                    .foregroundStyle(levelColor(for: entry.level))
                            }
                            .width(min: 70, ideal: 80, max: 90)

                            TableColumn("Target") { entry in
                                Text(entry.target)
                                    .font(.system(.caption, design: .monospaced))
                                    .lineLimit(1)
                            }
                            .width(min: 170, ideal: 220, max: 320)

                            TableColumn("Message") { entry in
                                Text(entry.message)
                                    .font(.body)
                                    .lineLimit(2)
                            }
                        }
                        .tableStyle(.inset)

                        if let selectedLogEntry = viewModel.selectedLogEntry {
                            Divider()
                            VStack(alignment: .leading, spacing: 6) {
                                Text("Raw Line")
                                    .font(.caption.weight(.semibold))
                                    .foregroundStyle(.secondary)
                                ScrollView {
                                    Text(selectedLogEntry.rawLine)
                                        .font(.system(.caption, design: .monospaced))
                                        .textSelection(.enabled)
                                        .frame(maxWidth: .infinity, alignment: .leading)
                                }
                            }
                            .padding(10)
                            .frame(height: 140)
                            .background(Color.primary.opacity(0.02))
                        }
                    }
                }
            }
        }
    }

    private func metricCard(title: String, value: String) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(title)
                .font(.caption.weight(.semibold))
                .foregroundStyle(.secondary)
            Text(value)
                .font(.title2.monospacedDigit())
                .lineLimit(1)
                .minimumScaleFactor(0.6)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(12)
        .background(Color.primary.opacity(0.04))
        .clipShape(RoundedRectangle(cornerRadius: 8))
    }

    private func formatMetricValue(_ value: Double?) -> String {
        guard let value else {
            return "-"
        }
        if value == value.rounded() {
            return String(Int64(value))
        }
        return String(value)
    }

    private func formattedTimestamp(for entry: ObservabilityLogEntry) -> String {
        if let timestamp = entry.timestamp {
            return Self.timestampFormatter.string(from: timestamp)
        }
        if !entry.timestampRaw.isEmpty {
            return entry.timestampRaw
        }
        return "-"
    }

    private func levelColor(for level: String) -> Color {
        switch level.uppercased() {
        case "ERROR":
            return .red
        case "WARN", "WARNING":
            return .orange
        case "INFO":
            return .blue
        case "DEBUG", "TRACE":
            return .secondary
        default:
            return .primary
        }
    }
}
