import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import type { VueWrapper } from '@vue/test-utils'
import { createVuetify } from 'vuetify'
import type { Material } from '@/api'
import MaterialsPanel from './MaterialsPanel.vue'

const vuetify = createVuetify()

const materials: Material[] = [
  { itemId: 1, name: '螺栓', quantity: 63, foundInRaid: false },
  { itemId: 2, name: '电钻', quantity: 12, foundInRaid: true },
  { itemId: 3, name: '一套工具', quantity: 1200, foundInRaid: true },
]

function mountPanel(items: Material[]) {
  return mount(MaterialsPanel, { props: { materials: items }, global: { plugins: [vuetify] } })
}

function bodyRows(wrapper: VueWrapper) {
  return wrapper.find('tbody').findAll('tr')
}

async function clickFilter(wrapper: VueWrapper, label: string) {
  const button = wrapper.findAll('button').find(node => node.text() === label)
  if (!button) throw new Error(`filter button ${label} not found`)
  await button.trigger('click')
}

describe('MaterialsPanel', () => {
  it('renders all materials with formatted quantities', () => {
    const wrapper = mountPanel(materials)
    expect(bodyRows(wrapper)).toHaveLength(3)
    expect(wrapper.text()).toContain('1,200')
    expect(wrapper.text()).toContain('3 种')
  })

  it('filters found-in-raid materials', async () => {
    const wrapper = mountPanel(materials)
    await clickFilter(wrapper, '带勾')
    const rows = bodyRows(wrapper)
    expect(rows).toHaveLength(2)
    expect(rows[0].text()).toContain('电钻')
    expect(wrapper.text()).not.toContain('螺栓')
  })

  it('filters non-found-in-raid materials', async () => {
    const wrapper = mountPanel(materials)
    await clickFilter(wrapper, '非带勾')
    const rows = bodyRows(wrapper)
    expect(rows).toHaveLength(1)
    expect(rows[0].text()).toContain('螺栓')
  })

  it('shows the empty state when there are no materials', () => {
    const wrapper = mountPanel([])
    expect(wrapper.text()).toContain('所有设施均已达到满级，没有剩余材料')
  })
})
