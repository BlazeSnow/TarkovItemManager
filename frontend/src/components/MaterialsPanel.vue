<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Material } from '@/api'

const props = defineProps<{ materials: Material[]; saving: boolean }>()
const emit = defineEmits<{ toggle: [id: string, checked: boolean] }>()
const filter = ref<'all' | 'open' | 'checked'>('all')
const visible = computed(() => props.materials.filter((item) => filter.value === 'all' || (filter.value === 'checked' ? item.checked : !item.checked)))
</script>

<template>
  <v-sheet class="materials-panel" color="surface" border>
    <div class="d-flex align-center justify-space-between flex-wrap ga-3 mb-4">
      <div><div class="text-overline text-secondary">MATERIALS</div><h2 class="text-h6">所需材料</h2></div>
      <v-btn-toggle v-model="filter" color="primary" density="compact" mandatory>
        <v-btn value="all">全部 {{ materials.length }}</v-btn>
        <v-btn value="open">未勾选</v-btn>
        <v-btn value="checked">已勾选</v-btn>
      </v-btn-toggle>
    </div>
    <v-table density="comfortable" fixed-header height="250">
      <thead><tr><th>状态</th><th>物品</th><th class="text-right">数量</th></tr></thead>
      <tbody>
        <tr v-for="material in visible" :key="material.id">
          <td><v-checkbox-btn :model-value="material.checked" :aria-label="`勾选${material.name}`" :disabled="saving" @update:model-value="emit('toggle', material.id, Boolean($event))" /></td>
          <td :class="{ 'text-decoration-line-through text-medium-emphasis': material.checked }">{{ material.name }}</td>
          <td class="text-right font-weight-bold">{{ material.quantity }}</td>
        </tr>
        <tr v-if="!visible.length"><td colspan="3" class="text-center text-medium-emphasis py-6">所有设施均已达到满级，没有剩余材料</td></tr>
      </tbody>
    </v-table>
  </v-sheet>
</template>
