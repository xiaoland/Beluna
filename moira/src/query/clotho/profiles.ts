import { computed, onMounted, reactive, ref } from 'vue'

import { hasTauriBridge } from '@/bridge/env'
import { listProfileDocuments, loadProfileDocument, saveProfileDocument } from '@/bridge/clotho'
import {
  compareProfileDocumentSummary,
  normalizeProfileDocument,
  normalizeProfileDocumentSummary,
  type ProfileDocumentSummary,
} from '@/projection/clotho'

const DEFAULT_PROFILE_TEMPLATE = `{
  // Add Core config fields here.
}
`

export function useClothoProfileControl() {
  const usingTauri = hasTauriBridge()
  const issue = ref<string | null>(null)
  const profileDialogOpen = ref(false)
  const profiles = ref<ProfileDocumentSummary[]>([])
  const selectedProfileId = ref<string | null>(null)
  const loadedProfilePath = ref<string | null>(null)

  const draft = reactive({
    profileId: '',
    contents: DEFAULT_PROFILE_TEMPLATE,
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
      draft.contents.trim().length > 0,
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
      selectedProfileId.value = document.profileId
      loadedProfilePath.value = document.profilePath
      draft.profileId = document.profileId
      draft.contents = document.contents
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
    draft.contents = DEFAULT_PROFILE_TEMPLATE
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

  function updateDraftField(field: 'profileId' | 'contents', value: string): void {
    draft[field] = value
    if (field === 'profileId') {
      loadedProfilePath.value = null
    }
  }

  async function saveCurrentProfile(): Promise<void> {
    if (!canSave.value) {
      return
    }

    issue.value = null
    loading.save = true
    try {
      const payload = await saveProfileDocument({
        profileId: draft.profileId.trim(),
        contents: draft.contents,
      })
      const document = normalizeProfileDocument(payload)
      selectedProfileId.value = document.profileId
      loadedProfilePath.value = document.profilePath
      draft.profileId = document.profileId
      draft.contents = document.contents
      await refreshProfiles()
      profileDialogOpen.value = false
    } catch (error) {
      issue.value = `Unable to save profile document: ${errorMessage(error)}`
    } finally {
      loading.save = false
    }
  }

  return {
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
    saveCurrentProfile,
    selectNoProfile,
    selectedProfileId,
    startNewProfile,
    updateDraftField,
    usingTauri,
  }
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}
