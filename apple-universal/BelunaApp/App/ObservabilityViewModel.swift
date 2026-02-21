import Foundation

#if os(macOS)
import AppKit
#endif

struct ObservabilityLogFile: Identifiable, Equatable {
    let path: String
    let name: String
    let sizeBytes: Int64
    let modifiedAt: Date?

    var id: String { path }
}

struct ObservabilityLogEntry: Identifiable, Equatable {
    let id: String
    let timestamp: Date?
    let timestampRaw: String
    let level: String
    let target: String
    let message: String
    let rawLine: String
    let lineIndex: Int
}

@MainActor
final class ObservabilityViewModel: ObservableObject {
    @Published var logDirectoryPathDraft: String
    @Published private(set) var logDirectoryPath: String
    @Published var metricsEndpointDraft: String
    @Published private(set) var metricsEndpoint: String
    @Published private(set) var files: [ObservabilityLogFile] = []
    @Published private(set) var selectedFilePath: String?
    @Published private(set) var selectedFileContent: String = ""
    @Published private(set) var selectedFileSizeBytes: Int64 = 0
    @Published private(set) var selectedFileModifiedAt: Date?
    @Published private(set) var selectedFileEntries: [ObservabilityLogEntry] = []
    @Published var selectedLogEntryID: ObservabilityLogEntry.ID?
    @Published private(set) var statusText: String = "Ready"
    @Published private(set) var isRefreshing: Bool = false
    @Published private(set) var lastRefreshedAt: Date?
    @Published private(set) var metricsStatusText: String = "Ready"
    @Published private(set) var isMetricsRefreshing: Bool = false
    @Published private(set) var metricsLastRefreshedAt: Date?
    @Published private(set) var metricsCycleID: Double?
    @Published private(set) var metricsActDescriptorCatalogCount: Double?
    @Published private(set) var metricsRawExcerpt: String = ""

    var canApplyLogDirectoryPath: Bool {
        let normalized = Self.normalizeDirectoryPath(logDirectoryPathDraft)
        return !normalized.isEmpty && normalized != logDirectoryPath
    }

    var canApplyMetricsEndpoint: Bool {
        let normalized = Self.normalizeMetricsEndpoint(metricsEndpointDraft)
        return !normalized.isEmpty && normalized != metricsEndpoint
    }

    var selectedFileName: String {
        guard let selectedFilePath else { return "No File Selected" }
        return URL(fileURLWithPath: selectedFilePath).lastPathComponent
    }

    var selectedLogEntry: ObservabilityLogEntry? {
        guard let selectedLogEntryID else {
            return nil
        }
        return selectedFileEntries.first(where: { $0.id == selectedLogEntryID })
    }

    private var started = false

    private nonisolated static let maxReadBytes: Int64 = 512 * 1024
    private static let defaultLogDirectoryPath = "~/logs/core"
    private static let defaultMetricsEndpoint = "http://127.0.0.1:9464/metrics"
    private static let logDirectoryPathDefaultsKey = "beluna.apple-universal.log_directory_path"
    private static let logDirectoryBookmarkDefaultsKey = "beluna.apple-universal.log_directory_bookmark"
    private static let metricsEndpointDefaultsKey = "beluna.apple-universal.metrics_endpoint"

    init(logDirectoryPath: String? = nil) {
        let persistedPath = Self.normalizeDirectoryPath(
            UserDefaults.standard.string(forKey: Self.logDirectoryPathDefaultsKey)
        )
        let requestedPath = Self.normalizeDirectoryPath(logDirectoryPath)
        let resolvedPath = requestedPath.isEmpty ? persistedPath : requestedPath
        let initialPath = resolvedPath.isEmpty
            ? Self.normalizeDirectoryPath(Self.defaultLogDirectoryPath)
            : resolvedPath
        let persistedMetricsEndpoint = Self.normalizeMetricsEndpoint(
            UserDefaults.standard.string(forKey: Self.metricsEndpointDefaultsKey)
        )
        let initialMetricsEndpoint = persistedMetricsEndpoint.isEmpty
            ? Self.defaultMetricsEndpoint
            : persistedMetricsEndpoint

        self.logDirectoryPath = initialPath
        self.logDirectoryPathDraft = initialPath
        self.metricsEndpoint = initialMetricsEndpoint
        self.metricsEndpointDraft = initialMetricsEndpoint
    }

