<script setup lang="ts">
import { computed } from 'vue'

import { stateTone } from '@/presentation/format'
import type { RuntimeStatus } from '@/projection/atropos'
import type { ProfileDocumentSummary } from '@/projection/clotho'
import AtroposRuntimeCard from './AtroposRuntimeCard.vue'
import ClothoRegistrationCard from './ClothoRegistrationCard.vue'
import ForceKillDialog from './ForceKillDialog.vue'
import ProfileLibraryCard from './ProfileLibraryCard.vue'

const props = defineProps<{
  buildDraft: {
    buildId: string
    executablePath: string
    workingDir: string
    sourceDir: string
  }
  canForceKill: boolean
  canRegister: boolean
  canSaveProfile: boolean
  canStop: boolean
  canWake: boolean
  issue: string | null
  loading: {
    forceKill: boolean
    profileList: boolean
    profileLoad: boolean
    profileSave: boolean
    register: boolean
    runtime: boolean
    wake: boolean
    stop: boolean
  }
  profileDraft: {
    profileId: string
    contents: string
  }
  profileIssue: string | null
  profilePathHint: string | null
  profiles: ProfileDocumentSummary[]
  runtime: RuntimeStatus | null
  selectedBuildId: string | null
  selectedProfileId: string | null
  showForceKillConfirm: boolean
}>()

const emit = defineEmits<{
  cancelForceKill: []
  confirmForceKill: []
  openProfile: [profileId: string]
  refreshProfiles: []
  refresh: []
  register: []
  saveProfile: []
  selectNoProfile: []
  startNewProfile: []
  stop: []
  requestForceKill: []
  updateBuildField: [field: 'buildId' | 'executablePath' | 'workingDir' | 'sourceDir', value: string]
  updateProfileField: [field: 'profileId' | 'contents', value: string]
  wake: []
}>()

const runtimeTone = computed(() => stateTone(props.runtime?.phase ?? null))
</script>

<template>
  <section class="panel-shell control-panel">
    <div class="panel-head control-head">
      <div>
        <p class="panel-kicker">Wake Control</p>
        <h2 class="panel-title">Clotho + Atropos</h2>
        <p class="panel-subtitle">
          Register a known local Core build, manage JSONC profile documents, then wake or stop the supervised process
          without leaking raw path logic into the operator surface.
        </p>
      </div>

      <div class="head-actions">
        <span class="status-badge" :class="`status-${runtimeTone}`">
          {{ runtime?.phase ?? (loading.runtime ? 'loading' : 'idle') }}
        </span>
        <button class="button-secondary" type="button" :disabled="loading.runtime" @click="emit('refresh')">
          Refresh Runtime
        </button>
      </div>
    </div>

    <div class="control-grid">
      <ClothoRegistrationCard
        :can-register="canRegister"
        :draft="buildDraft"
        :loading="{ register: loading.register }"
        :selected-build-id="selectedBuildId"
        @register="emit('register')"
        @update-field="(field, value) => emit('updateBuildField', field, value)"
      />

      <ProfileLibraryCard
        :can-save="canSaveProfile"
        :draft="profileDraft"
        :issue="profileIssue"
        :loading="{ list: loading.profileList, load: loading.profileLoad, save: loading.profileSave }"
        :path-hint="profilePathHint"
        :profiles="profiles"
        :selected-profile-id="selectedProfileId"
        @open-profile="(profileId) => emit('openProfile', profileId)"
        @refresh="emit('refreshProfiles')"
        @save="emit('saveProfile')"
        @select-no-profile="emit('selectNoProfile')"
        @start-new="emit('startNewProfile')"
        @update-field="(field, value) => emit('updateProfileField', field, value)"
      />

      <AtroposRuntimeCard
        :can-force-kill="canForceKill"
        :can-stop="canStop"
        :can-wake="canWake"
        :loading="{ forceKill: loading.forceKill, stop: loading.stop, wake: loading.wake }"
        :runtime="runtime"
        :selected-build-id="selectedBuildId"
        @request-force-kill="emit('requestForceKill')"
        @stop="emit('stop')"
        @wake="emit('wake')"
      />
    </div>

    <p v-if="issue" class="issue-banner">{{ issue }}</p>

    <ForceKillDialog
      :loading="loading.forceKill"
      :open="showForceKillConfirm"
      :runtime="runtime"
      @cancel="emit('cancelForceKill')"
      @confirm="emit('confirmForceKill')"
    />
  </section>
</template>

<style scoped>
.control-panel {
  position: relative;
  z-index: 1;
  margin-bottom: 1rem;
}

.control-head {
  padding-bottom: 0.95rem;
}

.head-actions {
  display: flex;
  align-items: center;
  gap: 0.6rem;
}

.control-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1.15fr) minmax(0, 0.95fr);
  gap: 1rem;
  padding: 0 1rem 1rem;
}

.issue-banner {
  margin: 0 1rem 1rem;
  padding: 0.72rem 0.85rem;
  border: 1px solid rgba(162, 77, 68, 0.2);
  background: rgba(162, 77, 68, 0.08);
  color: var(--err);
}

@media (max-width: 1080px) {
  .control-grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 780px) {
  .head-actions {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
