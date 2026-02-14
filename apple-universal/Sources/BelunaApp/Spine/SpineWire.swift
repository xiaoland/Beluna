import Foundation

let appleEndpointID = "macos-app.01"
let appleCapabilityID = "present.message"

struct RouteDescriptor: Codable, Equatable {
    let endpointID: String
    let capabilityID: String

    enum CodingKeys: String, CodingKey {
        case endpointID = "endpoint_id"
        case capabilityID = "capability_id"
    }
}

struct CostVectorWire: Codable, Equatable {
    let survivalMicro: Int64
    let timeMS: UInt64
    let ioUnits: UInt64
    let tokenUnits: UInt64

    enum CodingKeys: String, CodingKey {
        case survivalMicro = "survival_micro"
        case timeMS = "time_ms"
        case ioUnits = "io_units"
        case tokenUnits = "token_units"
    }
}

struct EndpointDescriptorWire: Codable, Equatable {
    let route: RouteDescriptor
    let payloadSchema: JSONValue
    let maxPayloadBytes: Int
    let defaultCost: CostVectorWire
    let metadata: [String: String]

    enum CodingKeys: String, CodingKey {
        case route
        case payloadSchema = "payload_schema"
        case maxPayloadBytes = "max_payload_bytes"
        case defaultCost = "default_cost"
        case metadata
    }
}

struct EndpointRegisterWire: Codable {
    let type = "body_endpoint_register"
    let endpointID: String
    let descriptor: EndpointDescriptorWire

    enum CodingKeys: String, CodingKey {
        case type
        case endpointID = "endpoint_id"
        case descriptor
    }
}

struct SenseWire: Codable {
    let type = "sense"
    let senseID: String
    let source: String
    let payload: JSONValue

    enum CodingKeys: String, CodingKey {
        case type
        case senseID = "sense_id"
        case source
        case payload
    }
}

struct AdmittedActionWire: Decodable, Equatable {
    let neuralSignalID: String
    let capabilityInstanceID: String
    let endpointID: String
    let capabilityID: String
    let normalizedPayload: JSONValue
    let reservedCost: CostVectorWire

    enum CodingKeys: String, CodingKey {
        case neuralSignalID = "neural_signal_id"
        case capabilityInstanceID = "capability_instance_id"
        case endpointID = "endpoint_id"
        case capabilityID = "capability_id"
        case normalizedPayload = "normalized_payload"
        case reservedCost = "reserved_cost"
    }
}

struct CoreActWire: Decodable, Equatable {
    let actID: String
    let capabilityInstanceID: String
    let endpointID: String
    let capabilityID: String
    let normalizedPayload: JSONValue
    let requestedResources: CostVectorWire

    enum CodingKeys: String, CodingKey {
        case actID = "act_id"
        case capabilityInstanceID = "capability_instance_id"
        case endpointID = "endpoint_id"
        case capabilityID = "capability_id"
        case normalizedPayload = "normalized_payload"
        case requestedResources = "requested_resources"
    }

    var asAdmittedAction: AdmittedActionWire {
        AdmittedActionWire(
            neuralSignalID: actID,
            capabilityInstanceID: capabilityInstanceID,
            endpointID: endpointID,
            capabilityID: capabilityID,
            normalizedPayload: normalizedPayload,
            reservedCost: requestedResources
        )
    }
}

enum ServerWireMessage: Decodable, Equatable {
    case act(action: AdmittedActionWire)
    case ignored(type: String)

    enum CodingKeys: String, CodingKey {
        case type
        case action
        case act
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        let type = try container.decode(String.self, forKey: .type)

        guard type == "act" else {
            self = .ignored(type: type)
            return
        }

        if let action = try container.decodeIfPresent(AdmittedActionWire.self, forKey: .action) {
            self = .act(action: action)
            return
        }

        if let coreAct = try container.decodeIfPresent(CoreActWire.self, forKey: .act) {
            self = .act(action: coreAct.asAdmittedAction)
            return
        }

        throw DecodingError.dataCorruptedError(
            forKey: .action,
            in: container,
            debugDescription: "act message is missing both action and act payload"
        )
    }
}

func makeAppleEndpointRegisterEnvelope() -> EndpointRegisterWire {
    EndpointRegisterWire(
        endpointID: appleEndpointID,
        descriptor: EndpointDescriptorWire(
            route: RouteDescriptor(
                endpointID: appleEndpointID,
                capabilityID: appleCapabilityID
            ),
            payloadSchema: .object([
                "type": .string("object"),
                "required": .array([.string("conversation_id"), .string("response")])
            ]),
            maxPayloadBytes: 32_768,
            defaultCost: CostVectorWire(
                survivalMicro: 120,
                timeMS: 100,
                ioUnits: 1,
                tokenUnits: 64
            ),
            metadata: ["app": "apple-universal"]
        )
    )
}

func makeUserSenseEnvelope(conversationID: String, text: String) -> SenseWire {
    SenseWire(
        senseID: "sense:apple:\(UUID().uuidString.lowercased())",
        source: "apple.universal.chat",
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
}

func makeActResultSenseEnvelope(
    action: AdmittedActionWire,
    status: String,
    referenceID: String,
    reasonCode: String? = nil
) -> SenseWire {
    var payload: [String: JSONValue] = [
        "kind": .string("present_message_result"),
        "status": .string(status),
        "neural_signal_id": .string(action.neuralSignalID),
        "capability_instance_id": .string(action.capabilityInstanceID),
        "endpoint_id": .string(action.endpointID),
        "capability_id": .string(action.capabilityID),
        "reference_id": .string(referenceID)
    ]
    if let reasonCode {
        payload["reason_code"] = .string(reasonCode)
    }

    return SenseWire(
        senseID: "sense:apple:\(UUID().uuidString.lowercased())",
        source: "apple.universal.chat",
        payload: .object(payload)
    )
}

func extractAssistantTexts(from normalizedPayload: JSONValue) throws -> [String] {
    guard let payload = normalizedPayload.objectValue else {
        return []
    }

    var result: [String] = []
    result.append(contentsOf: extractTextsFromResponsesOutput(payload["response"]))
    result.append(contentsOf: extractTextsFromOutputItems(payload["output"]?.arrayValue))
    result.append(contentsOf: extractTextsFromChoices(payload["choices"]?.arrayValue))

    for key in ["output_text", "text", "message"] {
        if let value = payload[key]?.stringValue {
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
    try JSONDecoder().decode(ServerWireMessage.self, from: line)
}
