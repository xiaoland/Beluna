import type {
  ProfileDocumentPayload,
  ProfileDocumentSummaryPayload,
} from '@/bridge/contracts/clotho'
import type { ProfileDocument, ProfileDocumentSummary } from './models'

export function normalizeProfileDocumentSummary(
  payload: ProfileDocumentSummaryPayload,
): ProfileDocumentSummary {
  return {
    profileId: normalizeString(payload.profileId),
    profilePath: normalizeString(payload.profilePath),
  }
}

export function normalizeProfileDocument(payload: ProfileDocumentPayload): ProfileDocument {
  return {
    profileId: normalizeString(payload.profileId),
    profilePath: normalizeString(payload.profilePath),
    contents: typeof payload.contents === 'string' ? payload.contents : '',
  }
}

export function compareProfileDocumentSummary(
  left: ProfileDocumentSummary,
  right: ProfileDocumentSummary,
): number {
  return left.profileId.localeCompare(right.profileId)
}

function normalizeString(value: string | null | undefined): string {
  return typeof value === 'string' ? value.trim() : ''
}
