import Foundation

enum MoiraRuntimeLifecycle: String, Decodable, Equatable, Sendable {
    case opening
    case ready
    case degraded
    case closing
    case closed
    case unavailable
}

enum MoiraResourceKind: String, Decodable, Equatable, Sendable {
    case directory
    case telemetryStore
    case otlpReceiver
    case coreSupervisor
    case platformAdapter
}

enum MoiraResourceState: String, Decodable, Equatable, Sendable {
    case available
    case claiming
    case claimed
    case degraded
    case conflict
    case faulted
}

struct MoiraResourceStatus: Decodable, Equatable, Identifiable, Sendable {
    var kind: MoiraResourceKind
    var state: MoiraResourceState
    var label: String
    var detail: String?

    var id: String {
        "\(kind.rawValue):\(label)"
    }
}

struct MoiraReceiverStatus: Decodable, Equatable, Sendable {
    var endpoint: String
    var wakeState: String
    var dbPath: String
    var lastBatchAt: String?
    var lastError: String?
    var rawEventCount: Int
    var wakeCount: Int
    var tickCount: Int
}

struct MoiraCoreStatus: Decodable, Equatable, Sendable {
    var phase: String
    var targetLabel: String?
    var executablePath: String?
    var workingDir: String?
    var profilePath: String?
    var pid: Int?
    var terminalReason: String?
}

struct MoiraRuntimeSnapshot: Decodable, Equatable, Sendable {
    var lifecycle: MoiraRuntimeLifecycle
    var resources: [MoiraResourceStatus]
    var receiver: MoiraReceiverStatus
    var core: MoiraCoreStatus
    var updatedAt: Date?

    var attentionResources: [MoiraResourceStatus] {
        resources.filter { resource in
            switch resource.state {
            case .degraded, .conflict, .faulted:
                true
            case .available, .claiming, .claimed:
                false
            }
        }
    }

    static func unavailable(reason: String) -> Self {
        Self(
            lifecycle: .unavailable,
            resources: [
                MoiraResourceStatus(
                    kind: .platformAdapter,
                    state: .degraded,
                    label: "Rust binding",
                    detail: reason
                ),
            ],
            receiver: MoiraReceiverStatus(
                endpoint: "Unavailable",
                wakeState: "unavailable",
                dbPath: "",
                lastBatchAt: nil,
                lastError: reason,
                rawEventCount: 0,
                wakeCount: 0,
                tickCount: 0
            ),
            core: MoiraCoreStatus(
                phase: "unavailable",
                targetLabel: nil,
                executablePath: nil,
                workingDir: nil,
                profilePath: nil,
                pid: nil,
                terminalReason: nil
            ),
            updatedAt: nil
        )
    }
}
