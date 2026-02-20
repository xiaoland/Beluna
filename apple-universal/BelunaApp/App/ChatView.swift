import SwiftUI

struct ChatView: View {
    @ObservedObject var viewModel: ChatViewModel
    @Environment(\.openSettings) private var openSettings
    @Environment(\.openWindow) private var openWindow

    var body: some View {
        VStack(spacing: 0) {
            header
            if viewModel.isSleeping {
                sleepingNotice
            }
            Divider()
            messageList
            Divider()
            composer
        }
        .frame(minWidth: 420, minHeight: 560)
        .onAppear {
            viewModel.startIfNeeded()
        }
    }

    private var sleepingNotice: some View {
        HStack(spacing: 8) {
            Image(systemName: viewModel.isConnectionEnabled ? "moon.stars.fill" : "bolt.slash.fill")
                .foregroundStyle(.orange)
            Text(viewModel.sleepingTitle)
                .font(.subheadline.weight(.semibold))
            Spacer()
            Text(viewModel.sleepingHint)
                .font(.caption)
                .foregroundStyle(.secondary)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(Color.orange.opacity(0.12))
    }

    private var header: some View {
        HStack(alignment: .center, spacing: 12) {
            VStack(alignment: .leading, spacing: 2) {
                Text("Beluna")
                    .font(.title3.bold())
                Text("Apple Universal Chat Endpoint")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            Spacer()

            HStack(spacing: 8) {
                statusPill(
                    title: "Connection",
                    value: viewModel.connectionState.rawValue,
                    tint: viewModel.connectionState.tint
                )
                statusPill(
                    title: "Beluna",
                    value: viewModel.belunaState.rawValue,
                    tint: viewModel.belunaState.tint
                )
            }

            Button(viewModel.connectButtonTitle) {
                viewModel.toggleConnection()
            }
            .buttonStyle(.borderedProminent)

            Button("Retry") {
                viewModel.retryConnection()
            }
            .buttonStyle(.bordered)
            .disabled(!viewModel.canRetry)

            Button {
                openWindow(id: "observability")
            } label: {
                Image(systemName: "doc.text.magnifyingglass")
            }
            .buttonStyle(.bordered)
            .help("Open Observability")

            Button {
                openSettings()
            } label: {
                Image(systemName: "gearshape")
            }
            .buttonStyle(.bordered)
            .help("Open Settings")
        }
        .padding(12)
        .background(Color(NSColor.windowBackgroundColor))
    }

    private var messageList: some View {
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(spacing: 10) {
                    ForEach(viewModel.messages) { message in
                        MessageRow(message: message)
                            .id(message.id)
                    }
                }
                .padding(12)
            }
            .background(Color(NSColor.textBackgroundColor).opacity(0.4))
            .onChange(of: viewModel.messages.count) { _, _ in
                if let lastID = viewModel.messages.last?.id {
                    withAnimation(.easeOut(duration: 0.2)) {
                        proxy.scrollTo(lastID, anchor: .bottom)
                    }
                }
            }
        }
    }

    private var composer: some View {
        HStack(spacing: 8) {
            TextField(viewModel.isSleeping ? "Beluna is sleeping..." : "Message Beluna...", text: $viewModel.draft, axis: .vertical)
                .lineLimit(1...4)
                .textFieldStyle(.roundedBorder)
                .onSubmit {
                    viewModel.sendCurrentDraft()
                }

            Button(action: {
                viewModel.sendCurrentDraft()
            }) {
                Text("Send")
                    .fontWeight(.semibold)
            }
            .buttonStyle(.borderedProminent)
            .disabled(viewModel.draft.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || !viewModel.canSend)
        }
        .padding(12)
        .background(Color(NSColor.windowBackgroundColor))
    }

    private func statusPill(title: String, value: String, tint: Color) -> some View {
        HStack(spacing: 6) {
            Circle()
                .fill(tint)
                .frame(width: 8, height: 8)
            Text("\(title): \(value)")
                .font(.caption.weight(.semibold))
                .foregroundStyle(.secondary)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .background(.regularMaterial, in: Capsule())
    }
}

private struct MessageRow: View {
    let message: ChatMessage

    var body: some View {
        switch message.role {
        case .system, .debug:
            CenterNoticeBubble(message: message)
        case .user, .assistant:
            MessageBubble(message: message)
        }
    }
}

private struct CenterNoticeBubble: View {
    let message: ChatMessage

    var body: some View {
        HStack {
            Spacer()
            Text(message.text)
                .font(.caption)
                .foregroundStyle(textColor)
                .multilineTextAlignment(.center)
                .textSelection(.enabled)
                .padding(.horizontal, 12)
                .padding(.vertical, 8)
                .background(backgroundColor, in: Capsule())
            Spacer()
        }
    }

    private var backgroundColor: Color {
        switch message.role {
        case .system:
            return Color.orange.opacity(0.18)
        case .debug:
            return Color.gray.opacity(0.2)
        case .user, .assistant:
            return .clear
        }
    }

    private var textColor: Color {
        message.role == .debug ? .secondary : .primary
    }
}

private struct MessageBubble: View {
    let message: ChatMessage

    var body: some View {
        HStack {
            if message.role == .user {
                Spacer(minLength: 36)
            }

            VStack(alignment: .leading, spacing: 6) {
                Text(roleLabel)
                    .font(.caption2.weight(.semibold))
                    .foregroundStyle(.secondary)

                Text(message.text)
                    .font(.body)
                    .textSelection(.enabled)
                    .foregroundStyle(textColor)
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 10)
            .background(backgroundColor, in: RoundedRectangle(cornerRadius: 14, style: .continuous))

            if message.role != .user {
                Spacer(minLength: 36)
            }
        }
    }

    private var roleLabel: String {
        switch message.role {
        case .user:
            return "You"
        case .assistant:
            return "Beluna"
        case .system:
            return "System"
        case .debug:
            return "Debug"
        }
    }

    private var backgroundColor: Color {
        switch message.role {
        case .user:
            return Color.accentColor.opacity(0.88)
        case .assistant:
            return Color.gray.opacity(0.18)
        case .system:
            return Color.orange.opacity(0.18)
        case .debug:
            return Color.gray.opacity(0.2)
        }
    }

    private var textColor: Color {
        switch message.role {
        case .user:
            return .white
        case .assistant, .system:
            return .primary
        case .debug:
            return .secondary
        }
    }
}
