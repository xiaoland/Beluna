import Foundation

@MainActor
final class ChatViewModel: ObservableObject {
    @Published var messages: [ChatMessage] = []
    @Published var draft: String = ""
    @Published var connectionState: ConnectionState = .disconnected
    @Published var belunaState: BelunaState = .unknown
    @Published var socketPathDraft: String
    @Published private(set) var socketPath: String
    @Published private(set) var isConnectionEnabled: Bool

    var isSleeping: Bool {
        belunaState == .sleeping
    }

    var canSend: Bool {
        isConnectionEnabled && connectionState == .connected
    }

    var canApplySocketPath: Bool {
        let normalized = Self.normalizeSocketPath(socketPathDraft)
        return !normalized.isEmpty && normalized != socketPath
    }

    var connectButtonTitle: String {
        isConnectionEnabled ? "Disconnect" : "Connect"
    }

    var canRetry: Bool {
        isConnectionEnabled && connectionState != .connected
    }

    var sleepingTitle: String {
        switch belunaState {
        case .awake:
            return "Beluna is awake"
        case .sleeping:
            return "Beluna is sleeping"
        case .unknown:
            return isConnectionEnabled ? "Beluna status unknown" : "Beluna is disconnected"
        }
    }

    var sleepingHint: String {
        if !isConnectionEnabled {
            return "Click Connect to reconnect."
        }

        switch connectionState {
        case .connected:
            return "Beluna Core is connected."
        case .connecting:
            return "Connecting..."
        case .disconnected:
            return "Retry to reconnect."
        }
    }

    private let conversationID: String
    private let spineBodyEndpoint: SpineUnixSocketBodyEndpoint
    private let sleepingNoticeText = "Beluna is sleeping."
    private let disconnectedNoticeText = "Beluna is disconnected. Click Connect to reconnect."
    private var started = false
    private var hasEverConnected = false
    private var handledActionIDs = Set<String>()
    private var handledActionOrder: [String] = []
    private let handledActionLimit = 256
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
        let persistedAutoConnect = UserDefaults.standard.object(forKey: Self.autoConnectDefaultsKey)
            as? Bool
        let initialAutoConnect = persistedAutoConnect
            ?? !AppRuntimeEnvironment.isXcodeSession

        self.conversationID = "conv_\(UUID().uuidString.lowercased())"
        self.spineBodyEndpoint = SpineUnixSocketBodyEndpoint(socketPath: initialSocketPath)
        self.socketPath = initialSocketPath
        self.socketPathDraft = initialSocketPath
        self.isConnectionEnabled = initialAutoConnect

        messages.append(
            ChatMessage(
                role: .system,
                text: initialMessageText(initialAutoConnect: initialAutoConnect)
            )
        )

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
        reconnectForCurrentSettings(announce: true)
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

    func retryConnection() {
        guard isConnectionEnabled else {
            connect()
            return
        }

        appendSystemMessage("Manual retry...")
        log("manual retry")
        reconnectForCurrentSettings(announce: false)
    }

    private func connectInternal(announce: Bool) {
        belunaState = .unknown
        if announce {
            appendSystemMessage("Connecting to \(socketPath)...")
        }

        Task {
            await spineBodyEndpoint.start()
        }
    }

    private func disconnectInternal(announce: Bool) {
        connectionState = .disconnected
        belunaState = .unknown
        if announce {
            appendSystemMessage(disconnectedNoticeText)
        }

        Task {
            await spineBodyEndpoint.stop()
        }
    }

