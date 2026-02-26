import Foundation

/// Kind of organ log event extracted from NDJSON log files.
enum OrganLogEventKind: Sendable {
    case input
    case output

    var sortOrder: Int {
        switch self {
        case .input:
            return 0
        case .output:
            return 1
        }
    }
}

/// A single parsed organ log event from an NDJSON log file.
struct OrganLogEvent: Sendable {
    let eventID: String
    let kind: OrganLogEventKind
    let timestamp: Date?
    let cycleID: UInt64
    let awakeSequence: UInt64?
    let stage: String
    let payload: String
}

/// Watches a Beluna Core log directory for changes using `DispatchSource` file system
/// object sources, delivering parsed organ log events incrementally as files are written.
///
/// Replaces the previous polling approach with near-instant event delivery:
/// - Monitors the directory for new/rotated log files via kqueue.
/// - Monitors individual log files for writes, reading only new bytes from tracked offsets.
/// - Falls back to a 5-second retry timer when the directory does not yet exist.
/// - Initial scan reads the latest 2 files, sorts events across files, and delivers them
///   in a single batch to ensure consistent cycle pairing.
/// - Incremental reads buffer partial trailing lines across read boundaries.
final class LogDirectoryWatcher: @unchecked Sendable {
    typealias EventHandler = @Sendable ([OrganLogEvent], String) -> Void

    private let directoryPath: String
    private let onEvents: EventHandler
    private let watchQueue = DispatchQueue(label: "beluna.log-directory-watcher", qos: .utility)

    private var directoryFD: Int32 = -1
    private var directorySource: DispatchSourceFileSystemObject?
    private var watchedFiles: [String: WatchedFile] = [:]
    private var retrySource: DispatchSourceTimer?
    private var isRunning = false
    private var initialScanCompleted = false

    static let coreLogFilePrefix = "core.log"
    private static let maxInitialReadBytes: UInt64 = 512 * 1024

    private struct WatchedFile {
        let url: URL
        let awakeSequence: UInt64?
        var fd: Int32
        var source: DispatchSourceFileSystemObject?
        var offset: UInt64
        var trailingPartialLine: String
    }

    init(directoryPath: String, onEvents: @escaping EventHandler) {
        self.directoryPath = directoryPath
        self.onEvents = onEvents
    }

    deinit {
        teardown()
    }

    /// Begin watching the configured directory.
    func start() {
        watchQueue.async { [self] in
            guard !isRunning else { return }
            isRunning = true
            attemptDirectoryWatch()
        }
    }

    /// Stop watching and release all file descriptors and dispatch sources.
    func stop() {
        watchQueue.async { [self] in
            guard isRunning else { return }
            isRunning = false
            teardown()
        }
    }

    // MARK: - Lifecycle

    private func attemptDirectoryWatch() {
        guard isRunning else { return }

        var isDir: ObjCBool = false
        guard FileManager.default.fileExists(atPath: directoryPath, isDirectory: &isDir),
              isDir.boolValue
        else {
            onEvents([], "Log directory does not exist: \(directoryPath). Retrying...")
            startRetryTimer()
            return
        }

        cancelRetryTimer()
        setupDirectoryWatch()
    }

    private func setupDirectoryWatch() {
        let fd = Darwin.open(directoryPath, O_EVTONLY)
        guard fd >= 0 else {
            onEvents([], "Cannot open log directory: \(directoryPath)")
            return
        }

        directoryFD = fd

        let source = DispatchSource.makeFileSystemObjectSource(
            fileDescriptor: fd,
            eventMask: [.write],
            queue: watchQueue
        )

        source.setEventHandler { [weak self] in
            self?.scanDirectory()
        }

        directorySource = source
        source.resume()

        scanDirectory()
    }

    private func startRetryTimer() {
        cancelRetryTimer()

        let timer = DispatchSource.makeTimerSource(queue: watchQueue)
        timer.schedule(deadline: .now() + 5, repeating: 5.0, leeway: .seconds(1))
        timer.setEventHandler { [weak self] in
            self?.attemptDirectoryWatch()
        }
        retrySource = timer
        timer.resume()
    }

