import Foundation

private struct LocalSenseActHistorySnapshot: Codable {
    let schemaVersion: Int
    let updatedAt: Date
    let messages: [ChatMessage]
}

final class LocalSenseActHistoryStore {
    private static let schemaVersion = 1
    private let fileManager: FileManager
    private let fileURL: URL

    init(fileManager: FileManager = .default) {
        self.fileManager = fileManager
        self.fileURL = Self.resolveHistoryFileURL(fileManager: fileManager)
    }

    func load(maxCount: Int) -> [ChatMessage] {
        guard let data = try? Data(contentsOf: fileURL) else {
            return []
        }

        let decoder = JSONDecoder()
        decoder.dateDecodingStrategy = .iso8601
        guard let snapshot = try? decoder.decode(LocalSenseActHistorySnapshot.self, from: data),
              snapshot.schemaVersion == Self.schemaVersion
        else {
            return []
        }

        guard maxCount > 0, snapshot.messages.count > maxCount else {
            return snapshot.messages
        }
        return Array(snapshot.messages.suffix(maxCount))
    }

    func save(messages: [ChatMessage], maxCount: Int) {
        let trimmedMessages: [ChatMessage]
        if maxCount > 0, messages.count > maxCount {
            trimmedMessages = Array(messages.suffix(maxCount))
        } else {
            trimmedMessages = messages
        }

        let snapshot = LocalSenseActHistorySnapshot(
            schemaVersion: Self.schemaVersion,
            updatedAt: Date(),
            messages: trimmedMessages
        )

        do {
            let encoder = JSONEncoder()
            encoder.dateEncodingStrategy = .iso8601
            let data = try encoder.encode(snapshot)
            try Self.ensureParentDirectoryExists(for: fileURL, fileManager: fileManager)
            try data.write(to: fileURL, options: [.atomic])
        } catch {
            fputs("[BelunaApp] failed to persist local sense/act history: \(error.localizedDescription)\n", stderr)
        }
    }

    func clear() {
        do {
            if fileManager.fileExists(atPath: fileURL.path) {
                try fileManager.removeItem(at: fileURL)
            }
        } catch {
            fputs("[BelunaApp] failed to clear local sense/act history: \(error.localizedDescription)\n", stderr)
        }
    }

    private static func resolveHistoryFileURL(fileManager: FileManager) -> URL {
        let appSupportDirectory = (try? fileManager.url(
            for: .applicationSupportDirectory,
            in: .userDomainMask,
            appropriateFor: nil,
            create: true
        )) ?? URL(fileURLWithPath: NSTemporaryDirectory(), isDirectory: true)

        return appSupportDirectory
            .appendingPathComponent("Beluna", isDirectory: true)
            .appendingPathComponent("AppleUniversal", isDirectory: true)
            .appendingPathComponent("local-sense-act-history.json", isDirectory: false)
    }

    private static func ensureParentDirectoryExists(for fileURL: URL, fileManager: FileManager) throws {
        let parentURL = fileURL.deletingLastPathComponent()
        var isDirectory: ObjCBool = false

        if fileManager.fileExists(atPath: parentURL.path, isDirectory: &isDirectory) {
            if isDirectory.boolValue {
                return
            }
            try fileManager.removeItem(at: parentURL)
        }

        try fileManager.createDirectory(at: parentURL, withIntermediateDirectories: true)
    }
}
