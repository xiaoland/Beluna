import Foundation

let appleEndpointName = "macos-app"
let appleActNeuralSignalDescriptorID = "present_text_message"
let appleUserSenseNeuralSignalDescriptorID = "apple.chat.user_message"
let appleActResultSenseNeuralSignalDescriptorID = "apple.chat.present_message_result"

enum NeuralSignalTypeWire: String, Codable, Equatable {
    case sense
    case act
}

struct NeuralSignalDescriptorWire: Codable, Equatable {
    let type: NeuralSignalTypeWire
    let endpointID: String
    let neuralSignalDescriptorID: String
    let payloadSchema: JSONValue

    enum CodingKeys: String, CodingKey {
        case type
        case endpointID = "endpoint_id"
        case neuralSignalDescriptorID = "neural_signal_descriptor_id"
        case payloadSchema = "payload_schema"
    }
}

struct NDJSONEnvelope<Body: Codable>: Codable {
    let method: String
    let id: String
    let timestamp: UInt64
    let body: Body
}

struct AuthBodyWire: Codable {
    let endpointName: String
    let capabilities: [NeuralSignalDescriptorWire]

    enum CodingKeys: String, CodingKey {
        case endpointName = "endpoint_name"
        case capabilities
    }
}

struct SenseBodyWire: Codable {
    let senseID: String
    let neuralSignalDescriptorID: String
    let payload: JSONValue

    enum CodingKeys: String, CodingKey {
        case senseID = "sense_id"
        case neuralSignalDescriptorID = "neural_signal_descriptor_id"
        case payload
    }
}

struct ActAckBodyWire: Codable {
    let actID: String

    enum CodingKeys: String, CodingKey {
        case actID = "act_id"
    }
}

struct InboundActWire: Decodable, Equatable {
    let actID: String
    let endpointID: String
    let neuralSignalDescriptorID: String
    let payload: JSONValue

    enum CodingKeys: String, CodingKey {
        case actID = "act_id"
        case endpointID = "endpoint_id"
        case neuralSignalDescriptorID = "neural_signal_descriptor_id"
        case payload
    }
}

private struct InboundEnvelopeWire: Decodable {
    let method: String
    let id: String
    let timestamp: UInt64
    let body: JSONValue
}

private struct InboundActBodyWire: Decodable {
    let act: InboundActWire
}

enum ServerWireMessage: Equatable {
    case act(action: InboundActWire)
    case ignored(type: String)
}

private func makeEnvelope<Body: Codable>(method: String, body: Body) -> NDJSONEnvelope<Body> {
    NDJSONEnvelope(
        method: method,
        id: UUID().uuidString.lowercased(),
        timestamp: UInt64(Date().timeIntervalSince1970 * 1_000),
        body: body
    )
}

func makeAppleEndpointRegisterEnvelope() -> NDJSONEnvelope<AuthBodyWire> {
    makeEnvelope(
        method: "auth",
        body: AuthBodyWire(
            endpointName: appleEndpointName,
            capabilities: [
                NeuralSignalDescriptorWire(
                    type: .act,
                    endpointID: appleEndpointName,
                    neuralSignalDescriptorID: appleActNeuralSignalDescriptorID,
                    payloadSchema: .object([
                        "type": .string("string"),
                        "description": .string("The text message you want to present")
                    ])
                ),
                NeuralSignalDescriptorWire(
                    type: .sense,
                    endpointID: appleEndpointName,
                    neuralSignalDescriptorID: appleUserSenseNeuralSignalDescriptorID,
                    payloadSchema: .object([
                        "type": .string("object"),
                        "required": .array([.string("conversation_id"), .string("input")]),
                        "properties": .object([
                            "conversation_id": .object([
                                "type": .string("string")
                            ]),
                            "input": .object([
                                "type": .string("array"),
                                "items": .object([
                                    "type": .string("object"),
                                    "required": .array([.string("type"), .string("role"), .string("content")]),
                                    "properties": .object([
                                        "type": .object([
                                            "type": .string("string"),
                                            "const": .string("message")
                                        ]),
                                        "role": .object([
                                            "type": .string("string"),
                                            "const": .string("user")
                                        ]),
                                        "content": .object([
                                            "type": .string("array"),
                                            "items": .object([
                                                "type": .string("object"),
                                                "required": .array([.string("type"), .string("text")]),
                                                "properties": .object([
                                                    "type": .object([
                                                        "type": .string("string"),
                                                        "const": .string("input_text")
                                                    ]),
                                                    "text": .object([
                                                        "type": .string("string")
                                                    ])
                                                ])
                                            ])
                                        ])
                                    ])
                                ])
                            ])
                        ])
                    ])
                ),
                NeuralSignalDescriptorWire(
                    type: .sense,
                    endpointID: appleEndpointName,
                    neuralSignalDescriptorID: appleActResultSenseNeuralSignalDescriptorID,
                    payloadSchema: .object([
                        "type": .string("object"),
                        "required": .array([
                            .string("kind"),
                            .string("act_id"),
                            .string("status"),
                            .string("reference_id")
                        ]),
                        "properties": .object([
                            "kind": .object([
                                "type": .string("string"),
                                "const": .string("present_message_result")
                            ]),
                            "act_id": .object([
                                "type": .string("string")
                            ]),
                            "status": .object([
                                "type": .string("string")
                            ]),
                            "reference_id": .object([
                                "type": .string("string")
                            ]),
                            "reason_code": .object([
                                "type": .string("string")
                            ])
                        ])
                    ])
                )
            ]
        )
    )
}

