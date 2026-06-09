# JX3-Tools UI 重设计 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 按已批准的设计文档 `docs/superpowers/specs/2026-06-10-ui-redesign-design.md`，把三个页面与主布局统一为「新中式」视觉语言（宣纸/夜宣纸双主题、印章字导航、墨线页头），不改任何业务逻辑。

**Architecture:** 设计 token 用 CSS 变量定义（`:root` / `.dark`），同一份色值映射进 naive-ui `themeOverrides`；新增 `useTheme`（三态主题）与 `PageHeader`（统一页头）两个可复用单元；各页面只改 template/style，composables/stores/services 接口不动；唯一功能新增是 `useRecentOps`（localStorage 最近操作记录）。

**Tech Stack:** Vue 3 `<script setup>` + naive-ui + UnoCSS + VueUse（`useStorage`）。无新依赖。

**前置说明（执行者必读）:**

- 仓库根目录是 `/Users/jiabinbin/myself/tauri/jx3-tools`，所有命令在根目录执行。
- 当前工作区有一批**未提交的依赖升级改动**（package.json / Cargo.toml / window.rs 等）。本计划的提交只 `git add` 各任务明确列出的文件，**不要** `git add -A`。
- 组件自动导入：`n-*`（naive-ui）和 `m-*`（shuimo-ui，全局注册）组件在模板里直接用，无需 import；`unplugin-icons` 图标用 `~icons/...` import。
- 验证命令：`pnpm type-check`（无输出即通过）、`pnpm lint`（自动修复，无 error 即通过）、`pnpm exec vitest run <文件>`（单测）。
- `pnpm tauri:dev` 可能已在后台运行（Vite 端口 5400，热更新生效），目检时直接看应用窗口即可。
- 提交信息格式：Conventional Commits 英文，结尾带 `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>`。

---

### Task 1: 主题 token 与通用样式（theme.css）

**Files:**
- Create: `src/assets/theme.css`
- Modify: `src/assets/main.css`

- [ ] **Step 1: 创建 `src/assets/theme.css`**

```css
/* 新中式主题 token —— 浅色「宣纸」 / 深色「夜宣纸」 */
:root {
  --paper-bg: #f4eedd;
  --paper-card: #fffdf6;
  --paper-card-soft: #f7f2e4;
  --line: #ddd3bb;
  --line-soft: #efe8d4;
  --ink: #2c2a26;
  --ink-secondary: #6f675a;
  --ink-muted: #a09a8c;
  --cinnabar: #9c2f23;
  --cinnabar-hover: #b03a2e;
  --cinnabar-pressed: #7e251c;
  --bamboo: #5a7247;
  --ochre: #b07936;
  --indigo: #44617b;
  --ink-error: #8f2727;
  --cinnabar-tint: rgba(156, 47, 35, 0.08);
  --bamboo-tint: rgba(90, 114, 71, 0.1);
  --ochre-tint: rgba(176, 121, 54, 0.1);
  --indigo-tint: rgba(68, 97, 123, 0.08);
  --sider-bg: #2c2a26;
  --sider-text: #9a9183;
  --sider-active-text: #f5efe2;
  --sider-line: rgba(245, 239, 226, 0.18);
}

.dark {
  --paper-bg: #211f1b;
  --paper-card: #2b2823;
  --paper-card-soft: #26231f;
  --line: #3d382f;
  --line-soft: #353027;
  --ink: #e8e0cd;
  --ink-secondary: #b3a994;
  --ink-muted: #7d7665;
  --cinnabar: #c24a3a;
  --cinnabar-hover: #d05c4b;
  --cinnabar-pressed: #a63c2e;
  --bamboo: #7e9a68;
  --ochre: #c99850;
  --indigo: #7395b3;
  --ink-error: #c0564a;
  --cinnabar-tint: rgba(194, 74, 58, 0.12);
  --bamboo-tint: rgba(126, 154, 104, 0.15);
  --ochre-tint: rgba(201, 152, 80, 0.12);
  --indigo-tint: rgba(115, 149, 179, 0.12);
  --sider-bg: #1a1816;
}

/* 页面底：宣纸色 + 细噪点纹理（替代古塔背景图） */
.paper-bg {
  background-color: var(--paper-bg);
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='180' height='180'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2' stitchTiles='stitch'/%3E%3CfeColorMatrix type='saturate' values='0'/%3E%3CfeComponentTransfer%3E%3CfeFuncA type='linear' slope='0.04'/%3E%3C/feComponentTransfer%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E");
}

/* 卡片 */
.paper-card {
  background: var(--paper-card);
  border: 1px solid var(--line);
  border-radius: 8px;
  box-shadow: 0 1px 2px rgba(44, 42, 38, 0.04);
}

/* 宋体页面标题 */
.page-title {
  font-family: 'Songti SC', 'STSong', 'SimSun', serif;
  font-size: 20px;
  letter-spacing: 3px;
  color: var(--ink);
}

/* 朱砂小方印 */
.seal-dot {
  display: inline-block;
  width: 8px;
  height: 8px;
  background: var(--cinnabar);
  border-radius: 1px;
}

/* 浓→淡渐变墨线（页头分隔） */
.ink-line {
  height: 1px;
  background: linear-gradient(90deg, var(--ink) 0%, transparent 100%);
  opacity: 0.55;
}

/* 等宽（MAC 地址、热键值） */
.text-mono {
  font-family: ui-monospace, Menlo, Consolas, monospace;
}

/* 键帽输入框（热键录入） */
.keycap-input {
  font-family: ui-monospace, Menlo, Consolas, monospace;
  font-size: 12px;
  background: var(--paper-card-soft);
  border: 1px solid var(--line);
  border-bottom-width: 2px;
  border-radius: 5px;
  padding: 4px 14px;
  color: var(--ink);
  width: 150px;
  text-align: center;
  outline: none;
  cursor: pointer;
}
.keycap-input:focus {
  border-color: var(--cinnabar);
}
.keycap-input::placeholder {
  color: var(--ink-muted);
}
```

- [ ] **Step 2: 在 `src/assets/main.css` 引入**

把 `src/assets/main.css` 改为：

```css
@import './base.css';
@import './theme.css';
```

- [ ] **Step 3: 验证**

Run: `pnpm type-check && pnpm lint`
Expected: 两个命令都无 error 退出。

- [ ] **Step 4: Commit**

```bash
git add src/assets/theme.css src/assets/main.css
git commit -m "feat(ui): add ink-paper theme tokens and base styles

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 2: useTheme 三态主题 composable（TDD）

**Files:**
- Create: `src/composables/useTheme.ts`
- Test: `src/composables/__tests__/useTheme.spec.ts`

- [ ] **Step 1: 写失败的测试**

创建 `src/composables/__tests__/useTheme.spec.ts`：

```ts
import { beforeEach, describe, expect, it } from 'vitest'
import { nextTick } from 'vue'
import { useTheme } from '../useTheme'

