<script setup lang="ts">
import { ref, computed } from 'vue'
import vueTsSource from '../templates/VersionUpdateNotification.vue?raw'
import vueJsSource from '../templates/VersionUpdateNotification.vue.js?raw'
import vue2TsSource from '../templates/VersionUpdateNotification.vue2.ts?raw'
import vue2JsSource from '../templates/VersionUpdateNotification.vue2?raw'
import reactSource from '../templates/VersionUpdateNotification.tsx?raw'
import angularSource from '../templates/VersionUpdateNotification.angular.ts?raw'
import svelteSource from '../templates/VersionUpdateNotification.svelte?raw'
import jsSource from '../templates/VersionUpdateNotification.js?raw'

const toastMessage = ref('')
const selectedFramework = ref<'vue3' | 'vue2' | 'react' | 'angular' | 'svelte' | 'js'>('vue3')
const vueVariant = ref<'ts' | 'js'>('ts')

const frameworkOptions = [
  { value: 'vue3' as const, label: 'Vue 3', icon: '🟢' },
  { value: 'vue2' as const, label: 'Vue 2', icon: '🟢' },
  { value: 'react' as const, label: 'React', icon: '🔵' },
  { value: 'angular' as const, label: 'Angular', icon: '🔴' },
  { value: 'svelte' as const, label: 'Svelte', icon: '🟠' },
  { value: 'js' as const, label: '纯 JS', icon: '🟡' },
]

// 下载到用户项目时的标准文件名（模板源文件名仅用于仓库内部区分变体）
const downloadNameMap: Record<string, string> = {
  'vue3-ts': 'VersionUpdateNotification.vue',
  'vue3-js': 'VersionUpdateNotification.vue',
  'vue2-ts': 'VersionUpdateNotification.vue',
  'vue2-js': 'VersionUpdateNotification.vue',
  react: 'VersionUpdateNotification.tsx',
  angular: 'version-update-notification.component.ts',
  svelte: 'VersionUpdateNotification.svelte',
  js: 'VersionUpdateNotification.js',
}

const frameworkSourceMap: Record<string, string> = {
  'vue3-ts': vueTsSource, 'vue3-js': vueJsSource,
  'vue2-ts': vue2TsSource, 'vue2-js': vue2JsSource,
  react: reactSource, angular: angularSource, svelte: svelteSource, js: jsSource,
}

const sourceKey = computed(() => {
  const fw = selectedFramework.value
  return (fw === 'vue3' || fw === 'vue2') ? `${fw}-${vueVariant.value}` : fw
})

const currentLabel = computed(() => {
  const fw = frameworkOptions.find(f => f.value === selectedFramework.value)!
  const isVue = selectedFramework.value === 'vue3' || selectedFramework.value === 'vue2'
  return isVue ? `${fw.label} (${vueVariant.value === 'ts' ? 'TS' : 'JS'})` : fw.label
})

const downloadFilename = computed(() => downloadNameMap[sourceKey.value])

const isVueFramework = computed(() => selectedFramework.value === 'vue3' || selectedFramework.value === 'vue2')

const projectLinks = [
  {
    label: '下载桌面端',
    href: 'https://github.com/YongHangPu/deploygo/releases',
    icon: 'download'
  },
  {
    label: 'GitHub 主页',
    href: 'https://github.com/YongHangPu',
    icon: 'github'
  },
  {
    label: 'GitHub 仓库',
    href: 'https://github.com/YongHangPu/deploygo',
    icon: 'repository'
  },
  {
    label: 'Gitee 仓库',
    href: 'https://gitee.com/yonghangpu/deploygo',
    icon: 'gitee'
  }
]

const fitCheckCards = [
  {
    title: '适合静态站点团队',
    summary: '适合 Vue、React、Angular、Svelte 等带 hash 静态资源的 SPA 项目，强调发布稳定性、版本追踪与回滚能力。',
    tone: 'fit'
  },
  {
    title: '接入动作很轻',
    summary: '选择对应框架的更新通知组件挂载到应用根组件中，让页面具备稳定的版本更新提示能力。',
    tone: 'check'
  },
  {
    title: '发布流程很完整',
    summary: '构建目录识别、version.json 生成、远端发布、历史记录与回滚，统一在 deploygo 中完成。',
    tone: 'caution'
  }
]

