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

    private var socketPath: String
    private let reconnectDelayNanos: UInt64 = 800_000_000
    private let missingSocketRetryDelayNanos: UInt64 = 1_500_000_000

    private var socketFD: Int32?
    private var readerBuffer = Data()
    private var stopped = true
    private var runTask: Task<Void, Never>?

    var onStateChange: StateHandler?
    var onServerMessage: MessageHandler?

    init(socketPath: String) {
        self.socketPath = socketPath
    }

    func updateSocketPath(_ socketPath: String) {
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

    private func runLoop() async {
        while !stopped && !Task.isCancelled {
            if !socketFileExists() {
                onStateChange?(.disconnected)
                cleanupConnection()

                if stopped || Task.isCancelled {
                    break
                }

                try? await Task.sleep(nanoseconds: missingSocketRetryDelayNanos)
                continue
            }

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

    private func socketFileExists() -> Bool {
        FileManager.default.fileExists(atPath: socketPath)
    }

    private func connectAndReadLoop() async throws {
        let fd = try openUnixSocket(path: socketPath)
        socketFD = fd
        onStateChange?(.connected)
        try await sendRegister()

        for try await chunk in makeReadStream(for: fd) {
            if stopped || Task.isCancelled {
                return
            }
            if chunk.isEmpty {
                continue
            }
            try parseIncomingData(chunk)
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
}