    func startIfNeeded() {
        guard !started else {
            return
        }
        started = true
        refresh()
        refreshMetrics()
    }

    func applyLogDirectoryPathDraft() {
        let normalized = Self.normalizeDirectoryPath(logDirectoryPathDraft)
        guard !normalized.isEmpty else {
            statusText = "Log directory path cannot be empty."
            return
        }
        guard normalized != logDirectoryPath else {
            return
        }

        logDirectoryPath = normalized
        logDirectoryPathDraft = normalized
        selectedFilePath = nil
        selectedFileContent = ""
        selectedFileSizeBytes = 0
        selectedFileModifiedAt = nil
        selectedFileEntries = []
        selectedLogEntryID = nil
        persistLogDirectoryPath()
        clearLogDirectoryBookmarkIfMismatched(currentPath: normalized)
        refresh()
    }

    func applyMetricsEndpointDraft() {
        let normalized = Self.normalizeMetricsEndpoint(metricsEndpointDraft)
        guard !normalized.isEmpty else {
            metricsStatusText = "Metrics endpoint cannot be empty."
            return
        }
        guard normalized != metricsEndpoint else {
            return
        }

        metricsEndpoint = normalized
        metricsEndpointDraft = normalized
        persistMetricsEndpoint()
        refreshMetrics()
    }

    #if os(macOS)
    func chooseLogDirectory() {
        let panel = NSOpenPanel()
        panel.title = "Choose Beluna Core Log Folder"
        panel.message = "Select a folder containing Beluna Core logs."
        panel.canChooseFiles = false
        panel.canChooseDirectories = true
        panel.allowsMultipleSelection = false
        panel.canCreateDirectories = true
        panel.prompt = "Choose"
        panel.directoryURL = URL(fileURLWithPath: logDirectoryPath)

        guard panel.runModal() == .OK, let chosenURL = panel.url else {
            return
        }

        do {
            let bookmark = try chosenURL.bookmarkData(
                options: [.withSecurityScope],
                includingResourceValuesForKeys: nil,
                relativeTo: nil
            )
            UserDefaults.standard.set(bookmark, forKey: Self.logDirectoryBookmarkDefaultsKey)
        } catch {
            statusText = "Folder chosen, but bookmark creation failed: \(error.localizedDescription)"
        }

        logDirectoryPath = chosenURL.standardizedFileURL.path
        logDirectoryPathDraft = logDirectoryPath
        selectedFilePath = nil
        selectedFileContent = ""
        selectedFileSizeBytes = 0
        selectedFileModifiedAt = nil
        selectedFileEntries = []
        selectedLogEntryID = nil
        persistLogDirectoryPath()
        refresh()
    }
    #else
    func chooseLogDirectory() {
        statusText = "Folder picker is only supported on macOS."
    }
    #endif

    func refresh() {
        guard !isRefreshing else {
            return
        }

        let directoryPath = logDirectoryPath
        let currentSelectedFilePath = selectedFilePath
        let bookmarkData = UserDefaults.standard.data(
            forKey: Self.logDirectoryBookmarkDefaultsKey
        )

        isRefreshing = true
        statusText = "Refreshing logs..."

        Task.detached(priority: .userInitiated) {
            let snapshot = Self.loadDirectorySnapshot(
                directoryPath: directoryPath,
                selectedFilePath: currentSelectedFilePath,
                bookmarkData: bookmarkData
            )
            await MainActor.run {
                self.apply(snapshot: snapshot, preferredSelectedFilePath: currentSelectedFilePath)
            }
        }
    }

    func refreshMetrics() {
        guard !isMetricsRefreshing else {
            return
        }

        let endpoint = metricsEndpoint
        isMetricsRefreshing = true
        metricsStatusText = "Refreshing metrics..."

        Task.detached(priority: .userInitiated) {
            let snapshot = await Self.loadMetricsSnapshot(endpoint: endpoint)
            await MainActor.run {
                self.apply(metricsSnapshot: snapshot)
            }
        }
    }

