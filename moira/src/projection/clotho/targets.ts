import type {
  LaunchTargetReadinessPayload,
  LaunchTargetRefPayload,
  LaunchTargetSummaryPayload,
} from '@/bridge/contracts/clotho'
import type { LaunchTargetReadiness, LaunchTargetRef, LaunchTargetSummary } from './models'

const VALID_READINESS: LaunchTargetReadiness[] = ['ready', 'stale']

export function normalizeLaunchTargetRef(payload: LaunchTargetRefPayload): LaunchTargetRef {
  if (payload.kind === 'installedArtifact') {
    return {
      kind: 'installedArtifact',
      releaseTag: normalizeString(payload.releaseTag) ?? '',
      rustTargetTriple: normalizeString(payload.rustTargetTriple) ?? '',
    }
  }

  return {
    kind: 'knownLocalBuild',
    buildId: normalizeString(payload.buildId) ?? '',
  }
}

export function normalizeLaunchTargetSummary(
  payload: LaunchTargetSummaryPayload,
): LaunchTargetSummary {
  const target = normalizeLaunchTargetRef(payload.target)

  return {
    key: launchTargetKey(target),
    target,
    label: normalizeString(payload.label) ?? 'Untitled target',
    provenance: payload.provenance,
    readiness: normalizeReadiness(payload.readiness),
    issue: normalizeString(payload.issue),
    executablePath: normalizeString(payload.executablePath),
    workingDir: normalizeString(payload.workingDir),
    sourceDir: normalizeString(payload.sourceDir),
    installDir: normalizeString(payload.installDir),
    releaseTag: normalizeString(payload.releaseTag),
    rustTargetTriple: normalizeString(payload.rustTargetTriple),
    checksumVerified: Boolean(payload.checksumVerified),
  }
}

export function compareLaunchTargetSummary(left: LaunchTargetSummary, right: LaunchTargetSummary): number {
  if (left.readiness !== right.readiness) {
    return left.readiness === 'ready' ? -1 : 1
  }

  return left.label.localeCompare(right.label)
}

export function launchTargetKey(target: LaunchTargetRef): string {
  if (target.kind === 'installedArtifact') {
    return `installed:${target.releaseTag}:${target.rustTargetTriple}`
  }

  return `known:${target.buildId}`
}

function normalizeReadiness(value: LaunchTargetReadinessPayload): LaunchTargetReadiness {
  return VALID_READINESS.includes(value) ? value : 'stale'
}

function normalizeString(value: string | null | undefined): string | null {
  return typeof value === 'string' && value.trim().length > 0 ? value : null
}
