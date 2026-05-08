import Foundation

struct MoiraLoomSelection: Equatable, Sendable {
    var runID: String?
    var tick: UInt64?

    static let none = Self(runID: nil, tick: nil)
}

struct MoiraLoomSnapshot: Decodable, Equatable, Sendable {
    var status: MoiraRuntimeSnapshot
    var launchTargets: [MoiraLaunchTargetSummary]
    var profiles: [MoiraProfileDocumentSummary]
    var runs: [MoiraRunSummary]
    var selectedRunID: String?
    var ticks: [MoiraTickSummary]
    var selectedTick: UInt64?
    var tickDetail: MoiraTickDetail?
    var updatedAt: Date?

    enum CodingKeys: String, CodingKey {
        case status
        case launchTargets
        case profiles
        case runs
        case selectedRunID = "selectedRunId"
        case ticks
        case selectedTick
        case tickDetail
    }

    static func unavailable(reason: String) -> Self {
        Self(
            status: .unavailable(reason: reason),
            launchTargets: [],
            profiles: [],
            runs: [],
            selectedRunID: nil,
            ticks: [],
            selectedTick: nil,
            tickDetail: nil,
            updatedAt: nil
        )
    }
}

struct MoiraLaunchTargetSummary: Decodable, Equatable, Identifiable, Sendable {
    var target: MoiraLaunchTargetRef
    var label: String
    var provenance: String
    var readiness: String
    var issue: String?
    var executablePath: String?
    var workingDir: String?
    var sourceDir: String?
    var installDir: String?
    var releaseTag: String?
    var rustTargetTriple: String?
    var checksumVerified: Bool

    var id: String {
        target.id
    }
}

struct MoiraLaunchTargetRef: Decodable, Equatable, Sendable {
    var kind: String
    var buildID: String?
    var releaseTag: String?
    var rustTargetTriple: String?

    enum CodingKeys: String, CodingKey {
        case kind
        case buildID = "buildId"
        case releaseTag
        case rustTargetTriple
    }

    var id: String {
        switch kind {
        case "knownLocalBuild":
            "knownLocalBuild:\(buildID ?? "")"
        case "installedArtifact":
            "installedArtifact:\(releaseTag ?? ""):\(rustTargetTriple ?? "")"
        default:
            kind
        }
    }
}

struct MoiraProfileDocumentSummary: Decodable, Equatable, Identifiable, Sendable {
    var profileID: String
    var profilePath: String

    enum CodingKeys: String, CodingKey {
        case profileID = "profileId"
        case profilePath
    }

    var id: String {
        profileID
    }
}

struct MoiraRunSummary: Decodable, Equatable, Identifiable, Sendable {
    var runID: String
    var firstSeenAt: String
    var lastSeenAt: String
    var eventCount: Int
    var warningCount: Int
    var errorCount: Int
    var latestTick: UInt64?

    enum CodingKeys: String, CodingKey {
        case runID = "runId"
        case firstSeenAt
        case lastSeenAt
        case eventCount
        case warningCount
        case errorCount
        case latestTick
    }

    var id: String {
        runID
    }
}

struct MoiraTickSummary: Decodable, Equatable, Identifiable, Sendable {
    var runID: String
    var tick: UInt64
    var traceID: String?
    var firstSeenAt: String
    var lastSeenAt: String
    var eventCount: Int
    var warningCount: Int
    var errorCount: Int
    var cortexHandled: Bool

    enum CodingKeys: String, CodingKey {
        case runID = "runId"
        case tick
        case traceID = "traceId"
        case firstSeenAt
        case lastSeenAt
        case eventCount
        case warningCount
        case errorCount
        case cortexHandled
    }

    var id: String {
        "\(runID):\(tick)"
    }
}

struct MoiraTickDetail: Decodable, Equatable, Sendable {
    var summary: MoiraTickSummary
    var cortex: [MoiraEventRecord]
    var stem: [MoiraEventRecord]
    var spine: [MoiraEventRecord]
    var raw: [MoiraEventRecord]
}

struct MoiraEventRecord: Decodable, Equatable, Identifiable, Sendable {
    var rawEventID: String
    var receivedAt: String
    var observedAt: String
    var severityText: String
    var recordKind: String
    var scopeName: String?
    var eventName: String?
    var traceID: String?
    var spanID: String?
    var traceFlags: UInt32?
    var target: String?
    var family: String?
    var subsystem: String?
    var runID: String?
    var tick: UInt64?
    var messageText: String?
    var attributes: JSONValue
    var body: JSONValue
    var resource: JSONValue
    var scope: JSONValue

    enum CodingKeys: String, CodingKey {
        case rawEventID = "rawEventId"
        case receivedAt
        case observedAt
        case severityText
        case recordKind
        case scopeName
        case eventName
        case traceID = "traceId"
        case spanID = "spanId"
        case traceFlags
        case target
        case family
        case subsystem
        case runID = "runId"
        case tick
        case messageText
        case attributes
        case body
        case resource
        case scope
    }

    var id: String {
        rawEventID
    }

    var displayTitle: String {
        eventName ?? messageText ?? rawEventID
    }

    var ownerText: String {
        subsystem ?? family ?? scopeName ?? recordKind
    }
}
