import SwiftUI

struct MoiraTickGanttView: View {
    var snapshot: MoiraTickGanttSnapshot
    @Binding var selectedRawEventID: String?

    var body: some View {
        if snapshot.lanes.isEmpty {
            MoiraO11yEmptyPane(
                title: "Gantt Empty",
                systemImage: "chart.bar.xaxis",
                detail: "The selected tick has no source records for timeline projection."
            )
        } else {
            VStack(alignment: .leading, spacing: 0) {
                summaryBar
                Divider()
                laneScroll
                Divider()
                MoiraTickGanttDetailPane(item: selectedItem)
                    .frame(minHeight: 160, idealHeight: 190, maxHeight: 230)
            }
        }
    }

    private var selectedItem: MoiraTickGanttItem? {
        snapshot.selectedItem(containing: selectedRawEventID)
    }

    private var summaryBar: some View {
        HStack(spacing: 18) {
            MoiraO11yMetricLabel(title: "Lanes", value: "\(snapshot.lanes.count)")
            MoiraO11yMetricLabel(title: "Items", value: "\(snapshot.itemCount)")
            MoiraO11yMetricLabel(title: "Events", value: "\(snapshot.eventCount)")
            MoiraO11yMetricLabel(title: "Duration", value: snapshot.durationText)

            Spacer()

            VStack(alignment: .trailing, spacing: 2) {
                Text(snapshot.startLabel)
                    .font(.caption.monospaced())
                    .lineLimit(1)
                Text(snapshot.endLabel)
                    .font(.caption.monospaced())
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
            .textSelection(.enabled)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 10)
    }

    private var laneScroll: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 14) {
                ForEach(snapshot.lanes) { lane in
                    MoiraTickGanttLaneRow(
                        lane: lane,
                        selectedItemID: selectedItem?.id
                    ) { item in
                        selectedRawEventID = item.rawEventIDs.first
                    }
                }
            }
            .padding(16)
        }
    }
}

private struct MoiraTickGanttLaneRow: View {
    var lane: MoiraTickGanttLane
    var selectedItemID: String?
    var selectItem: (MoiraTickGanttItem) -> Void

    var body: some View {
        HStack(alignment: .top, spacing: 14) {
            laneLabel
                .frame(width: 160, alignment: .leading)

            timeline
                .frame(height: timelineHeight)
        }
    }

    private var laneLabel: some View {
        VStack(alignment: .leading, spacing: 3) {
            Text(lane.title)
                .font(.caption.weight(.semibold))
                .lineLimit(1)

            Text(lane.subtitle)
                .font(.caption2)
                .foregroundStyle(.secondary)
                .lineLimit(1)
        }
        .textSelection(.enabled)
    }

    private var timeline: some View {
        GeometryReader { geometry in
            ZStack(alignment: .leading) {
                ForEach(Array(lane.items.enumerated()), id: \.element.id) { index, item in
                    let y = itemY(index)

                    Capsule()
                        .fill(.quaternary)
                        .frame(height: 4)
                        .position(x: geometry.size.width / 2, y: y)

                    itemView(item, width: geometry.size.width, y: y)
                }
            }
        }
        .frame(height: timelineHeight)
    }

    private var timelineHeight: CGFloat {
        max(24, CGFloat(lane.items.count) * 24)
    }

    @ViewBuilder
    private func itemView(_ item: MoiraTickGanttItem, width: CGFloat, y: CGFloat) -> some View {
        switch item.kind {
        case .interval:
            MoiraTickGanttIntervalBlock(
                item: item,
                selected: selectedItemID == item.id
            ) {
                selectItem(item)
            }
            .frame(width: intervalWidth(for: item, width: width))
            .position(
                x: intervalMidpoint(for: item, width: width),
                y: y
            )
        case .point:
            MoiraTickGanttMarker(
                item: item,
                selected: selectedItemID == item.id
            ) {
                selectItem(item)
            }
            .position(
                x: positionX(item.startPosition, width: width),
                y: y
            )
        }
    }

    private func itemY(_ index: Int) -> CGFloat {
        12 + CGFloat(index) * 24
    }

    private func intervalMidpoint(for item: MoiraTickGanttItem, width: CGFloat) -> CGFloat {
        let start = positionX(item.startPosition, width: width)
        let end = positionX(item.endPosition, width: width)
        return (start + end) / 2
    }

    private func intervalWidth(for item: MoiraTickGanttItem, width: CGFloat) -> CGFloat {
        let start = positionX(item.startPosition, width: width)
        let end = positionX(item.endPosition, width: width)
        return max(abs(end - start), 18)
    }

    private func positionX(_ position: Double, width: CGFloat) -> CGFloat {
        let inset: CGFloat = 9
        let span = max(width - inset * 2, 1)
        return inset + span * CGFloat(position)
    }
}

private struct MoiraTickGanttIntervalBlock: View {
    var item: MoiraTickGanttItem
    var selected: Bool
    var action: () -> Void

    var body: some View {
        Button(action: action) {
            RoundedRectangle(cornerRadius: 4)
                .fill(markerColor)
                .overlay {
                    RoundedRectangle(cornerRadius: 4)
                        .strokeBorder(selected ? Color.primary : Color.clear, lineWidth: 2)
                }
                .frame(height: 16)
        }
        .buttonStyle(.plain)
        .help(helpText)
        .accessibilityLabel("\(item.title), \(item.durationText ?? item.offsetText)")
    }

    private var helpText: String {
        let end = item.endObservedAt.map { " -> \($0)" } ?? ""
        return "\(item.title)  \(item.startObservedAt)\(end)"
    }

    private var markerColor: Color {
        MoiraTickGanttColor.color(for: item.severityText)
    }
}

private struct MoiraTickGanttMarker: View {
    var item: MoiraTickGanttItem
    var selected: Bool
    var action: () -> Void

    var body: some View {
        Button(action: action) {
            Circle()
                .fill(MoiraTickGanttColor.color(for: item.severityText))
                .overlay {
                    if selected {
                        Circle()
                            .strokeBorder(.primary, lineWidth: 2)
                    }
                }
                .frame(width: 14, height: 14)
        }
        .buttonStyle(.plain)
        .help("\(item.title)  \(item.startObservedAt)")
        .accessibilityLabel("\(item.title), \(item.offsetText)")
    }
}

private enum MoiraTickGanttColor {
    static func color(for severityText: String) -> Color {
        let severity = severityText.uppercased()
        if severity.contains("ERROR") {
            return .red
        }
        if severity.contains("WARN") {
            return .orange
        }
        return .accentColor
    }
}
