import Foundation

struct MessageBufferWindow {
    let messages: [ChatMessage]
    let bufferedMessageCount: Int
    let visibleMessageCount: Int
    let hasOlderBufferedMessages: Bool
    let hasNewerBufferedMessages: Bool
    let autoScrollMessageID: UUID?
}

struct MessageBuffer {
    private let pageSize: Int
    private var storage: [ChatMessage] = []
    private var visibleRange: Range<Int> = 0..<0

    init(pageSize: Int) {
        self.pageSize = pageSize
    }

    var persistedSenseActMessages: [ChatMessage] {
        storage.filter { $0.signalOrigin == .sense || $0.signalOrigin == .act }
    }

    func lastMessageMatches(role: ChatRole, text: String) -> Bool {
        guard let last = storage.last else {
            return false
        }
        return last.role == role && last.text == text
    }

    mutating func restore(_ messages: [ChatMessage]) -> MessageBufferWindow {
        storage = messages

        guard !storage.isEmpty else {
            visibleRange = 0..<0
            return window(autoScrollToLatest: false)
        }

        let end = storage.count
        let start = max(0, end - pageSize)
        visibleRange = start..<end
        return window(autoScrollToLatest: false)
    }

    mutating func clear() -> MessageBufferWindow {
        storage.removeAll(keepingCapacity: false)
        visibleRange = 0..<0
        return window(autoScrollToLatest: false)
    }

    mutating func append(
        _ message: ChatMessage,
        capacity: Int,
        preferredAutoScroll: Bool? = nil
    ) -> MessageBufferWindow {
        let shouldAutoScroll = preferredAutoScroll ?? isShowingLatestWindow
        let previousVisibleCount = visibleRange.count

        storage.append(message)
        trimToCapacity(capacity, preferLatestWindow: shouldAutoScroll)

        guard !storage.isEmpty else {
            visibleRange = 0..<0
            return window(autoScrollToLatest: false)
        }

        if shouldAutoScroll || visibleRange.isEmpty {
            let desiredVisibleCount = max(previousVisibleCount, pageSize)
            let end = storage.count
            let start = max(0, end - desiredVisibleCount)
            visibleRange = start..<end
            return window(autoScrollToLatest: true)
        }

        return window(autoScrollToLatest: false)
    }

    mutating func setCapacity(_ capacity: Int) -> MessageBufferWindow {
        trimToCapacity(capacity, preferLatestWindow: true)
        return window(autoScrollToLatest: true)
    }

    mutating func loadOlderPageIfNeeded() -> MessageBufferWindow {
        guard visibleRange.lowerBound > 0 else {
            return window(autoScrollToLatest: false)
        }

        let newLowerBound = max(0, visibleRange.lowerBound - pageSize)
        visibleRange = newLowerBound..<visibleRange.upperBound
        return window(autoScrollToLatest: false)
    }

    mutating func loadNewerPageIfNeeded() -> MessageBufferWindow {
        guard visibleRange.upperBound < storage.count else {
            return window(autoScrollToLatest: false)
        }

        let newUpperBound = min(storage.count, visibleRange.upperBound + pageSize)
        visibleRange = visibleRange.lowerBound..<newUpperBound
        return window(autoScrollToLatest: false)
    }

    private var isShowingLatestWindow: Bool {
        visibleRange.upperBound == storage.count
    }

    private mutating func trimToCapacity(_ capacity: Int, preferLatestWindow: Bool) {
        guard capacity > 0, storage.count > capacity else {
            return
        }

        let overflow = storage.count - capacity
        storage.removeFirst(overflow)

        let shiftedLowerBound = max(0, visibleRange.lowerBound - overflow)
        let shiftedUpperBound = max(shiftedLowerBound, visibleRange.upperBound - overflow)
        visibleRange = shiftedLowerBound..<min(shiftedUpperBound, storage.count)

        guard !storage.isEmpty else {
            visibleRange = 0..<0
            return
        }

        if preferLatestWindow {
            let desiredVisibleCount = max(visibleRange.count, pageSize)
            let end = storage.count
            let start = max(0, end - desiredVisibleCount)
            visibleRange = start..<end
            return
        }

        if visibleRange.isEmpty {
            let end = min(storage.count, pageSize)
            let start = max(0, end - pageSize)
            visibleRange = start..<end
        }
    }

    private mutating func window(autoScrollToLatest: Bool) -> MessageBufferWindow {
        guard !storage.isEmpty else {
            visibleRange = 0..<0
            return MessageBufferWindow(
                messages: [],
                bufferedMessageCount: 0,
                visibleMessageCount: 0,
                hasOlderBufferedMessages: false,
                hasNewerBufferedMessages: false,
                autoScrollMessageID: nil
            )
        }

        let lowerBound = min(max(0, visibleRange.lowerBound), storage.count)
        let upperBound = min(max(lowerBound, visibleRange.upperBound), storage.count)
        visibleRange = lowerBound..<upperBound

        let visibleMessages = Array(storage[visibleRange])
        return MessageBufferWindow(
            messages: visibleMessages,
            bufferedMessageCount: storage.count,
            visibleMessageCount: visibleMessages.count,
            hasOlderBufferedMessages: visibleRange.lowerBound > 0,
            hasNewerBufferedMessages: visibleRange.upperBound < storage.count,
            autoScrollMessageID: autoScrollToLatest ? visibleMessages.last?.id : nil
        )
    }
}
