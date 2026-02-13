import Foundation

@MainActor
final class ChatViewModel: ObservableObject {
    @Published var messages: [ChatMessage] = []
    @Published var draft: String = ""
    @Published var connectionState: ConnectionState = .disconnected

    var isSleeping: Bool {
        connectionState != .connected
    }

    var canSend: Bool {
        connectionState == .connected
    }

    private let conversationID: String
    private let spineBodyEndpoint: SpineUnixSocketBodyEndpoint
    private let sleepingNoticeText = "Beluna is sleeping."
    private let sleepingHelpText = "Beluna is sleeping. Start Beluna Core to wake it up."
    private var started = false

    init(socketPath: String = "/tmp/beluna.sock") {
        self.conversationID = "conv_\(UUID().uuidString.lowercased())"
        self.spineBodyEndpoint = SpineUnixSocketBodyEndpoint(socketPath: socketPath)

        messages.append(ChatMessage(role: .system, text: sleepingHelpText))

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
        Task {
            await spineBodyEndpoint.start()
        }
    }

    func sendCurrentDraft() {
        let text = draft.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty else {
            return
        }

        guard canSend else {
            appendSystemMessage(sleepingHelpText)
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
        let previousState = connectionState
        connectionState = state

        if previousState != .connected, state == .connected {
            appendSystemMessage("Beluna is awake.")
            return
        }

        if previousState != .disconnected, state == .disconnected {
            appendSystemMessage(sleepingNoticeText)
        }
    }

    private func appendSystemMessage(_ text: String) {
        if let last = messages.last, last.role == .system, last.text == text {
            return
        }

        messages.append(ChatMessage(role: .system, text: text))
    }
}
