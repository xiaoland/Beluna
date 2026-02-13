import Foundation

@MainActor
final class ChatViewModel: ObservableObject {
    @Published var messages: [ChatMessage] = []
    @Published var draft: String = ""
    @Published var connectionState: ConnectionState = .disconnected

    private let conversationID: String
    private let spineBodyEndpoint: SpineUnixSocketBodyEndpoint
    private var started = false

    init(socketPath: String = "/tmp/beluna.sock") {
        self.conversationID = "conv_\(UUID().uuidString.lowercased())"
        self.spineBodyEndpoint = SpineUnixSocketBodyEndpoint(socketPath: socketPath)

        messages.append(
            ChatMessage(
                role: .system,
                text: "Connected app endpoint will register as chat body over Spine UnixSocket."
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
        Task {
            await spineBodyEndpoint.start()
        }
    }

    func sendCurrentDraft() {
        let text = draft.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty else {
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
                        self?.connectionState = state
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
        case let .endpointInvoke(requestID, action):
            await handleEndpointInvoke(requestID: requestID, action: action)
        }
    }

    private func handleEndpointInvoke(requestID: String, action: AdmittedActionWire) async {
        guard action.affordanceKey == appleAffordanceKey,
              action.capabilityHandle == appleCapabilityHandle
        else {
            await rejectInvoke(
                requestID: requestID,
                actionID: action.actionID,
                reasonCode: "unsupported_route"
            )
            return
        }

        do {
            let texts = try extractAssistantTexts(from: action.normalizedPayload)
            if texts.isEmpty {
                await rejectInvoke(
                    requestID: requestID,
                    actionID: action.actionID,
                    reasonCode: "invalid_payload"
                )
                appendSystemMessage("Received chat invoke with empty assistant output.")
                return
            }

            for text in texts {
                messages.append(ChatMessage(role: .assistant, text: text))
            }

            try await spineBodyEndpoint.sendEndpointResult(
                requestID: requestID,
                outcome: .applied(
                    actualCostMicro: max(0, action.reservedCost.survivalMicro),
                    referenceID: "apple-universal:chat:\(action.actionID)"
                )
            )
        } catch {
            await rejectInvoke(
                requestID: requestID,
                actionID: action.actionID,
                reasonCode: "invalid_payload"
            )
            appendSystemMessage("Failed to decode assistant payload: \(error.localizedDescription)")
        }
    }

    private func rejectInvoke(requestID: String, actionID: String, reasonCode: String) async {
        do {
            try await spineBodyEndpoint.sendEndpointResult(
                requestID: requestID,
                outcome: .rejected(
                    reasonCode: reasonCode,
                    referenceID: "apple-universal:chat:reject:\(actionID)"
                )
            )
        } catch {
            appendSystemMessage("Failed to send body_endpoint_result: \(error.localizedDescription)")
        }
    }

    private func appendSystemMessage(_ text: String) {
        messages.append(ChatMessage(role: .system, text: text))
    }
}
