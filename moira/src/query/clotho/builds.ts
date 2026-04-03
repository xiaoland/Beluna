import { computed, onMounted, reactive, ref } from 'vue'

import { registerKnownLocalBuild } from '@/bridge/clotho'
import { hasTauriBridge } from '@/bridge/env'

interface KnownLocalBuildDraft {
  buildId: string
  executablePath: string
  workingDir: string
  sourceDir: string
}

interface SelectedKnownLocalBuild {
  buildId: string
  executablePath: string
  workingDir: string
  sourceDir: string | null
}

export function useClothoBuildControl() {
  const usingTauri = hasTauriBridge()
  const issue = ref<string | null>(null)
  const registerDialogOpen = ref(false)
  const selectedBuild = ref<SelectedKnownLocalBuild | null>(null)

  const draft = reactive<KnownLocalBuildDraft>({
    buildId: '',
    executablePath: '',
    workingDir: '',
    sourceDir: '',
  })

  const loading = reactive({
    register: false,
  })

  const selectedBuildId = computed(() => selectedBuild.value?.buildId ?? null)
  const canRegister = computed(
    () => usingTauri && draft.buildId.trim().length > 0 && draft.executablePath.trim().length > 0 && !loading.register,
  )

  onMounted(() => {
    if (!usingTauri) {
      issue.value =
        'Build registration requires the Tauri bridge. Start Moira through the desktop shell to register local Core builds.'
    }
  })

  function openRegisterDialog(): void {
    if (!usingTauri) {
      issue.value =
        'Build registration requires the Tauri bridge. Start Moira through the desktop shell to register local Core builds.'
      return
    }

    issue.value = null
    registerDialogOpen.value = true
  }

  function closeRegisterDialog(): void {
    if (loading.register) {
      return
    }

    registerDialogOpen.value = false
  }

  function updateDraftField(
    field: 'buildId' | 'executablePath' | 'workingDir' | 'sourceDir',
    value: string,
  ): void {
    draft[field] = value
  }

  async function registerBuild(): Promise<void> {
    if (!canRegister.value) {
      return
    }

    issue.value = null
    loading.register = true

    const registration = {
      buildId: draft.buildId.trim(),
      executablePath: draft.executablePath.trim(),
      workingDir: normalizeOptionalField(draft.workingDir),
      sourceDir: normalizeOptionalField(draft.sourceDir),
    }

    try {
      const payload = await registerKnownLocalBuild(registration)
      selectedBuild.value = {
        buildId: payload.buildId,
        executablePath: registration.executablePath,
        workingDir: registration.workingDir ?? deriveWorkingDir(registration.executablePath),
        sourceDir: registration.sourceDir,
      }
      registerDialogOpen.value = false
    } catch (error) {
      issue.value = `Unable to register known local build: ${errorMessage(error)}`
    } finally {
      loading.register = false
    }
  }

  return {
    canRegister,
    closeRegisterDialog,
    draft,
    issue,
    loading,
    openRegisterDialog,
    registerBuild,
    registerDialogOpen,
    selectedBuild,
    selectedBuildId,
    updateDraftField,
    usingTauri,
  }
}

function normalizeOptionalField(value: string): string | null {
  const trimmed = value.trim()
  return trimmed.length > 0 ? trimmed : null
}

function deriveWorkingDir(executablePath: string): string {
  const lastSlash = executablePath.lastIndexOf('/')
  return lastSlash > 0 ? executablePath.slice(0, lastSlash) : executablePath
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}
