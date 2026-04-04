<script setup lang="ts">
import { computed } from 'vue'

import AtroposRuntimePanel from '@/presentation/atropos/runtime/AtroposRuntimePanel.vue'
import ClothoWorkshopPanel from '@/presentation/clotho/workshop/ClothoWorkshopPanel.vue'
import LachesisWorkspacePanel from '@/presentation/lachesis/workspace/LachesisWorkspacePanel.vue'
import LoomFeatureTabs from '@/presentation/loom/chrome/LoomFeatureTabs.vue'
import StatusHeader from '@/presentation/loom/chrome/StatusHeader.vue'
import { useAtroposRuntime } from '@/query/atropos/runtime'
import { useClothoBuildControl } from '@/query/clotho/builds'
import { useClothoProfileControl } from '@/query/clotho/profiles'
import { useLachesisWorkspace } from '@/query/lachesis/workspace'
import { useLoomNavigation } from '@/query/loom/navigation'

const { activeFeature, selectFeature } = useLoomNavigation()
const {
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
  issue: buildIssue,
  launchTargets,
  loading: buildLoading,
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
  selectedTargetKey,
  selectedTargetLabel,
  selectedTargetRef,
  updateForgeDraftField,
  updateRegisterDraftField,
} = useClothoBuildControl()
const {
  canSave,
  closeProfileDialog,
  draft: profileDraft,
  issue: profileIssue,
  loading: profileLoading,
  openProfileEditor,
  pathHint: profilePathHint,
  profileDialogOpen,
  profiles,
  refreshProfiles,
  saveCurrentProfile,
  selectNoProfile,
  selectedProfileId,
  startNewProfile,
  updateDraftField: updateProfileField,
} = useClothoProfileControl()
const {
  activeTab,
  issue,
  loading,
  refreshVisibleState,
  selectTick,
  selectWake,
  selectedRunId,
  selectedTick,
  selectedTickDetail,
  status,
  tickTimeline,
  wakeSessions,
} = useLachesisWorkspace()
const {
  cancelForceKillConfirmation,
  canForceKill,
  canStop,
  canWake,
  confirmForceKill,
  forceKillConfirmOpen,
  issue: runtimeIssue,
  loading: runtimeLoading,
  refreshRuntimeStatus,
  requestForceKillConfirmation,
  runtime,
  stopRuntime,
  wakeSelectedTarget: wakeRuntime,
} = useAtroposRuntime(selectedTargetRef)

const wakeCount = computed(() => wakeSessions.value.length)

async function wakeSelectedTarget(): Promise<void> {
  await wakeRuntime(selectedProfileId.value)
}
</script>

<template>
  <div class="app-shell">
    <div class="bg-orb orb-a"></div>
    <div class="bg-orb orb-b"></div>

    <StatusHeader
      :status="status"
      :loading="loading.status"
      :issue="issue"
      @refresh="refreshVisibleState"
    />

    <LoomFeatureTabs
      :active-tab="activeFeature"
      :runtime-phase="runtime?.phase ?? null"
      :selected-target-label="selectedTargetLabel"
      :selected-profile-id="selectedProfileId"
      :wake-count="wakeCount"
      @select="selectFeature"
    />

    <main class="feature-stack">
      <LachesisWorkspacePanel
        v-show="activeFeature === 'lachesis'"
        :active-tab="activeTab"
        :loading="{
          detail: loading.detail,
          ticks: loading.ticks,
          wakes: loading.wakes,
        }"
        :selected-run-id="selectedRunId"
        :selected-tick="selectedTick"
        :selected-tick-detail="selectedTickDetail"
        :tick-timeline="tickTimeline"
        :wake-sessions="wakeSessions"
        @select-tick="selectTick"
        @select-wake="selectWake"
        @update:tab="activeTab = $event"
      />

      <AtroposRuntimePanel
        v-show="activeFeature === 'atropos'"
        :can-force-kill="canForceKill"
        :can-stop="canStop"
        :can-wake="canWake"
        :issue="runtimeIssue"
        :loading="runtimeLoading"
        :runtime="runtime"
        :selected-target-label="selectedTargetLabel"
        :selected-profile-id="selectedProfileId"
        :show-force-kill-confirm="forceKillConfirmOpen"
        @cancel-force-kill="cancelForceKillConfirmation"
        @confirm-force-kill="confirmForceKill"
        @refresh="refreshRuntimeStatus"
        @request-force-kill="requestForceKillConfirmation"
        @stop="stopRuntime"
        @wake="wakeSelectedTarget"
      />

      <ClothoWorkshopPanel
        v-show="activeFeature === 'clotho'"
        :build-issue="buildIssue"
        :build-loading="buildLoading"
        :can-forge="canForge"
        :can-install="canInstall"
        :can-register="canRegister"
        :can-save-profile="canSave"
        :forge-dialog-open="forgeDialogOpen"
        :forge-draft="forgeDraft"
        :install-dialog-open="installDialogOpen"
        :launch-targets="launchTargets"
        :profile-dialog-open="profileDialogOpen"
        :profile-draft="profileDraft"
        :profile-issue="profileIssue"
        :profile-loading="profileLoading"
        :profile-path-hint="profilePathHint"
        :profiles="profiles"
        :published-releases="publishedReleases"
        :register-dialog-open="registerDialogOpen"
        :register-draft="registerDraft"
        :selected-profile-id="selectedProfileId"
        :selected-release-key="selectedReleaseKey"
        :selected-target-key="selectedTargetKey"
        @close-forge-dialog="closeForgeDialog"
        @close-install-dialog="closeInstallDialog"
        @close-profile-dialog="closeProfileDialog"
        @close-register-dialog="closeRegisterDialog"
        @forge="forgeBuild"
        @install-release="installRelease"
        @open-forge-dialog="openForgeDialog"
        @open-install-dialog="openInstallDialog"
        @open-profile="openProfileEditor"
        @open-register-dialog="openRegisterDialog"
        @refresh-profiles="refreshProfiles"
        @refresh-published-releases="refreshPublishedReleases"
        @refresh-targets="refreshLaunchTargets"
        @register="registerBuild"
        @save-profile="saveCurrentProfile"
        @select-no-profile="selectNoProfile"
        @select-published-release="selectPublishedRelease"
        @select-target="selectTarget"
        @start-new-profile="startNewProfile"
        @update-forge-field="updateForgeDraftField"
        @update-register-field="updateRegisterDraftField"
        @update-profile-field="updateProfileField"
      />
    </main>
  </div>
</template>

<style scoped>
.feature-stack {
  display: grid;
  gap: 1rem;
}
</style>
