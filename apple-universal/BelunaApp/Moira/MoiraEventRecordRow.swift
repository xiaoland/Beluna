import SwiftUI

struct MoiraEventRecordRow: View {
    var record: MoiraEventRecord

    var body: some View {
        DisclosureGroup {
            VStack(alignment: .leading, spacing: 6) {
                LabeledContent("Record", value: record.recordKind)
                LabeledContent("Owner", value: record.ownerText)
                if let traceID = record.traceID {
                    pathText("Trace", traceID)
                }
                if let spanID = record.spanID {
                    pathText("Span", spanID)
                }
                if let messageText = record.messageText {
                    Text(messageText)
                        .font(.caption)
                        .textSelection(.enabled)
                }
                jsonPreview("Body", record.body)
                jsonPreview("Attributes", record.attributes)
            }
            .padding(.top, 4)
        } label: {
            HStack(spacing: 8) {
                Text(record.severityText)
                    .font(.caption.monospaced().weight(.semibold))
                    .frame(width: 56, alignment: .leading)

                Text(record.displayTitle)
                    .lineLimit(1)

                Spacer()

                Text(record.observedAt)
                    .font(.caption.monospaced())
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
        }
    }

    private func pathText(_ label: String, _ value: String) -> some View {
        LabeledContent(label) {
            Text(value)
                .font(.caption.monospaced())
                .lineLimit(2)
                .textSelection(.enabled)
        }
    }

    private func jsonPreview(_ label: String, _ value: JSONValue) -> some View {
        LabeledContent(label) {
            Text(MoiraJSONFormatter.compactString(value))
                .font(.caption.monospaced())
                .lineLimit(5)
                .textSelection(.enabled)
        }
    }
}
