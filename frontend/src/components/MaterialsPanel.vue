<script setup lang="ts">
import type { Material } from '@/api'
defineProps<{ materials: Material[] }>()
</script>

<template>
  <v-sheet class="materials-panel" color="surface" border>
    <div class="d-flex align-center justify-space-between flex-wrap ga-3 mb-4"><div><div class="text-overline text-secondary">MATERIALS</div><h2 class="text-h6">剩余材料</h2></div><span class="text-body-2">{{ materials.length }} 种</span></div>
    <v-table density="comfortable" fixed-header height="300"><thead><tr><th>物品</th><th>要求</th><th class="text-right">数量</th></tr></thead><tbody>
      <tr v-for="material in materials" :key="`${material.itemId}-${material.foundInRaid}`"><td>{{ material.name }}</td><td><v-chip v-if="material.foundInRaid" color="secondary" size="x-small">带勾</v-chip><span v-else class="text-medium-emphasis">非带勾</span></td><td class="text-right font-weight-bold">{{ material.quantity }}</td></tr>
      <tr v-if="!materials.length"><td colspan="3" class="text-center text-medium-emphasis py-6">所有设施均已达到满级，没有剩余材料</td></tr>
    </tbody></v-table>
  </v-sheet>
</template>
