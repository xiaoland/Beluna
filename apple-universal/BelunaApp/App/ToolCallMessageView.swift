import SwiftUI

struct ToolCallMessageView: View {
    let message: ChatMessage
    let payload: ToolCallMessagePayload

    @State private var isInputExpanded = false
    @State private var isOutputExpanded = true

    private static let timestampFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.dateFormat = "HH:mm:ss"
        return formatter
    }()

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(spacing: 8) {
                Image(systemName: "wrench.and.screwdriver.fill")
                    .foregroundStyle(Color.accentColor)
                Text("Tool Call")
                    .font(.caption.weight(.semibold))
                Text("cycle \(payload.cycleID)")
                    .font(.caption.monospaced())
                    .foregroundStyle(.secondary)
                Text(payload.stage)
                    .font(.caption.monospaced())
                    .foregroundStyle(.secondary)
                Spacer()
                Text(Self.timestampFormatter.string(from: message.timestamp))
                    .font(.caption2.monospaced())
                    .foregroundStyle(.secondary)
            }

            DisclosureGroup("Input", isExpanded: $isInputExpanded) {
                payloadText(payload.inputPayload)
            }
            .font(.caption)

            DisclosureGroup("Output", isExpanded: $isOutputExpanded) {
                payloadText(payload.outputPayload)
            }
            .font(.caption)
        }
        .padding(12)
        .background(Color.accentColor.opacity(0.08), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
    }

    private func payloadText(_ payload: String) -> some View {
        ScrollView {
            Text(payload)
                .font(.system(.caption, design: .monospaced))
                .textSelection(.enabled)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(8)
                .background(Color.primary.opacity(0.05), in: RoundedRectangle(cornerRadius: 8, style: .continuous))
        }
        .frame(maxHeight: 200)
    }
}
