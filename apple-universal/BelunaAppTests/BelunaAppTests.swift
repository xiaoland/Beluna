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
        let envelope = makeBodyEndpointRegisterEnvelope()

        #expect(envelope.method == "auth")
        #expect(envelope.body.endpointName == runtimeBodyEndpointID)
        #expect(envelope.body.capabilities.count == 4)
        #expect(UUID(uuidString: envelope.id) != nil)
        #expect(envelope.timestamp > 0)
    }

    @Test func authEnvelopeDeclaresActPayloadSchema() throws {
        let envelope = makeBodyEndpointRegisterEnvelope()
        guard let descriptor = descriptor(
            withID: bodyEndpointActPresentMessageTextDescriptorID,
            in: envelope.body.capabilities
        ) else {
            Issue.record("missing act descriptor \(bodyEndpointActPresentMessageTextDescriptorID)")
            return
        }

        guard let schema = descriptor.payloadSchema.objectValue else {
            Issue.record("act payload_schema should be an object")
            return
        }

        #expect(schema["type"]?.stringValue == "string")
        #expect(schema["required"] == nil)
        #expect(schema["properties"] == nil)
    }

    @Test func authEnvelopeDeclaresSensePayloadSchemas() throws {
        let envelope = makeBodyEndpointRegisterEnvelope()

        guard let userSense = descriptor(
            withID: bodyEndpointSenseUserMessageTextDescriptorID,
            in: envelope.body.capabilities
        ) else {
            Issue.record("missing sense descriptor \(bodyEndpointSenseUserMessageTextDescriptorID)")
            return
        }
        #expect(userSense.payloadSchema.objectValue?["type"]?.stringValue == "string")

        guard let successSense = descriptor(
            withID: bodyEndpointSensePresentMessageTextSuccessDescriptorID,
            in: envelope.body.capabilities
        ) else {
            Issue.record("missing sense descriptor \(bodyEndpointSensePresentMessageTextSuccessDescriptorID)")
            return
        }
        #expect(successSense.payloadSchema.objectValue?["type"]?.stringValue == "object")
        #expect(successSense.payloadSchema.objectValue?["additionalProperties"] == nil)

        guard let failureSense = descriptor(
            withID: bodyEndpointSensePresentMessageTextFailureDescriptorID,
            in: envelope.body.capabilities
        ) else {
            Issue.record("missing sense descriptor \(bodyEndpointSensePresentMessageTextFailureDescriptorID)")
            return
        }
        #expect(failureSense.payloadSchema.objectValue?["type"]?.stringValue == "object")
        #expect(failureSense.payloadSchema.objectValue?["additionalProperties"] == nil)
    }

    @Test func actAckEnvelopeMatchesCoreContract() throws {
        let actID = "0194f1f3-cc2f-7aa7-8d4c-486f9f2f7c0a"
        let envelope = makeActAckEnvelope(actID: actID)

        #expect(envelope.method == "act_ack")
        #expect(envelope.body.actID == actID)
        #expect(UUID(uuidString: envelope.id) != nil)
        #expect(envelope.timestamp > 0)
    }

    @Test func succeededSenseUsesMetadataForActID() throws {
        let action = InboundActWire(
            actID: "0194f1f3-cc2f-7aa7-8d4c-486f9f2f7c0a",
            endpointID: runtimeBodyEndpointID,
            neuralSignalDescriptorID: bodyEndpointActPresentMessageTextDescriptorID,
            payload: .string("hello")
        )
        let envelope = makeActPresentationSucceededSenseEnvelope(action: action)

        #expect(envelope.method == "sense")
        #expect(envelope.body.neuralSignalDescriptorID == bodyEndpointSensePresentMessageTextSuccessDescriptorID)
        #expect(UUID(uuidString: envelope.body.senseID) != nil)
        #expect(envelope.body.payload.objectValue == [:])
        #expect(envelope.body.metadata?.objectValue?["act_instance_id"]?.stringValue == action.actID)
    }

    @Test func rejectedSenseIncludesReasonCodeAndMetadataActID() throws {
        let action = InboundActWire(
            actID: "0194f1f3-cc2f-7aa7-8d4c-486f9f2f7c0a",
            endpointID: runtimeBodyEndpointID,
            neuralSignalDescriptorID: bodyEndpointActPresentMessageTextDescriptorID,
            payload: .string("hello")
        )
        let envelope = makeActPresentationRejectedSenseEnvelope(action: action, reasonCode: "invalid_payload")

        #expect(envelope.method == "sense")
        #expect(envelope.body.neuralSignalDescriptorID == bodyEndpointSensePresentMessageTextFailureDescriptorID)
        #expect(UUID(uuidString: envelope.body.senseID) != nil)
        #expect(envelope.body.payload.objectValue?["reason_code"]?.stringValue == "invalid_payload")
        #expect(envelope.body.metadata?.objectValue?["act_instance_id"]?.stringValue == action.actID)
    }

    @Test func decodesCoreActEnvelope() throws {
        let wire = """
        {
          "method":"act",
          "id":"2f8daebf-f529-4ea4-b322-7df109e86d66",
          "timestamp":1739500000000,
          "body":{
            "act":{
              "act_instance_id":"0194f1f3-cc2f-7aa7-8d4c-486f9f2f7c0a",
              "endpoint_id":"\(runtimeBodyEndpointID)",
              "neural_signal_descriptor_id":"present.message.text",
              "payload":"hello"
            }
          }
        }
        """

        let message = try decodeServerMessage(from: Data(wire.utf8))
        guard case let .act(action) = message else {
            Issue.record("expected act message")
            return
        }
        #expect(action.actID == "0194f1f3-cc2f-7aa7-8d4c-486f9f2f7c0a")
        #expect(action.endpointID == runtimeBodyEndpointID)
        #expect(action.neuralSignalDescriptorID == bodyEndpointActPresentMessageTextDescriptorID)
        #expect(action.payload.stringValue == "hello")
    }

    @Test func userSenseEnvelopeUsesStringPayload() throws {
        let envelope = makeUserTextSubmittedSenseEnvelope(text: "hello")
        guard let senseUUID = UUID(uuidString: envelope.body.senseID) else {
            Issue.record("sense_instance_id should be uuid string")
            return
        }

        #expect(envelope.method == "sense")
        #expect(envelope.body.neuralSignalDescriptorID == bodyEndpointSenseUserMessageTextDescriptorID)
        #expect(envelope.body.payload.stringValue == "hello")
        #expect(envelope.body.metadata == nil)
        #expect(uuidVersion(senseUUID) == 4)
    }

    @Test func extractPresentedTextRequiresStringPayload() throws {
        do {
            _ = try extractPresentedText(from: .object(["text": .string("hello")]))
            Issue.record("expected payload validation error")
        } catch let error as PresentTextPayloadError {
            #expect(error == .expectedString)
        } catch {
            Issue.record("unexpected error: \(error)")
        }
    }

    @Test func extractPresentedTextRejectsEmptyText() throws {
        do {
            _ = try extractPresentedText(from: .string("   "))
            Issue.record("expected payload validation error")
        } catch let error as PresentTextPayloadError {
            #expect(error == .emptyText)
        } catch {
            Issue.record("unexpected error: \(error)")
        }
    }

    @Test func extractPresentedTextReturnsTrimmedText() throws {
        let text = try extractPresentedText(from: .string("  hello  "))
        #expect(text == "hello")
    }

    private func uuidVersion(_ uuid: UUID) -> Int {
        withUnsafeBytes(of: uuid.uuid) { bytes in
            Int((bytes[6] & 0xF0) >> 4)
        }
    }

    private func descriptor(
        withID id: String,
        in capabilities: [NeuralSignalDescriptorWire]
    ) -> NeuralSignalDescriptorWire? {
        capabilities.first(where: { $0.neuralSignalDescriptorID == id })
    }

}
