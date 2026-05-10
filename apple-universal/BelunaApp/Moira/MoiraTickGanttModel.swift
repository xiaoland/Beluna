import Foundation

struct MoiraTickGanttSnapshot: Equatable {
    var lanes: [MoiraTickGanttLane]
    var startLabel: String
    var endLabel: String
    var durationText: String

    var eventCount: Int {
        lanes.reduce(0) { $0 + $1.items.reduce(0) { $0 + $1.records.count } }
    }

    var itemCount: Int {
        lanes.reduce(0) { $0 + $1.items.count }
    }

    static func make(records: [MoiraEventRecord]) -> Self {
        let timeline = MoiraTickGanttTimeline(records: records)
        guard !timeline.records.isEmpty else {
            return Self(lanes: [], startLabel: "", endLabel: "", durationText: "0 ms")
        }

        let lanes = Dictionary(grouping: timeline.positionedRecords, by: { laneKey(for: $0.record) })
            .map { key, records in
                makeLane(key: key, records: records)
            }
            .sorted { left, right in
                let leftPosition = left.items.first?.startPosition ?? 0
                let rightPosition = right.items.first?.startPosition ?? 0
                if leftPosition == rightPosition {
                    return left.title < right.title
                }
                return leftPosition < rightPosition
            }

        return Self(
            lanes: lanes,
            startLabel: timeline.records.first?.record.observedAt ?? "",
            endLabel: timeline.records.last?.record.observedAt ?? "",
            durationText: formattedDuration(timeline.totalDuration)
        )
    }

    func selectedItem(containing rawEventID: String?) -> MoiraTickGanttItem? {
        if let rawEventID,
           let item = lanes.lazy.flatMap(\.items).first(where: { item in
               item.rawEventIDs.contains(rawEventID)
           }) {
            return item
        }

        return lanes.lazy.flatMap(\.items).first
    }

    private static func makeLane(
        key: MoiraTickGanttLaneKey,
        records: [MoiraPositionedGanttRecord]
    ) -> MoiraTickGanttLane {
        MoiraTickGanttLane(
            id: key.id,
            title: key.title,
            subtitle: key.subtitle,
            items: pairItems(records: records)
        )
    }

    private static func pairItems(records: [MoiraPositionedGanttRecord]) -> [MoiraTickGanttItem] {
        let sorted = records.sortedByTimeline()
        var pending: [String: [MoiraPositionedGanttRecord]] = [:]
        var items: [MoiraTickGanttItem] = []

        for record in sorted {
            switch phase(for: record.record.eventName) {
            case let .start(key):
                pending[key, default: []].append(record)
            case let .end(key):
                if let start = pending[key]?.first {
                    pending[key]?.removeFirst()
                    if pending[key]?.isEmpty == true {
                        pending[key] = nil
                    }
                    items.append(intervalItem(start: start, end: record, key: key))
                } else {
                    items.append(pointItem(record))
                }
            case .instant:
                items.append(pointItem(record))
            }
        }

        for leftover in pending.values.flatMap({ $0 }) {
            items.append(pointItem(leftover))
        }

        return items.sorted { left, right in
            if left.startPosition == right.startPosition {
                return left.id < right.id
            }
            return left.startPosition < right.startPosition
        }
    }

    private static func intervalItem(
        start: MoiraPositionedGanttRecord,
        end: MoiraPositionedGanttRecord,
        key: String
    ) -> MoiraTickGanttItem {
        let intervalDuration = durationBetween(start: start, end: end)
        return MoiraTickGanttItem(
            id: "\(start.record.rawEventID)+\(end.record.rawEventID)",
            kind: .interval,
            title: intervalTitle(start: start.record, end: end.record, key: key),
            severityText: strongestSeverity([start.record.severityText, end.record.severityText]),
            startObservedAt: start.record.observedAt,
            endObservedAt: end.record.observedAt,
            startPosition: min(start.position, end.position),
            endPosition: max(start.position, end.position),
            offsetText: start.offsetText,
            durationText: intervalDuration.map(formattedDuration),
            records: [start.record, end.record]
        )
    }

    private static func pointItem(_ record: MoiraPositionedGanttRecord) -> MoiraTickGanttItem {
        MoiraTickGanttItem(
            id: record.record.rawEventID,
            kind: .point,
            title: record.record.displayTitle,
            severityText: record.record.severityText,
            startObservedAt: record.record.observedAt,
            endObservedAt: nil,
            startPosition: record.position,
            endPosition: record.position,
            offsetText: record.offsetText,
            durationText: nil,
            records: [record.record]
        )
    }

    private static func laneKey(for record: MoiraEventRecord) -> MoiraTickGanttLaneKey {
        let title = record.subsystem
            ?? record.family
            ?? record.scopeName
            ?? record.recordKind
        let subtitle = record.scopeName ?? record.recordKind
        return MoiraTickGanttLaneKey(title: title, subtitle: subtitle)
    }

    private static func phase(for eventName: String?) -> MoiraTickGanttPhase {
        guard let eventName else {
            return .instant
        }

        let lowercased = eventName.lowercased()
        for suffix in [".started", ".dispatched"] where lowercased.hasSuffix(suffix) {
            return .start(String(lowercased.dropLast(suffix.count)))
        }
        for suffix in [".finished", ".committed", ".commited"] where lowercased.hasSuffix(suffix) {
            return .end(String(lowercased.dropLast(suffix.count)))
        }

        switch lowercased {
        case "started", "dispatched":
            return .start("operation")
        case "finished", "committed", "commited":
            return .end("operation")
        default:
            return .instant
        }
    }

    private static func intervalTitle(
        start: MoiraEventRecord,
        end: MoiraEventRecord,
        key: String
    ) -> String {
        if key == "operation" {
            return "\(start.displayTitle) -> \(end.displayTitle)"
        }
        return key
    }

    private static func strongestSeverity(_ severities: [String]) -> String {
        if let error = severities.first(where: { $0.uppercased().contains("ERROR") }) {
            return error
        }
        if let warning = severities.first(where: { $0.uppercased().contains("WARN") }) {
            return warning
        }
        return severities.first ?? "INFO"
    }

    private static func durationBetween(
        start: MoiraPositionedGanttRecord,
        end: MoiraPositionedGanttRecord
    ) -> TimeInterval? {
        guard let startDate = start.date, let endDate = end.date else {
            return nil
        }
        return max(endDate.timeIntervalSince(startDate), 0)
    }

    static func formattedDuration(_ duration: TimeInterval) -> String {
        if duration < 1 {
            return "\(Int((duration * 1_000).rounded())) ms"
        }
        if duration < 60 {
            return String(format: "%.2f s", duration)
        }
        return String(format: "%.1f min", duration / 60)
    }
}

struct MoiraTickGanttLane: Equatable, Identifiable {
    var id: String
    var title: String
    var subtitle: String
    var items: [MoiraTickGanttItem]
}

struct MoiraTickGanttItem: Equatable, Identifiable {
    enum Kind: Equatable {
        case point
        case interval
    }

    var id: String
    var kind: Kind
    var title: String
    var severityText: String
    var startObservedAt: String
    var endObservedAt: String?
    var startPosition: Double
    var endPosition: Double
    var offsetText: String
    var durationText: String?
    var records: [MoiraEventRecord]

    var rawEventIDs: [String] {
        records.map(\.rawEventID)
    }
}

private enum MoiraTickGanttPhase {
    case start(String)
    case end(String)
    case instant
}

private struct MoiraTickGanttLaneKey: Hashable {
    var title: String
    var subtitle: String

    var id: String {
        "\(title)|\(subtitle)"
    }
}
