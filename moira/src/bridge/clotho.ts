import { invoke } from '@tauri-apps/api/core'

import type {
  ForgeLocalBuildRequestPayload,
  InstallPublishedReleaseRequestPayload,
  KnownLocalBuildRegistrationPayload,
  LaunchTargetRefPayload,
  LaunchTargetSummaryPayload,
  ProfileDocumentPayload,
  ProfileDocumentSummaryPayload,
  ProfileRefPayload,
  PublishedReleaseSummaryPayload,
  SaveProfileDocumentRequestPayload,
} from '@/bridge/contracts/clotho'

export async function listLaunchTargets(): Promise<LaunchTargetSummaryPayload[]> {
  return invoke<LaunchTargetSummaryPayload[]>('list_launch_targets')
}

export async function registerKnownLocalBuild(
  registration: KnownLocalBuildRegistrationPayload,
): Promise<LaunchTargetRefPayload> {
  return invoke<LaunchTargetRefPayload>('register_known_local_build', { registration })
}

export async function forgeLocalBuild(
  request: ForgeLocalBuildRequestPayload,
): Promise<LaunchTargetRefPayload> {
  return invoke<LaunchTargetRefPayload>('forge_local_build', { request })
}

export async function listPublishedReleases(): Promise<PublishedReleaseSummaryPayload[]> {
  return invoke<PublishedReleaseSummaryPayload[]>('list_published_releases')
}

export async function installPublishedRelease(
  request: InstallPublishedReleaseRequestPayload,
): Promise<LaunchTargetRefPayload> {
  return invoke<LaunchTargetRefPayload>('install_published_release', { request })
}

export async function listProfileDocuments(): Promise<ProfileDocumentSummaryPayload[]> {
  return invoke<ProfileDocumentSummaryPayload[]>('list_profile_documents')
}

export async function loadProfileDocument(profile: ProfileRefPayload): Promise<ProfileDocumentPayload> {
  return invoke<ProfileDocumentPayload>('load_profile_document', { profile })
}

export async function saveProfileDocument(
  request: SaveProfileDocumentRequestPayload,
): Promise<ProfileDocumentPayload> {
  return invoke<ProfileDocumentPayload>('save_profile_document', { request })
}