describe('useTheme', () => {
  beforeEach(() => {
    localStorage.clear()
    document.documentElement.classList.remove('dark')
  })

  it('cycles mode system -> light -> dark -> system', () => {
    const { mode, cycleMode } = useTheme()
    mode.value = 'system'
    cycleMode()
    expect(mode.value).toBe('light')
    cycleMode()
    expect(mode.value).toBe('dark')
    cycleMode()
    expect(mode.value).toBe('system')
  })

  it('toggles .dark class on documentElement', async () => {
    const { mode } = useTheme()
    mode.value = 'dark'
    await nextTick()
    expect(document.documentElement.classList.contains('dark')).toBe(true)
    mode.value = 'light'
    await nextTick()
    expect(document.documentElement.classList.contains('dark')).toBe(false)
  })

  it('exposes naive dark theme only when dark', async () => {
    const { mode, naiveTheme, isDark } = useTheme()
    mode.value = 'light'
    await nextTick()
    expect(isDark.value).toBe(false)
    expect(naiveTheme.value).toBeNull()
    mode.value = 'dark'
    await nextTick()
    expect(isDark.value).toBe(true)
    expect(naiveTheme.value).not.toBeNull()
  })
})
```

- [ ] **Step 2: 运行测试确认失败**

Run: `pnpm exec vitest run src/composables/__tests__/useTheme.spec.ts`
Expected: FAIL，报错 `Cannot find module '../useTheme'`（或类似找不到模块）。

- [ ] **Step 3: 实现 `src/composables/useTheme.ts`**

```ts
import type { GlobalTheme } from 'naive-ui'
import { useStorage } from '@vueuse/core'
import { darkTheme } from 'naive-ui'
import { computed, ref, watchEffect } from 'vue'

export type ThemeMode = 'system' | 'light' | 'dark'

// 模块级单例状态（与 useKeyboard 的共享模式一致）
const mode = useStorage<ThemeMode>('jx3-theme-mode', 'system')

// 系统主题：jsdom 没有 matchMedia，需判空（不要用 naive 的 useOsTheme，它依赖组件生命周期）
const prefersDark = typeof window.matchMedia === 'function'
  ? window.matchMedia('(prefers-color-scheme: dark)')
  : null
const osDark = ref(prefersDark?.matches ?? false)
prefersDark?.addEventListener('change', (e) => {
  osDark.value = e.matches
})

const isDark = computed(() =>
  mode.value === 'system' ? osDark.value : mode.value === 'dark',
)

// 根元素 .dark class 同步（模块加载时注册一次）
watchEffect(() => {
  document.documentElement.classList.toggle('dark', isDark.value)
})

export function useTheme() {
  const naiveTheme = computed<GlobalTheme | null>(() => (isDark.value ? darkTheme : null))

  function cycleMode() {
    mode.value = mode.value === 'system' ? 'light' : mode.value === 'light' ? 'dark' : 'system'
  }

  return { mode, isDark, naiveTheme, cycleMode }
}
```

- [ ] **Step 4: 运行测试确认通过**

Run: `pnpm exec vitest run src/composables/__tests__/useTheme.spec.ts`
Expected: 3 passed。

- [ ] **Step 5: 全量校验并提交**

Run: `pnpm type-check && pnpm lint`
Expected: 无 error。

```bash
git add src/composables/useTheme.ts src/composables/__tests__/useTheme.spec.ts
git commit -m "feat(ui): add three-state theme composable

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 3: naive-ui 主题映射 + App.vue 接线

**Files:**
- Create: `src/theme/naive.ts`
- Modify: `src/App.vue`

- [ ] **Step 1: 创建 `src/theme/naive.ts`**

```ts
import type { GlobalThemeOverrides } from 'naive-ui'

/** 与 src/assets/theme.css 的 CSS 变量保持同一份色值 */
export const lightOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: '#9c2f23',
    primaryColorHover: '#b03a2e',
    primaryColorPressed: '#7e251c',
    primaryColorSuppl: '#b03a2e',
    successColor: '#5a7247',
    warningColor: '#b07936',
    errorColor: '#8f2727',
    infoColor: '#44617b',
    textColorBase: '#2c2a26',
    textColor1: '#2c2a26',
    textColor2: '#4a443a',
    textColor3: '#a09a8c',
    bodyColor: '#f4eedd',
    cardColor: '#fffdf6',
    popoverColor: '#fffdf6',
    modalColor: '#fffdf6',
    inputColor: '#f7f2e4',
    borderColor: '#ddd3bb',
    dividerColor: '#efe8d4',
    borderRadius: '6px',
  },
}

export const darkOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: '#c24a3a',
    primaryColorHover: '#d05c4b',
    primaryColorPressed: '#a63c2e',
    primaryColorSuppl: '#d05c4b',
    successColor: '#7e9a68',
    warningColor: '#c99850',
    errorColor: '#c0564a',
    infoColor: '#7395b3',
    textColorBase: '#e8e0cd',
    textColor1: '#e8e0cd',
    textColor2: '#cfc5ad',
    textColor3: '#7d7665',
    bodyColor: '#211f1b',
    cardColor: '#2b2823',
    popoverColor: '#2b2823',
    modalColor: '#2b2823',
    inputColor: '#26231f',
    borderColor: '#3d382f',
    dividerColor: '#353027',
    borderRadius: '6px',
  },
}
```

- [ ] **Step 2: 重写 `src/App.vue`**（移除 m-rice-paper 包裹与旧的 useOsTheme）

```vue
<script setup lang="ts">
import { NConfigProvider, NMessageProvider } from 'naive-ui'
import { computed } from 'vue'
import { useTheme } from '@/composables/useTheme'
import { darkOverrides, lightOverrides } from '@/theme/naive'

const { naiveTheme, isDark } = useTheme()
const overrides = computed(() => (isDark.value ? darkOverrides : lightOverrides))
</script>

<template>
  <NConfigProvider :theme="naiveTheme" :theme-overrides="overrides" class="h-screen w-full">
    <NMessageProvider>
      <router-view />
    </NMessageProvider>
  </NConfigProvider>
</template>
```

- [ ] **Step 3: 验证 + 目检**

Run: `pnpm type-check && pnpm lint && pnpm exec vitest run`
Expected: 全部通过。应用窗口热更新后无白屏（此时整体观感是「半成品」状态，正常——MainLayout 还没改）。

- [ ] **Step 4: Commit**

```bash
git add src/theme/naive.ts src/App.vue
git commit -m "feat(ui): wire naive-ui theme overrides into app shell

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 4: PageHeader 通用页头组件（TDD）

**Files:**
- Create: `src/components/layout/PageHeader.vue`
- Test: `src/components/layout/__tests__/PageHeader.spec.ts`

- [ ] **Step 1: 写失败的测试**

创建 `src/components/layout/__tests__/PageHeader.spec.ts`：

```ts
import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'
import PageHeader from '../PageHeader.vue'