const startHereCards = [
  {
    title: '下载组件',
    summary: '选择你的前端框架，将更新通知组件下载到项目的 components 目录。',
    action: '下载组件',
    type: 'download'
  },
  {
    title: '挂载到根组件',
    summary: '在 App.vue / App.tsx / 根布局中挂载通知组件，让页面在发现新版本时给出清晰提示。',
    action: '复制挂载示例',
    type: 'copy'
  },
  {
    title: '执行构建并发布',
    summary: '先执行项目构建，再打开 deploygo 完成发布。',
    action: '复制构建命令',
    type: 'copy-build'
  }
]

const workflowSteps = [
  {
    step: '01',
    title: '完成本地构建',
    detail: '在业务项目中执行构建命令（如 npm run build），准备最新的静态资源产物。'
  },
  {
    step: '02',
    title: '准备发布内容',
    detail: 'deploygo 自动识别真实构建目录，生成 version.json 与 release_name，并整理上传内容。'
  },
  {
    step: '03',
    title: '执行远端发布',
    detail: '自动推导 releases 目录、创建远端目录、上传发布内容并执行零停机发布。'
  },
  {
    step: '04',
    title: '提示用户刷新',
    detail: '线上 version.json 更新后，通知组件会提示当前用户刷新以获取最新版本。'
  }
]

const capabilityCards = [
  {
    title: '项目侧职责',
    items: ['挂载更新通知组件', '按原有方式执行构建', '确保线上缓存策略合理']
  },
  {
    title: 'deploygo 负责',
    items: ['识别构建目录并生成版本信息', '执行远端发布与版本留存', '记录历史、回滚版本并拦截重复发布']
  },
  {
    title: '发布结果',
    items: ['在线页面持续可用', '新访问自然进入新版本', '发布记录完整可追踪']
  }
]

const desktopGuideSections = [
  {
    title: '添加服务器',
    id: 'add-server',
    summary: '填写 SSH 信息，让 deploygo 连接到目标 Linux 服务器执行发布。',
    fields: [
      { label: '服务器名称', desc: '用于区分不同环境或主机' },
      { label: '主机 / 端口', desc: '目标 Linux 服务器的 SSH 连接信息' },
      { label: '用户名 / 密码', desc: '用于远端连接和发布，密码可加密保存在本地' }
    ]
  },
  {
    title: '添加项目',
    id: 'add-project',
    summary: '填写本地构建产物目录和线上发布目录，deploygo 会自动推导 releases 目录。',
    fields: [
      { label: '项目名称', desc: '自定义项目名称' },
      { label: '本地构建产物目录', desc: '例如 dist，或其上一层由 deploygo 自动识别' },
      { label: '线上发布目录', desc: '例如 /mnt/data/app/dist' },
      { label: '保留版本数', desc: '控制历史版本保留数量' }
    ]
  },
  {
    title: '一键部署与回滚',
    id: 'deploy-and-rollback',
    summary: '每次部署只需先构建，再点击一键部署；需要时可从历史记录中直接回滚。',
    steps: [
      '先在业务项目执行构建',
      '回到 deploygo 选择项目并点击一键部署',
      'deploygo 自动完成发布、记录日志和历史',
      '若产物与最近一次发布完全相同，会提示重复发布并阻止继续部署',
      '若需要恢复旧版本，可在历史记录中直接回滚'
    ]
  }
]

