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

    @Published var socketPathDraft: String
    @Published private(set) var socketPath: String
    @Published private(set) var isConnectionEnabled: Bool

    @Published var metricsEndpointDraft: String
    @Published private(set) var metricsEndpoint: String
    @Published private(set) var metricsStatusText: String = "Metrics ready"
    @Published private(set) var metricsLastRefreshedAt: Date?
    @Published private(set) var metricsCycleID: Double?
    @Published private(set) var metricsActDescriptorCatalogCount: Double?
    @Published private(set) var isMetricsRefreshing: Bool = false

    @Published var logDirectoryPathDraft: String
    @Published private(set) var logDirectoryPath: String
    @Published private(set) var logStatusText: String = "Logs ready"
    @Published private(set) var logLastRefreshedAt: Date?

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

    var canApplyMetricsEndpoint: Bool {
        let normalized = Self.normalizeMetricsEndpoint(metricsEndpointDraft)
        return !normalized.isEmpty && normalized != metricsEndpoint
    }

    var canApplyLogDirectoryPath: Bool {
        let normalized = Self.normalizeDirectoryPath(logDirectoryPathDraft)
        return !normalized.isEmpty && normalized != logDirectoryPath
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

    var metricsCycleIDText: String {
        Self.formatMetricValue(metricsCycleID)
    }

    var metricsActDescriptorCatalogCountText: String {
        Self.formatMetricValue(metricsActDescriptorCatalogCount)
    }

    var metricsLastUpdatedLabel: String? {
        guard let metricsLastRefreshedAt else {
            return nil
        }
        return "Updated \(Self.metricsTimeFormatter.string(from: metricsLastRefreshedAt))"
    }

    private let bodyEndpointClient: UnixSocketBodyEndpointClient
    private let localSenseActHistoryStore: LocalSenseActHistoryStore
    private let hibernateNoticeText = "Beluna entered Hibernate."
    private let disconnectedNoticeText = "Beluna is disconnected. Click Connect to reconnect."

    private var started = false
    private var hasEverConnected = false
    private var handledActionIDs = Set<String>()
    private var handledActionOrder: [String] = []

    private var metricsPollingTask: Task<Void, Never>?
    private var logWatcher: LogDirectoryWatcher?

    private var messageBuffer: [ChatMessage] = []
    private var visibleMessageRange: Range<Int> = 0..<0
    private var persistedSenseActMessagesCache: [ChatMessage] = []

    private var cortexCycleMessageIDs: [String: UUID] = [:]
    private var pendingOrganInputs: [String: [OrganLogEvent]] = [:]
    private var pendingOrganOutputs: [String: [OrganLogEvent]] = [:]
    private var seenOrganEventIDs = Set<String>()
    private var seenOrganEventOrder: [String] = []

    private let handledActionLimit = 256
    private let pendingOrganEventLimitPerKey = 32
    private let seenOrganEventLimit = 8_192

    private static let defaultSocketPath = "/tmp/beluna.sock"
    private static let defaultMetricsEndpoint = "http://127.0.0.1:9464/metrics"
    private static let defaultLogDirectoryPath = "~/logs/core"
    private static let defaultMessageCapacity = 1000
    private static let minimumMessageCapacity = 100
    private static let maximumMessageCapacity = 20_000
    private static let messagePageSize = 80

    private static let socketPathDefaultsKey = "beluna.apple-universal.socket_path"
    private static let autoConnectDefaultsKey = "beluna.apple-universal.auto_connect"
    private static let metricsEndpointDefaultsKey = "beluna.apple-universal.metrics_endpoint"
    private static let logDirectoryPathDefaultsKey = "beluna.apple-universal.log_directory_path"
    private static let messageCapacityDefaultsKey = "beluna.apple-universal.message_capacity"



    private static let metricsTimeFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.dateFormat = "HH:mm:ss"
        return formatter
    }()

    init(socketPath: String? = nil) {
        let persistedSocketPath = Self.normalizeSocketPath(
            UserDefaults.standard.string(forKey: Self.socketPathDefaultsKey)
        )
        let requestedSocketPath = Self.normalizeSocketPath(socketPath)
        let resolvedSocketPath = requestedSocketPath.isEmpty ? persistedSocketPath : requestedSocketPath
        let initialSocketPath = resolvedSocketPath.isEmpty ? Self.defaultSocketPath : resolvedSocketPath

        let persistedAutoConnect = UserDefaults.standard.object(forKey: Self.autoConnectDefaultsKey) as? Bool
        let initialAutoConnect = persistedAutoConnect ?? !AppRuntimeEnvironment.isXcodeSession

        let persistedMetricsEndpoint = Self.normalizeMetricsEndpoint(
            UserDefaults.standard.string(forKey: Self.metricsEndpointDefaultsKey)
        )
        let initialMetricsEndpoint = persistedMetricsEndpoint.isEmpty
            ? Self.defaultMetricsEndpoint
            : persistedMetricsEndpoint

        let persistedLogDirectory = Self.normalizeDirectoryPath(
            UserDefaults.standard.string(forKey: Self.logDirectoryPathDefaultsKey)
        )
        let initialLogDirectory = persistedLogDirectory.isEmpty
            ? Self.normalizeDirectoryPath(Self.defaultLogDirectoryPath)
            : persistedLogDirectory

        let persistedMessageCapacity = Self.normalizeMessageCapacity(
            UserDefaults.standard.object(forKey: Self.messageCapacityDefaultsKey) as? Int
        )
        let initialMessageCapacity = persistedMessageCapacity ?? Self.defaultMessageCapacity

        self.bodyEndpointClient = UnixSocketBodyEndpointClient(socketPath: initialSocketPath)
        self.localSenseActHistoryStore = LocalSenseActHistoryStore()

        self.socketPath = initialSocketPath
        self.socketPathDraft = initialSocketPath
        self.isConnectionEnabled = initialAutoConnect

        self.metricsEndpoint = initialMetricsEndpoint
        self.metricsEndpointDraft = initialMetricsEndpoint

        self.logDirectoryPath = initialLogDirectory
        self.logDirectoryPathDraft = initialLogDirectory

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
        metricsPollingTask?.cancel()
        logWatcher?.stop()
        logWatcher = nil

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
        startMetricsPollingIfNeeded()
        startLogWatcher()

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
        persistMetricsSettings()

        Task {
            await refreshMetricsNow()
        }
    }

    func applyLogDirectoryPathDraft() {
        let normalized = Self.normalizeDirectoryPath(logDirectoryPathDraft)
        guard !normalized.isEmpty else {
            logStatusText = "Log directory cannot be empty."
            return
        }
        guard normalized != logDirectoryPath else {
            return
        }

        logDirectoryPath = normalized
        logDirectoryPathDraft = normalized
        resetOrganLogTracking()
        persistLogDirectorySettings()
        restartLogWatcher()
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
        resetOrganLogTracking()

        persistedSenseActMessagesCache.removeAll(keepingCapacity: false)
        persistedSenseActMessageCount = 0
        publishVisibleMessages(autoScrollToLatest: false)

        appendSystemMessage("Local Sense/Act history was cleared.")
    }

    func refreshMetrics() {
        Task {
            await refreshMetricsNow()
        }
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
        guard action.neuralSignalDescriptorID == bodyEndpointActPresentMessageTextDescriptorID else {
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
            Task {
                await refreshMetricsNow()
            }
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

    private func startMetricsPollingIfNeeded() {
        guard metricsPollingTask == nil else {
            return
        }

        metricsPollingTask = Task { [weak self] in
            guard let self else {
                return
            }

            while !Task.isCancelled {
                if self.isMetricsAutoPollingEnabled {
                    await self.refreshMetricsNow()
                } else {
                    self.setMetricsPollingPausedStatus()
                }
                try? await Task.sleep(nanoseconds: 5_000_000_000)
                if Task.isCancelled {
                    break
                }
            }
        }
    }

    private func startLogWatcher() {
        guard logWatcher == nil else {
            return
        }

        let directoryPath = logDirectoryPath
        logWatcher = LogDirectoryWatcher(directoryPath: directoryPath) { [weak self] events, statusText in
            Task { @MainActor [weak self] in
                self?.handleWatcherEvents(events, statusText: statusText)
            }
        }
        logWatcher?.start()
    }

    private func restartLogWatcher() {
        logWatcher?.stop()
        logWatcher = nil
        startLogWatcher()
    }

    private func handleWatcherEvents(_ events: [OrganLogEvent], statusText: String) {
        logStatusText = statusText
        logLastRefreshedAt = Date()

        for event in events {
            guard rememberSeenOrganEvent(event.eventID) else {
                continue
            }
            handleOrganLogEvent(event)
        }
    }

    private func refreshMetricsNow() async {
        guard !isMetricsRefreshing else {
            return
        }

        isMetricsRefreshing = true
        metricsStatusText = "Refreshing metrics..."

        let endpoint = metricsEndpoint
        let snapshot = await Self.loadMetricsSnapshot(endpoint: endpoint)
        if Task.isCancelled {
            isMetricsRefreshing = false
            return
        }

        isMetricsRefreshing = false
        metricsLastRefreshedAt = Date()
        metricsStatusText = snapshot.statusText
        metricsCycleID = snapshot.cycleID
        metricsActDescriptorCatalogCount = snapshot.actDescriptorCatalogCount
    }

    private var isMetricsAutoPollingEnabled: Bool {
        isConnectionEnabled && connectionState == .connected
    }

    private func setMetricsPollingPausedStatus() {
        if metricsStatusText != "Metrics polling paused (socket disconnected)." {
            metricsStatusText = "Metrics polling paused (socket disconnected)."
        }
    }

    private func handleOrganLogEvent(_ event: OrganLogEvent) {
        let key = Self.organPairKey(
            cycleID: event.cycleID,
            awakeSequence: event.awakeSequence,
            stage: event.stage
        )

        switch event.kind {
        case .input:
            if let output = popPendingEvent(from: &pendingOrganOutputs, key: key) {
                appendOrganActivityMessage(input: event, output: output)
            } else {
                appendPendingEvent(event, to: &pendingOrganInputs, key: key)
            }
        case .output:
            if let input = popPendingEvent(from: &pendingOrganInputs, key: key) {
                appendOrganActivityMessage(input: input, output: event)
            } else {
                appendPendingEvent(event, to: &pendingOrganOutputs, key: key)
            }
        }
    }

    private func appendOrganActivityMessage(input: OrganLogEvent, output: OrganLogEvent) {
        let timestamp = output.timestamp ?? input.timestamp ?? Date()
        let organActivityMessage = OrganActivityMessagePayload(
            stage: input.stage,
            inputPayload: input.payload,
            outputPayload: output.payload,
            timestamp: timestamp
        )

        appendCortexCycleMessage(
            cycleID: input.cycleID,
            awakeSequence: input.awakeSequence,
            organActivityMessage: organActivityMessage,
            timestamp: timestamp
        )
    }

    private func appendCortexCycleMessage(
        cycleID: UInt64,
        awakeSequence: UInt64?,
        organActivityMessage: OrganActivityMessagePayload,
        timestamp: Date
    ) {
        let cycleKey = Self.cortexCycleKey(cycleID: cycleID, awakeSequence: awakeSequence)
        if let existingMessageID = cortexCycleMessageIDs[cycleKey],
           let existingIndex = messageBuffer.firstIndex(where: { $0.id == existingMessageID }),
           case var .cortexCycle(existingPayload) = messageBuffer[existingIndex].body {
            existingPayload.organActivityMessages.append(organActivityMessage)
            messageBuffer[existingIndex].body = .cortexCycle(existingPayload)
            messageBuffer[existingIndex].timestamp = timestamp
            publishVisibleMessages(autoScrollToLatest: false)
            persistLocalSenseActHistoryIfNeeded()
            return
        }

        cortexCycleMessageIDs.removeValue(forKey: cycleKey)

        let payload = CortexCycleMessagePayload(
            cycleID: cycleID,
            awakeSequence: awakeSequence,
            organActivityMessages: [organActivityMessage]
        )
        let cycleMessage = ChatMessage(cortexCycle: payload, timestamp: timestamp)
        cortexCycleMessageIDs[cycleKey] = cycleMessage.id
        appendBufferedMessage(cycleMessage)
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
        let removedMessages = Array(messageBuffer.prefix(overflow))
        messageBuffer.removeFirst(overflow)
        forgetCortexCycleMessages(removedMessages)

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

    private func appendPendingEvent(
        _ event: OrganLogEvent,
        to storage: inout [String: [OrganLogEvent]],
        key: String
    ) {
        var queue = storage[key] ?? []
        queue.append(event)
        if queue.count > pendingOrganEventLimitPerKey {
            queue.removeFirst(queue.count - pendingOrganEventLimitPerKey)
        }
        storage[key] = queue
    }

    private func popPendingEvent(
        from storage: inout [String: [OrganLogEvent]],
        key: String
    ) -> OrganLogEvent? {
        guard var queue = storage[key], !queue.isEmpty else {
            return nil
        }
        let event = queue.removeFirst()
        if queue.isEmpty {
            storage.removeValue(forKey: key)
        } else {
            storage[key] = queue
        }
        return event
    }

    private func rememberSeenOrganEvent(_ eventID: String) -> Bool {
        if seenOrganEventIDs.contains(eventID) {
            return false
        }

        seenOrganEventIDs.insert(eventID)
        seenOrganEventOrder.append(eventID)

        if seenOrganEventOrder.count > seenOrganEventLimit {
            let removed = seenOrganEventOrder.removeFirst()
            seenOrganEventIDs.remove(removed)
        }

        return true
    }

    private func resetOrganLogTracking() {
        cortexCycleMessageIDs.removeAll(keepingCapacity: false)
        pendingOrganInputs.removeAll(keepingCapacity: false)
        pendingOrganOutputs.removeAll(keepingCapacity: false)
        seenOrganEventIDs.removeAll(keepingCapacity: false)
        seenOrganEventOrder.removeAll(keepingCapacity: false)
    }

    private func forgetCortexCycleMessages(_ messages: [ChatMessage]) {
        guard !messages.isEmpty else {
            return
        }

        for message in messages {
            guard case let .cortexCycle(payload) = message.body else {
                continue
            }

            let cycleKey = Self.cortexCycleKey(
                cycleID: payload.cycleID,
                awakeSequence: payload.awakeSequence
            )
            if cortexCycleMessageIDs[cycleKey] == message.id {
                cortexCycleMessageIDs.removeValue(forKey: cycleKey)
            }
        }
    }

    private func restoreMessageBuffer(from restoredMessages: [ChatMessage]) {
        guard !restoredMessages.isEmpty else {
            persistedSenseActMessagesCache = []
            persistedSenseActMessageCount = 0
            return
        }

        messageBuffer = restoredMessages
        rebuildCortexCycleMessageIndex()

        let end = messageBuffer.count
        let start = max(0, end - Self.messagePageSize)
        visibleMessageRange = start..<end
        publishVisibleMessages(autoScrollToLatest: false)

        persistedSenseActMessagesCache = currentPersistedSenseActMessages()
        persistedSenseActMessageCount = persistedSenseActMessagesCache.count
    }

    private func rebuildCortexCycleMessageIndex() {
        cortexCycleMessageIDs.removeAll(keepingCapacity: false)

        for message in messageBuffer {
            guard case let .cortexCycle(payload) = message.body else {
                continue
            }
            let key = Self.cortexCycleKey(cycleID: payload.cycleID, awakeSequence: payload.awakeSequence)
            cortexCycleMessageIDs[key] = message.id
        }
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

    private func persistMetricsSettings() {
        UserDefaults.standard.set(metricsEndpoint, forKey: Self.metricsEndpointDefaultsKey)
    }

    private func persistLogDirectorySettings() {
        UserDefaults.standard.set(logDirectoryPath, forKey: Self.logDirectoryPathDefaultsKey)
    }

    private func persistMessageBufferSettings() {
        UserDefaults.standard.set(messageCapacity, forKey: Self.messageCapacityDefaultsKey)
    }

    private nonisolated static func normalizeSocketPath(_ value: String?) -> String {
        (value ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private nonisolated static func normalizeMetricsEndpoint(_ value: String?) -> String {
        (value ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
    }

    private nonisolated static func normalizeDirectoryPath(_ value: String?) -> String {
        let trimmed = (value ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else {
            return ""
        }

        let expanded = (trimmed as NSString).expandingTildeInPath
        return URL(fileURLWithPath: expanded).standardizedFileURL.path
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

    private func appendDebugMessage(_ text: String) {
        log(text)
        appendMessage(role: .debug, text: text)
    }

    private func appendMessage(role: ChatRole, text: String) {
        if let last = messageBuffer.last, last.role == role, last.text == text {
            return
        }

        appendBufferedMessage(ChatMessage(role: role, text: text))
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

    private nonisolated static func formatMetricValue(_ value: Double?) -> String {
        guard let value else {
            return "-"
        }

        if value.rounded() == value {
            return String(Int(value))
        }

        return String(format: "%.2f", value)
    }

    private nonisolated static func loadMetricsSnapshot(endpoint: String) async -> MetricsSnapshot {
        guard let url = URL(string: endpoint) else {
            return MetricsSnapshot(
                cycleID: nil,
                actDescriptorCatalogCount: nil,
                statusText: "Invalid metrics endpoint URL: \(endpoint)"
            )
        }
        guard let scheme = url.scheme?.lowercased(), scheme == "http" || scheme == "https" else {
            return MetricsSnapshot(
                cycleID: nil,
                actDescriptorCatalogCount: nil,
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
                    statusText: "Metrics endpoint returned HTTP \(statusCode)."
                )
            }

            let body = String(decoding: data, as: UTF8.self)
            let cycleID = parsePrometheusGauge(named: "beluna_cortex_cycle_id", in: body)
            let catalogCount = parsePrometheusGauge(
                named: "beluna_cortex_input_ir_act_descriptor_catalog_count",
                in: body
            )

            let status: String
            if cycleID == nil && catalogCount == nil {
                status = "Metrics fetched, but target gauges were not found."
            } else {
                status = "Metrics loaded from \(endpoint)."
            }

            return MetricsSnapshot(
                cycleID: cycleID,
                actDescriptorCatalogCount: catalogCount,
                statusText: status
            )
        } catch {
            return MetricsSnapshot(
                cycleID: nil,
                actDescriptorCatalogCount: nil,
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

    private nonisolated static func organPairKey(
        cycleID: UInt64,
        awakeSequence: UInt64?,
        stage: String
    ) -> String {
        "\(awakeSequence.map(String.init) ?? "unknown")|\(cycleID)|\(stage)"
    }

    private nonisolated static func cortexCycleKey(cycleID: UInt64, awakeSequence: UInt64?) -> String {
        "\(awakeSequence.map(String.init) ?? "unknown")|\(cycleID)"
    }
}

private struct MetricsSnapshot: Sendable {
    let cycleID: Double?
    let actDescriptorCatalogCount: Double?
    let statusText: String
}