    private func reconnectForCurrentSettings(announce: Bool) {
        let shouldConnect = isConnectionEnabled
        let updatedSocketPath = socketPath
        connectionState = .disconnected
        belunaState = .unknown

        Task {
            await spineBodyEndpoint.stop()
            await spineBodyEndpoint.updateSocketPath(updatedSocketPath)
            if shouldConnect {
                if announce {
                    appendSystemMessage("Connecting to \(updatedSocketPath)...")
                }
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
                appendSystemMessage("Failed to send user message to core: \(describeError(error))")
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
                },
                onDebug: { [weak self] debugText in
                    Task { @MainActor in
                        self?.appendDebugMessage(debugText)
                    }
                }
            )
        }
    }

    private func handleServerMessage(_ message: ServerWireMessage) async {
        switch message {
        case let .act(action):
            await handleAct(action)
        case .ignored:
            break
        }
    }

    private func handleAct(_ action: InboundActWire) async {
        guard action.neuralSignalDescriptorID == appleActNeuralSignalDescriptorID else {
            appendDebugMessage(
                "Ignored act for unexpected descriptor \(action.neuralSignalDescriptorID) on endpoint \(action.endpointID)"
            )
            return
        }

        guard rememberHandledAction(action.actID) else {
            appendDebugMessage("Ignored duplicate act \(action.actID)")
            return
        }

        do {
            let texts = try extractAssistantTexts(from: action.payload)
            if texts.isEmpty {
                await rejectInvoke(action: action, reasonCode: "invalid_payload")
                appendSystemMessage("Received chat invoke with empty assistant output.")
                log("invalid assistant payload (empty text): \(action.payload)")
                return
            }

            for text in texts {
                messages.append(ChatMessage(role: .assistant, text: text))
            }

            try await spineBodyEndpoint.sendActResultSense(
                action: action,
                status: "applied",
                referenceID: "apple-universal:chat:\(action.actID)"
            )
        } catch {
            await rejectInvoke(action: action, reasonCode: "invalid_payload")
            appendSystemMessage("Failed to decode assistant payload: \(describeError(error))")
            log("failed to decode assistant payload: \(describeError(error)), payload=\(action.payload)")
        }
    }

    private func rejectInvoke(action: InboundActWire, reasonCode: String) async {
        do {
            try await spineBodyEndpoint.sendActResultSense(
                action: action,
                status: "rejected",
                referenceID: "apple-universal:chat:reject:\(action.actID)",
                reasonCode: reasonCode
            )
        } catch {
            appendSystemMessage("Failed to send invoke result sense: \(describeError(error))")
        }
    }

    private func handleConnectionStateChange(_ state: ConnectionState) {
        if !isConnectionEnabled && state == .connected {
            Task {
                await spineBodyEndpoint.stop()
            }
            connectionState = .disconnected
            belunaState = .unknown
            log("received connected state while disabled; forced stop")
            return
        }

        let previousState = connectionState
        connectionState = state
        if previousState != state {
            log("state \(previousState.rawValue) -> \(state.rawValue)")
        }

        if previousState != .connected, state == .connected {
            hasEverConnected = true
            belunaState = .awake
            appendSystemMessage("Beluna is awake.")
            return
        }

        if previousState == .connected, state == .disconnected, isConnectionEnabled {
            belunaState = .sleeping
            appendSystemMessage(sleepingNoticeText)
            return
        }

        if !isConnectionEnabled {
            belunaState = .unknown
            return
        }

        if state == .connecting {
            belunaState = hasEverConnected ? .sleeping : .unknown
            return
        }

        if state == .disconnected {
            belunaState = hasEverConnected ? .sleeping : .unknown
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

    private func initialMessageText(initialAutoConnect: Bool) -> String {
        if AppRuntimeEnvironment.isXcodeSession && !initialAutoConnect {
            return "Debug launch: auto-connect is off. Click Connect when ready."
        }
        return initialAutoConnect ? sleepingHelpText() : disconnectedNoticeText
    }

    private func log(_ message: String) {
        fputs("[BelunaApp] \(message)\n", stderr)
    }

    private func appendSystemMessage(_ text: String) {
        appendMessage(role: .system, text: text)
    }

    private func appendDebugMessage(_ text: String) {
        log(text)
        appendMessage(role: .debug, text: text)
    }

    private func appendMessage(role: ChatRole, text: String) {
        if let last = messages.last, last.role == role, last.text == text {
            return
        }

        messages.append(ChatMessage(role: role, text: text))
    }

    private func rememberHandledAction(_ actionID: String) -> Bool {
        if handledActionIDs.contains(actionID) {
            return false
        }

        handledActionIDs.insert(actionID)
        handledActionOrder.append(actionID)
        if handledActionOrder.count > handledActionLimit {
            let removed = handledActionOrder.removeFirst()
            handledActionIDs.remove(removed)
        }
        return true
    }

    private func describeError(_ error: Error) -> String {
        if let endpointError = error as? SpineBodyEndpointError {
            switch endpointError {
            case .notConnected:
                return "not connected"
            case let .connectionFailed(message):
                return message
            }
        }
        return error.localizedDescription
    }
}
