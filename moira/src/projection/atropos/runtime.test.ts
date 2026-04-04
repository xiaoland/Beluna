import { describe, expect, it } from 'vitest'

import { normalizeRuntimeStatus } from './runtime'

describe('atropos runtime projection', () => {
  it('normalizes target labels from runtime status payloads', () => {
    const runtime = normalizeRuntimeStatus({
      phase: 'running',
      targetLabel: 'v0.1.0 / aarch64-apple-darwin',
      executablePath: '/tmp/beluna',
      workingDir: '/tmp',
      profilePath: '/tmp/default.jsonc',
      pid: 42,
      terminalReason: null,
    })

    expect(runtime.phase).toBe('running')
    expect(runtime.targetLabel).toBe('v0.1.0 / aarch64-apple-darwin')
    expect(runtime.pid).toBe(42)
  })
})
