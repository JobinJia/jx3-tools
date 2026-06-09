import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import PageHeader from '../PageHeader.vue'

describe('pageHeader', () => {
  it('renders title and description', () => {
    const wrapper = mount(PageHeader, {
      props: { title: '改键', description: '在账号与角色之间复制键位配置' },
    })
    expect(wrapper.text()).toContain('改键')
    expect(wrapper.text()).toContain('在账号与角色之间复制键位配置')
  })

  it('renders without description', () => {
    const wrapper = mount(PageHeader, { props: { title: '按键' } })
    expect(wrapper.text()).toContain('按键')
  })

  it('renders extra slot', () => {
    const wrapper = mount(PageHeader, {
      props: { title: 'T' },
      slots: { extra: '<span class="probe">EXTRA</span>' },
    })
    expect(wrapper.find('.probe').exists()).toBe(true)
  })
})
