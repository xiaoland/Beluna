import { describe, expect, it } from 'vitest'

import { launchTargetKey, normalizeLaunchTargetSummary } from './targets'

describe('clotho launch target projection', () => {
  it('normalizes installed artifact summaries into stable keys', () => {
    const summary = normalizeLaunchTargetSummary({
      target: {
        kind: 'installedArtifact',
        releaseTag: 'v0.1.0',
        rustTargetTriple: 'aarch64-apple-darwin',
      },
      label: 'v0.1.0 / aarch64-apple-darwin',
      provenance: 'installed',
      readiness: 'ready',
      issue: null,
      executablePath: '/tmp/beluna',
      workingDir: '/tmp',
      installDir: '/tmp',
      releaseTag: 'v0.1.0',
      rustTargetTriple: 'aarch64-apple-darwin',
      checksumVerified: true,
    })

    expect(summary.key).toBe('installed:v0.1.0:aarch64-apple-darwin')
    expect(summary.checksumVerified).toBe(true)
    expect(summary.readiness).toBe('ready')
  })

  it('builds known-local target keys deterministically', () => {
    expect(
      launchTargetKey({
        kind: 'knownLocalBuild',
        buildId: 'dev-core',
      }),
    ).toBe('known:dev-core')
  })
})
