import Foundation

@MainActor
final class ChatViewModel: ObservableObject {
    @Published var messages: [ChatMessage] = []
    @Published var draft: String = ""
    @Published var connectionState: ConnectionState = .disconnected
    @Published var socketPathDraft: String
    @Published private(set) var socketPath: String
    @Published private(set) var isConnectionEnabled: Bool

    var isSleeping: Bool {
        connectionState != .connected
    }

    var canSend: Bool {
        connectionState == .connected
    }

    var canApplySocketPath: Bool {
        let normalized = Self.normalizeSocketPath(socketPathDraft)
        return !normalized.isEmpty && normalized != socketPath
    }

    var connectButtonTitle: String {
        isConnectionEnabled ? "Disconnect" : "Connect"
    }

    var sleepingTitle: String {
        isConnectionEnabled ? "Beluna is sleeping" : "Beluna is disconnected"
    }

    var sleepingHint: String {
        isConnectionEnabled
            ? "Start Beluna Core to wake it up."
            : "Click Connect to reconnect."
    }

    private let conversationID: String
    private let spineBodyEndpoint: SpineUnixSocketBodyEndpoint
    private let sleepingNoticeText = "Beluna is sleeping."
    private let disconnectedNoticeText = "Beluna is disconnected. Click Connect to reconnect."
    private var started = false
    private static let defaultSocketPath = "/tmp/beluna.sock"
    private static let socketPathDefaultsKey = "beluna.apple-universal.socket_path"
    private static let autoConnectDefaultsKey = "beluna.apple-universal.auto_connect"

    init(socketPath: String? = nil) {
        let persistedSocketPath = Self.normalizeSocketPath(
            UserDefaults.standard.string(forKey: Self.socketPathDefaultsKey)
        )
        let requestedSocketPath = Self.normalizeSocketPath(socketPath)
        let resolvedSocketPath = requestedSocketPath.isEmpty
            ? persistedSocketPath
            : requestedSocketPath
        let initialSocketPath = resolvedSocketPath.isEmpty
            ? Self.defaultSocketPath
            : resolvedSocketPath
        let initialAutoConnect = UserDefaults.standard.object(forKey: Self.autoConnectDefaultsKey)
            as? Bool ?? true

        self.conversationID = "conv_\(UUID().uuidString.lowercased())"
        self.spineBodyEndpoint = SpineUnixSocketBodyEndpoint(socketPath: initialSocketPath)
        self.socketPath = initialSocketPath
        self.socketPathDraft = initialSocketPath
        self.isConnectionEnabled = initialAutoConnect

        messages.append(
            ChatMessage(
                role: .system,
                text: initialAutoConnect ? sleepingHelpText() : disconnectedNoticeText
            )
        )

        persistConnectionSettings()
        bindSocketHandlers()
    }

    deinit {
        let bodyEndpoint = spineBodyEndpoint
        Task {
            await bodyEndpoint.stop()
        }
    }

    func startIfNeeded() {
        guard !started else {
            return
        }

        started = true

        guard isConnectionEnabled else {
            log("startup with connection disabled")
            return
        }

        log("startup connect to \(socketPath)")
        connectInternal(announce: false)
    }

    func applySocketPathDraft() {
        let normalized = Self.normalizeSocketPath(socketPathDraft)
        guard !normalized.isEmpty else {
            return
        }
        guard normalized != socketPath else {
            return
        }

        socketPath = normalized
        socketPathDraft = normalized
        persistConnectionSettings()
        appendSystemMessage("Socket path set to \(normalized)")
        log("socket path updated to \(normalized)")
        reconnectForCurrentSettings()
    }

    func toggleConnection() {
        if isConnectionEnabled {
            disconnect()
        } else {
            connect()
        }
    }

    func connect() {
        guard !isConnectionEnabled else {
            return
        }

        isConnectionEnabled = true
        persistConnectionSettings()
        log("manual connect to \(socketPath)")
        connectInternal(announce: true)
    }

    func disconnect() {
        guard isConnectionEnabled else {
            return
        }

        isConnectionEnabled = false
        persistConnectionSettings()
        log("manual disconnect")
        disconnectInternal(announce: true)
    }

    private func connectInternal(announce: Bool) {
        if announce {
            appendSystemMessage("Connecting to \(socketPath)...")
        }

        Task {
            await spineBodyEndpoint.start()
        }
    }