    private func cancelRetryTimer() {
        retrySource?.cancel()
        retrySource = nil
    }

    private func teardown() {
        cancelRetryTimer()

        directorySource?.cancel()
        directorySource = nil
        if directoryFD >= 0 {
            Darwin.close(directoryFD)
            directoryFD = -1
        }

        for (_, file) in watchedFiles {
            file.source?.cancel()
            if file.fd >= 0 {
                Darwin.close(file.fd)
            }
        }
        watchedFiles.removeAll()
    }

    // MARK: - Directory Scanning

    private func scanDirectory() {
        guard isRunning else { return }

        let directoryURL = URL(fileURLWithPath: directoryPath)
        let files: [LogFileMetadata]
        do {
            files = try Self.listLogFiles(in: directoryURL)
        } catch {
            onEvents([], "Failed to list log files: \(error.localizedDescription)")
            return
        }

        guard !files.isEmpty else {
            onEvents([], "No log files found in \(directoryPath)")
            return
        }

        let candidates = Array(files.suffix(2))
        let candidatePaths = Set(candidates.map(\.url.path))

        // Stop watching files that are no longer among the latest candidates.
        for path in watchedFiles.keys where !candidatePaths.contains(path) {
            if let file = watchedFiles.removeValue(forKey: path) {
                file.source?.cancel()
                if file.fd >= 0 {
                    Darwin.close(file.fd)
                }
            }
        }

        if !initialScanCompleted {
            performInitialScan(candidates: candidates)
            return
        }

        // Incremental mode: start or update watches for candidate files.
        for metadata in candidates {
            let path = metadata.url.path
            if watchedFiles[path] != nil {
                readNewContent(for: path)
            } else {
                startWatchingFile(metadata)
            }
        }
    }

    /// Reads all candidate files in one pass, sorts events across files, and delivers
    /// a single batch to the callback. Then sets up per-file watches for incremental delivery.
    private func performInitialScan(candidates: [LogFileMetadata]) {
        initialScanCompleted = true

        var allEvents: [OrganLogEvent] = []
        allEvents.reserveCapacity(512)

        for metadata in candidates {
            let path = metadata.url.path
            guard watchedFiles[path] == nil else { continue }

            let fd = Darwin.open(path, O_EVTONLY)
            guard fd >= 0 else { continue }

            let fileSize: UInt64
            do {
                let attrs = try FileManager.default.attributesOfItem(atPath: path)
                fileSize = (attrs[.size] as? NSNumber)?.uint64Value ?? 0
            } catch {
                Darwin.close(fd)
                continue
            }

            let initialOffset = fileSize > Self.maxInitialReadBytes
                ? fileSize - Self.maxInitialReadBytes
                : 0

            // Read from initialOffset to current end.
            var readOffset = initialOffset
            do {
                let handle = try FileHandle(forReadingFrom: metadata.url)
                defer { try? handle.close() }
                try handle.seek(toOffset: initialOffset)
                if let data = try handle.readToEnd(), !data.isEmpty {
                    readOffset = initialOffset + UInt64(data.count)
                    let content = String(decoding: data, as: UTF8.self)

                    // For the initial scan, separate the trailing partial line so we
                    // don't lose an event that spans into the next incremental read.
                    let (completeContent, trailing) = Self.splitTrailingPartialLine(content)

                    let events = Self.parseOrganLogEvents(
                        from: completeContent,
                        sourcePath: path,
                        sourceAwakeSequence: metadata.awakeSequence
                    )
                    allEvents.append(contentsOf: events)

                    // Set up the watch entry with the trailing partial line buffered.
                    let source = DispatchSource.makeFileSystemObjectSource(
                        fileDescriptor: fd,
                        eventMask: [.write, .extend, .delete, .rename],
                        queue: watchQueue
                    )

                    let watched = WatchedFile(
                        url: metadata.url,
                        awakeSequence: metadata.awakeSequence,
                        fd: fd,
                        source: source,
                        offset: readOffset,
                        trailingPartialLine: trailing
                    )
                    watchedFiles[path] = watched

                    source.setEventHandler { [weak self] in
                        guard let self else { return }
                        let flags = source.data
                        if flags.contains(.delete) || flags.contains(.rename) {
                            self.scanDirectory()
                        } else {
                            self.readNewContent(for: path)
                        }
                    }
                    source.resume()
                    continue
                }
            } catch {
                // Fall through to set up empty watch.
            }

            // File was empty or unreadable; still set up the watch for future writes.
            let source = DispatchSource.makeFileSystemObjectSource(
                fileDescriptor: fd,
                eventMask: [.write, .extend, .delete, .rename],
                queue: watchQueue
            )

            let watched = WatchedFile(
                url: metadata.url,
                awakeSequence: metadata.awakeSequence,
                fd: fd,
                source: source,
                offset: readOffset,
                trailingPartialLine: ""
            )
            watchedFiles[path] = watched

            source.setEventHandler { [weak self] in
                guard let self else { return }
                let flags = source.data
                if flags.contains(.delete) || flags.contains(.rename) {
                    self.scanDirectory()
                } else {
                    self.readNewContent(for: path)
                }
            }
            source.resume()
        }

        // Sort events across all files — matches the old polling behavior.
        allEvents.sort { lhs, rhs in
            let leftTimestamp = lhs.timestamp ?? .distantPast
            let rightTimestamp = rhs.timestamp ?? .distantPast
            if leftTimestamp != rightTimestamp {
                return leftTimestamp < rightTimestamp
            }
            if lhs.cycleID != rhs.cycleID {
                return lhs.cycleID < rhs.cycleID
            }
            if lhs.awakeSequence != rhs.awakeSequence {
                return (lhs.awakeSequence ?? 0) < (rhs.awakeSequence ?? 0)
            }
            if lhs.stage != rhs.stage {
                return lhs.stage < rhs.stage
            }
            return lhs.kind.sortOrder < rhs.kind.sortOrder
        }

        let fileCount = candidates.count
        onEvents(
            allEvents,
            "Loaded \(allEvents.count) organ log events from \(fileCount) file(s)."
        )
    }