const faqItems = [
  {
    question: '业务项目需要做哪些接入？',
    answer: '只需要选择对应框架的更新通知组件挂载到应用根组件，并保持正常构建流程。'
  },
  {
    question: '服务器需要预先准备什么？',
    answer: '只需要提供可用的 Linux 服务器与 SSH 访问权限，其余发布逻辑由 deploygo 自动完成。'
  },
  {
    question: '为什么页面里需要通知组件？',
    answer: '它负责轮询 version.json，并在检测到新版本时提醒用户刷新页面，完成最后一段版本感知体验。'
  },
  {
    question: '每次发版时要做什么？',
    answer: '先在业务项目中执行构建，然后回到 deploygo 点击一键部署即可。'
  }
]

const mountSnippets: Record<string, string> = {
  'vue3-ts': [
    '<script setup lang="ts">',
    "import VersionUpdateNotification from './components/VersionUpdateNotification.vue'",
    '<' + '/script>',
    '',
    '<template>',
    '  <RouterView />',
    '  <VersionUpdateNotification />',
    '<' + '/template>'
  ].join('\n'),
  'vue3-js': [
    '<script setup>',
    "import VersionUpdateNotification from './components/VersionUpdateNotification.vue'",
    '<' + '/script>',
    '',
    '<template>',
    '  <RouterView />',
    '  <VersionUpdateNotification />',
    '<' + '/template>'
  ].join('\n'),
  'vue2-ts': [
    '<script lang="ts">',
    "import VersionUpdateNotification from './components/VersionUpdateNotification.vue'",
    '',
    'export default {',
    "  components: { VersionUpdateNotification },",
    '}',
    '<' + '/script>',
    '',
    '<template>',
    '  <div id="app">',
    '    <router-view />',
    '    <VersionUpdateNotification />',
    '  </div>',
    '</template>'
  ].join('\n'),
  'vue2-js': [
    '<script>',
    "import VersionUpdateNotification from './components/VersionUpdateNotification.vue'",
    '',
    'export default {',
    "  components: { VersionUpdateNotification },",
    '}',
    '<' + '/script>',
    '',
    '<template>',
    '  <div id="app">',
    '    <router-view />',
    '    <VersionUpdateNotification />',
    '  </div>',
    '</template>'
  ].join('\n'),
  react: [
    "import VersionUpdateNotification from './components/VersionUpdateNotification'",
    '',
    'export default function App() {',
    '  return (',
    '    <>',
    '      <Router />',
    '      <VersionUpdateNotification />',
    '    </>',
    '  )',
    '}'
  ].join('\n'),
  angular: [
    "import { VersionUpdateNotificationComponent } from './components/version-update-notification.component'",
    '',
    '@Component({',
    '  // ...',
    "  imports: [VersionUpdateNotificationComponent],",
    '  template: `',
    '    <router-outlet />',
    '    <app-version-update-notification />',
    '  `',
    '})',
    'export class AppComponent {}'
  ].join('\n'),
  svelte: [
    '<script>',
    "  import VersionUpdateNotification from './lib/VersionUpdateNotification.svelte'",
    '<' + '/script>',
    '',
    '<Router />',
    '<VersionUpdateNotification />'
  ].join('\n'),
  js: [
    '<!-- 在 HTML 中引入 -->',
    '<script src="./components/VersionUpdateNotification.js"><' + '/script>',
    '<script>',
    '  VersionUpdateNotification.init({ basePath: \'/\' })',
    '<' + '/script>'
  ].join('\n'),
}

const mountSnippet = computed(() => mountSnippets[sourceKey.value] || mountSnippets['vue3-ts'])

