<script setup lang="ts">
import { computed, ref } from 'vue'
import type { Material } from '@/api'
import { formatQuantity } from '@/format'

const props = defineProps<{ materials: Material[] }>()
type MaterialFilter = 'all' | 'fir' | 'non-fir'

const filter = ref<MaterialFilter>('all')
const filterOptions = [
  { title: '全部', value: 'all' },
  { title: '带勾', value: 'fir' },
  { title: '非带勾', value: 'non-fir' },
]
const filteredMaterials = computed(() => {
  if (filter.value === 'fir') return props.materials.filter(material => material.foundInRaid)
  if (filter.value === 'non-fir') return props.materials.filter(material => !material.foundInRaid)
  return props.materials
})
</script>

<template>
  <v-sheet class="materials-panel" color="surface" border>
    <div class="d-flex align-center justify-space-between flex-wrap ga-3 mb-4"><div><div class="text-overline text-secondary">MATERIALS</div><h2 class="text-h6">剩余材料</h2></div><div class="d-flex align-center ga-3"><span class="text-body-2">{{ filteredMaterials.length }} 种</span><v-btn-toggle v-model="filter" color="primary" density="compact" mandatory><v-btn v-for="option in filterOptions" :key="option.value" :value="option.value">{{ option.title }}</v-btn></v-btn-toggle></div></div>
    <v-table density="comfortable" fixed-header height="420"><thead><tr><th>物品</th><th>要求</th><th class="text-right">数量</th></tr></thead><tbody>
      <tr v-for="material in filteredMaterials" :key="`${material.itemId}-${material.foundInRaid}`"><td>{{ material.name }}</td><td><v-chip v-if="material.foundInRaid" color="secondary" size="x-small">带勾</v-chip><span v-else class="text-medium-emphasis">非带勾</span></td><td class="text-right font-weight-bold">{{ formatQuantity(material.quantity) }}</td></tr>
      <tr v-if="!materials.length"><td colspan="3" class="text-center text-medium-emphasis py-6">所有设施均已达到满级，没有剩余材料</td></tr>
      <tr v-else-if="!filteredMaterials.length"><td colspan="3" class="text-center text-medium-emphasis py-6">当前筛选条件下没有材料</td></tr>
    </tbody></v-table>
  </v-sheet>
</template>
