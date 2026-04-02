export interface KnownLocalBuildRefPayload {
  buildId: string
}

export interface KnownLocalBuildRegistrationPayload {
  buildId: string
  executablePath: string
  workingDir?: string | null
  sourceDir?: string | null
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
  build: KnownLocalBuildRefPayload
  profile?: ProfileRefPayload | null
}