func makeUserSenseEnvelope(conversationID: String, text: String) -> NDJSONEnvelope<SenseBodyWire> {
    makeEnvelope(
        method: "sense",
        body: SenseBodyWire(
            senseID: UUID().uuidString.lowercased(),
            neuralSignalDescriptorID: appleUserSenseNeuralSignalDescriptorID,
            payload: .object([
                "conversation_id": .string(conversationID),
                "input": .array([
                    .object([
                        "type": .string("message"),
                        "role": .string("user"),
                        "content": .array([
                            .object([
                                "type": .string("input_text"),
                                "text": .string(text)
                            ])
                        ])
                    ])
                ])
            ])
        )
    )
}

func makeActResultSenseEnvelope(
    action: InboundActWire,
    status: String,
    referenceID: String,
    reasonCode: String? = nil
) -> NDJSONEnvelope<SenseBodyWire> {
    var payload: [String: JSONValue] = [
        "kind": .string("present_message_result"),
        "status": .string(status),
        "act_id": .string(action.actID),
        "reference_id": .string(referenceID)
    ]
    if let reasonCode {
        payload["reason_code"] = .string(reasonCode)
    }

    return makeEnvelope(
        method: "sense",
        body: SenseBodyWire(
            senseID: UUID().uuidString.lowercased(),
            neuralSignalDescriptorID: appleActResultSenseNeuralSignalDescriptorID,
            payload: .object(payload)
        )
    )
}

func makeActAckEnvelope(actID: String) -> NDJSONEnvelope<ActAckBodyWire> {
    makeEnvelope(method: "act_ack", body: ActAckBodyWire(actID: actID))
}

enum PresentTextPayloadError: LocalizedError, Equatable {
    case expectedString
    case emptyText

    var errorDescription: String? {
        switch self {
        case .expectedString:
            return "present_text_message payload must be a string"
        case .emptyText:
            return "present_text_message payload must not be empty"
        }
    }
}

func extractPresentedText(from payload: JSONValue) throws -> String {
    guard let text = payload.stringValue else {
        throw PresentTextPayloadError.expectedString
    }
    let trimmed = text.trimmingCharacters(in: .whitespacesAndNewlines)
    guard !trimmed.isEmpty else {
        throw PresentTextPayloadError.emptyText
    }
    return trimmed
}

func encodeLine<T: Encodable>(_ value: T) throws -> Data {
    let data = try JSONEncoder().encode(value)
    return data + Data([0x0A])
}

func decodeServerMessage(from line: Data) throws -> ServerWireMessage {
    let decoder = JSONDecoder()
    let envelope = try decoder.decode(InboundEnvelopeWire.self, from: line)
    guard envelope.method == "act" else {
        return .ignored(type: envelope.method)
    }

    let bodyData = try JSONEncoder().encode(envelope.body)
    let actBody = try decoder.decode(InboundActBodyWire.self, from: bodyData)
    return .act(action: actBody.act)
}