    func selectFile(path: String?) {
        guard selectedFilePath != path else {
            return
        }

        selectedFilePath = path
        guard let path else {
            selectedFileContent = ""
            selectedFileSizeBytes = 0
            selectedFileModifiedAt = nil
            selectedFileEntries = []
            selectedLogEntryID = nil
            return
        }

        let directoryPath = logDirectoryPath
        let bookmarkData = UserDefaults.standard.data(
            forKey: Self.logDirectoryBookmarkDefaultsKey
        )

        Task.detached(priority: .userInitiated) {
            let fileContent = Self.loadSingleFileContent(
                filePath: path,
                directoryPath: directoryPath,
                bookmarkData: bookmarkData
            )
            await MainActor.run {
                guard self.selectedFilePath == path else {
                    return
                }
                self.selectedFileContent = fileContent.content
                self.selectedFileSizeBytes = fileContent.sizeBytes
                self.selectedFileModifiedAt = fileContent.modifiedAt
                self.selectedFileEntries = fileContent.entries
                self.selectedLogEntryID = fileContent.entries.first?.id
            }
        }
    }

    private func apply(
        snapshot: DirectorySnapshot,
        preferredSelectedFilePath: String?
    ) {
        isRefreshing = false
        lastRefreshedAt = Date()
        files = snapshot.files
        statusText = snapshot.statusText

        let effectiveSelection = preferredSelectedFilePath.flatMap { requested in
            snapshot.files.first(where: { $0.path == requested })?.path
        } ?? snapshot.files.first?.path

        selectedFilePath = effectiveSelection
        if let selectedFile = snapshot.selectedFile {
            selectedFileContent = selectedFile.content
            selectedFileSizeBytes = selectedFile.sizeBytes
            selectedFileModifiedAt = selectedFile.modifiedAt
            selectedFileEntries = selectedFile.entries
            selectedLogEntryID = selectedFile.entries.first?.id
        } else {
            selectedFileContent = ""
            selectedFileSizeBytes = 0
            selectedFileModifiedAt = nil
            selectedFileEntries = []
            selectedLogEntryID = nil
        }
    }

    private func apply(metricsSnapshot: MetricsSnapshot) {
        isMetricsRefreshing = false
        metricsLastRefreshedAt = Date()
        metricsStatusText = metricsSnapshot.statusText
        metricsCycleID = metricsSnapshot.cycleID
        metricsActDescriptorCatalogCount = metricsSnapshot.actDescriptorCatalogCount
        metricsRawExcerpt = metricsSnapshot.rawExcerpt
    }

    private func persistLogDirectoryPath() {
        UserDefaults.standard.set(logDirectoryPath, forKey: Self.logDirectoryPathDefaultsKey)
    }

    private func persistMetricsEndpoint() {
        UserDefaults.standard.set(metricsEndpoint, forKey: Self.metricsEndpointDefaultsKey)
    }

    private func clearLogDirectoryBookmarkIfMismatched(currentPath: String) {
        guard let bookmarkData = UserDefaults.standard.data(forKey: Self.logDirectoryBookmarkDefaultsKey)
        else {
            return
        }

        var isStale = false
        guard let resolvedURL = try? URL(
            resolvingBookmarkData: bookmarkData,
            options: [.withSecurityScope, .withoutUI],
            relativeTo: nil,
            bookmarkDataIsStale: &isStale
        ) else {
            UserDefaults.standard.removeObject(forKey: Self.logDirectoryBookmarkDefaultsKey)
            return
        }

        if resolvedURL.standardizedFileURL.path != currentPath {
            UserDefaults.standard.removeObject(forKey: Self.logDirectoryBookmarkDefaultsKey)
        }
    }

