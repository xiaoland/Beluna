import { computed, onBeforeUnmount, onMounted, reactive, ref } from 'vue'

import { fetchRuntimeStatus, forceKillCore, stopCore, wakeCore } from '@/bridge/atropos'
import { registerKnownLocalBuild } from '@/bridge/clotho'
import { hasTauriBridge } from '@/bridge/env'
import { normalizeRuntimeStatus, type RuntimeStatus } from '@/projection/atropos'

const ACTIVE_PHASES = new Set(['waking', 'running', 'stopping'])
const FORCE_KILL_PHASES = new Set(['running', 'stopping'])
const POLL_INTERVAL_MS = 800

export function useWakeControl() {
  const usingTauri = hasTauriBridge()
  const issue = ref<string | null>(null)
  const runtime = ref<RuntimeStatus | null>(null)
  const selectedBuildId = ref<string | null>(null)

  const draft = reactive({
    buildId: '',
    executablePath: '',
    workingDir: '',
    sourceDir: '',
  })

  const loading = reactive({
    register: false,
    runtime: false,
    wake: false,
    stop: false,
    forceKill: false,
  })

  let pollTimer: number | null = null
  const forceKillConfirmOpen = ref(false)

  onMounted(async () => {
    if (!usingTauri) {
      issue.value =
        'Wake control requires the Tauri bridge. Start Moira through the desktop shell to register builds and supervise Core.'
      return
    }

    await refreshRuntimeStatus()
  })

  onBeforeUnmount(() => {
    stopPolling()
  })

  const canRegister = computed(
    () => draft.buildId.trim().length > 0 && draft.executablePath.trim().length > 0 && !loading.register,
  )
  const canWake = computed(
    () =>
      usingTauri &&
      !loading.wake &&
      !loading.register &&
      selectedBuildId.value != null &&
      !ACTIVE_PHASES.has(runtime.value?.phase ?? 'idle'),
  )
  const canStop = computed(
    () => usingTauri && !loading.stop && ACTIVE_PHASES.has(runtime.value?.phase ?? 'idle'),
  )
  const canForceKill = computed(
    () =>
      usingTauri &&
      !loading.forceKill &&
      runtime.value?.pid != null &&
      FORCE_KILL_PHASES.has(runtime.value.phase),
  )

  function updateDraftField(
    field: 'buildId' | 'executablePath' | 'workingDir' | 'sourceDir',
    value: string,
  ) {
    draft[field] = value
  }

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

  async function registerBuild(): Promise<void> {
    if (!canRegister.value) {
      return
    }

    issue.value = null
    loading.register = true

    try {
      const payload = await registerKnownLocalBuild({
        buildId: draft.buildId.trim(),
        executablePath: draft.executablePath.trim(),
        workingDir: normalizeOptionalField(draft.workingDir),
        sourceDir: normalizeOptionalField(draft.sourceDir),
      })
      selectedBuildId.value = payload.buildId
    } catch (error) {
      issue.value = `Unable to register known local build: ${errorMessage(error)}`
    } finally {
      loading.register = false
    }
  }

  async function wakeSelectedBuild(profileId: string | null): Promise<void> {
    if (!canWake.value || !selectedBuildId.value) {
      return
    }

    issue.value = null
    loading.wake = true

    try {
      const payload = await wakeCore({
        build: {
          buildId: selectedBuildId.value,
        },
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

  function syncPolling() {
    if (!canForceKill.value) {
      forceKillConfirmOpen.value = false
    }

    if (ACTIVE_PHASES.has(runtime.value?.phase ?? 'idle')) {
      startPolling()
      return
    }

    stopPolling()
  }

  function startPolling() {
    if (pollTimer) {
      return
    }

    pollTimer = window.setInterval(() => {
      void refreshRuntimeStatus()
    }, POLL_INTERVAL_MS)
  }

  function stopPolling() {
    if (!pollTimer) {
      return
    }

    window.clearInterval(pollTimer)
    pollTimer = null
  }

  return {
    canRegister,
    cancelForceKillConfirmation,
    canForceKill,
    canStop,
    canWake,
    confirmForceKill,
    draft,
    forceKillConfirmOpen,
    issue,
    loading,
    refreshRuntimeStatus,
    registerBuild,
    requestForceKillConfirmation,
    runtime,
    selectedBuildId,
    stopRuntime,
    updateDraftField,
    usingTauri,
    wakeSelectedBuild,
  }
}

function normalizeOptionalField(value: string): string | null {
  const trimmed = value.trim()
  return trimmed.length > 0 ? trimmed : null
}

function toProfileRef(profileId: string | null): { profileId: string } | null {
  const trimmed = profileId?.trim() ?? ''
  return trimmed.length > 0 ? { profileId: trimmed } : null
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}
