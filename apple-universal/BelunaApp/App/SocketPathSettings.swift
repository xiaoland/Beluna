import Foundation

struct SocketPathSettings {
    static let defaultSocketPath = "/tmp/beluna.sock"

    private static let socketPathDefaultsKey = "beluna.apple-universal.socket_path"
    private static let autoConnectDefaultsKey = "beluna.apple-universal.auto_connect"

    let socketPath: String
    let isConnectionEnabled: Bool

    static func load(
        requestedSocketPath: String?,
        userDefaults: UserDefaults = .standard
    ) -> SocketPathSettings {
        let persistedSocketPath = normalize(
            userDefaults.string(forKey: socketPathDefaultsKey)
        )
        let requestedSocketPath = normalize(requestedSocketPath)
        let resolvedSocketPath = requestedSocketPath.isEmpty ? persistedSocketPath : requestedSocketPath
        let initialSocketPath = resolvedSocketPath.isEmpty ? defaultSocketPath : resolvedSocketPath

        let persistedAutoConnect = userDefaults.object(forKey: autoConnectDefaultsKey) as? Bool
        let initialAutoConnect = persistedAutoConnect ?? !AppRuntimeEnvironment.isXcodeSession

        return SocketPathSettings(
            socketPath: initialSocketPath,
            isConnectionEnabled: initialAutoConnect
        )
    }

    static func normalize(_ value: String?) -> String {
        (value ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
    }

    static func persist(
        socketPath: String,
        isConnectionEnabled: Bool,
        userDefaults: UserDefaults = .standard
    ) {
        userDefaults.set(socketPath, forKey: socketPathDefaultsKey)
        userDefaults.set(isConnectionEnabled, forKey: autoConnectDefaultsKey)
    }
}
