<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { api, type Catalog } from '@/api'
import FacilityCard from '@/components/FacilityCard.vue'
import MaterialsPanel from '@/components/MaterialsPanel.vue'

const data = ref<Catalog | null>(null)
const loading = ref(true)
const savingFacilities = ref(false)
const savingMaterials = ref(false)
const error = ref('')

async function load() {
  loading.value = true; error.value = ''
  try { data.value = await api.catalog() } catch (reason) { error.value = reason instanceof Error ? reason.message : '无法加载数据' }
  finally { loading.value = false }
}
async function changeFacility(id: string, level: number) {
  if (!data.value) return
  const facility = data.value.facilities.find((item) => item.id === id)
  if (!facility) return
  facility.selectedLevel = level; savingFacilities.value = true
  try {
    await api.saveFacilities(data.value.facilities.map((item) => ({ facilityId: item.id, level: item.selectedLevel })))
    data.value = await api.catalog()
  } catch (reason) { error.value = reason instanceof Error ? reason.message : '保存失败'; await load() }
  finally { savingFacilities.value = false }
}
async function toggleMaterial(id: string, checked: boolean) {
  if (!data.value) return
  const material = data.value.materials.find((item) => item.id === id)
  if (!material) return
  material.checked = checked; savingMaterials.value = true
  try { await api.saveMaterials(data.value.materials.filter((item) => item.checked).map((item) => item.id)) }
  catch (reason) { error.value = reason instanceof Error ? reason.message : '保存失败'; await load() }
  finally { savingMaterials.value = false }
}
onMounted(load)
</script>

<template>
  <v-container class="workspace py-6">
    <div class="d-flex align-end justify-space-between flex-wrap ga-3 mb-6">
      <div><div class="text-overline text-secondary">HIDEOUT</div><h1 class="text-h4">升级规划</h1></div>
      <v-btn icon="mdi-refresh" variant="text" aria-label="刷新数据" :loading="loading" @click="load" />
    </div>
    <v-alert v-if="error" class="mb-5" closable density="comfortable" type="error" @click:close="error = ''">{{ error }}</v-alert>
    <v-progress-linear v-if="loading" color="primary" indeterminate class="mb-5" />
    <template v-else-if="data">
      <section class="facility-grid mb-6">
        <FacilityCard v-for="facility in data.facilities" :key="facility.id" :facility="facility" @change="changeFacility" />
      </section>
      <MaterialsPanel :materials="data.materials" :saving="savingMaterials || savingFacilities" @toggle="toggleMaterial" />
    </template>
    <v-sheet v-else class="pa-8 text-center" border><v-icon icon="mdi-database-alert-outline" size="40" class="mb-3" /><p>无法加载设施数据。</p><v-btn color="primary" @click="load">重试</v-btn></v-sheet>
  </v-container>
</template>
