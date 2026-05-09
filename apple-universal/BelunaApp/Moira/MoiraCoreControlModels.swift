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

struct MoiraProfileDocument: Decodable, Equatable, Sendable {
    var profileID: String
    var profilePath: String
    var contents: String

    enum CodingKeys: String, CodingKey {
        case profileID = "profileId"
        case profilePath
        case contents
    }
}

struct MoiraProfileSaveRequest: Encodable, Equatable, Sendable {
    var profileID: String
    var contents: String

    enum CodingKeys: String, CodingKey {
        case profileID = "profileId"
        case contents
    }
}

struct MoiraProfileDraftDocument: Decodable, Equatable, Sendable {
    var profileID: String
    var profilePath: String
    var coreConfig: String
    var envFiles: [MoiraProfileEnvFileDraft]
    var inlineEnvironment: [MoiraProfileInlineEnvironmentDraft]

    enum CodingKeys: String, CodingKey {
        case profileID = "profileId"
        case profilePath
        case coreConfig
        case envFiles
        case inlineEnvironment
    }
}

struct MoiraProfileDraftSaveRequest: Encodable, Equatable, Sendable {
    var profileID: String
    var coreConfig: String
    var envFiles: [MoiraProfileEnvFileDraft]
    var inlineEnvironment: [MoiraProfileInlineEnvironmentDraft]

    enum CodingKeys: String, CodingKey {
        case profileID = "profileId"
        case coreConfig
        case envFiles
        case inlineEnvironment
    }
}

struct MoiraProfileEnvFileDraft: Codable, Equatable, Sendable {
    var path: String
    var required: Bool
}

struct MoiraProfileInlineEnvironmentDraft: Codable, Equatable, Sendable {
    var key: String
    var value: String
}

struct MoiraKnownLocalBuildRegistration: Encodable, Equatable, Sendable {
    var buildID: String
    var executablePath: String
    var workingDir: String?
    var sourceDir: String?

    enum CodingKeys: String, CodingKey {
        case buildID = "buildId"
        case executablePath
        case workingDir
        case sourceDir
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
