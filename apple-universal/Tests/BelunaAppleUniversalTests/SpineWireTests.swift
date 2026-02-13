import XCTest
@testable import BelunaAppleUniversalApp

final class SpineWireTests: XCTestCase {
    func testRegisterEnvelopeRouteMatchesAppleChatEndpoint() throws {
        let envelope = makeAppleEndpointRegisterEnvelope()

        XCTAssertEqual(envelope.type, "body_endpoint_register")
        XCTAssertEqual(envelope.endpointID, appleEndpointID)
        XCTAssertEqual(envelope.descriptor.route.endpointID, appleEndpointID)
        XCTAssertEqual(envelope.descriptor.route.capabilityID, appleCapabilityID)
    }

    func testUserSenseEnvelopeMatchesResponsesSubset() throws {
        let envelope = makeUserSenseEnvelope(conversationID: "conv_1", text: "hi")
        let data = try JSONEncoder().encode(envelope)

        let decoded = try JSONDecoder().decode([String: JSONValue].self, from: data)
        XCTAssertEqual(decoded["type"], .string("sense"))
        XCTAssertEqual(decoded["source"], .string("apple.universal.chat"))

        let payload = decoded["payload"]?.objectValue
        XCTAssertEqual(payload?["conversation_id"], .string("conv_1"))

        let input = payload?["input"]?.arrayValue
        let firstMessage = input?.first?.objectValue
        XCTAssertEqual(firstMessage?["type"], .string("message"))
        XCTAssertEqual(firstMessage?["role"], .string("user"))
    }

    func testDecodeActAndExtractAssistantText() throws {
        let line = """
        {
          "type": "act",
          "action": {
            "neural_signal_id": "018f94da-9f92-7bc5-bc58-b5f01b0406f5",
            "capability_instance_id": "chat.1",
            "endpoint_id": "macos-app.01",
            "capability_id": "present.message",
            "normalized_payload": {
              "conversation_id": "conv_1",
              "response": {
                "object": "response",
                "id": "resp_1",
                "output": [
                  {
                    "type": "message",
                    "role": "assistant",
                    "content": [
                      { "type": "output_text", "text": "Hello from Beluna" }
                    ]
                  }
                ]
              }
            },
            "reserved_cost": {
              "survival_micro": 120,
              "time_ms": 100,
              "io_units": 1,
              "token_units": 64
            }
          }
        }
        """.data(using: .utf8)!

        let message = try decodeServerMessage(from: line)

        switch message {
        case let .act(action):
            XCTAssertEqual(action.neuralSignalID, "018f94da-9f92-7bc5-bc58-b5f01b0406f5")
            XCTAssertEqual(action.endpointID, appleEndpointID)
            XCTAssertEqual(action.capabilityID, appleCapabilityID)
            let texts = try extractAssistantTexts(from: action.normalizedPayload)
            XCTAssertEqual(texts, ["Hello from Beluna"])
        }
    }

    func testActResultSenseEnvelopeEchoesCorrelationFields() throws {
        let action = AdmittedActionWire(
            neuralSignalID: "018f94da-9f92-7bc5-bc58-b5f01b0406f5",
            capabilityInstanceID: "chat.1",
            endpointID: appleEndpointID,
            capabilityID: appleCapabilityID,
            normalizedPayload: .object([:]),
            reservedCost: CostVectorWire(
                survivalMicro: 120,
                timeMS: 100,
                ioUnits: 1,
                tokenUnits: 64
            )
        )

        let envelope = makeActResultSenseEnvelope(
            action: action,
            status: "applied",
            referenceID: "apple-universal:chat:018f94da-9f92-7bc5-bc58-b5f01b0406f5"
        )
        let data = try JSONEncoder().encode(envelope)
        let decoded = try JSONDecoder().decode([String: JSONValue].self, from: data)
        let payload = decoded["payload"]?.objectValue

        XCTAssertEqual(payload?["neural_signal_id"], .string(action.neuralSignalID))
        XCTAssertEqual(payload?["capability_instance_id"], .string(action.capabilityInstanceID))
        XCTAssertEqual(payload?["endpoint_id"], .string(action.endpointID))
        XCTAssertEqual(payload?["capability_id"], .string(action.capabilityID))
    }
}