    // MARK: - File Watching

    private func startWatchingFile(_ metadata: LogFileMetadata) {
        let path = metadata.url.path
        let fd = Darwin.open(path, O_EVTONLY)
        guard fd >= 0 else { return }

        let fileSize: UInt64
        do {
            let attrs = try FileManager.default.attributesOfItem(atPath: path)
            fileSize = (attrs[.size] as? NSNumber)?.uint64Value ?? 0
        } catch {
            Darwin.close(fd)
            return
        }

        // For post-initial files, read last 512KB to pick up recent history.
        let initialOffset = fileSize > Self.maxInitialReadBytes
            ? fileSize - Self.maxInitialReadBytes
            : 0

        let source = DispatchSource.makeFileSystemObjectSource(
            fileDescriptor: fd,
            eventMask: [.write, .extend, .delete, .rename],
            queue: watchQueue
        )

        let watched = WatchedFile(
            url: metadata.url,
            awakeSequence: metadata.awakeSequence,
            fd: fd,
            source: source,
            offset: initialOffset,
            trailingPartialLine: ""
        )

        watchedFiles[path] = watched

        source.setEventHandler { [weak self] in
            guard let self else { return }
            let flags = source.data
            if flags.contains(.delete) || flags.contains(.rename) {
                self.scanDirectory()
            } else {
                self.readNewContent(for: path)
            }
        }

        source.resume()

        readNewContent(for: path)
    }

