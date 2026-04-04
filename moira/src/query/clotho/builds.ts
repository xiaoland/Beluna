import { computed, onMounted, reactive, ref } from 'vue'

import {
  forgeLocalBuild,
  installPublishedRelease,
  listLaunchTargets,
  listPublishedReleases,
  registerKnownLocalBuild,
} from '@/bridge/clotho'
import { hasTauriBridge } from '@/bridge/env'
import {
  compareLaunchTargetSummary,
  comparePublishedReleaseSummary,
  launchTargetKey,
  normalizeLaunchTargetSummary,
  normalizePublishedReleaseSummary,
  type LaunchTargetRef,
  type LaunchTargetSummary,
  type PublishedReleaseSummary,
} from '@/projection/clotho'

interface KnownLocalBuildDraft {
  buildId: string
  executablePath: string
  workingDir: string
  sourceDir: string
}

interface ForgeBuildDraft {
  buildId: string
  sourceDir: string
}

const SUPPORTED_RELEASE_TARGET = 'aarch64-apple-darwin'

export function useClothoBuildControl() {
  const usingTauri = hasTauriBridge()
  const issue = ref<string | null>(null)
  const launchTargets = ref<LaunchTargetSummary[]>([])
  const publishedReleases = ref<PublishedReleaseSummary[]>([])
  const registerDialogOpen = ref(false)
  const forgeDialogOpen = ref(false)
  const installDialogOpen = ref(false)
  const selectedTargetKey = ref<string | null>(null)
  const selectedReleaseKey = ref<string | null>(null)

  const registerDraft = reactive<KnownLocalBuildDraft>({
    buildId: '',
    executablePath: '',
    workingDir: '',
    sourceDir: '',
  })

  const forgeDraft = reactive<ForgeBuildDraft>({
    buildId: '',
    sourceDir: '',
  })

  const loading = reactive({
    forge: false,
    install: false,
    listReleases: false,
    listTargets: false,
    register: false,
  })

  onMounted(async () => {
    if (!usingTauri) {
      issue.value =
        'Clotho launch-target management requires the Tauri bridge. Start Moira through the desktop shell to register, forge, or install Core targets.'
      return
    }

    await refreshLaunchTargets()
  })

  const selectedTarget = computed<LaunchTargetSummary | null>(
    () => launchTargets.value.find((target) => target.key === selectedTargetKey.value) ?? null,
  )
  const selectedTargetRef = computed<LaunchTargetRef | null>(() => selectedTarget.value?.target ?? null)
  const selectedTargetLabel = computed(() => selectedTarget.value?.label ?? null)

  const canRegister = computed(
    () =>
      usingTauri &&
      !loading.register &&
      registerDraft.buildId.trim().length > 0 &&
      registerDraft.executablePath.trim().length > 0,
  )
  const canForge = computed(
    () =>
      usingTauri &&
      !loading.forge &&
      forgeDraft.buildId.trim().length > 0 &&
      forgeDraft.sourceDir.trim().length > 0,
  )
  const canInstall = computed(
    () => usingTauri && !loading.install && selectedRelease.value != null,
  )
  const selectedRelease = computed(
    () => publishedReleases.value.find((release) => release.key === selectedReleaseKey.value) ?? null,
  )

  async function refreshLaunchTargets(): Promise<void> {
    if (!usingTauri) {
      return
    }

    loading.listTargets = true
    try {
      const payload = await listLaunchTargets()
      const nextTargets = payload.map(normalizeLaunchTargetSummary).sort(compareLaunchTargetSummary)
      launchTargets.value = nextTargets

      if (selectedTargetKey.value && !nextTargets.some((target) => target.key === selectedTargetKey.value)) {
        selectedTargetKey.value = null
      }
    } catch (error) {
      issue.value = `Unable to list Clotho launch targets: ${errorMessage(error)}`
    } finally {
      loading.listTargets = false
    }
  }

  async function refreshPublishedReleases(): Promise<void> {
    if (!usingTauri) {
      return
    }

    loading.listReleases = true
    try {
      const payload = await listPublishedReleases()
      const releases = payload.map(normalizePublishedReleaseSummary).sort(comparePublishedReleaseSummary)
      publishedReleases.value = releases
      if (selectedReleaseKey.value && !releases.some((release) => release.key === selectedReleaseKey.value)) {
        selectedReleaseKey.value = releases[0]?.key ?? null
      }
      if (!selectedReleaseKey.value && releases.length > 0) {
        selectedReleaseKey.value = releases[0].key
      }
    } catch (error) {
      issue.value = `Unable to list published releases: ${errorMessage(error)}`
    } finally {
      loading.listReleases = false
    }
  }

  function selectTarget(target: LaunchTargetRef): void {
    selectedTargetKey.value = launchTargetKey(target)
  }

  function openRegisterDialog(): void {
    if (!usingTauri) {
      issue.value =
        'Clotho launch-target management requires the Tauri bridge. Start Moira through the desktop shell to register, forge, or install Core targets.'
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

  function updateRegisterDraftField(
    field: 'buildId' | 'executablePath' | 'workingDir' | 'sourceDir',
    value: string,
  ): void {
    registerDraft[field] = value
  }

  async function registerBuild(): Promise<void> {
    if (!canRegister.value) {
      return
    }

    issue.value = null
    loading.register = true

    try {
      const target = await registerKnownLocalBuild({
        buildId: registerDraft.buildId.trim(),
        executablePath: registerDraft.executablePath.trim(),
        workingDir: normalizeOptionalField(registerDraft.workingDir),
        sourceDir: normalizeOptionalField(registerDraft.sourceDir),
      })
      await refreshLaunchTargets()
      selectTarget(target)
      registerDialogOpen.value = false
    } catch (error) {
      issue.value = `Unable to register known local build: ${errorMessage(error)}`
    } finally {
      loading.register = false
    }
  }

  function openForgeDialog(): void {
    if (!usingTauri) {
      issue.value =
        'Clotho launch-target management requires the Tauri bridge. Start Moira through the desktop shell to register, forge, or install Core targets.'
      return
    }

    issue.value = null
    forgeDialogOpen.value = true
  }

  function closeForgeDialog(): void {
    if (loading.forge) {
      return
    }

    forgeDialogOpen.value = false
  }

  function updateForgeDraftField(field: 'buildId' | 'sourceDir', value: string): void {
    forgeDraft[field] = value
  }

  async function forgeBuild(): Promise<void> {
    if (!canForge.value) {
      return
    }

    issue.value = null
    loading.forge = true

    try {
      const target = await forgeLocalBuild({
        buildId: forgeDraft.buildId.trim(),
        sourceDir: forgeDraft.sourceDir.trim(),
      })
      await refreshLaunchTargets()
      selectTarget(target)
      forgeDialogOpen.value = false
    } catch (error) {
      issue.value = `Unable to forge local build: ${errorMessage(error)}`
    } finally {
      loading.forge = false
    }
  }

  async function openInstallDialog(): Promise<void> {
    if (!usingTauri) {
      issue.value =
        'Clotho launch-target management requires the Tauri bridge. Start Moira through the desktop shell to register, forge, or install Core targets.'
      return
    }

    issue.value = null
    installDialogOpen.value = true
    await refreshPublishedReleases()
  }

  function closeInstallDialog(): void {
    if (loading.install) {
      return
    }

    installDialogOpen.value = false
  }

  function selectPublishedRelease(releaseKey: string): void {
    selectedReleaseKey.value = releaseKey
  }

  async function installRelease(): Promise<void> {
    if (!canInstall.value || !selectedRelease.value) {
      return
    }

    issue.value = null
    loading.install = true

    try {
      const target = await installPublishedRelease({
        releaseTag: selectedRelease.value.releaseTag,
        rustTargetTriple: selectedRelease.value.rustTargetTriple || SUPPORTED_RELEASE_TARGET,
      })
      await Promise.all([refreshLaunchTargets(), refreshPublishedReleases()])
      selectTarget(target)
      installDialogOpen.value = false
    } catch (error) {
      issue.value = `Unable to install published release: ${errorMessage(error)}`
    } finally {
      loading.install = false
    }
  }

  return {
    canForge,
    canInstall,
    canRegister,
    closeForgeDialog,
    closeInstallDialog,
    closeRegisterDialog,
    forgeBuild,
    forgeDialogOpen,
    forgeDraft,
    installDialogOpen,
    installRelease,
    issue,
    launchTargets,
    loading,
    openForgeDialog,
    openInstallDialog,
    openRegisterDialog,
    publishedReleases,
    refreshLaunchTargets,
    refreshPublishedReleases,
    registerBuild,
    registerDialogOpen,
    registerDraft,
    selectPublishedRelease,
    selectTarget,
    selectedReleaseKey,
    selectedTarget,
    selectedTargetKey,
    selectedTargetLabel,
    selectedTargetRef,
    updateForgeDraftField,
    updateRegisterDraftField,
    usingTauri,
  }
}

function normalizeOptionalField(value: string): string | null {
  const trimmed = value.trim()
  return trimmed.length > 0 ? trimmed : null
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}
