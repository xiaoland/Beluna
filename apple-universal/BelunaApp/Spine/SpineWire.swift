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

    static let zero = CostVectorWire(survivalMicro: 0, timeMS: 0, ioUnits: 0, tokenUnits: 0)
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

struct NDJSONEnvelope<Body: Codable>: Codable {
    let method: String
    let id: String
    let timestamp: UInt64
    let body: Body
}

struct AuthBodyWire: Codable {
    let endpointName: String
    let capabilities: [EndpointDescriptorWire]

    enum CodingKeys: String, CodingKey {
        case endpointName = "endpoint_name"
        case capabilities
    }
}

struct SenseBodyWire: Codable {
    let senseID: String
    let source: String
    let payload: JSONValue

    enum CodingKeys: String, CodingKey {
        case senseID = "sense_id"
        case source
        case payload
    }
}

struct ActAckBodyWire: Codable {
    let actID: String

    enum CodingKeys: String, CodingKey {
        case actID = "act_id"
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
    let bodyEndpointName: String
    let capabilityInstanceID: String
    let capabilityID: String
    let normalizedPayload: JSONValue
    let requestedResources: CostVectorWire

    enum CodingKeys: String, CodingKey {
        case actID = "act_id"
        case bodyEndpointName = "body_endpoint_name"
        case endpointID = "endpoint_id"
        case capabilityInstanceID = "capability_instance_id"
        case capabilityID = "capability_id"
        case normalizedPayload = "normalized_payload"
        case requestedResources = "requested_resources"
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        actID = try container.decode(String.self, forKey: .actID)
        capabilityInstanceID = try container.decode(String.self, forKey: .capabilityInstanceID)
        capabilityID = try container.decode(String.self, forKey: .capabilityID)
        normalizedPayload = try container.decode(JSONValue.self, forKey: .normalizedPayload)
        requestedResources =
            try container.decodeIfPresent(CostVectorWire.self, forKey: .requestedResources)
            ?? .zero
        if let endpoint = try container.decodeIfPresent(String.self, forKey: .bodyEndpointName) {
            bodyEndpointName = endpoint
        } else if let legacyEndpoint = try container.decodeIfPresent(String.self, forKey: .endpointID) {
            bodyEndpointName = legacyEndpoint
        } else {
            throw DecodingError.keyNotFound(
                CodingKeys.bodyEndpointName,
                DecodingError.Context(
                    codingPath: container.codingPath,
                    debugDescription: "missing both body_endpoint_name and endpoint_id"
                )
            )
        }
    }

    var asAdmittedAction: AdmittedActionWire {
        AdmittedActionWire(
            neuralSignalID: actID,
            capabilityInstanceID: capabilityInstanceID,
            endpointID: bodyEndpointName,
            capabilityID: capabilityID,
            normalizedPayload: normalizedPayload,
            reservedCost: requestedResources
        )
    }
}

private struct InboundEnvelopeWire: Decodable {
    let method: String
    let id: String
    let timestamp: UInt64
    let body: JSONValue
}

private struct InboundActBodyWire: Decodable {
    let act: CoreActWire
}

private struct LegacyInboundMessageWire: Decodable {
    let type: String
    let action: AdmittedActionWire?
    let act: CoreActWire?
}

enum ServerWireMessage: Equatable {
    case act(action: AdmittedActionWire)
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
            endpointName: appleEndpointID,
            capabilities: [
                EndpointDescriptorWire(
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
            ]
        )
    )
}

func makeUserSenseEnvelope(conversationID: String, text: String) -> NDJSONEnvelope<SenseBodyWire> {
    makeEnvelope(
        method: "sense",
        body: SenseBodyWire(
            senseID: UUID().uuidString.lowercased(),
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
    )
}

func makeActResultSenseEnvelope(
    action: AdmittedActionWire,
    status: String,
    referenceID: String,
    reasonCode: String? = nil
) -> NDJSONEnvelope<SenseBodyWire> {
    var payload: [String: JSONValue] = [
        "kind": .string("present_message_result"),
        "status": .string(status),
        "act_id": .string(action.neuralSignalID),
        "capability_instance_id": .string(action.capabilityInstanceID),
        "endpoint_id": .string(action.endpointID),
        "capability_id": .string(action.capabilityID),
        "reference_id": .string(referenceID)
    ]
    if let reasonCode {
        payload["reason_code"] = .string(reasonCode)
    }

    return makeEnvelope(
        method: "sense",
        body: SenseBodyWire(
            senseID: UUID().uuidString.lowercased(),
            source: "apple.universal.chat",
            payload: .object(payload)
        )
    )
}

func makeActAckEnvelope(actID: String) -> NDJSONEnvelope<ActAckBodyWire> {
    makeEnvelope(method: "act_ack", body: ActAckBodyWire(actID: actID))
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
    let decoder = JSONDecoder()

    do {
        let envelope = try decoder.decode(InboundEnvelopeWire.self, from: line)
        guard envelope.method == "act" else {
            return .ignored(type: envelope.method)
        }

        let bodyData = try JSONEncoder().encode(envelope.body)
        let actBody = try decoder.decode(InboundActBodyWire.self, from: bodyData)
        return .act(action: actBody.act.asAdmittedAction)
    } catch {
        let legacy = try decoder.decode(LegacyInboundMessageWire.self, from: line)
        guard legacy.type == "act" else {
            return .ignored(type: legacy.type)
        }
        if let action = legacy.action {
            return .act(action: action)
        }
        if let act = legacy.act {
            return .act(action: act.asAdmittedAction)
        }

        throw DecodingError.dataCorrupted(
            DecodingError.Context(
                codingPath: [],
                debugDescription: "legacy act message is missing both action and act payload"
            )
        )
    }
}