    private func readNewContent(for path: String) {
        guard isRunning, var watched = watchedFiles[path] else { return }

        do {
            let attrs = try FileManager.default.attributesOfItem(atPath: path)
            let currentSize = (attrs[.size] as? NSNumber)?.uint64Value ?? 0

            // Detect file truncation (rotation).
            if currentSize < watched.offset {
                watched.offset = 0
                watched.trailingPartialLine = ""
            }

            guard currentSize > watched.offset else {
                watchedFiles[path] = watched
                return
            }

            let handle = try FileHandle(forReadingFrom: watched.url)
            defer { try? handle.close() }

            try handle.seek(toOffset: watched.offset)
            guard let data = try handle.readToEnd(), !data.isEmpty else {
                return
            }

            let newOffset = watched.offset + UInt64(data.count)
            watched.offset = newOffset

            // Prepend any buffered trailing partial line from the previous read.
            let rawContent = String(decoding: data, as: UTF8.self)
            let content: String
            if watched.trailingPartialLine.isEmpty {
                content = rawContent
            } else {
                content = watched.trailingPartialLine + rawContent
            }

            // Separate trailing partial line for the next read.
            let (completeContent, trailing) = Self.splitTrailingPartialLine(content)
            watched.trailingPartialLine = trailing
            watchedFiles[path] = watched

            let events = Self.parseOrganLogEvents(
                from: completeContent,
                sourcePath: path,
                sourceAwakeSequence: watched.awakeSequence
            )

            if !events.isEmpty {
                onEvents(
                    events,
                    "Watching \(watched.url.lastPathComponent) — \(events.count) new events"
                )
            }
        } catch {
            // File may have been deleted or rotated; next directory scan will clean up.
        }
    }

    /// Splits content into complete lines and a trailing partial line (if any).
    /// A trailing partial line is content after the last newline that hasn't been
    /// terminated yet (the writer may still be flushing).
    private static func splitTrailingPartialLine(_ content: String) -> (complete: String, trailing: String) {
        guard let lastNewlineIndex = content.lastIndex(where: { $0.isNewline }) else {
            // No newline at all — entire content is a partial line.
            return ("", content)
        }

        let afterNewline = content.index(after: lastNewlineIndex)
        if afterNewline == content.endIndex {
            // Content ends with a newline — no trailing partial.
            return (content, "")
        }

        let complete = String(content[content.startIndex...lastNewlineIndex])
        let trailing = String(content[afterNewline...])
        return (complete, trailing)
    }

    // MARK: - File Listing

    private struct LogFileMetadata {
        let url: URL
        let modifiedAt: Date?
        let awakeSequence: UInt64?
    }

    private static func listLogFiles(in directoryURL: URL) throws -> [LogFileMetadata] {
        let resourceKeys: [URLResourceKey] = [
            .isRegularFileKey,
            .contentModificationDateKey,
        ]

        let entries = try FileManager.default.contentsOfDirectory(
            at: directoryURL,
            includingPropertiesForKeys: resourceKeys,
            options: [.skipsHiddenFiles]
        )

        var files: [LogFileMetadata] = []
        files.reserveCapacity(entries.count)

        for url in entries {
            let values = try url.resourceValues(forKeys: Set(resourceKeys))
            guard values.isRegularFile == true else {
                continue
            }
            let fileName = url.lastPathComponent
            guard fileName.hasPrefix(coreLogFilePrefix) else {
                continue
            }

            files.append(
                LogFileMetadata(
                    url: url.standardizedFileURL,
                    modifiedAt: values.contentModificationDate,
                    awakeSequence: parseAwakeSequence(from: fileName)
                )
            )
        }

        files.sort { lhs, rhs in
            let leftDate = lhs.modifiedAt ?? .distantPast
            let rightDate = rhs.modifiedAt ?? .distantPast
            if leftDate != rightDate {
                return leftDate < rightDate
            }
            return lhs.url.lastPathComponent < rhs.url.lastPathComponent
        }

        return files
    }

    // MARK: - NDJSON Parsing

