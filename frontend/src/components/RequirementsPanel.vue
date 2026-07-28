<script setup lang="ts">
import type { LevelEntry, SkillEntry } from '@/api'
const props = defineProps<{ merchants: LevelEntry[]; skills: SkillEntry[]; saving: boolean }>()
const emit = defineEmits<{ merchant: [id: number, level: number]; skill: [name: string, level: number] }>()
const merchantLevels = Array.from({ length: 5 }, (_, level) => ({ title: `Lv.${level}`, value: level }))
const skillLevels = (maxLevel: number) => Array.from({ length: maxLevel + 1 }, (_, level) => ({ title: `Lv.${level}`, value: level }))
</script>

<template>
  <section class="requirements-grid">
    <v-sheet class="requirements-panel" color="surface" border>
      <div class="text-overline text-secondary">TRADERS</div><h2 class="text-h6 mb-4">商人</h2>
      <div class="condition-grid">
        <v-select v-for="merchant in merchants" :key="merchant.id" :items="merchantLevels" :label="merchant.name" density="compact" hide-details :disabled="saving" :model-value="merchant.level" @update:model-value="emit('merchant', merchant.id, Number($event))" />
      </div>
    </v-sheet>
    <v-sheet class="requirements-panel" color="surface" border>
      <div class="text-overline text-secondary">SKILLS</div><h2 class="text-h6 mb-4">技能</h2>
      <div class="condition-grid">
        <v-select v-for="skill in skills" :key="skill.name" :items="skillLevels(skill.maxLevel)" :label="skill.name" density="compact" hide-details :disabled="saving" :model-value="skill.level" @update:model-value="emit('skill', skill.name, Number($event))" />
      </div>
    </v-sheet>
  </section>
</template>
