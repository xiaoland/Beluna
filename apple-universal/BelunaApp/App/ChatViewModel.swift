import Foundation

@MainActor
final class ChatViewModel: ObservableObject {
    @Published private(set) var messages: [ChatMessage] = []
    @Published private(set) var latestMessageIDForAutoScroll: UUID?
    @Published private(set) var hasOlderBufferedMessages: Bool = false
    @Published private(set) var hasNewerBufferedMessages: Bool = false
    @Published private(set) var bufferedMessageCount: Int = 0
    @Published private(set) var visibleMessageCount: Int = 0
    @Published var draft: String = ""
    @Published var connectionState: ConnectionState = .disconnected
    @Published var belunaState: BelunaState = .unknown
    @Published private(set) var reconnectStatus: ReconnectStatus = .idle

    @Published var socketPathDraft: String
    @Published private(set) var socketPath: String
    @Published private(set) var isConnectionEnabled: Bool

    @Published var messageCapacityDraft: String
    @Published private(set) var messageCapacity: Int
    @Published private(set) var persistedSenseActMessageCount: Int = 0

    var isHibernating: Bool {
        belunaState == .hibernate
    }

    var canSend: Bool {
        isConnectionEnabled && connectionState == .connected
    }

    var canApplySocketPath: Bool {
        let normalized = Self.normalizeSocketPath(socketPathDraft)
        return !normalized.isEmpty && normalized != socketPath
    }

    var canApplyMessageCapacity: Bool {
        guard let normalized = Self.normalizeMessageCapacity(messageCapacityDraft) else {
            return false
        }
        return normalized != messageCapacity
    }

    var connectButtonTitle: String {
        isConnectionEnabled ? "Disconnect" : "Connect"
    }

    var retryButtonTitle: String {
        switch reconnectStatus {
        case let .scheduled(_, _, delaySeconds):
            return "Retry in \(Self.formatRetryDelay(delaySeconds))"
        case .idle, .exhausted:
            return "Retry"
        }
    }

    var retryStatusText: String? {
        guard isConnectionEnabled else {
            return nil
        }

        switch reconnectStatus {
        case let .scheduled(attempt, maxAttempts, _):
            return "Auto reconnect \(attempt)/\(maxAttempts)"
        case let .exhausted(maxAttempts):
            return "Auto reconnect stopped after \(maxAttempts) attempts."
        case .idle:
            return nil
        }
    }

    var canRetry: Bool {
        isConnectionEnabled && connectionState != .connected
    }

    var canClearLocalSenseActHistory: Bool {
        persistedSenseActMessageCount > 0
    }

    var hibernateTitle: String {
        switch belunaState {
        case .awake:
            return "Beluna is awake"
        case .hibernate:
            return "Beluna is in Hibernate"
        case .unknown:
            return isConnectionEnabled ? "Beluna status unknown" : "Beluna is disconnected"
        }
    }

