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
  canRegister,
  closeRegisterDialog,
  draft: buildDraft,
  issue: buildIssue,
  loading: buildLoading,
  openRegisterDialog,
  registerBuild,
  registerDialogOpen,
  selectedBuild,
  selectedBuildId,
  updateDraftField: updateBuildField,
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
  wakeSelectedBuild: wakeRuntime,
} = useAtroposRuntime(selectedBuildId)

const wakeCount = computed(() => wakeSessions.value.length)

async function wakeSelectedBuild(): Promise<void> {
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
      :selected-build-id="selectedBuildId"
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
        :selected-build-id="selectedBuildId"
        :selected-profile-id="selectedProfileId"
        :show-force-kill-confirm="forceKillConfirmOpen"
        @cancel-force-kill="cancelForceKillConfirmation"
        @confirm-force-kill="confirmForceKill"
        @refresh="refreshRuntimeStatus"
        @request-force-kill="requestForceKillConfirmation"
        @stop="stopRuntime"
        @wake="wakeSelectedBuild"
      />

      <ClothoWorkshopPanel
        v-show="activeFeature === 'clotho'"
        :build-draft="buildDraft"
        :build-issue="buildIssue"
        :build-loading="buildLoading"
        :can-register="canRegister"
        :can-save-profile="canSave"
        :profile-dialog-open="profileDialogOpen"
        :profile-draft="profileDraft"
        :profile-issue="profileIssue"
        :profile-loading="profileLoading"
        :profile-path-hint="profilePathHint"
        :profiles="profiles"
        :register-dialog-open="registerDialogOpen"
        :selected-build="selectedBuild"
        :selected-profile-id="selectedProfileId"
        @close-profile-dialog="closeProfileDialog"
        @close-register-dialog="closeRegisterDialog"
        @open-profile="openProfileEditor"
        @open-register-dialog="openRegisterDialog"
        @refresh-profiles="refreshProfiles"
        @register="registerBuild"
        @save-profile="saveCurrentProfile"
        @select-no-profile="selectNoProfile"
        @start-new-profile="startNewProfile"
        @update-build-field="updateBuildField"
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
