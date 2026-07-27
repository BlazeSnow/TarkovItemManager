<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { api, type Catalog } from '@/api'
import FacilityCard from '@/components/FacilityCard.vue'
import MaterialsPanel from '@/components/MaterialsPanel.vue'
import RequirementsPanel from '@/components/RequirementsPanel.vue'

const data = ref<Catalog | null>(null)
const loading = ref(true)
const saving = ref(false)
const error = ref('')
async function load() { loading.value = true; error.value = ''; try { data.value = await api.catalog() } catch (reason) { error.value = reason instanceof Error ? reason.message : '无法加载数据' } finally { loading.value = false } }
async function saveFacilities(id: number, level: number) { if (!data.value) return; const facility = data.value.facilities.find(item => item.id === id); if (!facility) return; facility.currentLevel = level; saving.value = true; try { await api.saveFacilityLevels(data.value.facilities.map(item => ({ facilityId: item.id, level: item.currentLevel }))); await load() } catch (reason) { error.value = reason instanceof Error ? reason.message : '保存失败'; await load() } finally { saving.value = false } }
async function saveMerchant(id: number, level: number) { if (!data.value) return; const merchant = data.value.merchants.find(item => item.id === id); if (!merchant) return; merchant.level = level; saving.value = true; try { await api.saveMerchantLevels(data.value.merchants.map(item => ({ merchantId: item.id, level: item.level }))); await load() } catch (reason) { error.value = reason instanceof Error ? reason.message : '保存失败'; await load() } finally { saving.value = false } }
async function saveSkill(name: string, level: number) { if (!data.value) return; const skill = data.value.skills.find(item => item.name === name); if (!skill) return; skill.level = level; saving.value = true; try { await api.saveSkillLevels(data.value.skills.map(item => ({ name: item.name, level: item.level }))); await load() } catch (reason) { error.value = reason instanceof Error ? reason.message : '保存失败'; await load() } finally { saving.value = false } }
onMounted(load)
</script>

<template>
  <v-container class="workspace py-6">
    <div class="d-flex align-end justify-space-between flex-wrap ga-3 mb-6"><div><div class="text-overline text-secondary">{{ data?.gameMode ?? 'PVE' }} HIDEOUT</div><h1 class="text-h4">升级规划</h1></div><v-btn icon="mdi-refresh" variant="text" aria-label="刷新数据" :loading="loading" @click="load" /></div>
    <v-alert v-if="error" class="mb-5" closable density="comfortable" type="error" @click:close="error = ''">{{ error }}</v-alert>
    <v-progress-linear v-if="loading" color="primary" indeterminate class="mb-5" />
    <template v-else-if="data"><RequirementsPanel class="mb-6" :merchants="data.merchants" :skills="data.skills" :saving="saving" @merchant="saveMerchant" @skill="saveSkill" /><section class="facility-grid mb-6"><FacilityCard v-for="facility in data.facilities" :key="facility.id" :facility="facility" :saving="saving" @change="saveFacilities" /></section><MaterialsPanel :materials="data.materials" /></template>
    <v-sheet v-else class="pa-8 text-center" border><v-icon icon="mdi-database-alert-outline" size="40" class="mb-3" /><p>无法加载设施数据。</p><v-btn color="primary" @click="load">重试</v-btn></v-sheet>
  </v-container>
</template>
