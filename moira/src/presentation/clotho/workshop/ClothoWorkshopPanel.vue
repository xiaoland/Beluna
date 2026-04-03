<script setup lang="ts">
import BuildRegistrationDialog from '@/presentation/clotho/dialogs/BuildRegistrationDialog.vue'
import ProfileDocumentDialog from '@/presentation/clotho/dialogs/ProfileDocumentDialog.vue'
import ClothoBuildLibraryCard from './ClothoBuildLibraryCard.vue'
import ClothoProfileLibraryCard from './ClothoProfileLibraryCard.vue'

const props = defineProps<{
  buildDraft: {
    buildId: string
    executablePath: string
    workingDir: string
    sourceDir: string
  }
  buildIssue: string | null
  buildLoading: {
    register: boolean
  }
  canRegister: boolean
  canSaveProfile: boolean
  profileDialogOpen: boolean
  profileDraft: {
    profileId: string
    contents: string
  }
  profileIssue: string | null
  profileLoading: {
    list: boolean
    load: boolean
    save: boolean
  }
  profilePathHint: string | null
  profiles: Array<{
    profileId: string
    profilePath: string
  }>
  registerDialogOpen: boolean
  selectedBuild: {
    buildId: string
    executablePath: string
    workingDir: string
    sourceDir: string | null
  } | null
  selectedProfileId: string | null
}>()

const emit = defineEmits<{
  closeProfileDialog: []
  closeRegisterDialog: []
  openProfile: [profileId: string]
  openRegisterDialog: []
  refreshProfiles: []
  register: []
  saveProfile: []
  selectNoProfile: []
  startNewProfile: []
  updateBuildField: [field: 'buildId' | 'executablePath' | 'workingDir' | 'sourceDir', value: string]
  updateProfileField: [field: 'profileId' | 'contents', value: string]
}>()
</script>

<template>
  <section class="panel-shell workshop-panel">
    <div class="panel-head workshop-head">
      <div>
        <p class="panel-kicker">Clotho</p>
        <h2 class="panel-title">Preparation Workshop</h2>
        <p class="panel-subtitle">
          Keep build registration and profile curation together: build selection on the left, reusable profile documents
          on the right, editing in dialogs instead of a permanently expanded form.
        </p>
      </div>
    </div>

    <div class="workshop-grid">
      <ClothoBuildLibraryCard
        :issue="buildIssue"
        :loading="buildLoading"
        :selected-build="selectedBuild"
        @open-register="emit('openRegisterDialog')"
      />

      <ClothoProfileLibraryCard
        :issue="profileIssue"
        :loading="{ list: profileLoading.list, load: profileLoading.load }"
        :profiles="profiles"
        :selected-profile-id="selectedProfileId"
        @open-profile="emit('openProfile', $event)"
        @refresh="emit('refreshProfiles')"
        @select-no-profile="emit('selectNoProfile')"
        @start-new="emit('startNewProfile')"
      />
    </div>

    <BuildRegistrationDialog
      :open="registerDialogOpen"
      :can-register="canRegister"
      :draft="buildDraft"
      :issue="buildIssue"
      :loading="buildLoading"
      @close="emit('closeRegisterDialog')"
      @register="emit('register')"
      @update-field="(field, value) => emit('updateBuildField', field, value)"
    />

    <ProfileDocumentDialog
      :open="profileDialogOpen"
      :can-save="canSaveProfile"
      :draft="profileDraft"
      :issue="profileIssue"
      :loading="{ load: profileLoading.load, save: profileLoading.save }"
      :path-hint="profilePathHint"
      @close="emit('closeProfileDialog')"
      @save="emit('saveProfile')"
      @update-field="(field, value) => emit('updateProfileField', field, value)"
    />
  </section>
</template>

<style scoped>
.workshop-panel {
  position: relative;
  z-index: 1;
  overflow: hidden;
}

.workshop-head {
  padding-bottom: 0.95rem;
}

.workshop-grid {
  display: grid;
  grid-template-columns: minmax(0, 0.95fr) minmax(0, 1.2fr);
  gap: 1rem;
  padding: 0 1rem 1rem;
}

@media (max-width: 1080px) {
  .workshop-grid {
    grid-template-columns: 1fr;
  }
}
</style>
