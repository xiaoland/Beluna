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
        #expect(envelope.body.endpointName == appleEndpointName)
        #expect(envelope.body.capabilities.count == 3)
        #expect(UUID(uuidString: envelope.id) != nil)
        #expect(envelope.timestamp > 0)
    }

    @Test func authEnvelopeDeclaresActPayloadSchema() throws {
        let envelope = makeAppleEndpointRegisterEnvelope()
        guard let descriptor = descriptor(
            withID: appleActNeuralSignalDescriptorID,
            in: envelope.body.capabilities
        ) else {
            Issue.record("missing act descriptor \(appleActNeuralSignalDescriptorID)")
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
        let envelope = makeAppleEndpointRegisterEnvelope()

        guard let userSense = descriptor(
            withID: appleUserSenseNeuralSignalDescriptorID,
            in: envelope.body.capabilities
        ) else {
            Issue.record("missing sense descriptor \(appleUserSenseNeuralSignalDescriptorID)")
            return
        }

        guard let userSenseSchema = userSense.payloadSchema.objectValue else {
            Issue.record("user sense payload_schema should be an object")
            return
        }
        #expect(
            userSenseSchema["properties"]?.objectValue?["conversation_id"]?.objectValue?["type"]?.stringValue
                == "string"
        )
        #expect(userSenseSchema["properties"]?.objectValue?["input"]?.objectValue?["type"]?.stringValue == "array")

        guard let actResultSense = descriptor(
            withID: appleActResultSenseNeuralSignalDescriptorID,
            in: envelope.body.capabilities
        ) else {
            Issue.record("missing sense descriptor \(appleActResultSenseNeuralSignalDescriptorID)")
            return
        }

        guard let actResultSchema = actResultSense.payloadSchema.objectValue else {
            Issue.record("act result payload_schema should be an object")
            return
        }
        #expect(
            actResultSchema["required"]?.arrayValue?.contains(.string("kind")) == true
                && actResultSchema["required"]?.arrayValue?.contains(.string("act_id")) == true
                && actResultSchema["required"]?.arrayValue?.contains(.string("status")) == true
                && actResultSchema["required"]?.arrayValue?.contains(.string("reference_id")) == true
        )
        #expect(
            actResultSchema["properties"]?.objectValue?["kind"]?.objectValue?["const"]?.stringValue
                == "present_message_result"
        )
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
        let action = InboundActWire(
            actID: "0194f1f3-cc2f-7aa7-8d4c-486f9f2f7c0a",
            endpointID: "macos-app.1",
            neuralSignalDescriptorID: appleActNeuralSignalDescriptorID,
            payload: .string("hello")
        )
        let envelope = makeActResultSenseEnvelope(
            action: action,
            status: "applied",
            referenceID: "apple-universal:chat:\(action.actID)"
        )

        #expect(envelope.method == "sense")
        #expect(envelope.body.neuralSignalDescriptorID == appleActResultSenseNeuralSignalDescriptorID)
        #expect(UUID(uuidString: envelope.body.senseID) != nil)
        guard let payload = envelope.body.payload.objectValue else {
            Issue.record("sense payload should be an object")
            return
        }
        #expect(payload["act_id"]?.stringValue == action.actID)
        #expect(payload["status"]?.stringValue == "applied")
        #expect(payload["reference_id"]?.stringValue == "apple-universal:chat:\(action.actID)")
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
              "endpoint_id":"macos-app.1",
              "neural_signal_descriptor_id":"present_text_message",
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
        #expect(action.endpointID == "macos-app.1")
        #expect(action.neuralSignalDescriptorID == "present_text_message")
        #expect(action.payload.stringValue == "hello")
    }

    @Test func userSenseEnvelopeUsesUUIDv4SenseID() throws {
        let envelope = makeUserSenseEnvelope(conversationID: "conv_1", text: "hello")
        guard let senseUUID = UUID(uuidString: envelope.body.senseID) else {
            Issue.record("sense_id should be uuid string")
            return
        }

        #expect(envelope.method == "sense")
        #expect(envelope.body.neuralSignalDescriptorID == appleUserSenseNeuralSignalDescriptorID)
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
