import Foundation

struct MoiraTargetEditorDraft: Equatable {
    var buildID = ""
    var executablePath = ""
    var workingDir = ""
    var sourceDir = ""

    init() {}

    init?(target: MoiraLaunchTargetSummary) {
        guard target.target.kind == "knownLocalBuild",
              let buildID = target.target.buildID else {
            return nil
        }

        self.buildID = buildID
        self.executablePath = target.executablePath ?? ""
        self.workingDir = target.workingDir ?? ""
        self.sourceDir = target.sourceDir ?? ""
    }

    var isValid: Bool {
        !buildID.trimmedForMoiraManagement.isEmpty
            && !executablePath.trimmedForMoiraManagement.isEmpty
    }

    var registration: MoiraKnownLocalBuildRegistration {
        MoiraKnownLocalBuildRegistration(
            buildID: buildID.trimmedForMoiraManagement,
            executablePath: executablePath.trimmedForMoiraManagement,
            workingDir: workingDir.optionalMoiraManagementValue,
            sourceDir: sourceDir.optionalMoiraManagementValue
        )
    }
}

struct MoiraProfileEditorDraft: Equatable {
    var profileID = ""
    var loadedProfilePath: String?
    var coreConfig = "{\n}\n"
    var envFiles: [MoiraProfileEnvFileDraft] = []
    var inlineEnvironment: [MoiraProfileInlineEnvironmentDraft] = []

    init() {}

    init(profileID: String) {
        self.profileID = profileID
    }

    init(document: MoiraProfileDraftDocument) {
        self.profileID = document.profileID
        self.loadedProfilePath = document.profilePath
        self.coreConfig = document.coreConfig
        self.envFiles = document.envFiles
        self.inlineEnvironment = document.inlineEnvironment
    }

    var isValid: Bool {
        !profileID.trimmedForMoiraManagement.isEmpty
            && !coreConfig.trimmedForMoiraManagement.isEmpty
            && envFiles.allSatisfy { !$0.path.trimmedForMoiraManagement.isEmpty }
            && inlineEnvironment.allSatisfy { !$0.key.trimmedForMoiraManagement.isEmpty }
    }

    var saveRequest: MoiraProfileDraftSaveRequest {
        MoiraProfileDraftSaveRequest(
            profileID: profileID.trimmedForMoiraManagement,
            coreConfig: coreConfig,
            envFiles: envFiles,
            inlineEnvironment: inlineEnvironment
        )
    }
}

private extension String {
    var trimmedForMoiraManagement: String {
        trimmingCharacters(in: .whitespacesAndNewlines)
    }

    var optionalMoiraManagementValue: String? {
        let trimmed = trimmedForMoiraManagement
        return trimmed.isEmpty ? nil : trimmed
    }
}
