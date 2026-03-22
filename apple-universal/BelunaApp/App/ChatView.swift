import SwiftUI

struct ChatView: View {
    @ObservedObject var viewModel: ChatViewModel
    @Environment(\.openSettings) private var openSettings

    var body: some View {
        VStack(spacing: 0) {
            header
            if viewModel.isHibernating {
                hibernateNotice
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

    private var hibernateNotice: some View {
        HStack(spacing: 8) {
            Image(systemName: viewModel.isConnectionEnabled ? "moon.zzz.fill" : "bolt.slash.fill")
                .foregroundStyle(.orange)
            Text(viewModel.hibernateTitle)
                .font(.subheadline.weight(.semibold))
            Spacer()
            Text(viewModel.hibernateHint)
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
                Text("Apple Universal Body Endpoint")
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

            VStack(alignment: .leading, spacing: 2) {
                Button(viewModel.retryButtonTitle) {
                    viewModel.retryConnection()
                }
                .buttonStyle(.bordered)
                .disabled(!viewModel.canRetry)

                if let retryStatusText = viewModel.retryStatusText {
                    Text(retryStatusText)
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                }
            }

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
                    if viewModel.hasOlderBufferedMessages {
                        paginationHint("Scroll up to load older messages")
                    }

                    ForEach(viewModel.messages) { message in
                        MessageRow(message: message)
                            .id(message.id)
                            .onAppear {
                                viewModel.handleVisibleMessageAppeared(message.id)
                            }
                    }

                    if viewModel.hasNewerBufferedMessages {
                        paginationHint("Scroll down to load newer messages")
                    }
                }
                .padding(12)
            }
            .background(Color(NSColor.textBackgroundColor).opacity(0.4))
            .onChange(of: viewModel.latestMessageIDForAutoScroll) { _, latestID in
                guard let latestID else {
                    return
                }
                withAnimation(.easeOut(duration: 0.2)) {
                    proxy.scrollTo(latestID, anchor: .bottom)
                }
            }
        }
    }

    private var composer: some View {
        HStack(spacing: 8) {
            TextField(viewModel.isHibernating ? "Beluna is in Hibernate..." : "Message Beluna...", text: $viewModel.draft, axis: .vertical)
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

    private func paginationHint(_ text: String) -> some View {
        HStack {
            Spacer()
            Text(text)
                .font(.caption2)
                .foregroundStyle(.secondary)
                .padding(.horizontal, 10)
                .padding(.vertical, 6)
                .background(Color.primary.opacity(0.06), in: Capsule())
            Spacer()
        }
    }
}

private struct MessageRow: View {
    let message: ChatMessage

    var body: some View {
        switch message.role {
        case .system:
            CenterNoticeText(message: message)
        case .user, .assistant:
            MessageBubble(message: message)
        }
    }
}

private struct CenterNoticeText: View {
    let message: ChatMessage

    var body: some View {
        HStack {
            Spacer()
            Text(message.text)
                .font(.caption)
                .foregroundStyle(.secondary)
                .multilineTextAlignment(.center)
                .textSelection(.enabled)
                .padding(.vertical, 2)
            Spacer()
        }
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
        }
    }

    private var backgroundColor: Color {
        switch message.role {
        case .user:
            return Color.accentColor.opacity(0.88)
        case .assistant:
            return Color.gray.opacity(0.18)
        case .system:
            return .clear
        }
    }

    private var textColor: Color {
        switch message.role {
        case .user:
            return .white
        case .assistant, .system:
            return .primary
        }
    }
}
