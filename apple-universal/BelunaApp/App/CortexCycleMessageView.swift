import SwiftUI

struct CortexCycleMessageView: View {
    let message: ChatMessage
    let payload: CortexCycleMessagePayload
    @State private var isDetailPresented = false

    private static let timestampFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.dateFormat = "HH:mm:ss"
        return formatter
    }()

    var body: some View {
        Button {
            isDetailPresented = true
        } label: {
            cycleSummaryCard
        }
        .buttonStyle(.plain)
        .sheet(isPresented: $isDetailPresented) {
            CortexCycleDetailView(payload: payload)
        }
    }

    private var cycleSummaryCard: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(spacing: 8) {
                Image(systemName: "cpu")
                    .foregroundStyle(Color.accentColor)
                Text("Cortex Cycle")
                    .font(.caption.weight(.semibold))
                Text("\(payload.cycleID)")
                    .font(.caption.monospaced())
                    .foregroundStyle(.secondary)
                Spacer()
                Text(Self.timestampFormatter.string(from: message.timestamp))
                    .font(.caption2.monospaced())
                    .foregroundStyle(.secondary)
            }

            HStack(spacing: 8) {
                Text("\(payload.organActivityMessages.count) organ activity message(s)")
                    .font(.caption2.monospaced())
                    .foregroundStyle(.secondary)
                if let latestStage = payload.organActivityMessages.last?.stage {
                    Text("latest stage: \(latestStage)")
                        .font(.caption2.monospaced())
                        .foregroundStyle(.secondary)
                }
            }

            Text("Click to inspect this cycle's organ activity messages.")
                .font(.caption)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .padding(12)
        .background(Color.accentColor.opacity(0.08), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
    }
}

private struct CortexCycleDetailView: View {
    let payload: CortexCycleMessagePayload
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            ScrollView {
                LazyVStack(spacing: 12) {
                    ForEach(payload.organActivityMessages) { organActivity in
                        OrganActivityMessageDetailRow(message: organActivity)
                    }
                }
                .padding(12)
            }
            .navigationTitle("Cortex Cycle \(payload.cycleID)")
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
        }
        .frame(minWidth: 760, minHeight: 520)
    }
}

private struct OrganActivityMessageDetailRow: View {
    let message: OrganActivityMessagePayload

    private static let timestampFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.locale = Locale(identifier: "en_US_POSIX")
        formatter.dateFormat = "HH:mm:ss"
        return formatter
    }()

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(spacing: 8) {
                Text("Organ Activity")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)
                Text(message.stage)
                    .font(.caption.monospaced().weight(.semibold))
                    .foregroundStyle(.primary)
                Spacer()
                Text(Self.timestampFormatter.string(from: message.timestamp))
                    .font(.caption2.monospaced())
                    .foregroundStyle(.secondary)
            }

            payloadSection(title: "Input", payload: message.inputPayload)
            payloadSection(title: "Output", payload: message.outputPayload)
        }
        .padding(12)
        .background(Color.primary.opacity(0.05), in: RoundedRectangle(cornerRadius: 10, style: .continuous))
    }

    private func payloadSection(title: String, payload: String) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(title)
                .font(.caption.weight(.semibold))
                .foregroundStyle(.secondary)

            Text(payload.isEmpty ? "(empty)" : payload)
                .font(.system(.caption, design: .monospaced))
                .textSelection(.enabled)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(8)
                .background(Color.primary.opacity(0.05), in: RoundedRectangle(cornerRadius: 8, style: .continuous))
        }
    }
}
