import Foundation

let appleEndpointName = "macos-app"
let appleActNeuralSignalDescriptorID = "present.message"
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
                        "type": .string("object")
                    ])
                ),
                NeuralSignalDescriptorWire(
                    type: .sense,
                    endpointID: appleEndpointName,
                    neuralSignalDescriptorID: appleUserSenseNeuralSignalDescriptorID,
                    payloadSchema: .object([
                        "type": .string("object"),
                        "required": .array([.string("conversation_id"), .string("input")])
                    ])
                ),
                NeuralSignalDescriptorWire(
                    type: .sense,
                    endpointID: appleEndpointName,
                    neuralSignalDescriptorID: appleActResultSenseNeuralSignalDescriptorID,
                    payloadSchema: .object([
                        "type": .string("object"),
                        "required": .array([.string("act_id"), .string("status"), .string("reference_id")])
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

func extractAssistantTexts(from payload: JSONValue) throws -> [String] {
    guard let object = payload.objectValue else {
        return []
    }

    var result: [String] = []
    result.append(contentsOf: extractTextsFromResponsesOutput(object["response"]))
    result.append(contentsOf: extractTextsFromOutputItems(object["output"]?.arrayValue))
    result.append(contentsOf: extractTextsFromChoices(object["choices"]?.arrayValue))

    for key in ["output_text", "text", "message"] {
        if let value = object[key]?.stringValue {
            result.append(value)
        }
    }

    return dedupeNonEmptyTexts(result)
}

private func extractTextsFromResponsesOutput(_ response: JSONValue?) -> [String] {
    guard let response else {
        return []
    }

    var result: [String] = []
    if let object = response.objectValue {
        if let outputText = object["output_text"]?.stringValue {
            result.append(outputText)
        }
        if let text = object["text"]?.stringValue {
            result.append(text)
        }
        if let message = object["message"] {
            result.append(contentsOf: extractTextsFromMessage(message, role: object["role"]?.stringValue))
        }
        result.append(contentsOf: extractTextsFromOutputItems(object["output"]?.arrayValue))
        result.append(contentsOf: extractTextsFromChoices(object["choices"]?.arrayValue))
    } else {
        result.append(contentsOf: extractTextsFromMessage(response, role: nil))
    }
    return result
}

private func extractTextsFromOutputItems(_ items: [JSONValue]?) -> [String] {
    guard let items else {
        return []
    }

    var result: [String] = []
    for item in items {
        guard let object = item.objectValue else {
            continue
        }

        if let role = object["role"]?.stringValue,
           !role.isEmpty,
           role != "assistant"
        {
            continue
        }

        if let text = object["output_text"]?.stringValue {
            result.append(text)
        }
        if let text = object["text"]?.stringValue {
            result.append(text)
        }

        if let message = object["message"] {
            result.append(contentsOf: extractTextsFromMessage(message, role: object["role"]?.stringValue))
        }

        if let content = object["content"] {
            result.append(contentsOf: extractTextsFromMessage(content, role: object["role"]?.stringValue))
        }
    }

    return result
}

private func extractTextsFromChoices(_ choices: [JSONValue]?) -> [String] {
    guard let choices else {
        return []
    }

    var result: [String] = []
    for choice in choices {
        guard let object = choice.objectValue else {
            continue
        }

        if let message = object["message"] {
            let role = message.objectValue?["role"]?.stringValue
            result.append(contentsOf: extractTextsFromMessage(message, role: role))
            continue
        }

        if let text = object["text"]?.stringValue {
            result.append(text)
        }
    }
    return result
}

private func extractTextsFromMessage(_ message: JSONValue, role: String?) -> [String] {
    if let role, !role.isEmpty, role != "assistant" {
        return []
    }

    if let text = message.stringValue {
        return [text]
    }

    if let array = message.arrayValue {
        var result: [String] = []
        for entry in array {
            guard let content = entry.objectValue else {
                if let text = entry.stringValue {
                    result.append(text)
                }
                continue
            }

            if let entryRole = content["role"]?.stringValue,
               !entryRole.isEmpty,
               entryRole != "assistant"
            {
                continue
            }

            if let text = content["text"]?.stringValue {
                result.append(text)
            }
            if let text = content["output_text"]?.stringValue {
                result.append(text)
            }
            if let nested = content["content"] {
                result.append(contentsOf: extractTextsFromMessage(nested, role: content["role"]?.stringValue))
            }
        }
        return result
    }

    guard let object = message.objectValue else {
        return []
    }

    if let text = object["text"]?.stringValue {
        return [text]
    }
    if let text = object["output_text"]?.stringValue {
        return [text]
    }
    if let nested = object["content"] {
        return extractTextsFromMessage(nested, role: object["role"]?.stringValue)
    }
    return []
}

private func dedupeNonEmptyTexts(_ texts: [String]) -> [String] {
    var seen = Set<String>()
    var result: [String] = []
    for text in texts {
        let trimmed = text.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty, !seen.contains(trimmed) else {
            continue
        }
        seen.insert(trimmed)
        result.append(trimmed)
    }
    return result
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