describe('PageHeader', () => {
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
```

- [ ] **Step 2: 运行测试确认失败**

Run: `pnpm exec vitest run src/components/layout/__tests__/PageHeader.spec.ts`
Expected: FAIL，找不到 `../PageHeader.vue`。

- [ ] **Step 3: 实现 `src/components/layout/PageHeader.vue`**

```vue
<script setup lang="ts">
defineProps<{
  title: string
  description?: string
}>()
</script>

<template>
  <div class="mb-3.5">
    <div class="flex items-end justify-between">
      <div>
        <div class="page-title">
          {{ title }} <span class="seal-dot align-middle ml-0.5" />
        </div>
        <div v-if="description" class="text-xs mt-1" style="color: var(--ink-muted)">
          {{ description }}
        </div>
      </div>
      <div class="flex items-center">
        <slot name="extra" />
      </div>
    </div>
    <div class="ink-line mt-2.5" />
  </div>
</template>
```

- [ ] **Step 4: 运行测试确认通过**

Run: `pnpm exec vitest run src/components/layout/__tests__/PageHeader.spec.ts`
Expected: 3 passed。

- [ ] **Step 5: 校验并提交**

Run: `pnpm type-check && pnpm lint`
Expected: 无 error。

```bash
git add src/components/layout/PageHeader.vue src/components/layout/__tests__/PageHeader.spec.ts
git commit -m "feat(ui): add PageHeader component

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 5: 主布局印章侧栏 + 路由 meta

**Files:**
- Modify: `src/router/index.ts`
- Modify: `src/components/layout/MainLayout.vue`

- [ ] **Step 1: 重写 `src/router/index.ts`**（emoji 图标 → sealChar；增加 RouteMeta 类型扩充）

```ts
import { createRouter, createWebHistory } from 'vue-router'
import MainLayout from '@/components/layout/MainLayout.vue'
import HotkeyView from '@/views/hotkey/HotkeyView.vue'
import KeyboardView from '@/views/keyboard/KeyboardView.vue'
import MacIdView from '@/views/mac-id/MacId.vue'

declare module 'vue-router' {
  interface RouteMeta {
    title?: string
    sealChar?: string
  }
}

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      component: MainLayout,
      redirect: '/keyboard',
      children: [
        {
          path: 'keyboard',
          name: 'Keyboard',
          component: KeyboardView,
          meta: { title: '改键', sealChar: '改' },
        },
        {
          path: 'mac-id',
          name: 'MacId',
          component: MacIdView,
          meta: { title: 'MAC地址', sealChar: '网' },
        },
        {
          path: 'hotkey',
          name: 'Hotkey',
          component: HotkeyView,
          meta: { title: '按键', sealChar: '按' },
        },
      ],
    },
  ],
})

export default router
```

- [ ] **Step 2: 重写 `src/components/layout/MainLayout.vue`**（纯 div + flex；移除 n-layout/n-menu/MRicePaper/古塔背景）

```vue
<script setup lang="ts">
import type { RouteRecordRaw } from 'vue-router'
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useTheme } from '@/composables/useTheme'
import pkg from '../../../package.json'

const router = useRouter()
const route = useRoute()
const { mode, cycleMode } = useTheme()

interface NavItem {
  name: string
  title: string
  sealChar: string
}

const navItems = computed<NavItem[]>(() => {
  const rootRoute = router.options.routes.find(r => r.path === '/') as RouteRecordRaw
  return (rootRoute?.children || []).map(r => ({
    name: String(r.name),
    title: r.meta?.title ?? '',
    sealChar: r.meta?.sealChar ?? '',
  }))
})

const themeIcon = computed(() => (mode.value === 'system' ? '◐' : mode.value === 'light' ? '☀' : '☾'))
const themeTitle = computed(() => (mode.value === 'system' ? '跟随系统' : mode.value === 'light' ? '浅色' : '深色'))
</script>

<template>
  <div class="flex h-screen w-full">
    <aside class="sider">
      <nav class="flex flex-col items-center gap-3">
        <n-tooltip v-for="item in navItems" :key="item.name" placement="right">
          <template #trigger>
            <button
              class="seal-nav"
              :class="{ active: route.name === item.name }"
              @click="router.push({ name: item.name })"
            >
              {{ item.sealChar }}
            </button>
          </template>
          {{ item.title }}
        </n-tooltip>
      </nav>
      <div class="mt-auto flex flex-col items-center gap-2">
        <n-tooltip placement="right">
          <template #trigger>
            <button class="theme-toggle" @click="cycleMode">
              {{ themeIcon }}
            </button>
          </template>
          主题：{{ themeTitle }}
        </n-tooltip>
        <span class="version">v{{ pkg.version }}</span>
      </div>
    </aside>
    <main class="flex-1 min-w-0 overflow-y-auto paper-bg">
      <router-view />
    </main>
  </div>
</template>

<style scoped>
.sider {
  width: 58px;
  flex-shrink: 0;
  background: var(--sider-bg);
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 16px 0 12px;
}

.seal-nav {
  width: 36px;
  height: 36px;
  border-radius: 5px;
  cursor: pointer;
  font-family: 'Songti SC', 'STSong', 'SimSun', serif;
  font-size: 17px;
  color: var(--sider-text);
  background: transparent;
  border: 1px solid var(--sider-line);
  transition: all 0.2s;
}

.seal-nav:hover {
  color: var(--sider-active-text);
  border-color: rgba(245, 239, 226, 0.4);
}

.seal-nav.active {
  background: var(--cinnabar);
  color: var(--sider-active-text);
  border-color: transparent;
  box-shadow: inset 0 0 0 1px rgba(245, 239, 226, 0.25);
}

.theme-toggle {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  border: 1px solid var(--sider-line);
  background: transparent;
  color: var(--sider-text);
  cursor: pointer;
  font-size: 13px;
}

.theme-toggle:hover {
  color: var(--sider-active-text);
}

.version {
  font-size: 9px;
  color: #5c564b;
}
</style>
```

- [ ] **Step 3: 验证 + 目检**

Run: `pnpm type-check && pnpm lint && pnpm exec vitest run`
Expected: 全部通过。目检应用窗口：左侧 58px 墨色侧栏，三个印章字（改/网/按），当前页朱砂底；底部 ◐ 按钮点击循环切换主题（宣纸 ↔ 夜宣纸），内容区有宣纸底色和噪点纹理；古塔图和 emoji 不再出现。

- [ ] **Step 4: Commit**

```bash
git add src/router/index.ts src/components/layout/MainLayout.vue
git commit -m "feat(ui): redesign sidebar with seal-character nav and theme toggle

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 6: MAC地址页重构

**Files:**
- Modify: `src/views/mac-id/MacId.vue`（整文件替换）

- [ ] **Step 1: 重写 `src/views/mac-id/MacId.vue`**

脚本逻辑不变（useMac 的调用与字段完全一致），仅新增 `helpExpanded`；模板按「地址主卡 + 设置卡 + 折叠帮助」重排；删除组件内自定义的 .flex/.gap-2 等工具类 CSS（用 UnoCSS）。

```vue
<script setup lang="ts">
import { onMounted, ref } from 'vue'
import PageHeader from '@/components/layout/PageHeader.vue'
import { useMac } from '@/composables/useMac'

const {
  originalAddress,
  currentAddress,
  autoRestoreEnabled,
  loading,
  changing,
  restoring,
  error,
  isChanged,
  fetchMacAddress,
  changeMacAddress,
  restoreMacAddress,
  setAutoRestore,
} = useMac()

const helpExpanded = ref(false)

onMounted(() => {
  fetchMacAddress()
})
</script>

<template>
  <div class="p-5 h-full">
    <PageHeader title="MAC地址" description="查看、随机修改与还原网卡物理地址" />

    <n-alert v-if="error" type="error" :title="error" class="max-w-[430px] mx-auto mb-3" />

    <n-spin :show="loading">
      <div class="max-w-[430px] mx-auto">
        <div class="paper-card p-5 text-center">
          <div class="text-[10px] tracking-[2px]" style="color: var(--ink-muted)">
            当前地址
          </div>
          <div class="text-mono text-[26px] tracking-[3px] mt-2" style="color: var(--ink)">
            {{ currentAddress || '——' }}
          </div>
          <div class="mt-2.5">
            <n-tag :type="isChanged ? 'warning' : 'success'" size="small">
              {{ isChanged ? '已修改' : '原始地址' }}
            </n-tag>
          </div>
          <div class="text-[10px] mt-3" style="color: var(--ink-muted)">
            原始地址 <span class="text-mono tracking-wider ml-1">{{ originalAddress || '——' }}</span>
          </div>
          <div class="flex gap-2.5 justify-center mt-4.5">
            <n-button type="primary" :loading="changing" @click="changeMacAddress">
              随机修改
            </n-button>
            <n-button :loading="restoring" @click="restoreMacAddress">
              还原地址
            </n-button>
          </div>
        </div>

        <div class="paper-card px-4 py-3 mt-3 flex items-center justify-between">
          <div>
            <div class="text-xs" style="color: var(--ink)">
              重启自动还原
            </div>
            <div class="text-[10px] mt-0.5" style="color: var(--ink-muted)">
              开机时自动恢复原始地址（计划任务）
            </div>
          </div>
          <n-switch v-model:value="autoRestoreEnabled" @update:value="setAutoRestore" />
        </div>

        <div class="text-center mt-3 text-[10px]" style="color: var(--ink-muted)">
          什么是 MAC 地址？为什么要修改？
          <a class="cursor-pointer" style="color: var(--indigo)" @click="helpExpanded = !helpExpanded">
            {{ helpExpanded ? '收起说明 ▴' : '展开说明 ▾' }}
          </a>
        </div>
        <div
          v-if="helpExpanded"
          class="paper-card p-4 mt-2 text-xs leading-relaxed"
          style="color: var(--ink-secondary)"
        >
          <p><b>什么是MAC地址？</b></p>
          <p>MAC地址是网络设备的唯一标识符，由48位二进制数字组成，通常表示为12个十六进制数字。</p>
          <p class="mt-2">
            <b>为什么要修改MAC地址？</b>
          </p>
          <p>修改MAC地址可用于增强隐私保护、绕过基于MAC地址的网络访问控制等。请确保您的操作符合相关法律法规。</p>
        </div>
      </div>
    </n-spin>
  </div>
</template>
```

（注意：旧文件底部的整个 `<style scoped>` 块删除，不再需要。）

- [ ] **Step 2: 验证 + 目检**

Run: `pnpm type-check && pnpm lint`
Expected: 无 error。目检 MAC 页：居中地址主卡（等宽大字 + 状态标签）、设置卡开关、底部可展开帮助；深色主题下同样正常。

- [ ] **Step 3: Commit**

```bash
git add src/views/mac-id/MacId.vue
git commit -m "feat(ui): redesign MAC address page

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 7: 按键页重构

**Files:**
- Modify: `src/views/hotkey/HotkeyView.vue`

- [ ] **Step 1: 修改 script 部分**

在现有 `<script setup>` 中做三处小改动（其余逻辑一行不动）：

1. import 区新增 PageHeader：

```ts
import PageHeader from '@/components/layout/PageHeader.vue'
```

2. 在 `const stopHotkeyFocused = ref(false)` 之后新增：

```ts
const helpExpanded = ref(false)
```

3. 把现有的 `statusType` computed 整体替换为：

```ts
const statusClass = computed(() => {
  if (status.value.running)
    return 'running'
  if (!status.value.registered)
    return 'warn'
  return 'idle'
})
```

- [ ] **Step 2: 重写 template 与 style**

整个 `<template>` 替换为（所有事件处理函数与原来同名，直接复用）：

```vue
<template>
  <div class="p-5 h-full">
    <PageHeader title="按键" description="全局热键控制的自动按键">
      <template #extra>
        <div class="status-badge" :class="statusClass">
          <span class="dot" />
          <span>{{ statusText }}</span>
          <span class="hint">{{ formValue.startHotkey || '—' }} 开始 · {{ formValue.stopHotkey || '—' }} 停止</span>
        </div>
      </template>
    </PageHeader>

    <n-alert v-if="status.lastError" type="error" title="错误" class="max-w-[480px] mx-auto mb-3">
      {{ status.lastError }}
    </n-alert>

    <n-spin :show="loading">
      <div v-if="config" class="max-w-[480px] mx-auto">
        <div class="paper-card px-4.5 py-4">
          <div class="section-label">按键行为</div>

          <div class="form-row">
            <span>按键模式</span>
            <div class="flex items-center gap-2">
              <n-radio-group
                :value="formValue.keyMode"
                size="small"
                :disabled="!isWindows"
                @update:value="(val: string) => formValue.keyMode = val as 'global' | 'window'"
              >
                <n-radio-button value="global">
                  全局
                </n-radio-button>
                <n-radio-button value="window">
                  窗口
                </n-radio-button>
              </n-radio-group>
              <n-popover trigger="hover" placement="top" style="max-width: 280px">
                <template #trigger>
                  <span class="info-icon">ⓘ</span>
                </template>
                <p>窗口模式：按键发送到指定窗口，窗口不在前台也能接收。</p>
                <p>部分游戏或应用可能屏蔽此方式的按键。</p>
                <p>目标窗口关闭后任务会自动停止。仅支持 Windows。</p>
              </n-popover>
              <n-text v-if="!isWindows" depth="3" class="text-xs">
                (仅 Windows)
              </n-text>
            </div>
          </div>

          <div v-if="isWindowMode && isWindows" class="form-row !items-start">
            <span class="pt-1">目标窗口</span>
            <div class="flex-1 ml-4">
              <n-input-group>
                <n-input
                  v-model:value="windowFilter"
                  size="small"
                  placeholder="关键词过滤…"
                  clearable
                  @update:value="fetchWindows"
                />
                <n-button size="small" :loading="windowLoading" @click="fetchWindows">
                  刷新
                </n-button>
              </n-input-group>
              <n-select
                class="mt-2"
                size="small"
                :value="formValue.targetWindow?.hwnd ?? null"
                :options="windowOptions"
                placeholder="选择目标窗口"
                filterable
                clearable
                :loading="windowLoading"
                @update:value="handleWindowSelect"
              />
            </div>
          </div>

          <div class="form-row">
            <span>触发按键</span>
            <input
              class="keycap-input"
              readonly
              :value="triggerKeyFocused ? '' : formValue.triggerKey"
              :placeholder="triggerKeyFocused ? '请按下按键…' : '点击录入'"
              @focus="triggerKeyFocused = true"
              @blur="triggerKeyFocused = false"
              @keydown="handleTriggerKeyDown"
            >
          </div>

          <div class="form-row">
            <span>触发频率</span>
            <n-input-number
              v-model:value="formValue.intervalMs"
              size="small"
              :min="20"
              :max="60000"
              :step="50"
            >
              <template #suffix>
                毫秒
              </template>
            </n-input-number>
          </div>

          <div class="card-divider" />
          <div class="section-label">
            控制热键 <span class="font-normal">（软件后台时也生效）</span>
          </div>

          <div class="form-row">
            <span>开始热键</span>
            <input
              class="keycap-input"
              readonly
              :value="startHotkeyFocused ? '' : formValue.startHotkey"
              :placeholder="startHotkeyFocused ? '请按下按键…' : '点击录入（支持组合键）'"
              @focus="startHotkeyFocused = true"
              @blur="startHotkeyFocused = false"
              @keydown="handleStartHotkeyKeyDown"
            >
          </div>

          <div class="form-row">
            <span>结束热键</span>
            <input
              class="keycap-input"
              readonly
              :value="stopHotkeyFocused ? '' : formValue.stopHotkey"
              :placeholder="stopHotkeyFocused ? '请按下按键…' : '点击录入（支持组合键）'"
              @focus="stopHotkeyFocused = true"
              @blur="stopHotkeyFocused = false"
              @keydown="handleStopHotkeyKeyDown"
            >
          </div>

          <div class="flex gap-2.5 mt-4 items-center">
            <n-button type="primary" :loading="saving" @click="saveConfig">
              保存配置
            </n-button>
            <n-button :disabled="!status.running" @click="hotkeyStore.stopTask">
              停止任务
            </n-button>
          </div>
        </div>

        <div class="text-center mt-3 text-[10px]" style="color: var(--ink-muted)">
          点击热键输入框后按下键盘即可录入 · macOS 需授权辅助功能
          <a class="cursor-pointer" style="color: var(--indigo)" @click="helpExpanded = !helpExpanded">
            {{ helpExpanded ? '收起 ▴' : '更多说明 ▾' }}
          </a>
        </div>
        <div
          v-if="helpExpanded"
          class="paper-card p-4 mt-2 text-xs leading-relaxed"
          style="color: var(--ink-secondary)"
        >
          <p>
            1. 保存后即可使用 <b>{{ formValue.startHotkey || '开始热键' }}</b> / <b>{{ formValue.stopHotkey || '结束热键' }}</b>
            控制任务（Windows 与 macOS 均支持）。
          </p>
          <p class="mt-1">
            2. 软件最小化或在后台时同样生效，请避免与系统或其他软件热键冲突；macOS 需在「系统设置 → 隐私与安全性 → 辅助功能」中允许应用控制键盘。
          </p>
          <p class="mt-1">
            3. 支持字母、数字、功能键、小键盘、方向键、修饰键、媒体键等。
          </p>
        </div>
      </div>
    </n-spin>
  </div>
</template>
```

整个 `<style scoped>` 替换为：

```vue
<style scoped>
.section-label {
  font-size: 11px;
  letter-spacing: 1px;
  color: var(--ink-muted);
  margin-bottom: 6px;
}

.form-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 0;
  font-size: 12px;
  color: var(--ink);
}

.card-divider {
  height: 1px;
  background: var(--line-soft);
  margin: 10px 0;
}

.info-icon {
  color: var(--ink-muted);
  font-size: 12px;
  cursor: help;
}

.status-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  border-radius: 6px;
  padding: 6px 12px;
  font-size: 11px;
  border: 1px solid var(--line);
  color: var(--ink-secondary);
  background: var(--paper-card);
}

.status-badge .dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--ink-muted);
}

.status-badge .hint {
  font-size: 10px;
  color: var(--ink-muted);
  border-left: 1px solid var(--line);
  padding-left: 8px;
}

.status-badge.running {
  color: var(--bamboo);
  border-color: var(--bamboo);
  background: var(--bamboo-tint);
}

.status-badge.running .dot {
  background: var(--bamboo);
  animation: breath 1.6s ease-in-out infinite;
}

.status-badge.warn {
  color: var(--ochre);
  border-color: var(--ochre);
  background: var(--ochre-tint);
}

.status-badge.warn .dot {
  background: var(--ochre);
}

@keyframes breath {
  0%,
  100% {
    opacity: 1;
  }

  50% {
    opacity: 0.35;
  }
}
</style>
```

- [ ] **Step 3: 验证 + 目检**

Run: `pnpm type-check && pnpm lint`
Expected: 无 error。目检按键页：页头右侧状态徽章（停止时灰色、保存配置后按 F11 应变竹青呼吸灯——macOS 上热键不可用则只验证样式）；键帽输入框点击后录入按键正常；「更多说明」可展开收起。

- [ ] **Step 4: Commit**

```bash
git add src/views/hotkey/HotkeyView.vue
git commit -m "feat(ui): redesign hotkey page

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 8: useRecentOps 最近操作记录（TDD）

**Files:**
- Create: `src/composables/useRecentOps.ts`
- Test: `src/composables/__tests__/useRecentOps.spec.ts`

- [ ] **Step 1: 写失败的测试**

创建 `src/composables/__tests__/useRecentOps.spec.ts`：

```ts
import { beforeEach, describe, expect, it } from 'vitest'
import { useRecentOps } from '../useRecentOps'

describe('useRecentOps', () => {
  beforeEach(() => {
    localStorage.clear()
    const { recentOps } = useRecentOps()
    recentOps.value = []
  })

  it('adds newest record first', () => {
    const { recentOps, addRecentOp } = useRecentOps()
    addRecentOp('角色甲', '角色乙')
    addRecentOp('角色丙', '角色丁')
    expect(recentOps.value).toHaveLength(2)
    expect(recentOps.value[0]!.source).toBe('角色丙')
    expect(recentOps.value[1]!.target).toBe('角色乙')
  })

  it('caps records at 10', () => {
    const { recentOps, addRecentOp } = useRecentOps()
    for (let i = 0; i < 13; i++)
      addRecentOp(`s${i}`, `t${i}`)
    expect(recentOps.value).toHaveLength(10)
    expect(recentOps.value[0]!.source).toBe('s12')
  })

  it('formats today as HH:mm and older days as M/D', () => {
    const { formatOpTime } = useRecentOps()
    const now = new Date()
    now.setHours(14, 2, 0, 0)
    expect(formatOpTime(now.getTime())).toBe('14:02')

    const old = new Date(now)
    old.setDate(old.getDate() - 3)
    expect(formatOpTime(old.getTime())).toBe(`${old.getMonth() + 1}/${old.getDate()}`)
  })
})
```

- [ ] **Step 2: 运行测试确认失败**

Run: `pnpm exec vitest run src/composables/__tests__/useRecentOps.spec.ts`
Expected: FAIL，找不到 `../useRecentOps`。

- [ ] **Step 3: 实现 `src/composables/useRecentOps.ts`**

```ts
import { useStorage } from '@vueuse/core'

export interface RecentOp {
  source: string
  target: string
  at: number
}

const MAX_RECENT_OPS = 10

// 模块级单例（与 useKeyboard 的共享模式一致）
const recentOps = useStorage<RecentOp[]>('keyboard-recent-ops', [])

export function useRecentOps() {
  function addRecentOp(source: string, target: string) {
    recentOps.value.unshift({ source, target, at: Date.now() })
    if (recentOps.value.length > MAX_RECENT_OPS)
      recentOps.value.length = MAX_RECENT_OPS
  }

  function formatOpTime(at: number): string {
    const d = new Date(at)
    const today = new Date()
    if (d.toDateString() === today.toDateString()) {
      const hh = String(d.getHours()).padStart(2, '0')
      const mm = String(d.getMinutes()).padStart(2, '0')
      return `${hh}:${mm}`
    }
    const yesterday = new Date(today)
    yesterday.setDate(today.getDate() - 1)
    if (d.toDateString() === yesterday.toDateString())
      return '昨天'
    return `${d.getMonth() + 1}/${d.getDate()}`
  }

  return { recentOps, addRecentOp, formatOpTime }
}
```

- [ ] **Step 4: 运行测试确认通过**

Run: `pnpm exec vitest run src/composables/__tests__/useRecentOps.spec.ts`
Expected: 3 passed。

- [ ] **Step 5: 校验并提交**

Run: `pnpm type-check && pnpm lint`
Expected: 无 error。

```bash
git add src/composables/useRecentOps.ts src/composables/__tests__/useRecentOps.spec.ts
git commit -m "feat(ui): add recent operations composable

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 9: 改键页布局 + 操作面板重构

**Files:**
- Modify: `src/views/keyboard/components/Result.vue`（整文件替换）
- Modify: `src/views/keyboard/KeyboardView.vue`（整文件替换）

- [ ] **Step 1: 重写 `src/views/keyboard/components/Result.vue`**

```vue
<script setup lang="ts">
import type { UserSelect } from '@/types'
import { ref } from 'vue'
import { useKeyboard } from '@/composables/useKeyboard'
import { useRecentOps } from '@/composables/useRecentOps'

const props = defineProps<{
  userSelect: UserSelect
}>()

const { copyKeyboardConfig, copyLoading } = useKeyboard()
const { recentOps, addRecentOp, formatOpTime } = useRecentOps()

const faqExpanded = ref(false)

async function handleCopy() {
  const ok = await copyKeyboardConfig(props.userSelect)
  if (ok)
    addRecentOp(props.userSelect.source, props.userSelect.target)
}
</script>

<template>
  <div class="flex flex-col gap-3 h-full min-w-[170px] flex-1">
    <div class="paper-card p-3.5">
      <div class="text-[11px] mb-2" style="color: var(--ink-muted)">
        本次操作
      </div>
      <div class="text-xs leading-7" style="color: var(--ink)">
        <div>
          <span class="mr-2" style="color: var(--cinnabar)">源</span>{{ userSelect.source || '未选择' }}
        </div>
        <div class="pl-3.5" style="color: var(--ink-muted)">
          ↓ 覆盖
        </div>
        <div>
          <span class="mr-2" style="color: var(--indigo)">目</span>{{ userSelect.target || '未选择' }}
        </div>
      </div>
      <button
        class="copy-btn"
        :disabled="!(userSelect.source && userSelect.target) || copyLoading"
        @click="handleCopy"
      >
        {{ copyLoading ? '复制中…' : '复 制 键 位' }}
      </button>
      <div class="text-[9px] text-center mt-1.5" style="color: var(--ink-muted)">
        目标角色现有键位将被覆盖，不可撤销
      </div>
    </div>

    <div class="paper-card p-3 flex-1 overflow-y-auto min-h-0">
      <div class="text-[11px] mb-1.5" style="color: var(--ink-muted)">
        最近操作
      </div>
      <div v-if="recentOps.length === 0" class="text-[10px]" style="color: var(--ink-muted)">
        暂无记录
      </div>
      <div
        v-for="(op, i) in recentOps"
        :key="i"
        class="text-[10px] leading-7 truncate"
        style="color: var(--ink-secondary)"
      >
        <span style="color: var(--bamboo)">✓</span> {{ op.source }} → {{ op.target }}
        <span class="float-right" style="color: var(--ink-muted)">{{ formatOpTime(op.at) }}</span>
      </div>
    </div>

    <div class="text-[10px]" style="color: var(--ink-muted)">
      <a class="cursor-pointer" style="color: var(--indigo)" @click="faqExpanded = !faqExpanded">
        常见问题 {{ faqExpanded ? '▴' : '▾' }}
      </a>
      <div v-if="faqExpanded" class="mt-1.5 leading-relaxed">
        <p><b>自己带键位的账号</b>需要在游戏里<b>关闭同步到服务器</b>，这样键位才能在本地得到保存。</p>
        <p class="mt-1"><b>新账号</b>登入到角色选择界面后选中角色（不进入游戏），点击「⟳ 刷新」即可搜索到该角色。</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.copy-btn {
  width: 100%;
  margin-top: 12px;
  padding: 8px 0;
  border: none;
  border-radius: 6px;
  background: var(--cinnabar);
  color: #f7f2e4;
  font-size: 13px;
  letter-spacing: 4px;
  font-family: 'Songti SC', 'STSong', 'SimSun', serif;
  cursor: pointer;
  transition: background 0.2s;
}

.copy-btn:hover:not(:disabled) {
  background: var(--cinnabar-hover);
}

.copy-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
</style>
```

- [ ] **Step 2: 重写 `src/views/keyboard/KeyboardView.vue`**

```vue
<script setup lang="ts">
import type { KeyboardTemplate, UserSelect } from '@/types'
import { ref } from 'vue'
import PageHeader from '@/components/layout/PageHeader.vue'
import { useKeyboard } from '@/composables/useKeyboard'
import Result from './components/Result.vue'
import SourceTree from './components/SourceTree.vue'
import TemplateList from './components/TemplateList.vue'

const { basePath, changeDirectory, templates, loadTree } = useKeyboard()

const sourceTab = ref<'all' | 'favorites'>('all')

const userSelect = ref<UserSelect>({
  source: '',
  sourcePath: '',
  target: '',
  targetPath: '',
})

function setSource(val: { name: string, path: string }) {
  userSelect.value.source = val.name
  userSelect.value.sourcePath = val.path
}

function setTarget(val: { name: string, path: string }) {
  userSelect.value.target = val.name
  userSelect.value.targetPath = val.path
}

function selectTemplate(template: KeyboardTemplate) {
  userSelect.value.source = template.name
  userSelect.value.sourcePath = template.sourcePath
}
</script>

<template>
  <div class="p-5 h-full flex flex-col">
    <!-- 空状态：未选择 userdata 目录 -->
    <div v-if="!basePath" class="flex-1 flex items-center justify-center">
      <div class="paper-card p-8 text-center max-w-[360px]">
        <div class="empty-seal">
          键
        </div>
        <div class="mt-3 text-sm" style="color: var(--ink)">
          初次使用需要选择游戏 userdata 目录
        </div>
        <div class="mt-1 text-xs" style="color: var(--ink-muted)">
          通常位于 …/SeasunGame/Game/JX3/bin/userdata
        </div>
        <n-button type="primary" class="mt-4" @click="changeDirectory">
          选择 userdata 目录
        </n-button>
      </div>
    </div>

    <template v-else>
      <PageHeader title="改键" description="在账号与角色之间复制键位配置">
        <template #extra>
          <div class="path-capsule">
            <span style="color: var(--ink-muted)">📁</span>
            <n-tooltip trigger="hover">
              <template #trigger>
                <span class="path-text">{{ basePath }}</span>
              </template>
              {{ basePath }}
            </n-tooltip>
            <a @click="changeDirectory">切换</a>
            <n-tooltip trigger="hover">
              <template #trigger>
                <a @click="loadTree">⟳</a>
              </template>
              刷新所有键位数据
            </n-tooltip>
          </div>
        </template>
      </PageHeader>

      <div class="flex-1 flex gap-3.5 min-h-0">
        <!-- 源卡片 -->
        <div class="paper-card w-[270px] flex flex-col overflow-hidden flex-shrink-0">
          <div class="card-head">
            <span class="text-xs font-600" style="color: var(--ink)">源角色</span>
            <n-radio-group v-model:value="sourceTab" size="small">
              <n-radio-button value="all">
                全部
              </n-radio-button>
              <n-radio-button value="favorites">
                常用 {{ templates.length }}
              </n-radio-button>
            </n-radio-group>
          </div>
          <div class="flex-1 overflow-hidden">
            <SourceTree v-show="sourceTab === 'all'" type="source" @source="setSource" />
            <TemplateList
              v-show="sourceTab === 'favorites'"
              :selected-path="userSelect.sourcePath"
              @select="selectTemplate"
            />
          </div>
        </div>

        <!-- 流向箭头 -->
        <div class="flex items-center justify-center w-6 flex-shrink-0 text-base" style="color: var(--cinnabar)">
          ➜
        </div>

        <!-- 目标卡片 -->
        <div class="paper-card w-[270px] flex flex-col overflow-hidden flex-shrink-0">
          <div class="card-head">
            <span class="text-xs font-600" style="color: var(--ink)">目标角色</span>
            <span class="text-[10px]" style="color: var(--ink-muted)">键位将被覆盖</span>
          </div>
          <div class="flex-1 overflow-hidden">
            <SourceTree type="target" placeholder="搜索没有键位的账号/角色" @source="setTarget" />
          </div>
        </div>

        <!-- 操作面板 -->
        <Result :user-select="userSelect" />
      </div>
    </template>
  </div>
</template>

<style scoped>
.card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 12px;
  border-bottom: 1px solid var(--line-soft);
}

.path-capsule {
  display: flex;
  align-items: center;
  gap: 6px;
  background: var(--paper-card);
  border: 1px solid var(--line);
  border-radius: 6px;
  padding: 5px 10px;
  font-size: 11px;
  color: var(--ink-secondary);
}

.path-capsule .path-text {
  max-width: 240px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  direction: rtl;
}

.path-capsule a {
  color: var(--cinnabar);
  cursor: pointer;
}

.empty-seal {
  width: 56px;
  height: 56px;
  margin: 0 auto;
  background: var(--cinnabar);
  border-radius: 8px;
  color: #f5efe2;
  font-size: 26px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: 'Songti SC', 'STSong', 'SimSun', serif;
}
</style>
```

- [ ] **Step 3: 验证 + 目检**

Run: `pnpm type-check && pnpm lint && pnpm exec vitest run`
Expected: 全部通过。目检改键页：路径胶囊在页头右侧；源/目标两张卡片 + 朱砂箭头；右侧「本次操作」卡选齐源目标后按钮可点击；复制成功后「最近操作」出现记录（macOS 上可用本地任意目录测试树加载）。SourceTree 此时仍是旧样式（Task 10 处理），布局塞在卡片里可能有溢出——属预期中间态。

- [ ] **Step 4: Commit**

```bash
git add src/views/keyboard/KeyboardView.vue src/views/keyboard/components/Result.vue
git commit -m "feat(ui): redesign keyboard page layout and operation panel

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 10: SourceTree / TemplateList 视觉适配

**Files:**
- Modify: `src/views/keyboard/components/SourceTree.vue`（只动 template 与 style，script 一行不动）
- Modify: `src/views/keyboard/components/TemplateList.vue`（只动 template 的 class 绑定与新增 style）

- [ ] **Step 1: 重写 SourceTree 的 template**

把 `<template>` 整体替换为（注意：根容器从 `h-screen w-[300px]` 改为 `h-full w-full`，树区域改 flex-1；NTree 的全部 props/事件保持原样）：

```vue
<template>
  <div class="h-full w-full flex flex-col p-2.5">
    <NInput v-model:value="pattern" size="small" :placeholder="placeholder" clearable />
    <div class="flex gap-1 mt-1.5">
      <n-button size="tiny" quaternary @click="expandAll">
        展开全部
      </n-button>
      <n-button size="tiny" quaternary @click="collapseAll">
        收起全部
      </n-button>
    </div>
    <div class="flex-1 overflow-y-auto mt-1.5 min-h-0">
      <NTree
        :key="treeKey"
        class="compact-tree"
        :indent="10"
        :pattern="pattern"
        show-line
        :override-default-node-click-behavior="override"
        :data="treeData"
        block-line
        key-field="id"
        label-field="name"
        :filter="handleFilter"
        :show-irrelevant-nodes="false"
        expand-on-click
        :render-prefix="renderPrefix"
        :render-label="renderLabel"
        :on-update:selected-keys="handleSelectedKeys"
        :default-expand-all="expand"
      />
    </div>

    <!-- 收藏弹窗 -->
    <m-dialog v-model:visible="showFavoriteModal" title="保存为常用键位">
      <div class="flex flex-col h-full">
        <div class="flex-1 p-4">
          <m-input v-model="favoriteName" placeholder="保存名称(如：万灵)" />
        </div>
        <div class="flex justify-end gap-2 p-4 border-t" style="border-color: var(--line-soft)">
          <m-button @click="showFavoriteModal = false">
            取消
          </m-button>
          <m-button type="primary" @click="confirmFavorite">
            保存
          </m-button>
        </div>
      </div>
    </m-dialog>
  </div>
</template>
```

（对比旧版的删除项：顶部「展开全部/收起全部」n-button 行移到搜索框下方并改 quaternary；`h-screen`、`w-[300px]`、`h-[calc(100vh-20px)]`、`overflow-y-scroll` 等尺寸 hack 全部移除；旧的 `border-gray-200` 改为 token 边框色。）

- [ ] **Step 2: 更新 SourceTree 的 style**

`<style scoped>` 整体替换为：

```vue
<style scoped>
:global(.m-model-close-btn) {
  left: 93% !important;
}

.compact-tree :deep(.n-tree-node) {
  padding: 2px 0;
}

.compact-tree :deep(.n-tree-node-content) {
  padding: 2px 4px;
}

/* hover 时显示图标 */
.compact-tree :deep(.label-with-icons .folder-icon),
.compact-tree :deep(.label-with-icons .favorite-icon) {
  opacity: 0;
  transition: opacity 0.2s;
}

.compact-tree :deep(.n-tree-node:hover .label-with-icons .folder-icon),
.compact-tree :deep(.n-tree-node:hover .label-with-icons .favorite-icon),
.compact-tree :deep(.label-with-icons .favorite-icon.is-favorited) {
  opacity: 1;
}
</style>
```

- [ ] **Step 3: 更新 TemplateList 的列表项配色**

把 template 中列表项 div 的 `:class` 绑定整段：

```html
:class="[
  selectedPath === template.sourcePath
    ? 'bg-green-50 dark:bg-green-900 border-green-500'
    : 'bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-700 hover:border-gray-400',
]"
```

替换为：

```html
:class="selectedPath === template.sourcePath ? 'tpl-item tpl-selected' : 'tpl-item'"
```

并在文件末尾新增：

```vue
<style scoped>
.tpl-item {
  background: var(--paper-card);
  border-color: var(--line);
}

.tpl-item:hover {
  border-color: var(--ink-muted);
}

.tpl-selected {
  background: var(--bamboo-tint);
  border-color: var(--bamboo);
}
</style>
```

- [ ] **Step 4: 验证 + 目检**

Run: `pnpm type-check && pnpm lint && pnpm exec vitest run`
Expected: 全部通过。目检改键页：两棵树嵌在卡片内不溢出、搜索/展开收起/选中/收藏/打开文件夹都正常；「常用」tab 下模板列表选中态是竹青描边；深浅两主题分别看一遍。

- [ ] **Step 5: Commit**

```bash
git add src/views/keyboard/components/SourceTree.vue src/views/keyboard/components/TemplateList.vue
git commit -m "feat(ui): restyle source tree and template list with theme tokens

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 11: 清理遗留资源 + 全量验证 + 文档

**Files:**
- Delete (mv 到 /tmp): `src/assets/pagoda-8018757_1280.jpg`
- Modify: `CLAUDE.md`

- [ ] **Step 1: 确认古塔图无引用后移除**

Run: `grep -rn "pagoda" src/ index.html uno.config.ts 2>/dev/null`
Expected: 无输出（Task 5 已移除唯一引用）。若有输出，先处理引用再继续。

```bash
mv src/assets/pagoda-8018757_1280.jpg /tmp/
git add src/assets/pagoda-8018757_1280.jpg
```

（注意：`src/assets/YingZhangXingShu-2.ttf` 虽未使用，但不在本次批准范围内，**保留不动**。）

- [ ] **Step 2: 更新 CLAUDE.md**

在 `CLAUDE.md` 的 Frontend 小节（`- '@/' alias → ...` 那一行之前）插入一行：

```markdown
- Theme system: `assets/theme.css` (ink-paper CSS variable tokens, light + dark), `composables/useTheme.ts` (three-state mode), `src/theme/naive.ts` (naive-ui overrides — keep in sync with theme.css), `components/layout/PageHeader.vue` (unified page header)
```

- [ ] **Step 3: 全量验证**

Run: `pnpm type-check && pnpm lint && pnpm exec vitest run && pnpm build-only`
Expected: 全部通过、构建成功。

目检清单（深浅两主题各过一遍）：
1. 侧栏印章字导航 + 主题切换循环正常
2. 改键页：空状态（可临时清掉 localStorage 的 `keyboard-base-path` 验证）、路径胶囊、三栏卡片、复制流程、最近操作
3. MAC页：地址主卡、开关、帮助展开
4. 按键页：状态徽章、键帽录入、popover、帮助展开

- [ ] **Step 4: Commit**

```bash
git add CLAUDE.md
git commit -m "chore(ui): remove legacy pagoda asset and document theme system

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## 验收对照（来自设计文档）

- §2 设计 token → Task 1、3
- §2 主题切换（三态 + 持久化） → Task 2、5
- §3 通用规则（宋体标题/墨线/纸纹理/卡片/页头模板） → Task 1、4
- §4 主布局（58px 印章侧栏/主题钮/版本号/sealChar meta/纯 div+flex） → Task 5
- §5 改键页（路径胶囊/三栏卡片/箭头/操作面板/最近操作/空状态/树行为不变） → Task 8、9、10
- §6 MAC页 → Task 6
- §7 按键页 → Task 7
- §8 实现要点（theme.css/useTheme/PageHeader/无 webfont/清理） → Task 1、2、4、11
- §9 不变更清单 → 各任务均只动 template/style；新增 useRecentOps 不触碰既有 composables