    var hibernateHint: String {
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

    private let bodyEndpointClient: UnixSocketBodyEndpointClient
    private let localSenseActHistoryStore: LocalSenseActHistoryStore
    private let hibernateNoticeText = "Beluna entered Hibernate."
    private let disconnectedNoticeText = "Beluna is disconnected. Click Connect to reconnect."

    private var started = false
    private var hasEverConnected = false
    private var handledActionIDs = Set<String>()
    private var handledActionOrder: [String] = []

    private var messageBuffer: [ChatMessage] = []
    private var visibleMessageRange: Range<Int> = 0..<0
    private var persistedSenseActMessagesCache: [ChatMessage] = []

    private let handledActionLimit = 256

    private static let defaultSocketPath = "/tmp/beluna.sock"
    private static let defaultMessageCapacity = 1000
    private static let minimumMessageCapacity = 100
    private static let maximumMessageCapacity = 20_000
    private static let messagePageSize = 80

    private static let socketPathDefaultsKey = "beluna.apple-universal.socket_path"
    private static let autoConnectDefaultsKey = "beluna.apple-universal.auto_connect"
    private static let messageCapacityDefaultsKey = "beluna.apple-universal.message_capacity"

    init(socketPath: String? = nil) {
        let persistedSocketPath = Self.normalizeSocketPath(
            UserDefaults.standard.string(forKey: Self.socketPathDefaultsKey)
        )
        let requestedSocketPath = Self.normalizeSocketPath(socketPath)
        let resolvedSocketPath = requestedSocketPath.isEmpty ? persistedSocketPath : requestedSocketPath
        let initialSocketPath = resolvedSocketPath.isEmpty ? Self.defaultSocketPath : resolvedSocketPath

        let persistedAutoConnect = UserDefaults.standard.object(forKey: Self.autoConnectDefaultsKey) as? Bool
        let initialAutoConnect = persistedAutoConnect ?? !AppRuntimeEnvironment.isXcodeSession

        let persistedMessageCapacity = Self.normalizeMessageCapacity(
            UserDefaults.standard.object(forKey: Self.messageCapacityDefaultsKey) as? Int
        )
        let initialMessageCapacity = persistedMessageCapacity ?? Self.defaultMessageCapacity

        self.bodyEndpointClient = UnixSocketBodyEndpointClient(socketPath: initialSocketPath)
        self.localSenseActHistoryStore = LocalSenseActHistoryStore()

        self.socketPath = initialSocketPath
        self.socketPathDraft = initialSocketPath
        self.isConnectionEnabled = initialAutoConnect

        self.messageCapacity = initialMessageCapacity
        self.messageCapacityDraft = String(initialMessageCapacity)

        let restoredSenseActMessages = localSenseActHistoryStore.load(maxCount: initialMessageCapacity)
        restoreMessageBuffer(from: restoredSenseActMessages)

        appendBufferedMessage(
            ChatMessage(
                role: .system,
                text: initialMessageText(initialAutoConnect: initialAutoConnect)
            ),
            preferredAutoScroll: true
        )

        bindSocketHandlers()
    }

    deinit {
        let bodyEndpoint = bodyEndpointClient
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
        guard !normalized.isEmpty, normalized != socketPath else {
            return
        }

        socketPath = normalized
        socketPathDraft = normalized
        persistConnectionSettings()
        appendSystemMessage("Socket path set to \(normalized)")
        log("socket path updated to \(normalized)")
        reconnectForCurrentSettings(announce: true)
    }

    func applyMessageCapacityDraft() {
        guard let normalized = Self.normalizeMessageCapacity(messageCapacityDraft) else {
            appendSystemMessage(
                "Message capacity must be an integer between \(Self.minimumMessageCapacity) and \(Self.maximumMessageCapacity)."
            )
            messageCapacityDraft = String(messageCapacity)
            return
        }
        guard normalized != messageCapacity else {
            return
        }

        messageCapacity = normalized
        messageCapacityDraft = String(normalized)
        persistMessageBufferSettings()
        trimMessageBufferToCapacity(preferLatestWindow: true)
        publishVisibleMessages(autoScrollToLatest: true)
        persistLocalSenseActHistoryIfNeeded()
        appendSystemMessage("Message buffer capacity set to \(normalized).")
    }

    func clearLocalSenseActHistory() {
        localSenseActHistoryStore.clear()

        handledActionIDs.removeAll(keepingCapacity: false)
        handledActionOrder.removeAll(keepingCapacity: false)
        messageBuffer.removeAll(keepingCapacity: false)
        visibleMessageRange = 0..<0

        persistedSenseActMessagesCache.removeAll(keepingCapacity: false)
        persistedSenseActMessageCount = 0
        publishVisibleMessages(autoScrollToLatest: false)

        appendSystemMessage("Local Sense/Act history was cleared.")
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
        updateReconnectStatus(.idle)
        persistConnectionSettings()
        log("manual connect to \(socketPath)")
        connectInternal(announce: true)
    }

    func disconnect() {
        guard isConnectionEnabled else {
            return
        }

        isConnectionEnabled = false
        updateReconnectStatus(.idle)
        persistConnectionSettings()
        log("manual disconnect")
        disconnectInternal(announce: true)
    }

    func retryConnection() {
        guard isConnectionEnabled else {
            connect()
            return
        }

        updateReconnectStatus(.idle)
        log("manual retry")
        reconnectForCurrentSettings(announce: false)
    }

    func sendCurrentDraft() {
        let text = draft.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty else {
            return
        }

        guard canSend else {
            appendSystemMessage(isConnectionEnabled ? hibernateHelpText() : disconnectedNoticeText)
            return
        }

        draft = ""
        appendBufferedMessage(ChatMessage(role: .user, signalOrigin: .sense, text: text))

        Task {
            do {
                try await bodyEndpointClient.sendUserTextSubmittedSense(text: text)
            } catch {
                appendSystemMessage("Failed to send user message to core: \(describeError(error))")
            }
        }
    }

    func handleVisibleMessageAppeared(_ messageID: UUID) {
        guard !messages.isEmpty else {
            return
        }

        if messageID == messages.first?.id {
            loadOlderMessagePageIfNeeded()
        }
        if messageID == messages.last?.id {
            loadNewerMessagePageIfNeeded()
        }
    }

    private func connectInternal(announce: Bool) {
        belunaState = .unknown
        updateReconnectStatus(.idle)
        if announce {
            appendSystemMessage("Connecting to \(socketPath)...")
        }

        Task {
            await bodyEndpointClient.start()
        }
    }

    private func disconnectInternal(announce: Bool) {
        connectionState = .disconnected
        belunaState = .unknown
        updateReconnectStatus(.idle)
        if announce {
            appendSystemMessage(disconnectedNoticeText)
        }

        Task {
            await bodyEndpointClient.stop()
        }
    }

    private func reconnectForCurrentSettings(announce: Bool) {
        let shouldConnect = isConnectionEnabled
        let updatedSocketPath = socketPath
        connectionState = .disconnected
        belunaState = .unknown
        updateReconnectStatus(.idle)

        Task {
            await bodyEndpointClient.stop()
            await bodyEndpointClient.updateSocketPath(updatedSocketPath)
            if shouldConnect {
                if announce {
                    appendSystemMessage("Connecting to \(updatedSocketPath)...")
                }
                await bodyEndpointClient.start()
            }
        }
    }

    private func bindSocketHandlers() {
        Task { [weak self] in
            guard let self else {
                return
            }

            await bodyEndpointClient.setHandlers(
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
                        self?.log("debug: \(debugText)")
                    }
                },
                onReconnectStatus: { [weak self] status in
                    Task { @MainActor in
                        self?.updateReconnectStatus(status)
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
        guard action.neuralSignalDescriptorID == bodyEndpointActPresentMessageTextDescriptorID else {
            log(
                "Ignored act for unexpected descriptor \(action.neuralSignalDescriptorID) on endpoint \(action.endpointID)"
            )
            return
        }

        guard rememberHandledAction(action.actID) else {
            log("Ignored duplicate act \(action.actID)")
            return
        }

        do {
            let text = try extractPresentedText(from: action.payload)
            appendBufferedMessage(ChatMessage(role: .assistant, signalOrigin: .act, text: text))

            try await bodyEndpointClient.sendActPresentationSucceededSense(action: action)
        } catch {
            await rejectInvoke(action: action, reasonCode: "invalid_payload")
            appendSystemMessage("Failed to decode act payload: \(describeError(error))")
            log("failed to decode act payload: \(describeError(error)), payload=\(action.payload)")
        }
    }

    private func rejectInvoke(action: InboundActWire, reasonCode: String) async {
        do {
            try await bodyEndpointClient.sendActPresentationRejectedSense(
                action: action,
                reasonCode: reasonCode
            )
        } catch {
            appendSystemMessage("Failed to send invoke result sense: \(describeError(error))")
        }
    }

    private func handleConnectionStateChange(_ state: ConnectionState) {
        if !isConnectionEnabled && state == .connected {
            Task {
                await bodyEndpointClient.stop()
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
            belunaState = .hibernate
            appendSystemMessage(hibernateNoticeText)
            return
        }

        if !isConnectionEnabled {
            belunaState = .unknown
            return
        }

        if state == .connecting {
            belunaState = hasEverConnected ? .hibernate : .unknown
            return
        }

        if state == .disconnected {
            belunaState = hasEverConnected ? .hibernate : .unknown
        }
    }

    private func appendBufferedMessage(_ message: ChatMessage, preferredAutoScroll: Bool? = nil) {
        let shouldAutoScroll = preferredAutoScroll ?? isShowingLatestMessageWindow
        let previousVisibleCount = visibleMessageRange.count

        messageBuffer.append(message)
        trimMessageBufferToCapacity(preferLatestWindow: shouldAutoScroll)

        if messageBuffer.isEmpty {
            visibleMessageRange = 0..<0
            publishVisibleMessages(autoScrollToLatest: false)
            persistLocalSenseActHistoryIfNeeded()
            return
        }

        if shouldAutoScroll || visibleMessageRange.isEmpty {
            let desiredVisibleCount = max(previousVisibleCount, Self.messagePageSize)
            let end = messageBuffer.count
            let start = max(0, end - desiredVisibleCount)
            visibleMessageRange = start..<end
            publishVisibleMessages(autoScrollToLatest: true)
            persistLocalSenseActHistoryIfNeeded()
            return
        }

        publishVisibleMessages(autoScrollToLatest: false)
        persistLocalSenseActHistoryIfNeeded()
    }

    private func loadOlderMessagePageIfNeeded() {
        guard visibleMessageRange.lowerBound > 0 else {
            return
        }

        let newLowerBound = max(0, visibleMessageRange.lowerBound - Self.messagePageSize)
        visibleMessageRange = newLowerBound..<visibleMessageRange.upperBound
        publishVisibleMessages(autoScrollToLatest: false)
    }

    private func loadNewerMessagePageIfNeeded() {
        guard visibleMessageRange.upperBound < messageBuffer.count else {
            return
        }

        let newUpperBound = min(messageBuffer.count, visibleMessageRange.upperBound + Self.messagePageSize)
        visibleMessageRange = visibleMessageRange.lowerBound..<newUpperBound
        publishVisibleMessages(autoScrollToLatest: false)
    }

    private func trimMessageBufferToCapacity(preferLatestWindow: Bool) {
        guard messageBuffer.count > messageCapacity else {
            return
        }

        let overflow = messageBuffer.count - messageCapacity
        messageBuffer.removeFirst(overflow)

        let shiftedLowerBound = max(0, visibleMessageRange.lowerBound - overflow)
        let shiftedUpperBound = max(shiftedLowerBound, visibleMessageRange.upperBound - overflow)
        visibleMessageRange = shiftedLowerBound..<min(shiftedUpperBound, messageBuffer.count)

        guard !messageBuffer.isEmpty else {
            visibleMessageRange = 0..<0
            return
        }

        if preferLatestWindow {
            let desiredVisibleCount = max(visibleMessageRange.count, Self.messagePageSize)
            let end = messageBuffer.count
            let start = max(0, end - desiredVisibleCount)
            visibleMessageRange = start..<end
            return
        }

        if visibleMessageRange.isEmpty {
            let end = min(messageBuffer.count, Self.messagePageSize)
            let start = max(0, end - Self.messagePageSize)
            visibleMessageRange = start..<end
        }
    }

    private func publishVisibleMessages(autoScrollToLatest: Bool) {
        guard !messageBuffer.isEmpty else {
            visibleMessageRange = 0..<0
            messages = []
            bufferedMessageCount = 0
            visibleMessageCount = 0
            hasOlderBufferedMessages = false
            hasNewerBufferedMessages = false
            return
        }

        let clampedLowerBound = min(max(0, visibleMessageRange.lowerBound), messageBuffer.count)
        let clampedUpperBound = min(max(clampedLowerBound, visibleMessageRange.upperBound), messageBuffer.count)
        visibleMessageRange = clampedLowerBound..<clampedUpperBound

        messages = Array(messageBuffer[visibleMessageRange])
        bufferedMessageCount = messageBuffer.count
        visibleMessageCount = messages.count
        hasOlderBufferedMessages = visibleMessageRange.lowerBound > 0
        hasNewerBufferedMessages = visibleMessageRange.upperBound < messageBuffer.count

        if autoScrollToLatest, let latestID = messages.last?.id {
            latestMessageIDForAutoScroll = latestID
        }
    }

    private var isShowingLatestMessageWindow: Bool {
        visibleMessageRange.upperBound == messageBuffer.count
    }

    private func restoreMessageBuffer(from restoredMessages: [ChatMessage]) {
        guard !restoredMessages.isEmpty else {
            persistedSenseActMessagesCache = []
            persistedSenseActMessageCount = 0
            return
        }

        messageBuffer = restoredMessages
        let end = messageBuffer.count
        let start = max(0, end - Self.messagePageSize)
        visibleMessageRange = start..<end
        publishVisibleMessages(autoScrollToLatest: false)

        persistedSenseActMessagesCache = currentPersistedSenseActMessages()
        persistedSenseActMessageCount = persistedSenseActMessagesCache.count
    }

    private func persistLocalSenseActHistoryIfNeeded() {
        let currentSenseActMessages = currentPersistedSenseActMessages()
        persistedSenseActMessageCount = currentSenseActMessages.count

        guard currentSenseActMessages != persistedSenseActMessagesCache else {
            return
        }

        persistedSenseActMessagesCache = currentSenseActMessages
        localSenseActHistoryStore.save(messages: currentSenseActMessages, maxCount: messageCapacity)
    }

    private func currentPersistedSenseActMessages() -> [ChatMessage] {
        messageBuffer.filter { $0.signalOrigin == .sense || $0.signalOrigin == .act }
    }

    private func persistConnectionSettings() {
        UserDefaults.standard.set(socketPath, forKey: Self.socketPathDefaultsKey)
        UserDefaults.standard.set(isConnectionEnabled, forKey: Self.autoConnectDefaultsKey)
    }

    private func persistMessageBufferSettings() {
        UserDefaults.standard.set(messageCapacity, forKey: Self.messageCapacityDefaultsKey)
    }

    private nonisolated static func normalizeSocketPath(_ value: String?) -> String {
        (value ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private nonisolated static func normalizeMessageCapacity(_ value: String?) -> Int? {
        let trimmed = (value ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty, let parsed = Int(trimmed) else {
            return nil
        }
        return normalizeMessageCapacity(parsed)
    }

    private nonisolated static func normalizeMessageCapacity(_ value: Int?) -> Int? {
        guard let value else {
            return nil
        }
        let clamped = min(max(value, minimumMessageCapacity), maximumMessageCapacity)
        return clamped
    }

    private nonisolated static func formatRetryDelay(_ seconds: Double) -> String {
        if seconds >= 10 {
            return "\(Int(seconds.rounded()))s"
        }
        return String(format: "%.1fs", seconds)
    }

    private func hibernateHelpText() -> String {
        "Beluna is in Hibernate. Start Beluna Core to wake it up."
    }

    private func initialMessageText(initialAutoConnect: Bool) -> String {
        if AppRuntimeEnvironment.isXcodeSession && !initialAutoConnect {
            return "Debug launch: auto-connect is off. Click Connect when ready."
        }
        return initialAutoConnect ? hibernateHelpText() : disconnectedNoticeText
    }

    private func log(_ message: String) {
        fputs("[BelunaApp] \(message)\n", stderr)
    }

    private func appendSystemMessage(_ text: String) {
        appendMessage(role: .system, text: text)
    }

    private func appendMessage(role: ChatRole, text: String) {
        if let last = messageBuffer.last, last.role == role, last.text == text {
            return
        }

        appendBufferedMessage(ChatMessage(role: role, text: text))
    }

    private func updateReconnectStatus(_ status: ReconnectStatus) {
        guard reconnectStatus != status else {
            return
        }
        reconnectStatus = status
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
        if let endpointError = error as? BodyEndpointClientError {
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
