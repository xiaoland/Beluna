import SwiftUI

struct MoiraO11yEmptyPane: View {
    var title: String
    var systemImage: String
    var detail: String

    var body: some View {
        VStack(spacing: 10) {
            Image(systemName: systemImage)
                .font(.title2)
                .foregroundStyle(.secondary)
            Text(title)
                .font(.headline)
            Text(detail)
                .font(.caption)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
                .frame(maxWidth: 320)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .padding(24)
    }
}

struct MoiraO11yMetadataLine: View {
    var title: String
    var value: String
    var monospaced = false

    var body: some View {
        LabeledContent(title) {
            Text(value)
                .font(monospaced ? .caption.monospaced() : .caption)
                .lineLimit(3)
                .textSelection(.enabled)
        }
    }
}

struct MoiraO11yMetricLabel: View {
    var title: String
    var value: String

    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(title)
                .font(.caption2)
                .foregroundStyle(.secondary)
            Text(value)
                .font(.caption.monospaced().weight(.semibold))
                .textSelection(.enabled)
        }
        .frame(minWidth: 72, alignment: .leading)
    }
}

struct MoiraJSONBlock: View {
    var title: String
    var value: JSONValue

    var body: some View {
        DisclosureGroup(title) {
            ScrollView(.horizontal) {
                Text(MoiraJSONFormatter.prettyString(value))
                    .font(.caption.monospaced())
                    .textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(.top, 4)
            }
        }
    }
}
