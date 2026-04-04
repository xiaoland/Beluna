export type LaunchTargetRefPayload =
  | {
      kind: 'knownLocalBuild'
      buildId: string
    }
  | {
      kind: 'installedArtifact'
      releaseTag: string
      rustTargetTriple: string
    }

export type LaunchTargetProvenancePayload = 'registered' | 'forged' | 'installed'
export type LaunchTargetReadinessPayload = 'ready' | 'stale'

export interface LaunchTargetSummaryPayload {
  target: LaunchTargetRefPayload
  label: string
  provenance: LaunchTargetProvenancePayload
  readiness: LaunchTargetReadinessPayload
  issue?: string | null
  executablePath?: string | null
  workingDir?: string | null
  sourceDir?: string | null
  installDir?: string | null
  releaseTag?: string | null
  rustTargetTriple?: string | null
  checksumVerified: boolean
}

export interface KnownLocalBuildRegistrationPayload {
  buildId: string
  executablePath: string
  workingDir?: string | null
  sourceDir?: string | null
}

export interface ForgeLocalBuildRequestPayload {
  buildId: string
  sourceDir: string
}

export interface InstallPublishedReleaseRequestPayload {
  releaseTag: string
  rustTargetTriple: string
}

export interface PublishedReleaseSummaryPayload {
  releaseTag: string
  displayName: string
  rustTargetTriple: string
  archiveAssetName: string
  checksumAssetName: string
  prerelease: boolean
  publishedAt?: string | null
  alreadyInstalled: boolean
}

export interface ProfileRefPayload {
  profileId: string
}

export interface ProfileDocumentSummaryPayload {
  profileId: string
  profilePath: string
}

export interface ProfileDocumentPayload {
  profileId: string
  profilePath: string
  contents: string
}

export interface SaveProfileDocumentRequestPayload {
  profileId: string
  contents: string
}

export interface WakeInputRequestPayload {
  target: LaunchTargetRefPayload
  profile?: ProfileRefPayload | null
}
