import { describe, expect, it } from 'vitest'

import { comparePublishedReleaseSummary, normalizePublishedReleaseSummary } from './releases'

describe('clotho release projection', () => {
  it('normalizes release summaries and preserves install state', () => {
    const summary = normalizePublishedReleaseSummary({
      releaseTag: 'v0.1.1',
      displayName: 'v0.1.1',
      rustTargetTriple: 'aarch64-apple-darwin',
      archiveAssetName: 'beluna-core-aarch64-apple-darwin.tar.gz',
      checksumAssetName: 'SHA256SUMS',
      prerelease: true,
      publishedAt: '2026-04-04T00:00:00Z',
      alreadyInstalled: false,
    })

    expect(summary.key).toBe('v0.1.1:aarch64-apple-darwin')
    expect(summary.alreadyInstalled).toBe(false)
  })

  it('sorts not-yet-installed releases ahead of installed ones', () => {
    const pending = normalizePublishedReleaseSummary({
      releaseTag: 'v0.1.2',
      displayName: 'v0.1.2',
      rustTargetTriple: 'aarch64-apple-darwin',
      archiveAssetName: 'beluna-core-aarch64-apple-darwin.tar.gz',
      checksumAssetName: 'SHA256SUMS',
      prerelease: true,
      publishedAt: '2026-04-04T00:00:00Z',
      alreadyInstalled: false,
    })
    const installed = normalizePublishedReleaseSummary({
      releaseTag: 'v0.1.1',
      displayName: 'v0.1.1',
      rustTargetTriple: 'aarch64-apple-darwin',
      archiveAssetName: 'beluna-core-aarch64-apple-darwin.tar.gz',
      checksumAssetName: 'SHA256SUMS',
      prerelease: true,
      publishedAt: '2026-04-03T00:00:00Z',
      alreadyInstalled: true,
    })

    expect(comparePublishedReleaseSummary(pending, installed)).toBeLessThan(0)
  })
})
