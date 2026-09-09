<script setup lang="ts">
import { ref, computed } from 'vue'
import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'
import vueTsSource from '../../templates/VersionUpdateNotification.vue?raw'
import vueJsSource from '../../templates/VersionUpdateNotification.vue.js?raw'
import vue2TsSource from '../../templates/VersionUpdateNotification.vue2.ts?raw'
import vue2JsSource from '../../templates/VersionUpdateNotification.vue2?raw'
import reactSource from '../../templates/VersionUpdateNotification.tsx?raw'
import angularSource from '../../templates/VersionUpdateNotification.angular.ts?raw'
import svelteSource from '../../templates/VersionUpdateNotification.svelte?raw'
import jsSource from '../../templates/VersionUpdateNotification.js?raw'

const toastMessage = ref('')
const selectedFramework = ref<'vue3' | 'vue2' | 'react' | 'angular' | 'svelte' | 'js'>('vue3')
const vueVariant = ref<'ts' | 'js'>('ts')

const frameworkOptions = [
  { value: 'vue3' as const, label: 'Vue 3' },
  { value: 'vue2' as const, label: 'Vue 2' },
  { value: 'react' as const, label: 'React' },
  { value: 'angular' as const, label: 'Angular' },
  { value: 'svelte' as const, label: 'Svelte' },
  { value: 'js' as const, label: '纯 JS' },
]

