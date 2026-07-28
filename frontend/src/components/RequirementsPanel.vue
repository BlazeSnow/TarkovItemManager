<script setup lang="ts">
import type { LevelEntry, SkillEntry } from '@/api'
const props = defineProps<{ merchants: LevelEntry[]; skills: SkillEntry[]; saving: boolean }>()
const emit = defineEmits<{ merchant: [id: number, level: number]; skill: [name: string, level: number] }>()
const levels = Array.from({ length: 5 }, (_, level) => ({ title: `Lv.${level}`, value: level }))
</script>

<template>
  <v-sheet class="requirements-panel" color="surface" border>
    <div class="text-overline text-secondary">PROGRESS</div><h2 class="text-h6 mb-4">商人与技能</h2>
    <div class="condition-grid">
      <v-select v-for="merchant in merchants" :key="merchant.id" :items="levels" :label="merchant.name" density="compact" hide-details :disabled="saving" :model-value="merchant.level" @update:model-value="emit('merchant', merchant.id, Number($event))" />
      <v-select v-for="skill in skills" :key="skill.name" :items="levels" :label="skill.name" density="compact" hide-details :disabled="saving" :model-value="skill.level" @update:model-value="emit('skill', skill.name, Number($event))" />
    </div>
  </v-sheet>
</template>
