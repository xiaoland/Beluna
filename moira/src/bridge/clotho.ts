import { invoke } from '@tauri-apps/api/core'

import type {
  KnownLocalBuildRefPayload,
  KnownLocalBuildRegistrationPayload,
  ProfileDocumentPayload,
  ProfileDocumentSummaryPayload,
  ProfileRefPayload,
  SaveProfileDocumentRequestPayload,
} from '@/bridge/contracts/clotho'

export async function registerKnownLocalBuild(
  registration: KnownLocalBuildRegistrationPayload,
): Promise<KnownLocalBuildRefPayload> {
  return invoke<KnownLocalBuildRefPayload>('register_known_local_build', { registration })
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
