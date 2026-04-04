export type LaunchTargetRef =
  | {
      kind: 'knownLocalBuild'
      buildId: string
    }
  | {
      kind: 'installedArtifact'
      releaseTag: string
      rustTargetTriple: string
    }

export type LaunchTargetProvenance = 'registered' | 'forged' | 'installed'
export type LaunchTargetReadiness = 'ready' | 'stale'

export interface LaunchTargetSummary {
  key: string
  target: LaunchTargetRef
  label: string
  provenance: LaunchTargetProvenance
  readiness: LaunchTargetReadiness
  issue: string | null
  executablePath: string | null
  workingDir: string | null
  sourceDir: string | null
  installDir: string | null
  releaseTag: string | null
  rustTargetTriple: string | null
  checksumVerified: boolean
}

export interface PublishedReleaseSummary {
  key: string
  releaseTag: string
  displayName: string
  rustTargetTriple: string
  archiveAssetName: string
  checksumAssetName: string
  prerelease: boolean
  publishedAt: string | null
  alreadyInstalled: boolean
}

export interface ProfileDocumentSummary {
  profileId: string
  profilePath: string
}

export interface ProfileDocument {
  profileId: string
  profilePath: string
  contents: string
}
