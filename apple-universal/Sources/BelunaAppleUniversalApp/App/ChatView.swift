import SwiftUI

struct ChatView: View {
    @ObservedObject var viewModel: ChatViewModel

    var body: some View {
        VStack(spacing: 0) {
            header
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

    private var header: some View {
        HStack {
            VStack(alignment: .leading, spacing: 2) {
                Text("Beluna")
                    .font(.title3.bold())
                Text("Apple Universal Chat Endpoint")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            Spacer()

            HStack(spacing: 8) {
                Circle()
                    .fill(viewModel.connectionState.tint)
                    .frame(width: 10, height: 10)
                Text(viewModel.connectionState.rawValue)
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 6)
            .background(.regularMaterial, in: Capsule())
        }
        .padding(12)
        .background(Color(NSColor.windowBackgroundColor))
    }

    private var messageList: some View {
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(spacing: 10) {
                    ForEach(viewModel.messages) { message in
                        MessageBubble(message: message)
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
            TextField("Message Beluna...", text: $viewModel.draft, axis: .vertical)
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
            .disabled(viewModel.draft.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
        }
        .padding(12)
        .background(Color(NSColor.windowBackgroundColor))
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
            return Color.orange.opacity(0.18)
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
