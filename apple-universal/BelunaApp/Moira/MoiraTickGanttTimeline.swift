import Foundation

struct MoiraTickGanttTimeline {
    var records: [MoiraIndexedGanttRecord]
    var positionedRecords: [MoiraPositionedGanttRecord]
    var totalDuration: TimeInterval

    init(records: [MoiraEventRecord]) {
        let indexedRecords = records.indexedByTimeline()
        let dates = indexedRecords.compactMap(\.date)
        let start = dates.min()
        let end = dates.max()
        let duration = start.flatMap { startDate in
            end.map { max($0.timeIntervalSince(startDate), 0) }
        } ?? 0

        self.records = indexedRecords
        totalDuration = duration

        positionedRecords = indexedRecords.map { indexed in
            MoiraPositionedGanttRecord(
                record: indexed.record,
                ordinal: indexed.ordinal,
                date: indexed.date,
                position: Self.normalizedPosition(
                    ordinal: indexed.ordinal,
                    count: indexedRecords.count,
                    date: indexed.date,
                    start: start,
                    totalDuration: duration
                ),
                offsetText: Self.offsetText(
                    ordinal: indexed.ordinal,
                    date: indexed.date,
                    start: start
                )
            )
        }
    }

    private static func normalizedPosition(
        ordinal: Int,
        count: Int,
        date: Date?,
        start: Date?,
        totalDuration: TimeInterval
    ) -> Double {
        if let date, let start, totalDuration > 0 {
            return clamp(date.timeIntervalSince(start) / totalDuration)
        }
        guard count > 1 else {
            return 0.5
        }
        return clamp(Double(ordinal) / Double(count - 1))
    }

    private static func offsetText(ordinal: Int, date: Date?, start: Date?) -> String {
        guard let date, let start else {
            return "#\(ordinal + 1)"
        }
        return "+\(MoiraTickGanttSnapshot.formattedDuration(max(date.timeIntervalSince(start), 0)))"
    }

    private static func clamp(_ value: Double) -> Double {
        min(max(value, 0), 1)
    }
}

struct MoiraIndexedGanttRecord {
    var record: MoiraEventRecord
    var ordinal: Int
    var date: Date?
}

struct MoiraPositionedGanttRecord {
    var record: MoiraEventRecord
    var ordinal: Int
    var date: Date?
    var position: Double
    var offsetText: String
}

extension Array where Element == MoiraEventRecord {
    func indexedByTimeline() -> [MoiraIndexedGanttRecord] {
        enumerated()
            .map { offset, record in
                MoiraIndexedGanttRecord(
                    record: record,
                    ordinal: offset,
                    date: parseDate(record.observedAt)
                )
            }
            .sortedByTimeline()
            .enumerated()
            .map { offset, indexed in
                MoiraIndexedGanttRecord(
                    record: indexed.record,
                    ordinal: offset,
                    date: indexed.date
                )
            }
    }
}

extension Array where Element == MoiraIndexedGanttRecord {
    func sortedByTimeline() -> [MoiraIndexedGanttRecord] {
        sorted { left, right in
            switch (left.date, right.date) {
            case let (leftDate?, rightDate?):
                if leftDate == rightDate {
                    return left.ordinal < right.ordinal
                }
                return leftDate < rightDate
            case (_?, nil):
                return true
            case (nil, _?):
                return false
            case (nil, nil):
                return left.ordinal < right.ordinal
            }
        }
    }
}

extension Array where Element == MoiraPositionedGanttRecord {
    func sortedByTimeline() -> [MoiraPositionedGanttRecord] {
        sorted { left, right in
            if left.position == right.position {
                return left.ordinal < right.ordinal
            }
            return left.position < right.position
        }
    }
}

private func parseDate(_ text: String) -> Date? {
    let fractional = ISO8601DateFormatter()
    fractional.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
    if let date = fractional.date(from: text) {
        return date
    }

    let standard = ISO8601DateFormatter()
    standard.formatOptions = [.withInternetDateTime]
    return standard.date(from: text)
}
