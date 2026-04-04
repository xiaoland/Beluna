import { computed, onBeforeUnmount, onMounted, reactive, ref } from 'vue'

import { fetchRuntimeStatus, forceKillCore, stopCore, wakeCore } from '@/bridge/atropos'
import { hasTauriBridge } from '@/bridge/env'
import type { LaunchTargetRef } from '@/projection/clotho'
import { normalizeRuntimeStatus, type RuntimeStatus } from '@/projection/atropos'

const ACTIVE_PHASES = new Set(['waking', 'running', 'stopping'])
const FORCE_KILL_PHASES = new Set(['running', 'stopping'])
const POLL_INTERVAL_MS = 800

export function useAtroposRuntime(selectedTarget: { value: LaunchTargetRef | null }) {
  const usingTauri = hasTauriBridge()
  const issue = ref<string | null>(null)
  const runtime = ref<RuntimeStatus | null>(null)
  const forceKillConfirmOpen = ref(false)

  const loading = reactive({
    runtime: false,
    wake: false,
    stop: false,
    forceKill: false,
  })

  let pollTimer: number | null = null

  onMounted(async () => {
    if (!usingTauri) {
      issue.value =
        'Atropos supervision requires the Tauri bridge. Start Moira through the desktop shell to wake and supervise Core.'
      return
    }

    await refreshRuntimeStatus()
  })

  onBeforeUnmount(() => {
    stopPolling()
  })

  const canWake = computed(
    () =>
      usingTauri &&
      !loading.runtime &&
      !loading.wake &&
      selectedTarget.value != null &&
      !ACTIVE_PHASES.has(runtime.value?.phase ?? 'idle'),
  )
  const canStop = computed(() => usingTauri && !loading.stop && ACTIVE_PHASES.has(runtime.value?.phase ?? 'idle'))
  const canForceKill = computed(
    () =>
      usingTauri &&
      !loading.forceKill &&
      runtime.value?.pid != null &&
      FORCE_KILL_PHASES.has(runtime.value.phase),
  )

  async function refreshRuntimeStatus(): Promise<void> {
    if (!usingTauri) {
      return
    }

    loading.runtime = true
    try {
      const payload = await fetchRuntimeStatus()
      runtime.value = normalizeRuntimeStatus(payload)
      syncPolling()
    } catch (error) {
      issue.value = `Unable to load Atropos runtime: ${errorMessage(error)}`
      stopPolling()
    } finally {
      loading.runtime = false
    }
  }

  async function wakeSelectedTarget(profileId: string | null): Promise<void> {
    if (!canWake.value || !selectedTarget.value) {
      return
    }

    issue.value = null
    loading.wake = true

    try {
      const payload = await wakeCore({
        target: selectedTarget.value,
        profile: toProfileRef(profileId),
      })
      runtime.value = normalizeRuntimeStatus(payload)
      syncPolling()
    } catch (error) {
      issue.value = `Unable to wake supervised Core: ${errorMessage(error)}`
    } finally {
      loading.wake = false
    }
  }

  async function stopRuntime(): Promise<void> {
    if (!canStop.value) {
      return
    }

    issue.value = null
    loading.stop = true

    try {
      const payload = await stopCore()
      runtime.value = normalizeRuntimeStatus(payload)
      syncPolling()
    } catch (error) {
      issue.value = `Unable to stop supervised Core: ${errorMessage(error)}`
    } finally {
      loading.stop = false
    }
  }

  function requestForceKillConfirmation(): void {
    if (!canForceKill.value) {
      return
    }

    forceKillConfirmOpen.value = true
  }

  function cancelForceKillConfirmation(): void {
    forceKillConfirmOpen.value = false
  }

  async function confirmForceKill(): Promise<void> {
    if (!canForceKill.value) {
      forceKillConfirmOpen.value = false
      return
    }

    issue.value = null
    loading.forceKill = true

    try {
      const payload = await forceKillCore()
      runtime.value = normalizeRuntimeStatus(payload)
      forceKillConfirmOpen.value = false
      syncPolling()
    } catch (error) {
      issue.value = `Unable to force-kill supervised Core: ${errorMessage(error)}`
    } finally {
      loading.forceKill = false
    }
  }

  function syncPolling(): void {
    if (!canForceKill.value) {
      forceKillConfirmOpen.value = false
    }

    if (ACTIVE_PHASES.has(runtime.value?.phase ?? 'idle')) {
      startPolling()
      return
    }

    stopPolling()
  }

  function startPolling(): void {
    if (pollTimer) {
      return
    }

    pollTimer = window.setInterval(() => {
      void refreshRuntimeStatus()
    }, POLL_INTERVAL_MS)
  }

  function stopPolling(): void {
    if (!pollTimer) {
      return
    }

    window.clearInterval(pollTimer)
    pollTimer = null
  }

  return {
    cancelForceKillConfirmation,
    canForceKill,
    canStop,
    canWake,
    confirmForceKill,
    forceKillConfirmOpen,
    issue,
    loading,
    refreshRuntimeStatus,
    requestForceKillConfirmation,
    runtime,
    stopRuntime,
    usingTauri,
    wakeSelectedTarget,
  }
}

function toProfileRef(profileId: string | null): { profileId: string } | null {
  const trimmed = profileId?.trim() ?? ''
  return trimmed.length > 0 ? { profileId: trimmed } : null
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}
