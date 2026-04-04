import type { PublishedReleaseSummaryPayload } from '@/bridge/contracts/clotho'
import type { PublishedReleaseSummary } from './models'

export function normalizePublishedReleaseSummary(
  payload: PublishedReleaseSummaryPayload,
): PublishedReleaseSummary {
  const releaseTag = normalizeString(payload.releaseTag) ?? ''
  const rustTargetTriple = normalizeString(payload.rustTargetTriple) ?? ''

  return {
    key: `${releaseTag}:${rustTargetTriple}`,
    releaseTag,
    displayName: normalizeString(payload.displayName) ?? releaseTag,
    rustTargetTriple,
    archiveAssetName: normalizeString(payload.archiveAssetName) ?? '',
    checksumAssetName: normalizeString(payload.checksumAssetName) ?? '',
    prerelease: Boolean(payload.prerelease),
    publishedAt: normalizeString(payload.publishedAt),
    alreadyInstalled: Boolean(payload.alreadyInstalled),
  }
}

export function comparePublishedReleaseSummary(
  left: PublishedReleaseSummary,
  right: PublishedReleaseSummary,
): number {
  if (left.alreadyInstalled !== right.alreadyInstalled) {
    return left.alreadyInstalled ? 1 : -1
  }

  return right.releaseTag.localeCompare(left.releaseTag)
}

function normalizeString(value: string | null | undefined): string | null {
  return typeof value === 'string' && value.trim().length > 0 ? value : null
}
