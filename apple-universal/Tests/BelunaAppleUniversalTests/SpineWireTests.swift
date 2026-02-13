import XCTest
@testable import BelunaAppleUniversalApp

final class SpineWireTests: XCTestCase {
    func testRegisterEnvelopeRouteMatchesAppleChatEndpoint() throws {
        let envelope = makeAppleEndpointRegisterEnvelope()

        XCTAssertEqual(envelope.type, "body_endpoint_register")
        XCTAssertEqual(envelope.endpointID, appleEndpointID)
        XCTAssertEqual(envelope.descriptor.route.affordanceKey, appleAffordanceKey)
        XCTAssertEqual(envelope.descriptor.route.capabilityHandle, appleCapabilityHandle)
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

    func testDecodeEndpointInvokeAndExtractAssistantText() throws {
        let line = """
        {
          "type": "body_endpoint_invoke",
          "request_id": "req:1",
          "action": {
            "action_id": "act:1",
            "affordance_key": "chat.reply.emit",
            "capability_handle": "cap.apple.universal.chat",
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
        case let .endpointInvoke(requestID, action):
            XCTAssertEqual(requestID, "req:1")
            XCTAssertEqual(action.affordanceKey, appleAffordanceKey)
            let texts = try extractAssistantTexts(from: action.normalizedPayload)
            XCTAssertEqual(texts, ["Hello from Beluna"])
        }
    }
}
