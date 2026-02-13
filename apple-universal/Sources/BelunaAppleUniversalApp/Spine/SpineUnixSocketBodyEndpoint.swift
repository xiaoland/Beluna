import Foundation
import Network

enum SpineBodyEndpointError: Error {
    case notConnected
    case connectionFailed(String)
}

private final class ResumeOnce {
    private let lock = NSLock()
    private var resumed = false

    func run(_ body: () -> Void) {
        lock.lock()
        defer { lock.unlock() }

        guard !resumed else {
            return
        }

        resumed = true
        body()
    }
}

actor SpineUnixSocketBodyEndpoint {
    typealias StateHandler = @Sendable (ConnectionState) -> Void
    typealias MessageHandler = @Sendable (ServerWireMessage) -> Void

    private let socketPath: String
    private let readQueue = DispatchQueue(label: "beluna.apple-universal.spine")
    private let reconnectDelayNanos: UInt64 = 800_000_000

    private var connection: NWConnection?
    private var readerBuffer = Data()
    private var stopped = true
    private var runTask: Task<Void, Never>?

    var onStateChange: StateHandler?
    var onServerMessage: MessageHandler?

    init(socketPath: String) {
        self.socketPath = socketPath
    }

    func setHandlers(onStateChange: StateHandler?, onServerMessage: MessageHandler?) {
        self.onStateChange = onStateChange
        self.onServerMessage = onServerMessage
    }

    func start() {
        guard runTask == nil else {
            return
        }

        stopped = false
        runTask = Task {
            await self.runLoop()
        }
    }

    func stop() {
        stopped = true
        runTask?.cancel()
        runTask = nil
        connection?.cancel()
        connection = nil
    }

    func sendRegister() async throws {
        let envelope = makeAppleEndpointRegisterEnvelope()
        try await sendLine(envelope)
    }

    func sendUserSense(conversationID: String, text: String) async throws {
        let envelope = makeUserSenseEnvelope(conversationID: conversationID, text: text)
        try await sendLine(envelope)
    }

    func sendActResultSense(
        action: AdmittedActionWire,
        status: String,
        referenceID: String,
        reasonCode: String? = nil
    ) async throws {
        let envelope = makeActResultSenseEnvelope(
            action: action,
            status: status,
            referenceID: referenceID,
            reasonCode: reasonCode
        )
        try await sendLine(envelope)
    }

    private func runLoop() async {
        while !stopped && !Task.isCancelled {
            do {
                onStateChange?(.connecting)
                try await connectAndReadLoop()
            } catch {
                onStateChange?(.disconnected)
                cleanupConnection()
                if stopped || Task.isCancelled {
                    break
                }
                try? await Task.sleep(nanoseconds: reconnectDelayNanos)
            }
        }

        onStateChange?(.disconnected)
        cleanupConnection()
    }

    private func connectAndReadLoop() async throws {
        let connection = NWConnection(to: .unix(path: socketPath), using: .tcp)
        self.connection = connection

        try await waitUntilConnectionReady(connection)
        onStateChange?(.connected)
        try await sendRegister()

        while !stopped && !Task.isCancelled {
            let chunk = try await receiveChunk(connection)
            guard let chunk else {
                throw SpineBodyEndpointError.connectionFailed("connection closed")
            }

            if chunk.isEmpty {
                continue
            }

            try parseIncomingData(chunk)
        }
    }

    private func waitUntilConnectionReady(_ connection: NWConnection) async throws {
        try await withCheckedThrowingContinuation { continuation in
            let resumeOnce = ResumeOnce()

            connection.stateUpdateHandler = { state in
                switch state {
                case .ready:
                    resumeOnce.run {
                        continuation.resume(returning: ())
                    }
                case .failed(let error):
                    resumeOnce.run {
                        continuation.resume(
                            throwing: SpineBodyEndpointError.connectionFailed(error.localizedDescription)
                        )
                    }
                case .cancelled:
                    resumeOnce.run {
                        continuation.resume(
                            throwing: SpineBodyEndpointError.connectionFailed("cancelled")
                        )
                    }
                default:
                    break
                }
            }

            connection.start(queue: readQueue)
        }
    }

    private func receiveChunk(_ connection: NWConnection) async throws -> Data? {
        try await withCheckedThrowingContinuation { continuation in
            connection.receive(minimumIncompleteLength: 1, maximumLength: 4096) { data, _, isComplete, error in
                if let error {
                    continuation.resume(throwing: SpineBodyEndpointError.connectionFailed(error.localizedDescription))
                    return
                }

                if isComplete, (data?.isEmpty ?? true) {
                    continuation.resume(returning: nil)
                    return
                }

                continuation.resume(returning: data ?? Data())
            }
        }
    }

    private func parseIncomingData(_ chunk: Data) throws {
        readerBuffer.append(chunk)

        while let newlineIndex = readerBuffer.firstIndex(of: 0x0A) {
            let line = readerBuffer.prefix(upTo: newlineIndex)
            readerBuffer.removeSubrange(...newlineIndex)

            if line.isEmpty {
                continue
            }

            let message = try decodeServerMessage(from: Data(line))
            onServerMessage?(message)
        }
    }

    private func sendLine<T: Encodable>(_ envelope: T) async throws {
        guard let connection else {
            throw SpineBodyEndpointError.notConnected
        }

        let data = try encodeLine(envelope)
        try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
            connection.send(content: data, completion: .contentProcessed { error in
                if let error {
                    continuation.resume(throwing: SpineBodyEndpointError.connectionFailed(error.localizedDescription))
                } else {
                    continuation.resume(returning: ())
                }
            })
        }
    }

    private func cleanupConnection() {
        connection?.cancel()
        connection = nil
        readerBuffer.removeAll(keepingCapacity: false)
    }
}
