//
//  BelunaAppTests.swift
//  BelunaAppTests
//
//  Created by Lan_zhijiang on 2026/2/20.
//

import Foundation
import Testing
@testable import BelunaApp

struct BelunaAppTests {

    @Test func authEnvelopeMatchesCoreContract() throws {
        let envelope = makeAppleEndpointRegisterEnvelope()

        #expect(envelope.method == "auth")
        #expect(envelope.body.endpointName == appleEndpointID)
        #expect(envelope.body.capabilities.count == 1)
        #expect(UUID(uuidString: envelope.id) != nil)
        #expect(envelope.timestamp > 0)
    }

    @Test func actAckEnvelopeMatchesCoreContract() throws {
        let actID = "0194f1f3-cc2f-7aa7-8d4c-486f9f2f7c0a"
        let envelope = makeActAckEnvelope(actID: actID)

        #expect(envelope.method == "act_ack")
        #expect(envelope.body.actID == actID)
        #expect(UUID(uuidString: envelope.id) != nil)
        #expect(envelope.timestamp > 0)
    }

    @Test func correlatedSenseUsesActID() throws {
        let action = AdmittedActionWire(
            neuralSignalID: "0194f1f3-cc2f-7aa7-8d4c-486f9f2f7c0a",
            capabilityInstanceID: "chat.instance",
            endpointID: appleEndpointID,
            capabilityID: appleCapabilityID,
            normalizedPayload: .object(["text": .string("hello")]),
            reservedCost: .zero
        )
        let envelope = makeActResultSenseEnvelope(
            action: action,
            status: "applied",
            referenceID: "apple-universal:chat:0194f1f3-cc2f-7aa7-8d4c-486f9f2f7c0a"
        )

        #expect(envelope.method == "sense")
        #expect(UUID(uuidString: envelope.body.senseID) != nil)
        guard let payload = envelope.body.payload.objectValue else {
            Issue.record("sense payload should be an object")
            return
        }
        #expect(payload["act_id"]?.stringValue == action.neuralSignalID)
        #expect(payload["capability_instance_id"]?.stringValue == action.capabilityInstanceID)
        #expect(payload["endpoint_id"]?.stringValue == action.endpointID)
        #expect(payload["capability_id"]?.stringValue == action.capabilityID)
    }

    @Test func decodesCoreActEnvelope() throws {
        let wire = """
        {
          "method":"act",
          "id":"2f8daebf-f529-4ea4-b322-7df109e86d66",
          "timestamp":1739500000000,
          "body":{
            "act":{
              "act_id":"0194f1f3-cc2f-7aa7-8d4c-486f9f2f7c0a",
              "body_endpoint_name":"macos-app.01",
              "capability_id":"present.message",
              "capability_instance_id":"chat.instance",
              "normalized_payload":{"response":{"output_text":"hello"}},
              "requested_resources":{
                "survival_micro":120,
                "time_ms":100,
                "io_units":1,
                "token_units":64
              }
            }
          }
        }
        """

        let message = try decodeServerMessage(from: Data(wire.utf8))
        guard case let .act(action) = message else {
            Issue.record("expected act message")
            return
        }
        #expect(action.neuralSignalID == "0194f1f3-cc2f-7aa7-8d4c-486f9f2f7c0a")
        #expect(action.endpointID == "macos-app.01")
        #expect(action.capabilityID == "present.message")
        #expect(action.capabilityInstanceID == "chat.instance")
    }

    @Test func userSenseEnvelopeUsesUUIDv4SenseID() throws {
        let envelope = makeUserSenseEnvelope(conversationID: "conv_1", text: "hello")
        guard let senseUUID = UUID(uuidString: envelope.body.senseID) else {
            Issue.record("sense_id should be uuid string")
            return
        }

        #expect(envelope.method == "sense")
        #expect(uuidVersion(senseUUID) == 4)
    }

    private func uuidVersion(_ uuid: UUID) -> Int {
        withUnsafeBytes(of: uuid.uuid) { bytes in
            Int((bytes[6] & 0xF0) >> 4)
        }
    }

}