const vueExtMap: Record<string, Record<string, string>> = {
  vue3: { ts: '.vue', js: '.vue.js' },
  vue2: { ts: '.vue2.ts', js: '.vue2' },
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

const currentExt = computed(() => {
  const fw = selectedFramework.value
  if (fw === 'vue3' || fw === 'vue2') return vueExtMap[fw][vueVariant.value]
  return '.tsx'
})

const isVueFramework = computed(() => selectedFramework.value === 'vue3' || selectedFramework.value === 'vue2')

const projectLinks = [
  {
    label: 'GitHub 主页',
    href: 'https://github.com/YongHangPu',
    icon: 'github'
  }
]

const startHereCards = [
  {
    title: '下载组件',
    summary: '选择你的前端框架，将更新通知组件下载到业务项目中，推荐放在 components 目录。',
    action: '下载组件',
    type: 'download'
  },
  {
    title: '挂载到根组件',
    summary: '在根组件中挂载通知组件，让页面在发现新版本时给出清晰提示。',
    action: '复制挂载示例',
    type: 'copy'
  },
  {
    title: '执行构建并发布',
    summary: '每次发版前先执行构建，然后回到 deploygo 中点击一键部署。',
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
    detail: '自动推导 releases 目录、上传发布内容、执行零停机发布，并保留可回滚的历史版本。'
  },
  {
    step: '04',
    title: '提示用户刷新',
    detail: '当 version.json 更新后，已在线页面会通过通知组件提示用户刷新获取最新资源。'
  }
]

const desktopGuideSections = [
  {
    title: '添加服务器',
    id: 'add-server',
    summary: '配置 SSH 服务器信息，让 deploygo 连接到目标 Linux 服务器执行发布。',
    fields: [
      { label: '服务器名称', desc: '自定义标识，便于区分不同环境或机器' },
      { label: '主机地址', desc: '服务器 IP 或域名' },
      { label: '端口', desc: 'SSH 端口，默认 22' },
      { label: '用户名 / 密码', desc: 'SSH 登录凭据，密码支持本地加密保存' }
    ],
    note: 'deploygo 会直接执行内置的远端发布逻辑，无需额外脚本配置。'
  },
  {
    title: '添加项目',
    id: 'add-project',
    summary: '将本地构建产物目录与服务器关联起来，后续部署只需要选择项目。',
    fields: [
      { label: '项目名称', desc: '自定义项目名称，如官网、管理后台等' },
      { label: '本地构建产物目录', desc: '填写构建（如 npm run build）后生成的 dist 或其上层目录' },
      { label: '关联服务器', desc: '选择要部署到哪台服务器' },
      { label: '线上发布目录', desc: '例如 /mnt/data/app/dist，deploygo 会据此自动推导同级 releases' },
      { label: '保留版本数', desc: '控制服务器保留多少个历史版本，超出的旧版本会自动清理' }
    ],
    note: 'deploygo 会根据线上发布目录自动推导同级 releases 目录，并在远端完成创建。'
  },
  {
    title: '一键部署',
    id: 'one-click-deploy',
    summary: '选择项目后点击一键部署，deploygo 会自动完成整条发布链路。',
    steps: [
      '先在业务项目中完成构建',
      '回到 deploygo 选择对应项目',
      '建议先测试连接，确认 SSH 可用',
      '点击一键部署，deploygo 自动完成准备、上传、远端发布与历史记录写入',
      '如果当前构建产物与最近一次发布完全一致，会提示重复发布并阻止继续部署'
    ]
  },
  {
    title: '查看历史与回滚',
    id: 'view-history',
    summary: '每次部署都会写入历史记录，方便追踪版本与执行回滚。',
    features: [
      '支持按项目、服务器、状态、时间范围筛选',
      '可查看版本号、文件数量、大小、耗时和完整日志',
      '支持直接回滚到某个历史版本',
      '回滚同样复用内置远端发布流程，不依赖服务器已有脚本'
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
    question: '如果更新提示一直不出现怎么办？',
    answer: '优先检查线上缓存策略，确保 index.html 和 version.json 不被强缓存；带 hash 的静态资源则可以长期缓存。'
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
    "import VersionUpdateNotification from './components/VersionUpdateNotification.vue.js'",
    '<' + '/script>',
    '',
    '<template>',
    '  <RouterView />',
    '  <VersionUpdateNotification />',
    '<' + '/template>'
  ].join('\n'),
  'vue2-ts': [
    '<script lang="ts">',
    "import VersionUpdateNotification from './components/VersionUpdateNotification.vue2.ts'",
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
    "import VersionUpdateNotification from './components/VersionUpdateNotification.vue2'",
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

const nginxSnippet = [
  '# 1. 对 index.html 和 version.json 绝对禁缓存',
  'location ~* ^/(index\\.html|version\\.json)$ {',
  '    root /mnt/data/app/dist;',
  '    add_header Cache-Control "no-store, no-cache, must-revalidate, proxy-revalidate, max-age=0";',
  '    add_header Pragma "no-cache";',
  '    add_header Expires "0";',
  '    try_files $uri =404;',
  '}',
  '',
  '# 2. 对带有 hash 的静态资源配置一年强缓存',
  'location /assets/ {',
  '    root /mnt/data/app/dist;',
  '    add_header Cache-Control "public, max-age=31536000, immutable";',
  '    try_files $uri =404;',
  '}'
].join('\n')

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

const downloadComponent = async () => {
  const source = frameworkSourceMap[sourceKey.value]
  const filename = `VersionUpdateNotification${currentExt.value}`
  const filePath = await save({
    defaultPath: filename,
    filters: [{ name: '源文件', extensions: [currentExt.value.replace('.', '')] }]
  })

  if (!filePath) return

  try {
    await writeTextFile(filePath, source)
    toastMessage.value = `已保存至: ${filePath}`
  } catch (err) {
    toastMessage.value = `保存失败: ${err}`
  }

  window.setTimeout(() => {
    toastMessage.value = ''
  }, 3000)
}

const handleStartCardAction = async (type: string) => {
  if (type === 'download') {
    await downloadComponent()
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
  <div class="guide-view">
    <header class="gv-hero">
      <div class="gv-hero-copy">
        <p class="gv-eyebrow">使用指南</p>
        <h1>3 步接入 deploygo 发布流程</h1>
        <p class="gv-hero-lead">
          这份指南会带你完成组件接入、项目配置与一键部署。
          业务项目只需要挂载更新通知组件，再按原有方式执行项目构建。
        </p>
        <div class="gv-hero-actions">
          <a href="#gv-start-here" class="gv-primary-link">从这里开始</a>
          <a href="#gv-workflow" class="gv-secondary-link">发布流程</a>
          <a href="#gv-desktop-guide" class="gv-secondary-link">桌面端使用说明</a>
        </div>
        <p class="gv-hero-note">建议按顺序阅读：先完成接入，再查看发布流程，最后了解桌面端操作说明。</p>
      </div>
      <div class="gv-hero-metrics">
        <div class="gv-quick-links">
          <a
            v-for="item in projectLinks"
            :key="item.href"
            :href="item.href"
            class="gv-quick-link"
            target="_blank"
            rel="noreferrer"
            :title="item.label"
            :aria-label="item.label"
          >
            <svg v-if="item.icon === 'github'" viewBox="0 0 16 16" aria-hidden="true" class="gv-quick-link-icon">
              <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59c.4.07.55-.17.55-.38c0-.19-.01-.82-.01-1.49c-2.01.37-2.53-.49-2.69-.94c-.09-.23-.48-.94-.82-1.13c-.28-.15-.68-.52-.01-.53c.63-.01 1.08.58 1.23.82c.72 1.21 1.87.87 2.33.66c.07-.52.28-.87.5-1.07c-1.78-.2-3.64-.89-3.64-3.95c0-.87.31-1.59.82-2.15c-.08-.2-.36-1.02.08-2.12c0 0 .67-.21 2.2.82c.64-.18 1.32-.27 2-.27s1.36.09 2 .27c1.53-1.04 2.2-.82 2.2-.82c.44 1.1.16 1.92.08 2.12c.51.56.82 1.27.82 2.15c0 3.07-1.87 3.75-3.65 3.95c.29.25.54.73.54 1.48c0 1.07-.01 1.93-.01 2.2c0 .21.15.46.55.38A8.013 8.013 0 0 0 16 8c0-4.42-3.58-8-8-8Z" fill="currentColor" />
            </svg>
            <svg v-else-if="item.icon === 'repository'" viewBox="0 0 16 16" aria-hidden="true" class="gv-quick-link-icon">
              <path d="M2 2.75A1.75 1.75 0 0 1 3.75 1h8.5C13.216 1 14 1.784 14 2.75v10.5A1.75 1.75 0 0 1 12.25 15h-8.5A1.75 1.75 0 0 1 2 13.25Zm1.75-.25a.25.25 0 0 0-.25.25v10.5c0 .138.112.25.25.25H5v-2.25C5 10.56 5.56 10 6.25 10h3.5c.69 0 1.25.56 1.25 1.25v2.25h1.25a.25.25 0 0 0 .25-.25V2.75a.25.25 0 0 0-.25-.25Zm5.75 11v-2h-3v2Zm-3.75-8.75c0-.414.336-.75.75-.75h3.5a.75.75 0 0 1 0 1.5H6.5a.75.75 0 0 1-.75-.75Zm0 3c0-.414.336-.75.75-.75h3.5a.75.75 0 0 1 0 1.5H6.5a.75.75 0 0 1-.75-.75Z" fill="currentColor" />
            </svg>
            <svg v-else viewBox="0 0 16 16" aria-hidden="true" class="gv-quick-link-icon gv-quick-link-icon-gitee">
              <rect width="16" height="16" rx="3.2" fill="#C71D23" />
              <path d="M3.25 5.1A2.35 2.35 0 0 1 5.6 2.75h4.8a2.35 2.35 0 0 1 0 4.7H7.15v1.4h3.05a2.35 2.35 0 1 1 0 4.7H5.6a2.35 2.35 0 0 1-2.35-2.35V5.1Zm3.9.85h3.25a.85.85 0 0 0 0-1.7H5.6a.85.85 0 0 0-.85.85v6.1c0 .47.38.85.85.85h4.6a.85.85 0 1 0 0-1.7H6.4a.75.75 0 0 1-.75-.75V6.7c0-.414.336-.75.75-.75Z" fill="#fff" />
            </svg>
          </a>
        </div>
        <article>
          <span>项目接入</span>
          <strong>1 个组件</strong>
          <p>多框架支持</p>
        </article>
        <article>
          <span>发版动作</span>
          <strong>先构建，再发布</strong>
          <p>按原有方式执行项目构建</p>
        </article>
        <article>
          <span>桌面端能力</span>
          <strong>发布、回滚、追踪</strong>
          <p>统一收口在 deploygo 中完成</p>
        </article>
      </div>
    </header>

    <section id="gv-start-here" class="gv-section">
      <div class="gv-section-heading">
        <p class="gv-section-kicker">从这里开始</p>
        <h2>3 步开始使用</h2>
      </div>

      <!-- 框架选择器 -->
      <div class="gv-framework-selector">
        <span class="gv-framework-label">选择你的框架：</span>
        <div class="gv-framework-tabs">
          <button
            v-for="fw in frameworkOptions"
            :key="fw.value"
            class="gv-framework-tab"
            :class="{ active: selectedFramework === fw.value }"
            :title="fw.label"
            @click="selectedFramework = fw.value"
          >
            {{ fw.label }}
          </button>
        </div>
      </div>

      <!-- TS/JS 变体切换 -->
      <div v-if="isVueFramework" class="gv-variant-selector">
        <span class="gv-variant-label">语言版本：</span>
        <div class="gv-variant-tabs">
          <button class="gv-variant-tab" :class="{ active: vueVariant === 'ts' }" @click="vueVariant = 'ts'">TypeScript</button>
          <button class="gv-variant-tab" :class="{ active: vueVariant === 'js' }" @click="vueVariant = 'js'">JavaScript</button>
        </div>
      </div>

      <div class="gv-card-grid gv-card-grid-3">
        <article v-for="item in startHereCards" :key="item.title" class="gv-card gv-start-card">
          <h3>{{ item.title }}</h3>
          <p>{{ item.summary }}</p>
          <button type="button" class="gv-inline-btn" @click="handleStartCardAction(item.type)">
            {{ item.action }}
          </button>
        </article>
      </div>
      <div class="gv-code-panel">
        <div class="gv-code-heading">
          <p class="gv-section-kicker">{{ currentLabel }} 挂载示例</p>
          <button type="button" class="gv-inline-btn" @click="copyText(mountSnippet, '挂载示例')">复制代码</button>
        </div>
        <pre><code>{{ mountSnippet }}</code></pre>
      </div>
    </section>

    <section class="gv-section">
      <div class="gv-section-heading">
        <p class="gv-section-kicker">上线前检查</p>
        <h2>发布前补齐这 2 条缓存配置</h2>
      </div>
      <p class="gv-section-intro">
        请先确认线上 Nginx 已完成这 2 条缓存策略：`index.html` 与 `version.json` 禁缓存，带 hash 的静态资源长期缓存。
        否则页面可能无法及时感知新版本。
      </p>
      <div class="gv-code-panel">
        <div class="gv-code-heading">
          <p class="gv-section-kicker">可直接参考</p>
          <button type="button" class="gv-inline-btn" @click="copyText(nginxSnippet, 'Nginx 配置')">复制代码</button>
        </div>
        <pre><code>{{ nginxSnippet }}</code></pre>
      </div>
    </section>

    <section id="gv-workflow" class="gv-section">
      <div class="gv-section-heading">
        <p class="gv-section-kicker">发布流程</p>
        <h2>发布链路如何运转</h2>
      </div>
      <div class="gv-workflow-list">
        <article v-for="item in workflowSteps" :key="item.step" class="gv-workflow-card">
          <span class="gv-workflow-step">{{ item.step }}</span>
          <div>
            <h3>{{ item.title }}</h3>
            <p>{{ item.detail }}</p>
          </div>
        </article>
      </div>
    </section>

    <section class="gv-section">
      <div class="gv-section-heading">
        <p class="gv-section-kicker">常见问题</p>
        <h2>上手前常见问题</h2>
      </div>
      <div class="gv-faq-list">
        <article v-for="item in faqItems" :key="item.question" class="gv-faq-card">
          <p class="gv-faq-question">{{ item.question }}</p>
          <p class="gv-faq-answer">{{ item.answer }}</p>
        </article>
      </div>
    </section>

    <section id="gv-desktop-guide" class="gv-section">
      <div class="gv-section-heading">
        <p class="gv-section-kicker">桌面端使用说明</p>
        <h2>从连接服务器到完成发布</h2>
      </div>
      <div v-for="guide in desktopGuideSections" :key="guide.id" class="gv-guide-block">
        <h3 class="gv-guide-block-title">{{ guide.title }}</h3>
        <p class="gv-guide-block-summary">{{ guide.summary }}</p>

        <div v-if="guide.fields" class="gv-guide-field-list">
          <div v-for="field in guide.fields" :key="field.label" class="gv-guide-field-row">
            <span class="gv-guide-field-label">{{ field.label }}</span>
            <span class="gv-guide-field-desc">{{ field.desc }}</span>
          </div>
          <p v-if="guide.note" class="gv-guide-field-note">{{ guide.note }}</p>
        </div>

        <ol v-if="guide.steps" class="gv-guide-step-list">
          <li v-for="(step, si) in guide.steps" :key="si">{{ step }}</li>
        </ol>

        <ul v-if="guide.features" class="gv-guide-feature-list">
          <li v-for="(feat, fi) in guide.features" :key="fi">{{ feat }}</li>
        </ul>
      </div>
    </section>

    <div v-if="toastMessage" class="gv-toast">
      <div class="gv-toast-content">{{ toastMessage }}</div>
    </div>
  </div>
</template>

<style scoped>
.guide-view {
  padding: var(--content-padding-y) var(--content-padding-x);
  height: 100%;
  overflow-y: auto;
}

.gv-hero,
.gv-section,
.gv-card,
.gv-workflow-card,
.gv-hero-metrics article {
  background: var(--bg-elevated);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
}

.gv-hero {
  display: grid;
  grid-template-columns: minmax(0, 1.35fr) minmax(260px, 0.9fr);
  gap: 28px;
  padding: 32px;
  margin-bottom: 28px;
}

.gv-hero-copy {
  display: grid;
  gap: 14px;
}

.gv-eyebrow,
.gv-section-kicker {
  margin: 0;
  font-size: 12px;
  font-weight: 600;
  letter-spacing: 0.12em;
  text-transform: uppercase;
  color: var(--accent-blue);
}

.gv-hero-copy h1,
.gv-section-heading h2 {
  margin: 0;
  color: var(--text-primary);
  letter-spacing: 0;
}

.gv-hero-copy h1 {
  font-size: clamp(30px, 4vw, 46px);
  line-height: 1.1;
}

.gv-hero-lead,
.gv-card p,
.gv-workflow-card p,
.gv-guide-block-summary,
.gv-faq-answer {
  margin: 0;
  color: var(--text-secondary);
  line-height: 1.75;
}

.gv-hero-lead code {
  background: var(--terminal-bg);
  padding: 1px 6px;
  border-radius: 4px;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  color: var(--accent-blue);
}

.gv-hero-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.gv-primary-link,
.gv-secondary-link,
.gv-inline-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: 40px;
  padding: 0 16px;
  border-radius: 999px;
  text-decoration: none;
  cursor: pointer;
  transition:
    transform 220ms cubic-bezier(0.22, 1, 0.36, 1),
    border-color 220ms cubic-bezier(0.22, 1, 0.36, 1),
    background-color 220ms cubic-bezier(0.22, 1, 0.36, 1),
    color 220ms cubic-bezier(0.22, 1, 0.36, 1);
}

.gv-primary-link {
  background: var(--accent-blue);
  color: #fff;
}

.gv-secondary-link,
.gv-inline-btn {
  background: transparent;
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
}

.gv-primary-link:hover,
.gv-secondary-link:hover,
.gv-inline-btn:hover {
  transform: translateY(-1px);
}

.gv-hero-note {
  margin: 0;
  color: var(--text-muted);
  font-size: 13px;
}

.gv-hero-metrics {
  display: grid;
  gap: 12px;
}

.gv-quick-links {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.gv-quick-link {
  width: 40px;
  height: 40px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 12px;
  border: 1px solid var(--border-color);
  color: var(--text-secondary);
}

.gv-quick-link-icon {
  width: 18px;
  height: 18px;
}

.gv-quick-link-icon-gitee {
  border-radius: 4px;
}

.gv-hero-metrics article {
  padding: 16px 18px;
}

.gv-hero-metrics span {
  display: block;
  font-size: 11px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.08em;
}

.gv-hero-metrics strong {
  display: block;
  margin-top: 8px;
  font-size: 18px;
  color: var(--text-primary);
}

.gv-hero-metrics p {
  margin: 8px 0 0;
  color: var(--text-secondary);
  font-size: 13px;
}

.gv-section {
  margin-bottom: 28px;
  padding: 28px;
}

.gv-section-heading {
  display: grid;
  gap: 8px;
  margin-bottom: 18px;
}

.gv-section-intro {
  margin: 0 0 16px;
  color: var(--text-secondary);
  line-height: 1.75;
}

.gv-card-grid,
.gv-workflow-list,
.gv-faq-list {
  display: grid;
  gap: 14px;
}

.gv-card-grid-3 {
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

/* 框架选择器 */
.gv-framework-selector {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
  margin-bottom: 18px;
}

.gv-framework-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-muted);
}

.gv-framework-tabs {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.gv-framework-tab {
  padding: 6px 14px;
  border-radius: 8px;
  border: 1px solid var(--border-color);
  background: var(--bg-card);
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition:
    border-color 220ms cubic-bezier(0.22, 1, 0.36, 1),
    background-color 220ms cubic-bezier(0.22, 1, 0.36, 1),
    color 220ms cubic-bezier(0.22, 1, 0.36, 1);
  font-family: inherit;
}

.gv-framework-tab:hover {
  border-color: var(--accent-blue);
  color: var(--text-primary);
}

.gv-framework-tab.active {
  border-color: var(--accent-blue);
  background: rgba(var(--accent-blue-rgb), 0.1);
  color: var(--accent-blue);
}

/* TS/JS 变体切换 */
.gv-variant-selector {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  margin-bottom: 8px;
}
.gv-variant-label {
  font-size: 12px;
  color: var(--text-muted);
}
.gv-variant-tabs {
  display: flex;
  gap: 4px;
}
.gv-variant-tab {
  padding: 4px 12px;
  border-radius: 6px;
  border: 1px solid var(--border-color);
  background: var(--bg-card);
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition:
    border-color 220ms cubic-bezier(0.22, 1, 0.36, 1),
    background-color 220ms cubic-bezier(0.22, 1, 0.36, 1),
    color 220ms cubic-bezier(0.22, 1, 0.36, 1);
  font-family: inherit;
}
.gv-variant-tab:hover {
  border-color: var(--accent-blue);
  color: var(--text-primary);
}
.gv-variant-tab.active {
  border-color: var(--accent-green);
  background: rgba(var(--accent-green-rgb), 0.1);
  color: var(--accent-green);
}

.gv-card {
  padding: 20px;
}

.gv-card h3,
.gv-workflow-card h3,
.gv-guide-block-title {
  margin: 0 0 8px;
  color: var(--text-primary);
}

.gv-fit-card-fit {
  border-color: rgba(99, 207, 224, 0.25);
}

.gv-fit-card-check {
  border-color: rgba(59, 130, 246, 0.25);
}

.gv-fit-card-caution {
  border-color: rgba(245, 158, 11, 0.25);
}

.gv-start-card {
  display: grid;
  gap: 12px;
}

.gv-code-panel {
  margin-top: 16px;
  padding: 18px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  background: var(--bg-card);
}

.gv-code-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
}

.gv-code-panel pre {
  margin: 0;
  padding: 16px;
  overflow: auto;
  background: var(--terminal-bg);
  border-radius: var(--radius-md);
}

.gv-code-panel code {
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  color: var(--accent-blue);
  font-size: 13px;
  line-height: 1.7;
}

.gv-workflow-list {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.gv-workflow-card {
  display: grid;
  grid-template-columns: 50px 1fr;
  gap: 14px;
  padding: 18px;
}

.gv-workflow-step {
  width: 50px;
  height: 50px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 14px;
  background: var(--bg-chip);
  border: 1px solid rgba(var(--accent-blue-rgb), 0.24);
  color: var(--accent-blue);
  font-weight: 700;
}

.gv-checklist-list,
.gv-guide-feature-list {
  margin: 0;
  padding-left: 20px;
  display: grid;
  gap: 8px;
  color: var(--text-secondary);
}

.gv-faq-list {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.gv-faq-card {
  padding: 18px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  background: var(--bg-card);
}

.gv-faq-question {
  margin: 0 0 8px;
  font-weight: 600;
  color: var(--text-primary);
}

.gv-guide-block + .gv-guide-block {
  margin-top: 24px;
}

.gv-guide-field-list {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.gv-guide-field-row {
  display: flex;
  gap: 16px;
  padding: 12px 14px;
  border-bottom: 1px solid var(--border-color);
}

.gv-guide-field-row:last-child {
  border-bottom: none;
}

.gv-guide-field-label {
  flex-shrink: 0;
  min-width: 118px;
  color: var(--text-primary);
  font-weight: 600;
}

.gv-guide-field-desc,
.gv-guide-field-note,
.gv-guide-step-list {
  color: var(--text-secondary);
}

.gv-guide-field-note {
  margin: 12px 14px;
}

.gv-guide-step-list {
  margin: 0;
  padding-left: 20px;
  display: grid;
  gap: 8px;
}

.gv-toast {
  position: fixed;
  top: 16px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 1000;
}

.gv-toast-content {
  padding: 10px 18px;
  border-radius: 999px;
  background: var(--bg-elevated);
  border: 1px solid var(--accent-green);
  color: var(--text-primary);
}

@media (max-width: 980px) {
  .gv-hero,
  .gv-card-grid-3,
  .gv-workflow-list,
  .gv-faq-list {
    grid-template-columns: 1fr;
  }

  .gv-quick-links {
    justify-content: flex-start;
  }
}

@media (max-width: 640px) {
  .gv-section,
  .gv-hero {
    padding: 20px;
  }

  .gv-guide-field-row {
    flex-direction: column;
    gap: 4px;
  }

  .gv-guide-field-label {
    min-width: auto;
  }

  .gv-code-heading {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>

<style scoped>
.guide-view {
  max-width: var(--content-max-width);
  margin: 0 auto;
  padding: var(--content-padding-y) var(--content-padding-x);
}

.gv-hero,
.gv-section,
.gv-card,
.gv-workflow-card,
.gv-hero-metrics article {
  border-radius: 8px;
  box-shadow: none;
}

.gv-hero {
  gap: 24px;
  padding: 26px;
  border-top: 3px solid var(--accent-blue);
}

.gv-hero-copy h1 {
  font-size: clamp(28px, 3.5vw, 40px);
  font-weight: 750;
  letter-spacing: 0;
}

.gv-eyebrow,
.gv-section-kicker {
  font: 700 10px/1.2 'JetBrains Mono', monospace;
  letter-spacing: 0.1em;
}

.gv-primary-link,
.gv-secondary-link,
.gv-inline-btn {
  height: 36px;
  border-radius: 5px;
  font-size: 12px;
  font-weight: 700;
}

.gv-primary-link {
  background: var(--accent-blue);
}

.gv-primary-link:hover,
.gv-secondary-link:hover,
.gv-inline-btn:hover {
  transform: none;
  border-color: var(--accent-blue);
}

.gv-quick-link,
.gv-framework-tab,
.gv-variant-tab,
.gv-workflow-step,
.gv-code-panel,
.gv-code-panel pre,
.gv-guide-field-list,
.gv-faq-card {
  border-radius: 5px;
}

.gv-quick-link:hover {
  border-color: var(--accent-blue);
  color: var(--accent-blue);
}

.gv-hero-metrics article {
  padding: 14px 16px;
}

.gv-section {
  padding: 24px;
}

.gv-card,
.gv-workflow-card,
.gv-faq-card {
  padding: 16px;
}

.gv-workflow-step {
  width: 42px;
  height: 42px;
  background: var(--bg-chip);
  border: 1px solid rgba(var(--accent-blue-rgb), 0.24);
  color: var(--accent-blue);
  font: 700 11px/1 'JetBrains Mono', monospace;
}

.gv-code-panel {
  background: var(--bg-soft);
}

.gv-code-panel code {
  color: var(--accent-blue);
}

.gv-framework-tab.active,
.gv-variant-tab.active {
  font-weight: 700;
}
</style>
