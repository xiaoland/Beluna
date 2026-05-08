import Foundation

struct MoiraCoreWakeRequest: Encodable, Equatable, Sendable {
    var target: MoiraLaunchTargetRef
    var profile: MoiraProfileRef?
}

struct MoiraProfileRef: Encodable, Equatable, Sendable {
    var profileID: String

    enum CodingKeys: String, CodingKey {
        case profileID = "profileId"
    }
}

extension MoiraLaunchTargetRef: Encodable {
    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(kind, forKey: .kind)
        try container.encodeIfPresent(buildID, forKey: .buildID)
        try container.encodeIfPresent(releaseTag, forKey: .releaseTag)
        try container.encodeIfPresent(rustTargetTriple, forKey: .rustTargetTriple)
    }
}
