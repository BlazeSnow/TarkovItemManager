<script setup lang="ts">
import { computed } from 'vue'
import type { Facility } from '@/api'

const props = defineProps<{ facility: Facility; saving: boolean }>()
const emit = defineEmits<{ change: [id: number, level: number] }>()
const levels = computed(() => Array.from({ length: props.facility.maxLevel + 1 }, (_, level) => ({ title: `当前 Lv.${level}`, value: level })))
function formatTime(seconds: number) { if (!seconds) return '即时'; const hours = seconds / 3600; return hours >= 24 ? `${hours / 24} 天` : `${hours} 小时` }
</script>

<template>
  <v-card class="facility-card" border flat>
    <v-card-item>
      <template #prepend><v-avatar color="secondary" variant="tonal" icon="mdi-hammer-wrench" /></template>
      <v-card-title>{{ facility.name }}</v-card-title>
      <v-card-subtitle>最高等级 Lv.{{ facility.maxLevel }}</v-card-subtitle>
    </v-card-item>
    <v-card-text>
      <v-select :items="levels" label="当前等级" density="comfortable" hide-details :disabled="saving" :model-value="facility.currentLevel" @update:model-value="emit('change', facility.id, Number($event))" />
      <v-divider class="my-4" />
      <div v-for="upgrade in facility.upgrades" :key="upgrade.level" class="upgrade-plan mb-4">
        <div class="d-flex justify-space-between align-center mb-2"><strong>升级至 Lv.{{ upgrade.level }}</strong><span class="text-caption"><v-icon icon="mdi-clock-outline" size="15" class="mr-1" />{{ formatTime(upgrade.constructionTimeSeconds) }}</span></div>
        <div v-if="upgrade.requirements.length" class="text-caption mb-2">{{ upgrade.requirements.map(item => `${item.name} x${item.quantity}${item.foundInRaid ? ' [带勾]' : ''}`).join(' · ') }}</div>
        <div v-else class="text-caption text-medium-emphasis mb-2">无需材料</div>
        <div v-for="gate in upgrade.facilityPrerequisites" :key="`f-${gate.facilityId}-${gate.level}`" class="gate-row text-caption"><v-icon :color="gate.satisfied ? 'success' : 'error'" :icon="gate.satisfied ? 'mdi-check-circle' : 'mdi-alert-circle'" size="16" />{{ gate.name }}需达到 Lv.{{ gate.level }}</div>
        <div v-for="gate in upgrade.merchantPrerequisites" :key="`m-${gate.merchantId}-${gate.level}`" class="gate-row text-caption"><v-icon :color="gate.satisfied ? 'success' : 'error'" :icon="gate.satisfied ? 'mdi-check-circle' : 'mdi-alert-circle'" size="16" />{{ gate.name }}信誉需达到 Lv.{{ gate.level }}</div>
        <div v-for="gate in upgrade.skillPrerequisites" :key="`s-${gate.name}-${gate.level}`" class="gate-row text-caption"><v-icon :color="gate.satisfied ? 'success' : 'error'" :icon="gate.satisfied ? 'mdi-check-circle' : 'mdi-alert-circle'" size="16" />{{ gate.name }}需达到 Lv.{{ gate.level }}</div>
      </div>
      <div v-if="!facility.upgrades.length" class="text-caption text-success">已达到满级</div>
    </v-card-text>
  </v-card>
</template>