    private func disconnectInternal(announce: Bool) {
        connectionState = .disconnected
        if announce {
            appendSystemMessage(disconnectedNoticeText)
        }

        Task {
            await spineBodyEndpoint.stop()
        }
    }

    private func reconnectForCurrentSettings() {
        let shouldConnect = isConnectionEnabled
        let updatedSocketPath = socketPath
        connectionState = .disconnected

        Task {
            await spineBodyEndpoint.stop()
            await spineBodyEndpoint.updateSocketPath(updatedSocketPath)
            if shouldConnect {
                await spineBodyEndpoint.start()
            }
        }
    }

    func sendCurrentDraft() {
        let text = draft.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty else {
            return
        }

        guard canSend else {
            appendSystemMessage(isConnectionEnabled ? sleepingHelpText() : disconnectedNoticeText)
            return
        }

        draft = ""
        messages.append(ChatMessage(role: .user, text: text))

        Task {
            do {
                try await spineBodyEndpoint.sendUserSense(conversationID: conversationID, text: text)
            } catch {
                appendSystemMessage("Failed to send user message to core: \(error.localizedDescription)")
            }
        }
    }

    private func bindSocketHandlers() {
        Task { [weak self] in
            guard let self else {
                return
            }

            await spineBodyEndpoint.setHandlers(
                onStateChange: { [weak self] state in
                    Task { @MainActor in
                        self?.handleConnectionStateChange(state)
                    }
                },
                onServerMessage: { [weak self] message in
                    Task { @MainActor in
                        await self?.handleServerMessage(message)
                    }
                }
            )
        }
    }

    private func handleServerMessage(_ message: ServerWireMessage) async {
        switch message {
        case let .act(action):
            await handleAct(action)
        }
    }

    private func handleAct(_ action: AdmittedActionWire) async {
        guard action.endpointID == appleEndpointID,
              action.capabilityID == appleCapabilityID
        else {
            await rejectInvoke(action: action, reasonCode: "unsupported_route")
            return
        }

        do {
            let texts = try extractAssistantTexts(from: action.normalizedPayload)
            if texts.isEmpty {
                await rejectInvoke(action: action, reasonCode: "invalid_payload")
                appendSystemMessage("Received chat invoke with empty assistant output.")
                return
            }

            for text in texts {
                messages.append(ChatMessage(role: .assistant, text: text))
            }

            try await spineBodyEndpoint.sendActResultSense(
                action: action,
                status: "applied",
                referenceID: "apple-universal:chat:\(action.neuralSignalID)"
            )
        } catch {
            await rejectInvoke(action: action, reasonCode: "invalid_payload")
            appendSystemMessage("Failed to decode assistant payload: \(error.localizedDescription)")
        }
    }

    private func rejectInvoke(action: AdmittedActionWire, reasonCode: String) async {
        do {
            try await spineBodyEndpoint.sendActResultSense(
                action: action,
                status: "rejected",
                referenceID: "apple-universal:chat:reject:\(action.neuralSignalID)",
                reasonCode: reasonCode
            )
        } catch {
            appendSystemMessage("Failed to send invoke result sense: \(error.localizedDescription)")
        }
    }

    private func handleConnectionStateChange(_ state: ConnectionState) {
        if !isConnectionEnabled && state == .connected {
            Task {
                await spineBodyEndpoint.stop()
            }
            connectionState = .disconnected
            log("received connected state while disabled; forced stop")
            return
        }

        let previousState = connectionState
        connectionState = state
        if previousState != state {
            log("state \(previousState.rawValue) -> \(state.rawValue)")
        }

        if previousState != .connected, state == .connected {
            appendSystemMessage("Beluna is awake.")
            return
        }

        if previousState == .connected, state == .disconnected, isConnectionEnabled {
            appendSystemMessage(sleepingNoticeText)
        }
    }

    private func persistConnectionSettings() {
        UserDefaults.standard.set(socketPath, forKey: Self.socketPathDefaultsKey)
        UserDefaults.standard.set(isConnectionEnabled, forKey: Self.autoConnectDefaultsKey)
    }

    private static func normalizeSocketPath(_ value: String?) -> String {
        (value ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private func sleepingHelpText() -> String {
        "Beluna is sleeping. Start Beluna Core to wake it up."
    }

    private func log(_ message: String) {
        fputs("[BelunaAppleUniversalApp] \(message)\n", stderr)
    }

    private func appendSystemMessage(_ text: String) {
        if let last = messages.last, last.role == .system, last.text == text {
            return
        }

        messages.append(ChatMessage(role: .system, text: text))
    }
}
