import SwiftUI

struct MoiraRawEventInspector: View {
    var record: MoiraEventRecord?

    var body: some View {
        if let record {
            ScrollView {
                VStack(alignment: .leading, spacing: 14) {
                    header(record)
                    metadata(record)
                    jsonPayload(record)
                }
                .padding(16)
                .frame(maxWidth: .infinity, alignment: .leading)
            }
        } else {
            MoiraO11yEmptyPane(
                title: "Raw Event Empty",
                systemImage: "doc.text.magnifyingglass",
                detail: "Select a tick with ingested records."
            )
        }
    }

    private func header(_ record: MoiraEventRecord) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 8) {
                Text(record.severityText)
                    .font(.caption.monospaced().weight(.semibold))
                    .padding(.horizontal, 6)
                    .padding(.vertical, 3)
                    .background(.quaternary, in: RoundedRectangle(cornerRadius: 4))

                Text(record.recordKind)
                    .font(.caption)
                    .foregroundStyle(.secondary)

                Spacer()
            }

            Text(record.displayTitle)
                .font(.headline)
                .lineLimit(3)
                .textSelection(.enabled)

            Text(record.rawEventID)
                .font(.caption.monospaced())
                .foregroundStyle(.secondary)
                .lineLimit(2)
                .textSelection(.enabled)
        }
    }

    private func metadata(_ record: MoiraEventRecord) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Divider()

            MoiraO11yMetadataLine(title: "Observed", value: record.observedAt, monospaced: true)
            MoiraO11yMetadataLine(title: "Received", value: record.receivedAt, monospaced: true)
            MoiraO11yMetadataLine(title: "Owner", value: record.ownerText)

            if let scopeName = record.scopeName {
                MoiraO11yMetadataLine(title: "Scope", value: scopeName, monospaced: true)
            }
            if let traceID = record.traceID {
                MoiraO11yMetadataLine(title: "Trace", value: traceID, monospaced: true)
            }
            if let spanID = record.spanID {
                MoiraO11yMetadataLine(title: "Span", value: spanID, monospaced: true)
            }
            if let runID = record.runID {
                MoiraO11yMetadataLine(title: "Wake", value: runID, monospaced: true)
            }
            if let tick = record.tick {
                MoiraO11yMetadataLine(title: "Tick", value: "\(tick)", monospaced: true)
            }
            if let messageText = record.messageText {
                MoiraO11yMetadataLine(title: "Message", value: messageText)
            }
        }
    }

    private func jsonPayload(_ record: MoiraEventRecord) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Divider()

            MoiraJSONBlock(title: "Body", value: record.body)
            MoiraJSONBlock(title: "Attributes", value: record.attributes)
            MoiraJSONBlock(title: "Resource", value: record.resource)
            MoiraJSONBlock(title: "Scope", value: record.scope)
        }
    }
}
