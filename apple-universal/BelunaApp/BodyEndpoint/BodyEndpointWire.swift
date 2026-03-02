import Foundation

let appleUniversalBodyEndpointID = "apple-universal"
let macOSBodyEndpointID = "macos-app"
let iOSBodyEndpointID = "ios-app"

let bodyEndpointActPresentMessageTextDescriptorID = "present.message.text"
let bodyEndpointSenseUserMessageTextDescriptorID = "user.message.text"
let bodyEndpointSensePresentMessageTextSuccessDescriptorID = "present.message.text.success"
let bodyEndpointSensePresentMessageTextFailureDescriptorID = "present.message.text.failure"

func resolveRuntimeBodyEndpointID() -> String {
    #if os(macOS)
    return macOSBodyEndpointID
    #elseif os(iOS)
    return iOSBodyEndpointID
    #else
    return appleUniversalBodyEndpointID
    #endif
}

let runtimeBodyEndpointID = resolveRuntimeBodyEndpointID()

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
    let nsDescriptors: [NeuralSignalDescriptorWire]

    enum CodingKeys: String, CodingKey {
        case endpointName = "endpoint_name"
        case nsDescriptors = "ns_descriptors"
    }
}

struct SenseBodyWire: Codable {
    let senseID: String
    let neuralSignalDescriptorID: String
    let payload: String
    let weight: Double
    let actInstanceID: String?

    enum CodingKeys: String, CodingKey {
        case senseID = "sense_instance_id"
        case neuralSignalDescriptorID = "neural_signal_descriptor_id"
        case payload
        case weight
        case actInstanceID = "act_instance_id"
    }
}

struct ActAckBodyWire: Codable {
    let actID: String

    enum CodingKeys: String, CodingKey {
        case actID = "act_instance_id"
    }
}

struct InboundActWire: Decodable, Equatable {
    let actID: String
    let endpointID: String
    let neuralSignalDescriptorID: String
    let payload: JSONValue

    enum CodingKeys: String, CodingKey {
        case actID = "act_instance_id"
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

private func makeSenseDescriptor(id: String, payloadSchema: JSONValue) -> NeuralSignalDescriptorWire {
    NeuralSignalDescriptorWire(
        type: .sense,
        endpointID: runtimeBodyEndpointID,
        neuralSignalDescriptorID: id,
        payloadSchema: payloadSchema
    )
}

func makeBodyEndpointRegisterEnvelope() -> NDJSONEnvelope<AuthBodyWire> {
    makeEnvelope(
        method: "auth",
        body: AuthBodyWire(
            endpointName: runtimeBodyEndpointID,
            nsDescriptors: [
                NeuralSignalDescriptorWire(
                    type: .act,
                    endpointID: runtimeBodyEndpointID,
                    neuralSignalDescriptorID: bodyEndpointActPresentMessageTextDescriptorID,
                    payloadSchema: .object([
                        "type": .string("string"),
                        "description": .string("Text to present in the chat timeline")
                    ])
                ),
                makeSenseDescriptor(
                    id: bodyEndpointSenseUserMessageTextDescriptorID,
                    payloadSchema: .object([
                        "type": .string("string")
                    ])
                ),
                makeSenseDescriptor(
                    id: bodyEndpointSensePresentMessageTextSuccessDescriptorID,
                    payloadSchema: .object([
                        "type": .string("string")
                    ])
                ),
                makeSenseDescriptor(
                    id: bodyEndpointSensePresentMessageTextFailureDescriptorID,
                    payloadSchema: .object([
                        "type": .string("string")
                    ])
                )
            ]
        )
    )
}

func makeUserTextSubmittedSenseEnvelope(text: String) -> NDJSONEnvelope<SenseBodyWire> {
    makeEnvelope(
        method: "sense",
        body: SenseBodyWire(
            senseID: UUID().uuidString.lowercased(),
            neuralSignalDescriptorID: bodyEndpointSenseUserMessageTextDescriptorID,
            payload: text,
            weight: 1.0,
            actInstanceID: nil
        )
    )
}

func makeActPresentationSucceededSenseEnvelope(action: InboundActWire) -> NDJSONEnvelope<SenseBodyWire> {
    makeEnvelope(
        method: "sense",
        body: SenseBodyWire(
            senseID: UUID().uuidString.lowercased(),
            neuralSignalDescriptorID: bodyEndpointSensePresentMessageTextSuccessDescriptorID,
            payload: "presentation_result status=success",
            weight: 0.0,
            actInstanceID: action.actID
        )
    )
}

func makeActPresentationRejectedSenseEnvelope(
    action: InboundActWire,
    reasonCode: String
) -> NDJSONEnvelope<SenseBodyWire> {
    makeEnvelope(
        method: "sense",
        body: SenseBodyWire(
            senseID: UUID().uuidString.lowercased(),
            neuralSignalDescriptorID: bodyEndpointSensePresentMessageTextFailureDescriptorID,
            payload: "presentation_result status=failure reason_code=\(reasonCode)",
            weight: 1.0,
            actInstanceID: action.actID
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
            return "act payload must be a string"
        case .emptyText:
            return "act payload must not be empty"
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
