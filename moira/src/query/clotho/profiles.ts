import { computed, onMounted, reactive, ref } from 'vue'

import { hasTauriBridge } from '@/bridge/env'
import { listProfileDocuments, loadProfileDocument, saveProfileDocument } from '@/bridge/clotho'
import {
  compareProfileDocumentSummary,
  createEmptyEditableProfileDocument,
  parseEditableProfileDocument,
  normalizeProfileDocument,
  normalizeProfileDocumentSummary,
  serializeEditableProfileDocument,
  type EditableProfileDocument,
  type EditableProfileEnvironmentFile,
  type EditableProfileInlineEnvironment,
  type ProfileDocumentSummary,
} from '@/projection/clotho'

export function useClothoProfileControl() {
  const usingTauri = hasTauriBridge()
  const issue = ref<string | null>(null)
  const profileDialogOpen = ref(false)
  const profiles = ref<ProfileDocumentSummary[]>([])
  const selectedProfileId = ref<string | null>(null)
  const loadedProfilePath = ref<string | null>(null)

  const draft = reactive({
    profileId: '',
    coreConfig: createEmptyEditableProfileDocument().coreConfig,
    envFiles: [] as Array<EditableProfileEnvironmentFile & { id: string }>,
    inlineEnvironment: [] as Array<EditableProfileInlineEnvironment & { id: string }>,
  })

  const loading = reactive({
    list: false,
    load: false,
    save: false,
  })

  onMounted(async () => {
    if (!usingTauri) {
      issue.value =
        'Profile management requires the Tauri bridge. Start Moira through the desktop shell to create or edit local profile documents.'
      return
    }

    await refreshProfiles()
  })

  const canSave = computed(
    () =>
      usingTauri &&
      !loading.save &&
      !loading.load &&
      draft.profileId.trim().length > 0 &&
      draft.coreConfig.trim().length > 0,
  )

  const pathHint = computed(() => {
    const profileId = draft.profileId.trim()
    if (!profileId) {
      return null
    }

    return loadedProfilePath.value ?? `profiles/${profileId}.jsonc`
  })

  async function refreshProfiles(): Promise<void> {
    if (!usingTauri) {
      return
    }

    loading.list = true
    try {
      const payload = await listProfileDocuments()
      const nextProfiles = payload.map(normalizeProfileDocumentSummary).sort(compareProfileDocumentSummary)
      profiles.value = nextProfiles

      if (
        selectedProfileId.value &&
        !nextProfiles.some((profile) => profile.profileId === selectedProfileId.value)
      ) {
        selectedProfileId.value = null
      }
    } catch (error) {
      issue.value = `Unable to list profile documents: ${errorMessage(error)}`
    } finally {
      loading.list = false
    }
  }

  async function openProfileEditor(profileId: string): Promise<void> {
    if (!usingTauri) {
      return
    }

    issue.value = null
    loading.load = true
    try {
      const payload = await loadProfileDocument({ profileId })
      const document = normalizeProfileDocument(payload)
      const editable = parseEditableProfileDocument(document.contents)
      selectedProfileId.value = document.profileId
      loadedProfilePath.value = document.profilePath
      draft.profileId = document.profileId
      assignEditableDraft(editable)
      profileDialogOpen.value = true
    } catch (error) {
      issue.value = `Unable to load profile document: ${errorMessage(error)}`
    } finally {
      loading.load = false
    }
  }

  function startNewProfile(): void {
    if (!usingTauri) {
      issue.value =
        'Profile management requires the Tauri bridge. Start Moira through the desktop shell to create or edit local profile documents.'
      return
    }

    issue.value = null
    loadedProfilePath.value = null
    draft.profileId = ''
    assignEditableDraft(createEmptyEditableProfileDocument())
    profileDialogOpen.value = true
  }

  function closeProfileDialog(): void {
    if (loading.load || loading.save) {
      return
    }

    profileDialogOpen.value = false
  }

  function selectNoProfile(): void {
    selectedProfileId.value = null
  }

  function updateDraftField(field: 'profileId' | 'coreConfig', value: string): void {
    draft[field] = value
    if (field === 'profileId') {
      loadedProfilePath.value = null
    }
  }

  function addEnvFileRow(): void {
    draft.envFiles.push({
      id: nextDraftRowId('env-file'),
      path: '',
      required: true,
    })
  }

  function removeEnvFileRow(rowId: string): void {
    draft.envFiles = draft.envFiles.filter((entry) => entry.id !== rowId)
  }

  function updateEnvFileRow(field: 'path' | 'required', rowId: string, value: string | boolean): void {
    const entry = draft.envFiles.find((candidate) => candidate.id === rowId)
    if (!entry) {
      return
    }

    if (field === 'required' && typeof value === 'boolean') {
      entry.required = value
      return
    }
    if (field === 'path' && typeof value === 'string') {
      entry.path = value
    }
  }

  function addInlineEnvironmentRow(): void {
    draft.inlineEnvironment.push({
      id: nextDraftRowId('inline-env'),
      key: '',
      value: '',
    })
  }

  function removeInlineEnvironmentRow(rowId: string): void {
    draft.inlineEnvironment = draft.inlineEnvironment.filter((entry) => entry.id !== rowId)
  }

  function updateInlineEnvironmentRow(field: 'key' | 'value', rowId: string, value: string): void {
    const entry = draft.inlineEnvironment.find((candidate) => candidate.id === rowId)
    if (!entry) {
      return
    }

    entry[field] = value
  }

  async function saveCurrentProfile(): Promise<void> {
    if (!canSave.value) {
      return
    }

    issue.value = null
    loading.save = true
    try {
      const contents = serializeEditableProfileDocument({
        coreConfig: draft.coreConfig,
        envFiles: draft.envFiles.map(({ path, required }) => ({ path, required })),
        inlineEnvironment: draft.inlineEnvironment.map(({ key, value }) => ({ key, value })),
      })
      const payload = await saveProfileDocument({
        profileId: draft.profileId.trim(),
        contents,
      })
      const document = normalizeProfileDocument(payload)
      const editable = parseEditableProfileDocument(document.contents)
      selectedProfileId.value = document.profileId
      loadedProfilePath.value = document.profilePath
      draft.profileId = document.profileId
      assignEditableDraft(editable)
      await refreshProfiles()
      profileDialogOpen.value = false
    } catch (error) {
      issue.value = `Unable to save profile document: ${errorMessage(error)}`
    } finally {
      loading.save = false
    }
  }

  return {
    addEnvFileRow,
    addInlineEnvironmentRow,
    canSave,
    closeProfileDialog,
    draft,
    issue,
    loading,
    openProfileEditor,
    pathHint,
    profileDialogOpen,
    profiles,
    refreshProfiles,
    removeEnvFileRow,
    removeInlineEnvironmentRow,
    saveCurrentProfile,
    selectNoProfile,
    selectedProfileId,
    startNewProfile,
    updateEnvFileRow,
    updateDraftField,
    updateInlineEnvironmentRow,
    usingTauri,
  }

  function assignEditableDraft(editable: EditableProfileDocument): void {
    draft.coreConfig = editable.coreConfig
    draft.envFiles = editable.envFiles.map((entry) => ({
      id: nextDraftRowId('env-file'),
      ...entry,
    }))
    draft.inlineEnvironment = editable.inlineEnvironment.map((entry) => ({
      id: nextDraftRowId('inline-env'),
      ...entry,
    }))
  }
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

let draftRowCounter = 0

function nextDraftRowId(prefix: string): string {
  draftRowCounter += 1
  return `${prefix}-${draftRowCounter}`
}
