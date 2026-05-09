import SwiftUI

struct MoiraWakeRow: View {
    var run: MoiraRunSummary

    var body: some View {
        HStack(spacing: 10) {
            Image(systemName: "power")
                .foregroundStyle(.secondary)
                .frame(width: 16)

            VStack(alignment: .leading, spacing: 2) {
                Text(String(run.runID.prefix(18)))
                    .lineLimit(1)

                Text(detailText)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
        }
    }

    private var detailText: String {
        let latest = run.latestTick.map { ", latest tick \($0)" } ?? ""
        return "\(run.eventCount) events\(latest)"
    }
}

struct MoiraTickRow: View {
    var tick: MoiraTickSummary

    var body: some View {
        VStack(alignment: .leading, spacing: 3) {
            HStack(spacing: 8) {
                Text("Tick \(tick.tick)")
                    .lineLimit(1)

                Spacer()

                Text("\(tick.eventCount)")
                    .font(.caption.monospaced())
                    .foregroundStyle(.secondary)
            }

            Text("\(tick.firstSeenAt) -> \(tick.lastSeenAt)")
                .font(.caption)
                .foregroundStyle(.secondary)
                .lineLimit(1)
        }
        .padding(.vertical, 2)
    }
}

struct MoiraRawEventRow: View {
    var record: MoiraEventRecord

    var body: some View {
        VStack(alignment: .leading, spacing: 3) {
            HStack(spacing: 8) {
                Text(record.severityText)
                    .font(.caption.monospaced().weight(.semibold))
                    .frame(width: 48, alignment: .leading)

                Text(record.displayTitle)
                    .lineLimit(1)
            }

            Text("\(record.ownerText)  \(record.observedAt)")
                .font(.caption)
                .foregroundStyle(.secondary)
                .lineLimit(1)
        }
        .padding(.vertical, 2)
    }
}