const downloadFile = async (content: string, filename: string) => {
  try {
    const blob = new Blob([content], { type: 'application/octet-stream' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = filename
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
    toastMessage.value = `已开始下载: ${filename}`
  } catch (err) {
    toastMessage.value = `下载失败: ${err}`
  }

  window.setTimeout(() => {
    toastMessage.value = ''
  }, 3000)
}

const copyText = async (text: string, label: string) => {
  try {
    await navigator.clipboard.writeText(text)
    toastMessage.value = `已复制 ${label}`
  } catch {
    toastMessage.value = '复制失败，请手动选择'
  }

  window.setTimeout(() => {
    toastMessage.value = ''
  }, 1800)
}

const handleStartCardAction = async (type: string) => {
  if (type === 'download') {
    const source = frameworkSourceMap[sourceKey.value]
    await downloadFile(source, downloadFilename.value)
    return
  }

  if (type === 'copy') {
    await copyText(mountSnippet.value, '挂载示例')
    return
  }

  if (type === 'copy-build') {
    await copyText('npm run build  # 或 pnpm build / yarn build', '构建命令')
  }
}
</script>

<template>
  <div class="app-shell">
    <header class="hero-panel">
      <div class="hero-copy">
        <p class="eyebrow">静态站点发布工具</p>
        <h1>用 deploygo 完成稳定、可回滚的静态站点发布</h1>
        <p class="hero-lead">
          从本地构建产物到远端发布、版本留存与回滚，deploygo 提供一条完整工作流。
          业务项目只需要挂载更新通知组件，再按原有方式执行项目构建。
        </p>
        <div class="hero-actions">
          <a href="https://github.com/YongHangPu/deploygo/releases" class="primary-link" target="_blank" rel="noreferrer">下载桌面端</a>
          <a href="#start-here" class="secondary-link">极简接入</a>
          <a href="#workflow" class="secondary-link">发布流程</a>
          <a href="#desktop-guide" class="secondary-link">桌面端使用说明</a>
        </div>
        <p class="hero-note">适合希望降低发布维护成本，同时保留稳定性、可追踪性与回滚能力的前端团队。</p>
      </div>
      <div class="hero-metrics">
        <div class="hero-quick-links">
          <a
            v-for="item in projectLinks"
            :key="item.href"
            :href="item.href"
            class="hero-quick-link"
            target="_blank"
            rel="noreferrer"
            :title="item.label"
            :aria-label="item.label"
          >
            <svg v-if="item.icon === 'download'" viewBox="0 0 16 16" aria-hidden="true" class="hero-quick-link-icon">
              <path d="M2.75 14A1.75 1.75 0 0 1 1 12.25v-2.5a.75.75 0 0 1 1.5 0v2.5c0 .138.112.25.25.25h10.5a.25.25 0 0 0 .25-.25v-2.5a.75.75 0 0 1 1.5 0v2.5A1.75 1.75 0 0 1 13.25 14Z" fill="currentColor" />
              <path d="M7.25 1.75a.75.75 0 0 1 1.5 0v6.44l1.97-1.97a.75.75 0 1 1 1.06 1.06l-3.25 3.25a.75.75 0 0 1-1.06 0L4.22 7.28a.75.75 0 0 1 1.06-1.06l1.97 1.97Z" fill="currentColor" />
            </svg>
            <svg v-else-if="item.icon === 'github'" viewBox="0 0 16 16" aria-hidden="true" class="hero-quick-link-icon">
              <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59c.4.07.55-.17.55-.38c0-.19-.01-.82-.01-1.49c-2.01.37-2.53-.49-2.69-.94c-.09-.23-.48-.94-.82-1.13c-.28-.15-.68-.52-.01-.53c.63-.01 1.08.58 1.23.82c.72 1.21 1.87.87 2.33.66c.07-.52.28-.87.5-1.07c-1.78-.2-3.64-.89-3.64-3.95c0-.87.31-1.59.82-2.15c-.08-.2-.36-1.02.08-2.12c0 0 .67-.21 2.2.82c.64-.18 1.32-.27 2-.27s1.36.09 2 .27c1.53-1.04 2.2-.82 2.2-.82c.44 1.1.16 1.92.08 2.12c.51.56.82 1.27.82 2.15c0 3.07-1.87 3.75-3.65 3.95c.29.25.54.73.54 1.48c0 1.07-.01 1.93-.01 2.2c0 .21.15.46.55.38A8.013 8.013 0 0 0 16 8c0-4.42-3.58-8-8-8Z" fill="currentColor" />
            </svg>
            <svg v-else-if="item.icon === 'repository'" viewBox="0 0 16 16" aria-hidden="true" class="hero-quick-link-icon">
              <path d="M2 2.75A1.75 1.75 0 0 1 3.75 1h8.5C13.216 1 14 1.784 14 2.75v10.5A1.75 1.75 0 0 1 12.25 15h-8.5A1.75 1.75 0 0 1 2 13.25Zm1.75-.25a.25.25 0 0 0-.25.25v10.5c0 .138.112.25.25.25H5v-2.25C5 10.56 5.56 10 6.25 10h3.5c.69 0 1.25.56 1.25 1.25v2.25h1.25a.25.25 0 0 0 .25-.25V2.75a.25.25 0 0 0-.25-.25Zm5.75 11v-2h-3v2Zm-3.75-8.75c0-.414.336-.75.75-.75h3.5a.75.75 0 0 1 0 1.5H6.5a.75.75 0 0 1-.75-.75Zm0 3c0-.414.336-.75.75-.75h3.5a.75.75 0 0 1 0 1.5H6.5a.75.75 0 0 1-.75-.75Z" fill="currentColor" />
            </svg>
            <svg v-else viewBox="0 0 16 16" aria-hidden="true" class="hero-quick-link-icon hero-quick-link-icon-gitee">
              <rect width="16" height="16" rx="3.2" fill="#C71D23" />
              <path d="M3.25 5.1A2.35 2.35 0 0 1 5.6 2.75h4.8a2.35 2.35 0 0 1 0 4.7H7.15v1.4h3.05a2.35 2.35 0 1 1 0 4.7H5.6a2.35 2.35 0 0 1-2.35-2.35V5.1Zm3.9.85h3.25a.85.85 0 0 0 0-1.7H5.6a.85.85 0 0 0-.85.85v6.1c0 .47.38.85.85.85h4.6a.85.85 0 1 0 0-1.7H6.4a.75.75 0 0 1-.75-.75V6.7c0-.414.336-.75.75-.75Z" fill="#fff" />
            </svg>
            <svg viewBox="0 0 16 16" aria-hidden="true" class="hero-quick-link-arrow">
              <path d="M4.5 11.5L11.5 4.5M6 4.5h5.5V10" fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" />
            </svg>
          </a>
        </div>
        <article>
          <span>项目接入</span>
          <strong>1 个组件</strong>
          <p>支持 Vue / React / Angular / Svelte / JS</p>
        </article>
        <article>
          <span>发布前动作</span>
          <strong>按原有方式构建</strong>
          <p>按原有方式执行项目构建</p>
        </article>
        <article>
          <span>deploygo 能力</span>
          <strong>发布、回滚、追踪</strong>
          <p>统一收口在 deploygo 中完成</p>
        </article>
      </div>
    </header>

    <section id="fit-check" class="section-block section-spotlight section-fit-check">
      <div class="section-heading">
        <p class="section-kicker">快速判断</p>
        <h2>适合哪些项目</h2>
      </div>
      <div class="start-grid">
        <article v-for="item in fitCheckCards" :key="item.title" class="info-card start-card fit-card" :class="`fit-card-${item.tone}`">
          <h3>{{ item.title }}</h3>
          <p>{{ item.summary }}</p>
        </article>
      </div>
    </section>

    <section id="start-here" class="section-block section-guide section-start">
      <div class="section-heading">
        <p class="section-kicker">从这里开始</p>
        <h2>3 步完成接入</h2>
      </div>

      <!-- 框架选择器 -->
      <div class="framework-selector">
        <span class="framework-label">选择你的框架：</span>
        <div class="framework-tabs">
          <button
            v-for="fw in frameworkOptions"
            :key="fw.value"
            class="framework-tab"
            :class="{ active: selectedFramework === fw.value }"
            :title="fw.label"
            @click="selectedFramework = fw.value"
          >
            <span class="fw-icon">{{ fw.icon }}</span>
            <span class="fw-label">{{ fw.label }}</span>
          </button>
        </div>
      </div>

      <!-- TS/JS 变体切换（仅 Vue 支持） -->
      <div v-if="isVueFramework" class="variant-selector">
        <span class="variant-label">语言版本：</span>
        <div class="variant-tabs">
          <button
            class="variant-tab"
            :class="{ active: vueVariant === 'ts' }"
            @click="vueVariant = 'ts'"
          >TypeScript</button>
          <button
            class="variant-tab"
            :class="{ active: vueVariant === 'js' }"
            @click="vueVariant = 'js'"
          >JavaScript</button>
        </div>
      </div>

      <div class="start-grid step-grid">
        <article v-for="item in startHereCards" :key="item.title" class="info-card start-card">
          <h3>{{ item.title }}</h3>
          <p>{{ item.summary }}</p>
          <div class="start-card-actions">
            <button type="button" class="inline-btn" @click="handleStartCardAction(item.type)">
              {{ item.action }}
            </button>
          </div>
        </article>
      </div>
      <div class="starter-checklist">
        <div class="section-heading compact">
          <p class="section-kicker">挂载示例</p>
          <h2>把组件放到应用根组件中</h2>
        </div>
        <div class="code-shell">
          <div class="code-meta">
            <span>{{ currentLabel }} 挂载示例</span>
            <div class="code-meta-actions">
              <button type="button" class="copy-button" @click="copyText(mountSnippet, '挂载示例')">复制代码</button>
            </div>
          </div>
          <pre><code>{{ mountSnippet }}</code></pre>
        </div>
      </div>
    </section>

    <section id="workflow" class="section-block section-guide section-workflow">
      <div class="section-heading">
        <p class="section-kicker">发布流程</p>
        <h2>发布链路如何运转</h2>
      </div>
      <div class="workflow-list">
        <article v-for="item in workflowSteps" :key="item.step" class="workflow-card">
          <span class="workflow-step">{{ item.step }}</span>
          <div>
            <h3>{{ item.title }}</h3>
            <p>{{ item.detail }}</p>
          </div>
        </article>
      </div>
    </section>

    <section class="section-block section-principles">
      <div class="section-heading">
        <p class="section-kicker">产品能力</p>
        <h2>职责清晰，流程完整</h2>
      </div>
      <div class="principle-grid">
        <article v-for="item in capabilityCards" :key="item.title" class="info-card">
          <h3>{{ item.title }}</h3>
          <ul class="checklist-list compact">
            <li v-for="point in item.items" :key="point">{{ point }}</li>
          </ul>
        </article>
      </div>
    </section>

    <section id="desktop-guide" class="section-block section-guide section-desktop-guide">
      <div class="section-heading">
        <p class="section-kicker">桌面端使用说明</p>
        <h2>从连接服务器到完成发布</h2>
      </div>
      <div v-for="guide in desktopGuideSections" :key="guide.id" class="guide-block">
        <h3 class="guide-block-title">{{ guide.title }}</h3>
        <p class="guide-block-summary">{{ guide.summary }}</p>

        <div v-if="guide.fields" class="guide-field-list">
          <div v-for="field in guide.fields" :key="field.label" class="guide-field-row">
            <span class="guide-field-label">{{ field.label }}</span>
            <span class="guide-field-desc">{{ field.desc }}</span>
          </div>
        </div>

        <ol v-if="guide.steps" class="guide-step-list">
          <li v-for="(step, si) in guide.steps" :key="si">{{ step }}</li>
        </ol>
      </div>
    </section>

    <section class="section-block section-faq">
      <div class="section-heading">
        <p class="section-kicker">常见问题</p>
        <h2>部署前常见问题</h2>
      </div>
      <div class="faq-list">
        <article v-for="item in faqItems" :key="item.question" class="faq-card">
          <p class="faq-question">{{ item.question }}</p>
          <p class="faq-answer">{{ item.answer }}</p>
        </article>
      </div>
    </section>

    <div v-if="toastMessage" class="global-toast">
      <div class="toast-content">
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="20 6 9 17 4 12"></polyline>
        </svg>
        {{ toastMessage }}
      </div>
    </div>
  </div>
</template>

<style>
@import './style.css';
</style>
