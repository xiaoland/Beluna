import Foundation
import Darwin

enum SpineBodyEndpointError: Error {
    case notConnected
    case connectionFailed(String)
}

private func osErrorDescription(_ code: Int32) -> String {
    String(cString: strerror(code))
}

actor SpineUnixSocketBodyEndpoint {
    typealias StateHandler = @Sendable (ConnectionState) -> Void
    typealias MessageHandler = @Sendable (ServerWireMessage) -> Void
    typealias DebugHandler = @Sendable (String) -> Void

    private var socketPath: String
    private let maxReconnectAttempts = 5
    private let initialReconnectDelayNanos: UInt64 = 500_000_000
    private let maxReconnectDelayNanos: UInt64 = 8_000_000_000

    private var socketFD: Int32?
    private var readerBuffer = Data()
    private var stopped = true
    private var runTask: Task<Void, Never>?
    private var runLoopGeneration: UInt64 = 0
    private var connectionWasEstablished = false

    var onStateChange: StateHandler?
    var onServerMessage: MessageHandler?
    var onDebug: DebugHandler?

    init(socketPath: String) {
        self.socketPath = socketPath
    }

    func updateSocketPath(_ socketPath: String) {
        self.socketPath = socketPath
    }

    func setHandlers(
        onStateChange: StateHandler?,
        onServerMessage: MessageHandler?,
        onDebug: DebugHandler?
    ) {
        self.onStateChange = onStateChange
        self.onServerMessage = onServerMessage
        self.onDebug = onDebug
    }

    func start() {
        guard runTask == nil else {
            return
        }

        stopped = false
        connectionWasEstablished = false
        runLoopGeneration &+= 1
        let generation = runLoopGeneration
        runTask = Task {
            await self.runLoop(generation: generation)
        }
    }

    func restart() {
        stop()
        start()
    }

    func stop() {
        stopped = true
        runLoopGeneration &+= 1
        runTask?.cancel()
        runTask = nil
        cleanupConnection()
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

    private func runLoop(generation: UInt64) async {
        defer {
            if runLoopGeneration == generation {
                runTask = nil
            }
        }

        var retryAttempt = 0

        while !stopped && !Task.isCancelled && runLoopGeneration == generation {
            do {
                onStateChange?(.connecting)
                try await connectAndReadLoop()
            } catch {
                onStateChange?(.disconnected)
                cleanupConnection()
                if stopped || Task.isCancelled {
                    break
                }

                if connectionWasEstablished {
                    retryAttempt = 0
                    connectionWasEstablished = false
                }

                retryAttempt += 1
                guard retryAttempt <= maxReconnectAttempts else {
                    onDebug?("Reconnect stopped after \(maxReconnectAttempts) retries. You can retry manually.")
                    break
                }

                let delayNanos = reconnectDelayNanos(forAttempt: retryAttempt)
                let delaySeconds = Double(delayNanos) / 1_000_000_000
                onDebug?(
                    "Reconnect \(retryAttempt)/\(maxReconnectAttempts) in \(String(format: "%.1f", delaySeconds))s (\(error.localizedDescription))"
                )
                try? await Task.sleep(nanoseconds: delayNanos)
            }
        }

        onStateChange?(.disconnected)
        cleanupConnection()
    }

    private func connectAndReadLoop() async throws {
        let fd = try openUnixSocket(path: socketPath)
        socketFD = fd
        connectionWasEstablished = true
        onStateChange?(.connected)
        try await sendRegister()

        for try await chunk in makeReadStream(for: fd) {
            if stopped || Task.isCancelled {
                return
            }
            if chunk.isEmpty {
                continue
            }
            parseIncomingData(chunk)
        }

        throw SpineBodyEndpointError.connectionFailed("connection closed")
    }

    private func openUnixSocket(path: String) throws -> Int32 {
        let fd = Darwin.socket(AF_UNIX, SOCK_STREAM, 0)
        guard fd >= 0 else {
            throw SpineBodyEndpointError.connectionFailed(
                "socket failed: \(osErrorDescription(errno)) (\(errno))"
            )
        }

        do {
            var address = sockaddr_un()
            #if os(macOS)
            address.sun_len = UInt8(MemoryLayout<sockaddr_un>.size)
            #endif
            address.sun_family = sa_family_t(AF_UNIX)

            let pathBytes = Array(path.utf8)
            let maxPathLength = MemoryLayout.size(ofValue: address.sun_path)
            guard pathBytes.count < maxPathLength else {
                throw SpineBodyEndpointError.connectionFailed(
                    "socket path too long for AF_UNIX: \(path)"
                )
            }

            withUnsafeMutableBytes(of: &address.sun_path) { rawBytes in
                rawBytes.initializeMemory(as: UInt8.self, repeating: 0)
                rawBytes.copyBytes(from: pathBytes)
            }

            let connectResult = withUnsafePointer(to: &address) {
                $0.withMemoryRebound(to: sockaddr.self, capacity: 1) { sockaddrPtr in
                    Darwin.connect(fd, sockaddrPtr, socklen_t(MemoryLayout<sockaddr_un>.size))
                }
            }

            if connectResult != 0 {
                let code = errno
                throw SpineBodyEndpointError.connectionFailed(
                    "connect failed: \(osErrorDescription(code)) (\(code))"
                )
            }

            return fd
        } catch {
            Darwin.close(fd)
            throw error
        }
    }

    private func makeReadStream(for fd: Int32) -> AsyncThrowingStream<Data, Error> {
        AsyncThrowingStream { continuation in
            let readerTask = Task.detached(priority: .background) {
                var buffer = [UInt8](repeating: 0, count: 4096)

                while !Task.isCancelled {
                    let readCount: Int = buffer.withUnsafeMutableBytes { rawBuffer in
                        guard let baseAddress = rawBuffer.baseAddress else {
                            return -1
                        }
                        return Darwin.read(fd, baseAddress, rawBuffer.count)
                    }

                    if readCount > 0 {
                        continuation.yield(Data(buffer[0..<readCount]))
                        continue
                    }

                    if readCount == 0 {
                        continuation.finish()
                        return
                    }

                    let code = errno
                    if code == EINTR {
                        continue
                    }
                    if code == EAGAIN || code == EWOULDBLOCK {
                        usleep(10_000)
                        continue
                    }

                    continuation.finish(
                        throwing: SpineBodyEndpointError.connectionFailed(
                            "read failed: \(osErrorDescription(code)) (\(code))"
                        )
                    )
                    return
                }

                continuation.finish()
            }

            continuation.onTermination = { _ in
                readerTask.cancel()
            }
        }
    }

    private func writeAll(fd: Int32, data: Data) throws {
        try data.withUnsafeBytes { rawBuffer in
            guard let baseAddress = rawBuffer.baseAddress else {
                return
            }

            var offset = 0
            while offset < rawBuffer.count {
                let writeCount = Darwin.write(
                    fd,
                    baseAddress.advanced(by: offset),
                    rawBuffer.count - offset
                )

                if writeCount > 0 {
                    offset += writeCount
                    continue
                }

                let code = errno
                if code == EINTR {
                    continue
                }

                throw SpineBodyEndpointError.connectionFailed(
                    "write failed: \(osErrorDescription(code)) (\(code))"
                )
            }
        }
    }

    private func parseIncomingData(_ chunk: Data) {
        readerBuffer.append(chunk)

        while let newlineIndex = readerBuffer.firstIndex(of: 0x0A) {
            let line = readerBuffer.prefix(upTo: newlineIndex)
            readerBuffer.removeSubrange(...newlineIndex)

            if line.isEmpty {
                continue
            }

            do {
                let message = try decodeServerMessage(from: Data(line))
                switch message {
                case .act:
                    onServerMessage?(message)
                case let .ignored(type):
                    onDebug?("Ignored inbound \(type) message.")
                }
            } catch {
                onDebug?("Ignored malformed inbound message: \(error.localizedDescription)")
            }
        }
    }

    private func sendLine<T: Encodable>(_ envelope: T) async throws {
        guard let socketFD else {
            throw SpineBodyEndpointError.notConnected
        }

        let data = try encodeLine(envelope)
        try writeAll(fd: socketFD, data: data)
    }

    private func cleanupConnection() {
        if let fd = socketFD {
            Darwin.close(fd)
            socketFD = nil
        }
        readerBuffer.removeAll(keepingCapacity: false)
    }

    private func reconnectDelayNanos(forAttempt attempt: Int) -> UInt64 {
        let cappedAttempt = max(1, attempt)
        let exponent = min(cappedAttempt - 1, 16)
        let factor = UInt64(1) << UInt64(exponent)
        let delay = initialReconnectDelayNanos.saturatingMultiply(by: factor)
        return min(delay, maxReconnectDelayNanos)
    }
}

private extension UInt64 {
    func saturatingMultiply(by value: UInt64) -> UInt64 {
        let (result, overflow) = multipliedReportingOverflow(by: value)
        return overflow ? UInt64.max : result
    }
}
