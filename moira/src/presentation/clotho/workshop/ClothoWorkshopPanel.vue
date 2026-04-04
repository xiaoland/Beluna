<script setup lang="ts">
import ForgeBuildDialog from '@/presentation/clotho/dialogs/ForgeBuildDialog.vue'
import BuildRegistrationDialog from '@/presentation/clotho/dialogs/BuildRegistrationDialog.vue'
import InstallReleaseDialog from '@/presentation/clotho/dialogs/InstallReleaseDialog.vue'
import ProfileDocumentDialog from '@/presentation/clotho/dialogs/ProfileDocumentDialog.vue'
import ClothoLaunchTargetLibraryCard from './ClothoLaunchTargetLibraryCard.vue'
import ClothoProfileLibraryCard from './ClothoProfileLibraryCard.vue'

const props = defineProps<{
  buildIssue: string | null
  buildLoading: {
    forge: boolean
    install: boolean
    listReleases: boolean
    listTargets: boolean
    register: boolean
  }
  canSaveProfile: boolean
  canForge: boolean
  canInstall: boolean
  canRegister: boolean
  forgeDialogOpen: boolean
  forgeDraft: {
    buildId: string
    sourceDir: string
  }
  installDialogOpen: boolean
  launchTargets: Array<{
    checksumVerified: boolean
    executablePath: string | null
    installDir: string | null
    issue: string | null
    key: string
    label: string
    provenance: 'registered' | 'forged' | 'installed'
    readiness: 'ready' | 'stale'
    releaseTag: string | null
    rustTargetTriple: string | null
    sourceDir: string | null
    target:
      | { kind: 'knownLocalBuild'; buildId: string }
      | { kind: 'installedArtifact'; releaseTag: string; rustTargetTriple: string }
    workingDir: string | null
  }>
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
  publishedReleases: Array<{
    alreadyInstalled: boolean
    archiveAssetName: string
    checksumAssetName: string
    displayName: string
    key: string
    prerelease: boolean
    publishedAt: string | null
    releaseTag: string
    rustTargetTriple: string
  }>
  registerDialogOpen: boolean
  registerDraft: {
    buildId: string
    executablePath: string
    workingDir: string
    sourceDir: string
  }
  selectedProfileId: string | null
  selectedReleaseKey: string | null
  selectedTargetKey: string | null
}>()

const emit = defineEmits<{
  closeForgeDialog: []
  closeInstallDialog: []
  closeProfileDialog: []
  closeRegisterDialog: []
  forge: []
  installRelease: []
  openForgeDialog: []
  openInstallDialog: []
  openProfile: [profileId: string]
  openRegisterDialog: []
  refreshProfiles: []
  refreshPublishedReleases: []
  refreshTargets: []
  register: []
  saveProfile: []
  selectNoProfile: []
  selectPublishedRelease: [releaseKey: string]
  selectTarget: [target: { kind: 'knownLocalBuild'; buildId: string } | { kind: 'installedArtifact'; releaseTag: string; rustTargetTriple: string }]
  startNewProfile: []
  updateForgeField: [field: 'buildId' | 'sourceDir', value: string]
  updateRegisterField: [field: 'buildId' | 'executablePath' | 'workingDir' | 'sourceDir', value: string]
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
          Keep launch-target preparation and profile curation together: targets on the left, reusable profile documents on
          the right, and dense operations inside dialogs rather than a permanently expanded control slab.
        </p>
      </div>
    </div>

    <div class="workshop-grid">
      <ClothoLaunchTargetLibraryCard
        :issue="buildIssue"
        :launch-targets="launchTargets"
        :loading="buildLoading"
        :selected-target-key="selectedTargetKey"
        @open-forge="emit('openForgeDialog')"
        @open-install="emit('openInstallDialog')"
        @open-register="emit('openRegisterDialog')"
        @refresh-targets="emit('refreshTargets')"
        @select-target="emit('selectTarget', $event)"
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
      :draft="registerDraft"
      :issue="buildIssue"
      :loading="{ register: buildLoading.register }"
      @close="emit('closeRegisterDialog')"
      @register="emit('register')"
      @update-field="(field, value) => emit('updateRegisterField', field, value)"
    />

    <ForgeBuildDialog
      :open="forgeDialogOpen"
      :can-forge="canForge"
      :draft="forgeDraft"
      :issue="buildIssue"
      :loading="{ forge: buildLoading.forge }"
      @close="emit('closeForgeDialog')"
      @forge="emit('forge')"
      @update-field="(field, value) => emit('updateForgeField', field, value)"
    />

    <InstallReleaseDialog
      :open="installDialogOpen"
      :can-install="canInstall"
      :issue="buildIssue"
      :loading="{ install: buildLoading.install, list: buildLoading.listReleases }"
      :releases="publishedReleases"
      :selected-release-key="selectedReleaseKey"
      @close="emit('closeInstallDialog')"
      @install="emit('installRelease')"
      @refresh="emit('refreshPublishedReleases')"
      @select-release="emit('selectPublishedRelease', $event)"
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
  grid-template-columns: minmax(0, 1.02fr) minmax(0, 1.12fr);
  gap: 1rem;
  padding: 0 1rem 1rem;
}

@media (max-width: 1080px) {
  .workshop-grid {
    grid-template-columns: 1fr;
  }
}
</style>
