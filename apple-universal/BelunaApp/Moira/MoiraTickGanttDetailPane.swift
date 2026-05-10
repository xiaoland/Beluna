import SwiftUI

struct MoiraTickGanttDetailPane: View {
    var item: MoiraTickGanttItem?

    var body: some View {
        Group {
            if let item {
                ScrollView {
                    VStack(alignment: .leading, spacing: 12) {
                        header(item)

                        VStack(alignment: .leading, spacing: 8) {
                            ForEach(item.records) { record in
                                MoiraEventRecordRow(record: record)
                                if record.id != item.records.last?.id {
                                    Divider()
                                }
                            }
                        }
                    }
                    .padding(14)
                    .frame(maxWidth: .infinity, alignment: .leading)
                }
            } else {
                MoiraO11yEmptyPane(
                    title: "No Gantt Selection",
                    systemImage: "timeline.selection",
                    detail: "Select a point or interval to inspect its source records."
                )
            }
        }
    }

    private func header(_ item: MoiraTickGanttItem) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(alignment: .firstTextBaseline, spacing: 10) {
                Text(item.title)
                    .font(.headline)
                    .lineLimit(1)

                Text(item.kind == .interval ? "Interval" : "Event")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)

                Spacer()
            }

            HStack(spacing: 18) {
                MoiraO11yMetricLabel(title: "Records", value: "\(item.records.count)")
                MoiraO11yMetricLabel(title: "Offset", value: item.offsetText)
                if let durationText = item.durationText {
                    MoiraO11yMetricLabel(title: "Duration", value: durationText)
                }
            }

            MoiraO11yMetadataLine(title: "Start", value: item.startObservedAt, monospaced: true)
            if let endObservedAt = item.endObservedAt {
                MoiraO11yMetadataLine(title: "End", value: endObservedAt, monospaced: true)
            }
        }
    }
}
