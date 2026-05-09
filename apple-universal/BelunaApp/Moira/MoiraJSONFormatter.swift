import Foundation

enum MoiraJSONFormatter {
    static func prettyString(_ value: JSONValue) -> String {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys, .withoutEscapingSlashes]

        guard let data = try? encoder.encode(value),
              let text = String(data: data, encoding: .utf8) else {
            return compactString(value)
        }

        return text
    }

    static func compactString(_ value: JSONValue, limit: Int = 600) -> String {
        let text = compactString(value)
        if text.count > limit {
            return String(text.prefix(limit)) + "..."
        }
        return text
    }

    private static func compactString(_ value: JSONValue) -> String {
        switch value {
        case let .string(value):
            value
        case let .number(value):
            String(value)
        case let .bool(value):
            value ? "true" : "false"
        case .null:
            "null"
        case let .array(values):
            "[" + values.map { compactString($0) }.joined(separator: ", ") + "]"
        case let .object(values):
            "{" + values.keys.sorted().map { key in
                "\(key): \(compactString(values[key] ?? .null))"
            }.joined(separator: ", ") + "}"
        }
    }
}