    private nonisolated static func normalizeDirectoryPath(_ value: String?) -> String {
        let trimmed = (value ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else {
            return ""
        }

        let expanded = (trimmed as NSString).expandingTildeInPath
        return URL(fileURLWithPath: expanded).standardizedFileURL.path
    }

    private nonisolated static func normalizeMetricsEndpoint(_ value: String?) -> String {
        (value ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private nonisolated static func loadDirectorySnapshot(
        directoryPath: String,
        selectedFilePath: String?,
        bookmarkData: Data?
    ) -> DirectorySnapshot {
        let directoryURL = URL(fileURLWithPath: directoryPath).standardizedFileURL
        let access = resolveScopedAccessURL(directoryURL: directoryURL, bookmarkData: bookmarkData)
        defer {
            if access.didStartSecurityScope {
                access.url.stopAccessingSecurityScopedResource()
            }
        }

        do {
            var isDirectory: ObjCBool = false
            guard FileManager.default.fileExists(
                atPath: access.url.path,
                isDirectory: &isDirectory
            ) else {
                return DirectorySnapshot(
                    files: [],
                    selectedFile: nil,
                    statusText: "Log directory does not exist: \(access.url.path)"
                )
            }
            guard isDirectory.boolValue else {
                return DirectorySnapshot(
                    files: [],
                    selectedFile: nil,
                    statusText: "Configured path is not a directory: \(access.url.path)"
                )
            }

            let files = try listLogFiles(in: access.url)
            guard !files.isEmpty else {
                return DirectorySnapshot(
                    files: [],
                    selectedFile: nil,
                    statusText: "No log files found in \(access.url.path)"
                )
            }

            let resolvedSelection = selectedFilePath.flatMap { requested in
                files.first(where: { $0.path == requested })?.path
            } ?? files.first?.path

            let selectedFile = resolvedSelection.map { filePath in
                loadSingleFileContent(filePath: filePath, directoryURL: access.url)
            }

            return DirectorySnapshot(
                files: files,
                selectedFile: selectedFile,
                statusText: "Loaded \(files.count) files from \(access.url.path)"
            )
        } catch {
            return DirectorySnapshot(
                files: [],
                selectedFile: nil,
                statusText: "Failed to read log directory: \(error.localizedDescription)"
            )
        }
    }

    private nonisolated static func loadSingleFileContent(
        filePath: String,
        directoryPath: String,
        bookmarkData: Data?
    ) -> LoadedFileContent {
        let directoryURL = URL(fileURLWithPath: directoryPath).standardizedFileURL
        let access = resolveScopedAccessURL(directoryURL: directoryURL, bookmarkData: bookmarkData)
        defer {
            if access.didStartSecurityScope {
                access.url.stopAccessingSecurityScopedResource()
            }
        }

        return loadSingleFileContent(filePath: filePath, directoryURL: access.url)
    }

    private nonisolated static func loadSingleFileContent(
        filePath: String,
        directoryURL: URL
    ) -> LoadedFileContent {
        let fileURL = URL(fileURLWithPath: filePath).standardizedFileURL
        if fileURL.deletingLastPathComponent().path != directoryURL.path {
            return LoadedFileContent(
                content: "Selected file is outside the configured log directory.",
                sizeBytes: 0,
                modifiedAt: nil,
                entries: []
            )
        }

        do {
            let attributes = try FileManager.default.attributesOfItem(atPath: fileURL.path)
            let sizeBytes = (attributes[.size] as? NSNumber)?.int64Value ?? 0
            let modifiedAt = attributes[.modificationDate] as? Date

            let handle = try FileHandle(forReadingFrom: fileURL)
            defer { try? handle.close() }

            let offset = sizeBytes > maxReadBytes ? sizeBytes - maxReadBytes : 0
            if offset > 0 {
                try handle.seek(toOffset: UInt64(offset))
            } else {
                try handle.seek(toOffset: 0)
            }

            let data = try handle.readToEnd() ?? Data()
            var content = String(decoding: data, as: UTF8.self)
            if offset > 0 {
                content =
                    "[Showing last \(maxReadBytes) bytes of \(sizeBytes) bytes]\n\n" + content
            }
            let entries = parseLogEntries(from: content)

            return LoadedFileContent(
                content: content,
                sizeBytes: sizeBytes,
                modifiedAt: modifiedAt,
                entries: entries
            )
        } catch {
            return LoadedFileContent(
                content: "Failed to read file: \(error.localizedDescription)",
                sizeBytes: 0,
                modifiedAt: nil,
                entries: []
            )
        }
    }

    private nonisolated static let iso8601FormatterWithFractional: ISO8601DateFormatter = {
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        return formatter
    }()

    private nonisolated static let iso8601Formatter: ISO8601DateFormatter = {
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime]
        return formatter
    }()

    private nonisolated static func parseLogEntries(from content: String) -> [ObservabilityLogEntry] {
        let lines = content.split(whereSeparator: \.isNewline)
        var entries: [ObservabilityLogEntry] = []
        entries.reserveCapacity(lines.count)

        for (index, rawLineSlice) in lines.enumerated() {
            let rawLine = String(rawLineSlice).trimmingCharacters(in: .whitespacesAndNewlines)
            guard rawLine.first == "{" else {
                continue
            }
            guard let data = rawLine.data(using: .utf8),
                  let payload = (try? JSONSerialization.jsonObject(with: data))
                    as? [String: Any]
            else {
                continue
            }

            let timestampRaw = payload["timestamp"] as? String ?? ""
            let timestamp = parseTimestamp(timestampRaw)
            let level = (payload["level"] as? String)?.uppercased() ?? "-"
            let target = payload["target"] as? String ?? "-"

            let message: String
            if let fields = payload["fields"] as? [String: Any],
               let fieldMessage = fields["message"] as? String,
               !fieldMessage.isEmpty
            {
                message = fieldMessage
            } else if let topMessage = payload["message"] as? String, !topMessage.isEmpty {
                message = topMessage
            } else {
                message = "-"
            }

            let entry = ObservabilityLogEntry(
                id: "\(timestampRaw)|\(target)|\(level)|\(index)|\(rawLine.hashValue)",
                timestamp: timestamp,
                timestampRaw: timestampRaw,
                level: level,
                target: target,
                message: message,
                rawLine: rawLine,
                lineIndex: index
            )
            entries.append(entry)
        }

        entries.sort { lhs, rhs in
            switch (lhs.timestamp, rhs.timestamp) {
            case let (left?, right?):
                if left != right {
                    return left > right
                }
            case (.some, .none):
                return true
            case (.none, .some):
                return false
            case (.none, .none):
                break
            }
            return lhs.lineIndex > rhs.lineIndex
        }
        return entries
    }

    private nonisolated static func parseTimestamp(_ value: String) -> Date? {
        guard !value.isEmpty else {
            return nil
        }
        if let date = iso8601FormatterWithFractional.date(from: value) {
            return date
        }
        return iso8601Formatter.date(from: value)
    }

    private nonisolated static func listLogFiles(
        in directoryURL: URL
    ) throws -> [ObservabilityLogFile] {
        let resourceKeys: [URLResourceKey] = [
            .isRegularFileKey,
            .contentModificationDateKey,
            .fileSizeKey
        ]

        let candidates = try FileManager.default.contentsOfDirectory(
            at: directoryURL,
            includingPropertiesForKeys: resourceKeys,
            options: [.skipsHiddenFiles]
        )

        var files: [ObservabilityLogFile] = []
        files.reserveCapacity(candidates.count)
        for url in candidates {
            let values = try url.resourceValues(forKeys: Set(resourceKeys))
            guard values.isRegularFile == true else {
                continue
            }
            let file = ObservabilityLogFile(
                path: url.standardizedFileURL.path,
                name: url.lastPathComponent,
                sizeBytes: Int64(values.fileSize ?? 0),
                modifiedAt: values.contentModificationDate
            )
            files.append(file)
        }

        return files.sorted { lhs, rhs in
            let leftDate = lhs.modifiedAt ?? .distantPast
            let rightDate = rhs.modifiedAt ?? .distantPast
            if leftDate != rightDate {
                return leftDate > rightDate
            }
            return lhs.name < rhs.name
        }
    }

    private nonisolated static func loadMetricsSnapshot(endpoint: String) async -> MetricsSnapshot {
        guard let url = URL(string: endpoint) else {
            return MetricsSnapshot(
                cycleID: nil,
                actDescriptorCatalogCount: nil,
                rawExcerpt: "",
                statusText: "Invalid metrics endpoint URL: \(endpoint)"
            )
        }
        guard let scheme = url.scheme?.lowercased(), scheme == "http" || scheme == "https" else {
            return MetricsSnapshot(
                cycleID: nil,
                actDescriptorCatalogCount: nil,
                rawExcerpt: "",
                statusText: "Metrics endpoint must start with http:// or https://."
            )
        }

        var request = URLRequest(url: url)
        request.httpMethod = "GET"
        request.timeoutInterval = 5

        do {
            let (data, response) = try await URLSession.shared.data(for: request)
            let statusCode = (response as? HTTPURLResponse)?.statusCode ?? 0
            guard (200..<300).contains(statusCode) else {
                return MetricsSnapshot(
                    cycleID: nil,
                    actDescriptorCatalogCount: nil,
                    rawExcerpt: "",
                    statusText: "Metrics endpoint returned HTTP \(statusCode)."
                )
            }

            let body = String(decoding: data, as: UTF8.self)
            let cycleID = parsePrometheusGauge(
                named: "beluna_cortex_cycle_id",
                in: body
            )
            let catalogCount = parsePrometheusGauge(
                named: "beluna_cortex_input_ir_act_descriptor_catalog_count",
                in: body
            )
            let excerpt = relevantMetricsExcerpt(from: body)

            let status: String
            if cycleID == nil && catalogCount == nil {
                status = "Metrics fetched, but target gauges were not found."
            } else {
                status = "Metrics loaded from \(endpoint)."
            }

            return MetricsSnapshot(
                cycleID: cycleID,
                actDescriptorCatalogCount: catalogCount,
                rawExcerpt: excerpt,
                statusText: status
            )
        } catch {
            return MetricsSnapshot(
                cycleID: nil,
                actDescriptorCatalogCount: nil,
                rawExcerpt: "",
                statusText: "Failed to fetch metrics: \(error.localizedDescription)"
            )
        }
    }

    private nonisolated static func parsePrometheusGauge(
        named metricName: String,
        in payload: String
    ) -> Double? {
        var latestValue: Double?
        for rawLine in payload.split(whereSeparator: \.isNewline) {
            let line = rawLine.trimmingCharacters(in: .whitespacesAndNewlines)
            if line.isEmpty || line.hasPrefix("#") || !line.hasPrefix(metricName) {
                continue
            }

            let valueText: Substring
            if let closeBrace = line.firstIndex(of: "}") {
                valueText = line[line.index(after: closeBrace)...]
            } else {
                valueText = line.dropFirst(metricName.count)
            }

            let parts = valueText.split(whereSeparator: \.isWhitespace)
            guard let valueLiteral = parts.first, let value = Double(valueLiteral) else {
                continue
            }
            latestValue = value
        }
        return latestValue
    }

    private nonisolated static func relevantMetricsExcerpt(from payload: String) -> String {
        let metricNames = [
            "beluna_cortex_cycle_id",
            "beluna_cortex_input_ir_act_descriptor_catalog_count"
        ]

        let lines = payload
            .split(whereSeparator: \.isNewline)
            .map(String.init)
        let filtered = lines.filter { line in
            metricNames.contains(where: { line.contains($0) })
        }

        if !filtered.isEmpty {
            return filtered.joined(separator: "\n")
        }

        return lines.prefix(80).joined(separator: "\n")
    }

    private nonisolated static func resolveScopedAccessURL(
        directoryURL: URL,
        bookmarkData: Data?
    ) -> ScopedAccessURL {
        guard let bookmarkData else {
            return ScopedAccessURL(url: directoryURL, didStartSecurityScope: false)
        }

        var isStale = false
        guard let resolvedURL = try? URL(
            resolvingBookmarkData: bookmarkData,
            options: [.withSecurityScope, .withoutUI],
            relativeTo: nil,
            bookmarkDataIsStale: &isStale
        ) else {
            return ScopedAccessURL(url: directoryURL, didStartSecurityScope: false)
        }

        let normalizedResolved = resolvedURL.standardizedFileURL
        if normalizedResolved.path != directoryURL.path {
            return ScopedAccessURL(url: directoryURL, didStartSecurityScope: false)
        }

        let didStart = normalizedResolved.startAccessingSecurityScopedResource()
        return ScopedAccessURL(url: normalizedResolved, didStartSecurityScope: didStart)
    }
}

private struct ScopedAccessURL {
    let url: URL
    let didStartSecurityScope: Bool
}

private struct LoadedFileContent {
    let content: String
    let sizeBytes: Int64
    let modifiedAt: Date?
    let entries: [ObservabilityLogEntry]
}

private struct DirectorySnapshot {
    let files: [ObservabilityLogFile]
    let selectedFile: LoadedFileContent?
    let statusText: String
}

private struct MetricsSnapshot {
    let cycleID: Double?
    let actDescriptorCatalogCount: Double?
    let rawExcerpt: String
    let statusText: String
}
