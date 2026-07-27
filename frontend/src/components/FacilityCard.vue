<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Facility } from '@/api'

const props = defineProps<{ facility: Facility }>()
const emit = defineEmits<{ change: [id: string, level: number] }>()
const selected = ref(props.facility.selected_level)
const levels = computed(() => Array.from({ length: props.facility.max_level + 1 }, (_, level) => ({ title: level === 0 ? '未选择' : `目标 Lv.${level}`, value: level })))

function update(level: number) { selected.value = level; emit('change', props.facility.id, level) }
</script>

<template>
  <v-card class="facility-card" border flat>
    <v-card-item>
      <template #prepend><v-avatar color="secondary" variant="tonal" icon="mdi-hammer-wrench" /></template>
      <v-card-title>{{ facility.name }}</v-card-title>
      <v-card-subtitle>最高等级 Lv.{{ facility.max_level }}</v-card-subtitle>
    </v-card-item>
    <v-card-text>
      <v-select :items="levels" label="升级目标" density="comfortable" hide-details :model-value="selected" @update:model-value="update" />
      <div v-if="facility.prerequisites.length" class="mt-4">
        <div v-for="prerequisite in facility.prerequisites" :key="`${prerequisite.facilityId}-${prerequisite.level}`" class="d-flex align-center text-caption mb-1">
          <v-icon :color="prerequisite.satisfied ? 'success' : 'error'" size="16" :icon="prerequisite.satisfied ? 'mdi-check-circle' : 'mdi-alert-circle'" class="mr-1" />
          {{ prerequisite.facilityName }}需达到 Lv.{{ prerequisite.level }}
        </div>
      </div>
      <div v-else class="text-caption text-medium-emphasis mt-4">无前置条件</div>
    </v-card-text>
  </v-card>
</template>