    static func parseOrganLogEvents(
        from content: String,
        sourcePath: String,
        sourceAwakeSequence: UInt64?
    ) -> [OrganLogEvent] {
        var events: [OrganLogEvent] = []

        for rawLineSlice in content.split(whereSeparator: \.isNewline) {
            let rawLine = String(rawLineSlice).trimmingCharacters(in: .whitespacesAndNewlines)
            guard rawLine.first == "{" else {
                continue
            }
            guard let data = rawLine.data(using: .utf8),
                  let payload = (try? JSONSerialization.jsonObject(with: data)) as? [String: Any],
                  let fields = payload["fields"] as? [String: Any],
                  let message = fields["message"] as? String
            else {
                continue
            }

            let kind: OrganLogEventKind
            let payloadField: String
            switch message {
            case "cortex_organ_input":
                kind = .input
                payloadField = "input_payload"
            case "cortex_organ_output":
                kind = .output
                payloadField = "output_payload"
            default:
                continue
            }

            guard let cycleID = parseUInt64(fields["cycle_id"]),
                  let stage = fields["stage"] as? String,
                  !stage.isEmpty,
                  let eventPayload = stringifyLogField(fields[payloadField])
            else {
                continue
            }

            let timestampRaw = payload["timestamp"] as? String ?? ""
            let awakePart = sourceAwakeSequence.map(String.init) ?? "unknown"
            let eventID =
                "\(sourcePath)|\(awakePart)|\(timestampRaw)|\(message)|\(cycleID)|\(stage)|\(eventPayload.hashValue)"

            events.append(
                OrganLogEvent(
                    eventID: eventID,
                    kind: kind,
                    timestamp: parseTimestamp(timestampRaw),
                    cycleID: cycleID,
                    awakeSequence: sourceAwakeSequence,
                    stage: stage,
                    payload: eventPayload
                )
            )
        }

        return events
    }

    // MARK: - Value Parsing Utilities

    private static func parseUInt64(_ value: Any?) -> UInt64? {
        switch value {
        case let number as NSNumber:
            return number.uint64Value
        case let string as String:
            return UInt64(string)
        case let intValue as Int where intValue >= 0:
            return UInt64(intValue)
        case let int64Value as Int64 where int64Value >= 0:
            return UInt64(int64Value)
        case let uintValue as UInt:
            return UInt64(uintValue)
        case let uint64Value as UInt64:
            return uint64Value
        default:
            return nil
        }
    }

    private static func stringifyLogField(_ value: Any?) -> String? {
        guard let value else {
            return nil
        }

        if let text = value as? String {
            return text
        }

        if JSONSerialization.isValidJSONObject(value),
           let data = try? JSONSerialization.data(withJSONObject: value, options: [.prettyPrinted]),
           let text = String(data: data, encoding: .utf8)
        {
            return text
        }

        return String(describing: value)
    }

    private static func parseTimestamp(_ value: String) -> Date? {
        guard !value.isEmpty else {
            return nil
        }

        let formatterWithFractional = ISO8601DateFormatter()
        formatterWithFractional.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        if let date = formatterWithFractional.date(from: value) {
            return date
        }

        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime]
        return formatter.date(from: value)
    }

    static func parseAwakeSequence(from fileName: String) -> UInt64? {
        let prefix = "\(coreLogFilePrefix)."
        guard fileName.hasPrefix(prefix) else {
            return nil
        }
        let suffix = fileName.dropFirst(prefix.count)
        let parts = suffix.split(separator: ".", omittingEmptySubsequences: false)
        guard parts.count == 2 else {
            return nil
        }
        let datePart = String(parts[0])
        let sequencePart = String(parts[1])
        guard isDateLiteral(datePart), let sequence = UInt64(sequencePart), sequence > 0 else {
            return nil
        }
        return sequence
    }

    private static func isDateLiteral(_ value: String) -> Bool {
        let bytes = Array(value.utf8)
        guard bytes.count == 10 else {
            return false
        }
        return isASCIIDigit(bytes[0])
            && isASCIIDigit(bytes[1])
            && isASCIIDigit(bytes[2])
            && isASCIIDigit(bytes[3])
            && bytes[4] == 45
            && isASCIIDigit(bytes[5])
            && isASCIIDigit(bytes[6])
            && bytes[7] == 45
            && isASCIIDigit(bytes[8])
            && isASCIIDigit(bytes[9])
    }

    private static func isASCIIDigit(_ value: UInt8) -> Bool {
        value >= 48 && value <= 57
    }
}
