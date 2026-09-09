<script setup lang="ts">
import { ref, computed, provide, watch, onMounted, onUnmounted } from 'vue'
import { darkTheme, useOsTheme, zhCN, dateZhCN, NModal } from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import ProjectSidebar from './components/ProjectSidebar.vue'
import TerminalLog from './components/TerminalLog.vue'
import { useDeployStore } from './stores/deployStore'

const deployStore = useDeployStore()

const theme = ref<'light' | 'dark'>(
  (localStorage.getItem('theme') as 'light' | 'dark') || 'dark'
)

const osTheme = useOsTheme()
watch(osTheme, (val) => {
  if (!localStorage.getItem('theme')) {
    theme.value = val || 'dark'
  }
})

const toggleTheme = () => {
  theme.value = theme.value === 'dark' ? 'light' : 'dark'
  localStorage.setItem('theme', theme.value)
}

const naiveTheme = computed(() => theme.value === 'dark' ? darkTheme : null)
const themeOverrides = computed(() => {
  const isDark = theme.value === 'dark'
  const lightFieldBorder = '#cbd1d9'
  const lightFieldBorderHover = '#91a4c7'
  const lightFieldBorderFocus = '#416fca'

  return {
    common: {
      primaryColor: isDark ? '#5b8cff' : '#416fca',
      primaryColorHover: isDark ? '#7aa4ff' : '#5684dd',
      primaryColorPressed: isDark ? '#3d6fdd' : '#315aa9',
      primaryColorSuppl: isDark ? '#5b8cff' : '#416fca',
      successColor: isDark ? '#63cfe0' : '#238da3',
      successColorHover: isDark ? '#87ddea' : '#32a4bc',
      successColorPressed: isDark ? '#46b3c6' : '#1a7186',
      warningColor: isDark ? '#f1b95a' : '#ad741d',
      warningColorHover: isDark ? '#f7cb7d' : '#be862e',
      warningColorPressed: isDark ? '#d89d3d' : '#915c12',
      errorColor: isDark ? '#ff7b72' : '#c94843',
      errorColorHover: isDark ? '#ff988f' : '#d65c56',
      errorColorPressed: isDark ? '#e26058' : '#aa3430',
      infoColor: isDark ? '#7aa4ff' : '#416fca',
      infoColorHover: isDark ? '#9abaff' : '#5684dd',
      infoColorPressed: isDark ? '#5d89e8' : '#315aa9',
      textColorBase: isDark ? '#f1f3f6' : '#20242b',
      textColor1: isDark ? '#f1f3f6' : '#20242b',
      textColor2: isDark ? '#b8bec9' : '#59616d',
      textColor3: isDark ? '#89919e' : '#7d8590',
      placeholderColor: isDark ? '#727b88' : '#969da6',
      borderColor: isDark ? '#343942' : '#d9dfe7',
      dividerColor: isDark ? '#272c34' : '#e8ecf1',
      cardColor: isDark ? '#1e2229' : '#ffffff',
      modalColor: isDark ? '#1e2229' : '#ffffff',
      popoverColor: isDark ? '#1e2229' : '#ffffff',
      tableColor: isDark ? '#181c22' : '#ffffff',
      bodyColor: isDark ? '#151619' : '#f4f5f7',
      actionColor: isDark ? 'rgba(91, 140, 255, 0.1)' : 'rgba(65, 111, 202, 0.07)',
      hoverColor: isDark ? 'rgba(91, 140, 255, 0.08)' : 'rgba(65, 111, 202, 0.05)',
      closeIconColor: isDark ? '#89919e' : '#7d8590',
      closeIconColorHover: isDark ? '#f1f3f6' : '#20242b',
      closeColorHover: isDark ? 'rgba(255, 255, 255, 0.08)' : 'rgba(32, 36, 43, 0.06)',
      scrollbarColor: isDark ? '#444b57' : '#c5ccd6',
      scrollbarColorHover: isDark ? '#5c6574' : '#aab4c2',
      boxShadow1: isDark ? '0 12px 28px rgba(0, 0, 0, 0.24)' : '0 10px 24px rgba(32, 36, 43, 0.07)',
      boxShadow2: isDark ? '0 18px 40px rgba(0, 0, 0, 0.32)' : '0 18px 38px rgba(32, 36, 43, 0.1)',
      borderRadius: '8px',
      fontFamily: "'Manrope', 'Noto Sans SC', sans-serif",
      fontFamilyMono: "'JetBrains Mono', 'Cascadia Code', 'Consolas', monospace",
    },
    Card: {
      color: isDark ? '#1e2229' : '#ffffff',
      colorEmbedded: isDark ? '#191e25' : '#f7f8fa',
      borderColor: isDark ? '#343942' : '#d9dfe7',
    },
    DataTable: {
      thColor: isDark ? '#222730' : '#f0f3f7',
      tdColor: isDark ? '#181c22' : '#ffffff',
      tdColorHover: isDark ? 'rgba(91, 140, 255, 0.06)' : 'rgba(65, 111, 202, 0.04)',
      tdColorStriped: isDark ? 'rgba(255, 255, 255, 0.014)' : '#fafbfc',
      borderColor: isDark ? '#272c34' : '#e8ecf1',
    },
    Modal: {
      color: isDark ? '#1e2229' : '#ffffff',
      borderRadius: '10px',
    },
    Dialog: {
      color: isDark ? '#1e2229' : '#ffffff',
      borderRadius: '10px',
    },
    Input: {
      color: isDark ? '#111516' : '#ffffff',
      colorFocus: isDark ? '#111516' : '#ffffff',
      colorDisabled: isDark ? '#151a1b' : '#f0efeb',
      border: isDark ? '1px solid #313a38' : `1px solid ${lightFieldBorder}`,
      borderHover: isDark ? '1px solid #5b8cff' : `1px solid ${lightFieldBorderHover}`,
      borderFocus: isDark ? '1px solid #5b8cff' : `1px solid ${lightFieldBorderFocus}`,
      boxShadowFocus: isDark
        ? '0 0 0 3px rgba(91, 140, 255, 0.18)'
        : '0 0 0 3px rgba(65, 111, 202, 0.14)',
    },
    InputNumber: {
      buttonColor: isDark ? '#111516' : '#ffffff',
      buttonColorHover: isDark ? '#202727' : '#f8f6f1',
      buttonBorder: isDark ? '1px solid #313a38' : `1px solid ${lightFieldBorder}`,
      buttonBorderHover: isDark ? '1px solid #5b8cff' : `1px solid ${lightFieldBorderHover}`,
      peers: {
        Input: {
          color: isDark ? '#111516' : '#ffffff',
          colorFocus: isDark ? '#111516' : '#ffffff',
          border: isDark ? '1px solid #313a38' : `1px solid ${lightFieldBorder}`,
          borderHover: isDark ? '1px solid #5b8cff' : `1px solid ${lightFieldBorderHover}`,
          borderFocus: isDark ? '1px solid #5b8cff' : `1px solid ${lightFieldBorderFocus}`,
        },
      },
    },
    Select: {
      peers: {
        InternalSelection: {
          color: isDark ? '#111516' : '#ffffff',
          colorActive: isDark ? '#111516' : '#ffffff',
          border: isDark ? '1px solid #313a38' : `1px solid ${lightFieldBorder}`,
          borderHover: isDark ? '1px solid #5b8cff' : `1px solid ${lightFieldBorderHover}`,
          borderFocus: isDark ? '1px solid #5b8cff' : `1px solid ${lightFieldBorderFocus}`,
          boxShadowFocus: isDark
            ? '0 0 0 3px rgba(91, 140, 255, 0.18)'
            : '0 0 0 3px rgba(65, 111, 202, 0.14)',
        },
      },
    },
    DatePicker: {
      peers: {
        Input: {
          color: isDark ? '#111516' : '#ffffff',
          colorFocus: isDark ? '#111516' : '#ffffff',
          border: isDark ? '1px solid #313a38' : `1px solid ${lightFieldBorder}`,
          borderHover: isDark ? '1px solid #5b8cff' : `1px solid ${lightFieldBorderHover}`,
          borderFocus: isDark ? '1px solid #5b8cff' : `1px solid ${lightFieldBorderFocus}`,
        },
        InternalSelection: {
          color: isDark ? '#111516' : '#ffffff',
          colorActive: isDark ? '#111516' : '#ffffff',
          border: isDark ? '1px solid #313a38' : `1px solid ${lightFieldBorder}`,
          borderHover: isDark ? '1px solid #5b8cff' : `1px solid ${lightFieldBorderHover}`,
          borderFocus: isDark ? '1px solid #5b8cff' : `1px solid ${lightFieldBorderFocus}`,
        },
      },
    },
  }
})

const syncDocumentTheme = (value: 'light' | 'dark') => {
  document.documentElement.dataset.theme = value
  document.documentElement.style.colorScheme = value
  document.body.dataset.theme = value
  document.body.style.colorScheme = value
}

watch(theme, (value) => {
  syncDocumentTheme(value)
}, { immediate: true })

provide('theme', theme)
provide('toggleTheme', toggleTheme)

// ====== 关闭确认 ======
const showCloseConfirm = ref(false)
const showCloseBlocked = ref(false)
const closeBlockedMessage = ref('')
const closeProcessing = ref(false)
let closeConfirmResolve: ((confirmed: boolean) => void) | null = null
const closeConfirmContent = computed(() => deployStore.isRunning
  ? '当前部署尚未完成。退出前将先取消部署并等待远端任务停止。'
  : '确定要退出程序吗？')

const onCloseConfirmOk = () => {
  closeConfirmResolve?.(true)
  closeConfirmResolve = null
  showCloseConfirm.value = false
}

const onCloseConfirmCancel = () => {
  closeConfirmResolve?.(false)
  closeConfirmResolve = null
  showCloseConfirm.value = false
}

const waitForDeployToStop = async () => {
  if (!deployStore.isRunning) return
  await new Promise<void>((resolve) => {
    const stop = watch(() => deployStore.isRunning, (running) => {
      if (!running) {
        stop()
        resolve()
      }
    })
  })
}

onMounted(async () => {
  const win = getCurrentWindow()
  await win.onCloseRequested(async (event) => {
    event.preventDefault()
    if (closeProcessing.value || showCloseConfirm.value) return

    if (deployStore.isRunning && deployStore.deployStepStatus === 'committing') {
      closeBlockedMessage.value = '发布正在提交到线上目录，当前不能退出。请等待提交完成。'
      showCloseBlocked.value = true
      return
    }

    showCloseConfirm.value = true
    const confirmed = await new Promise<boolean>((resolve) => {
      closeConfirmResolve = resolve
    })
    if (!confirmed) return

    closeProcessing.value = true
    try {
      if (deployStore.isRunning) {
        const result = await deployStore.cancelDeploy()
        if (result === 'too_late') {
          closeBlockedMessage.value = '发布已经进入提交阶段，当前不能退出。请等待提交完成。'
          showCloseBlocked.value = true
          return
        }
        await waitForDeployToStop()
      }
      await win.destroy()
    } catch (error) {
      closeBlockedMessage.value = `无法安全退出：${String(error)}`
      showCloseBlocked.value = true
    } finally {
      closeProcessing.value = false
    }
  })
})

// ====== 已签名的桌面端更新 ======
const availableUpdate = ref<Update | null>(null)
const showUpdateDialog = ref(false)
const updateInstalling = ref(false)
const updateInstalled = ref(false)
const updateProgress = ref(0)
const updateError = ref('')
let updateCheckTimer: ReturnType<typeof setTimeout> | null = null

const checkForDesktopUpdate = async () => {
  try {
    const configured = await invoke<boolean>('is_updater_configured')
    if (!configured) return
    const update = await check({ timeout: 15_000 })
    if (update) {
      availableUpdate.value = update
      showUpdateDialog.value = true
    }
  } catch (error) {
    console.warn('Desktop update check failed', error)
  }
}

const installDesktopUpdate = async () => {
  const update = availableUpdate.value
  if (!update || updateInstalling.value) return
  if (deployStore.isRunning) {
    updateError.value = '部署进行中，完成后才能安装更新。'
    return
  }
  updateInstalling.value = true
  updateError.value = ''
  updateProgress.value = 0
  let downloaded = 0
  let total = 0
  try {
    await update.downloadAndInstall((event) => {
      if (event.event === 'Started') {
        total = event.data.contentLength || 0
      } else if (event.event === 'Progress') {
        downloaded += event.data.chunkLength
        updateProgress.value = total ? Math.min(99, Math.round(downloaded / total * 100)) : 0
      } else if (event.event === 'Finished') {
        updateProgress.value = 100
      }
    })
    updateInstalled.value = true
  } catch (error) {
    updateError.value = String(error)
  } finally {
    updateInstalling.value = false
  }
}

const restartAfterUpdate = async () => {
  if (deployStore.isRunning) {
    updateError.value = '部署进行中，完成后才能重启。'
    return
  }
  await relaunch()
}

onMounted(() => {
  updateCheckTimer = setTimeout(checkForDesktopUpdate, 5_000)
})

onUnmounted(() => {
  if (updateCheckTimer) clearTimeout(updateCheckTimer)
  availableUpdate.value?.close()
})

// ====== 可调整宽度的侧边栏 ======
const SIDEBAR_MIN = 200
const SIDEBAR_MAX = 280
const SIDEBAR_DEFAULT = 240

const sidebarWidth = ref(
  Math.min(SIDEBAR_MAX, Math.max(SIDEBAR_MIN, Number(localStorage.getItem('sidebarWidth')) || SIDEBAR_DEFAULT))
)
watch(sidebarWidth, (val) => {
  localStorage.setItem('sidebarWidth', String(val))
})

const startSidebarResize = (e: MouseEvent) => {
  e.preventDefault()
  const startX = e.clientX
  const startWidth = sidebarWidth.value

  const onMove = (ev: MouseEvent) => {
    const newWidth = Math.min(SIDEBAR_MAX, Math.max(SIDEBAR_MIN, startWidth + ev.clientX - startX))
    sidebarWidth.value = newWidth
  }

  const onUp = () => {
    document.removeEventListener('mousemove', onMove)
    document.removeEventListener('mouseup', onUp)
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
  }

  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
  document.addEventListener('mousemove', onMove)
  document.addEventListener('mouseup', onUp)
}
</script>

<template>
  <n-config-provider :theme="naiveTheme" :theme-overrides="themeOverrides" :locale="zhCN" :date-locale="dateZhCN">
    <n-message-provider>
      <n-notification-provider>
        <n-dialog-provider>
          <div class="app-layout" :class="`theme-${theme}`">
            <div class="app-body">
              <ProjectSidebar :style="{ width: sidebarWidth + 'px', minWidth: sidebarWidth + 'px' }" />
              <div class="sidebar-resize-handle" @mousedown="startSidebarResize" />
              <div class="main-area">
                <router-view v-slot="{ Component }">
                  <transition name="page" mode="out-in">
                    <component :is="Component" />
                  </transition>
                </router-view>
              </div>
            </div>
            <TerminalLog />
            <button class="theme-toggle" @click="toggleTheme" :title="theme === 'dark' ? '切换到亮色主题' : '切换到暗色主题'">
              <svg v-if="theme === 'dark'" viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" aria-hidden="true">
                <circle cx="12" cy="12" r="4" />
                <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41" />
              </svg>
              <svg v-else viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                <path d="M20.6 15.3A8.4 8.4 0 0 1 8.7 3.4 8.4 8.4 0 1 0 20.6 15.3Z" />
              </svg>
            </button>
          </div>
        </n-dialog-provider>
      </n-notification-provider>
    </n-message-provider>

    <!-- 关闭确认弹窗（放在 config-provider 内以继承主题） -->
    <n-modal
      v-model:show="showCloseConfirm"
      title="确认退出"
      preset="dialog"
      type="warning"
      :content="closeConfirmContent"
      positive-text="确定退出"
      negative-text="取消"
      :mask-closable="false"
      @positive-click="onCloseConfirmOk"
      @negative-click="onCloseConfirmCancel"
      @close="onCloseConfirmCancel"
    />

    <n-modal
      v-model:show="showCloseBlocked"
      preset="dialog"
      type="warning"
      title="暂时无法退出"
      :content="closeBlockedMessage"
      positive-text="我知道了"
      :mask-closable="false"
    />

    <n-modal
      v-model:show="showUpdateDialog"
      preset="card"
      title="发现桌面应用更新"
      style="width: min(480px, calc(100vw - 32px))"
      :mask-closable="!updateInstalling"
      :closable="!updateInstalling"
    >
      <div class="updater-content">
        <div class="updater-version">
          {{ availableUpdate?.currentVersion }} → {{ availableUpdate?.version }}
        </div>
        <p v-if="availableUpdate?.body" class="updater-notes">{{ availableUpdate.body }}</p>
        <div v-if="updateInstalling" class="updater-progress">
          <div class="updater-progress-track">
            <span :style="{ width: `${updateProgress}%` }" />
          </div>
          <span>{{ updateProgress ? `${updateProgress}%` : '正在下载并验证更新...' }}</span>
        </div>
        <p v-if="updateError" class="updater-error">更新失败，当前版本未被替换：{{ updateError }}</p>
        <p v-if="updateInstalled" class="updater-success">更新已安装，重启后生效。</p>
      </div>
      <template #footer>
        <div class="updater-actions">
          <n-button v-if="!updateInstalled" :disabled="updateInstalling" @click="showUpdateDialog = false">稍后</n-button>
          <n-button
            v-if="!updateInstalled"
            type="primary"
            :loading="updateInstalling"
            :disabled="deployStore.isRunning"
            @click="installDesktopUpdate"
          >下载并安装</n-button>
          <n-button v-else type="primary" :disabled="deployStore.isRunning" @click="restartAfterUpdate">立即重启</n-button>
        </div>
      </template>
    </n-modal>
  </n-config-provider>
</template>

<style>
* { box-sizing: border-box; }

html, body {
  margin: 0;
  padding: 0;
  height: 100%;
  overflow: hidden;
  font-family: 'Manrope', 'Noto Sans SC', sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

:root {
  --content-padding-x: clamp(12px, 3.4vw, 24px);
  --content-padding-y: clamp(12px, 2.4vw, 24px);
  --grid-gap: clamp(12px, 1.6vw, 24px);
  --content-max-width: 1400px;
  --terminal-min-height: 120px;
  --terminal-max-height: 500px;
  --terminal-default-height: 200px;
  --radius-xs: 4px;
  --radius-sm: 8px;
  --radius-md: 12px;
  --radius-lg: 18px;
  --radius-xl: 24px;
  --motion-fast: 0.18s ease;
  --motion-base: 0.24s ease;
}

:root[data-theme='dark'], .theme-dark {
  --bg-primary: #0d1117;
  --bg-secondary: #11161f;
  --bg-elevated: #151b24;
  --bg-card: #18202c;
  --bg-card-hover: #1d2735;
  --bg-soft: rgba(255, 255, 255, 0.03);
  --bg-panel: rgba(17, 22, 31, 0.88);
  --bg-panel-strong: rgba(21, 27, 36, 0.96);
  --bg-code: #0b1017;
  --bg-chip: rgba(88, 166, 255, 0.12);
  --bg-overlay: rgba(2, 6, 12, 0.72);
  --border-color: #2d3645;
  --border-subtle: #222b36;
  --text-primary: #e6edf3;
  --text-secondary: #b6c2cf;
  --text-muted: #7b8694;
  --accent-blue: #58a6ff;
  --accent-blue-rgb: 88, 166, 255;
  --accent-green: #3fb950;
  --accent-green-rgb: 63, 185, 80;
  --accent-red: #f85149;
  --accent-red-rgb: 248, 81, 73;
  --accent-yellow: #d29922;
  --accent-yellow-rgb: 210, 153, 34;
  --accent-purple: #bc8cff;
  --app-gradient:
    radial-gradient(circle at top right, rgba(88, 166, 255, 0.12), transparent 34%),
    radial-gradient(circle at 10% 10%, rgba(188, 140, 255, 0.08), transparent 28%),
    linear-gradient(180deg, rgba(13, 17, 23, 0.98), rgba(13, 17, 23, 0.92));
  --sidebar-bg: #0d1117;
  --sidebar-hover: rgba(255, 255, 255, 0.04);
  --sidebar-active: rgba(88, 166, 255, 0.1);
  --sidebar-active-border: #58a6ff;
  --terminal-bg: #0a0f16;
  --scrollbar-track: #11161f;
  --scrollbar-thumb: #3a4555;
  --theme-toggle-bg: rgba(21, 27, 36, 0.88);
  --theme-toggle-border: rgba(88, 166, 255, 0.14);
  --theme-toggle-color: #c9d1d9;
  --shadow-sm: 0 10px 24px rgba(0, 0, 0, 0.16);
  --shadow-md: 0 18px 42px rgba(0, 0, 0, 0.24);
  --shadow-lg: 0 28px 64px rgba(0, 0, 0, 0.34);
}

:root[data-theme='light'], .theme-light {
  --bg-primary: #f5f7fb;
  --bg-secondary: #eef2f7;
  --bg-elevated: #ffffff;
  --bg-card: #ffffff;
  --bg-card-hover: #f8fbff;
  --bg-soft: rgba(15, 23, 42, 0.03);
  --bg-panel: rgba(255, 255, 255, 0.84);
  --bg-panel-strong: rgba(255, 255, 255, 0.98);
  --bg-code: #f4f7fb;
  --bg-chip: rgba(9, 105, 218, 0.08);
  --bg-overlay: rgba(15, 23, 42, 0.08);
  --border-color: #d0d7de;
  --border-subtle: #e5e7eb;
  --text-primary: #1f2328;
  --text-secondary: #4b5563;
  --text-muted: #8c959f;
  --accent-blue: #0969da;
  --accent-blue-rgb: 9, 105, 218;
  --accent-green: #1f883d;
  --accent-green-rgb: 31, 136, 61;
  --accent-red: #cf222e;
  --accent-red-rgb: 207, 34, 46;
  --accent-yellow: #9a6700;
  --accent-yellow-rgb: 154, 103, 0;
  --accent-purple: #8250df;
  --app-gradient:
    radial-gradient(circle at top left, rgba(9, 105, 218, 0.1), transparent 32%),
    radial-gradient(circle at 88% 5%, rgba(130, 80, 223, 0.08), transparent 30%),
    linear-gradient(180deg, rgba(245, 247, 251, 0.98), rgba(241, 245, 249, 0.92));
  --sidebar-bg: #f8fafc;
  --sidebar-hover: rgba(9, 105, 218, 0.06);
  --sidebar-active: rgba(9, 105, 218, 0.1);
  --sidebar-active-border: #0969da;
  --terminal-bg: #f7f9fc;
  --scrollbar-track: #eef2f7;
  --scrollbar-thumb: #c7d0db;
  --theme-toggle-bg: rgba(255, 255, 255, 0.9);
  --theme-toggle-border: rgba(9, 105, 218, 0.14);
  --theme-toggle-color: #1f2328;
  --shadow-sm: 0 8px 24px rgba(15, 23, 42, 0.06);
  --shadow-md: 0 18px 36px rgba(15, 23, 42, 0.1);
  --shadow-lg: 0 26px 60px rgba(15, 23, 42, 0.12);
}

html, body, #app, .theme-dark, .theme-light {
  background: var(--bg-primary);
  color: var(--text-primary);
}

body {
  transition: background-color var(--motion-base), color var(--motion-base);
}

#app { height: 100%; }

.app-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  position: relative;
  background: var(--app-gradient);
  transition: background var(--motion-base), color var(--motion-base);
}

.app-body {
  display: flex;
  flex-direction: row;
  flex: 1;
  overflow: hidden;
}

.main-area {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  background: transparent;
  min-width: 0;
}

.theme-toggle {
  position: fixed;
  bottom: 16px;
  right: 16px;
  z-index: 9999;
  width: 42px;
  height: 42px;
  border-radius: 14px;
  border: 1px solid var(--theme-toggle-border);
  background: var(--theme-toggle-bg);
  color: var(--theme-toggle-color);
  font-size: 17px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0.9;
  transition: opacity var(--motion-fast), transform var(--motion-fast), box-shadow var(--motion-fast), background var(--motion-fast), border-color var(--motion-fast);
  box-shadow: var(--shadow-md);
  backdrop-filter: blur(14px);
}

.theme-toggle:hover {
  opacity: 1;
  transform: translateY(-2px);
  box-shadow: var(--shadow-lg);
}

.theme-toggle:focus-visible {
  outline: none;
  box-shadow: 0 0 0 3px rgba(var(--accent-blue-rgb), 0.18), var(--shadow-lg);
}

/* 侧边栏宽度调整手柄 */
.sidebar-resize-handle {
  width: 4px;
  cursor: col-resize;
  flex-shrink: 0;
  position: relative;
  z-index: 5;
  transition: background 0.15s;
  background: transparent;
}
.sidebar-resize-handle:hover,
.sidebar-resize-handle:active {
  background: var(--accent-blue);
  opacity: 0.4;
}
.sidebar-resize-handle::after {
  content: '';
  position: absolute;
  top: 0;
  bottom: 0;
  left: -3px;
  right: -3px;
}

.updater-content {
  display: grid;
  gap: 12px;
}

.updater-version {
  font: 600 14px/1.4 'JetBrains Mono', monospace;
  color: var(--accent-blue);
}

.updater-notes,
.updater-error,
.updater-success {
  margin: 0;
  white-space: pre-wrap;
  line-height: 1.6;
  color: var(--text-secondary);
}

.updater-error { color: var(--accent-red); }
.updater-success { color: var(--accent-green); }

.updater-progress {
  display: grid;
  gap: 6px;
  font-size: 12px;
  color: var(--text-muted);
}

.updater-progress-track {
  height: 6px;
  overflow: hidden;
  border-radius: 3px;
  background: var(--border-subtle);
}

.updater-progress-track span {
  display: block;
  height: 100%;
  background: var(--accent-blue);
  transition: width var(--motion-fast);
}

.updater-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

::-webkit-scrollbar { width: 8px; height: 8px; }
::-webkit-scrollbar-track { background: var(--scrollbar-track); }
::-webkit-scrollbar-thumb { background: var(--scrollbar-thumb); border-radius: 4px; }
::-webkit-scrollbar-thumb:hover { background: var(--text-muted); }

/* 浅色主题下，白色弹窗中的表单控件需要更明显的边缘对比度。 */
/* 使用 Naive UI 主题覆盖，避免通过自定义 box-shadow 进行补丁式处理。 */

/* 页面布局工具类 */
.page-container {
  padding: var(--content-padding-y) var(--content-padding-x);
  height: 100%;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: var(--grid-gap);
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-shrink: 0;
  gap: 12px;
  flex-wrap: wrap;
}

.page-header h2 {
  margin: 0;
  font-size: clamp(18px, 2vw, 22px);
  font-weight: 600;
  white-space: nowrap;
}

/* 面板卡片工具类 */
.panel-card {
  background: var(--bg-panel);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-sm);
  backdrop-filter: blur(12px);
  transition: box-shadow var(--motion-fast), transform var(--motion-fast), border-color var(--motion-fast);
}

/* 页面过渡 */
.page-enter-active { transition: opacity 260ms cubic-bezier(0.22, 1, 0.36, 1), transform 260ms cubic-bezier(0.22, 1, 0.36, 1); }
.page-leave-active { transition: opacity 180ms cubic-bezier(0.4, 0, 1, 1), transform 180ms cubic-bezier(0.4, 0, 1, 1); }
.page-enter-from { opacity: 0; transform: translateY(8px); }
.page-leave-to { opacity: 0; transform: translateY(-4px); }

/* 卡片入场动画 */
@keyframes card-in {
  from { opacity: 0; transform: translateY(12px); }
  to   { opacity: 1; transform: translateY(0); }
}
.animate-in { animation: card-in 0.35s ease both; }
.animate-in-d1 { animation-delay: 0.05s; }
.animate-in-d2 { animation-delay: 0.10s; }
.animate-in-d3 { animation-delay: 0.15s; }
.animate-in-d4 { animation-delay: 0.20s; }

/* 日志行入场动画 */
@keyframes log-in {
  from { opacity: 0; transform: translateX(-6px); }
  to   { opacity: 1; transform: translateX(0); }
}
</style>

<style>
/* Palette v2: editorial ink, porcelain, and vermilion. */
:root[data-theme='dark'], .theme-dark {
  --bg-primary: #121617;
  --bg-secondary: #171b1c;
  --bg-elevated: #1b2221;
  --bg-card: #202727;
  --bg-card-hover: #29302f;
  --bg-soft: rgba(242, 240, 235, 0.048);
  --bg-panel: rgba(27, 34, 33, 0.94);
  --bg-panel-strong: rgba(22, 27, 28, 0.98);
  --bg-code: #0c1011;
  --bg-chip: rgba(242, 118, 92, 0.14);
  --bg-overlay: rgba(7, 9, 10, 0.76);
  --border-color: #313a38;
  --border-subtle: #242c2a;
  --text-primary: #f2f0eb;
  --text-secondary: #b8bfbc;
  --text-muted: #89928f;
  --accent-blue: #f2765c;
  --accent-blue-rgb: 242, 118, 92;
  --accent-green: #64cbd4;
  --accent-green-rgb: 100, 203, 212;
  --accent-red: #ff7b72;
  --accent-red-rgb: 255, 123, 114;
  --accent-yellow: #e8b45f;
  --accent-yellow-rgb: 232, 180, 95;
  --accent-purple: #5a8dee;
  --app-gradient: #121617;
  --sidebar-bg: #161a1b;
  --sidebar-hover: rgba(242, 240, 235, 0.05);
  --sidebar-active: rgba(242, 118, 92, 0.15);
  --sidebar-active-border: #f2765c;
  --terminal-bg: #0b0f10;
  --scrollbar-track: #151a1a;
  --scrollbar-thumb: #414b48;
  --theme-toggle-bg: #202727;
  --theme-toggle-border: #39423f;
  --theme-toggle-color: #e0ddd6;
  --shadow-sm: 0 8px 20px rgba(0, 0, 0, 0.2);
  --shadow-md: 0 16px 36px rgba(0, 0, 0, 0.26);
  --shadow-lg: 0 24px 56px rgba(0, 0, 0, 0.32);
}

:root[data-theme='light'], .theme-light {
  --bg-primary: #f4f3ee;
  --bg-secondary: #ebeae5;
  --bg-elevated: #fffdfa;
  --bg-card: #fffdfa;
  --bg-card-hover: #f8f6f1;
  --bg-soft: rgba(32, 39, 37, 0.04);
  --bg-panel: rgba(255, 253, 250, 0.96);
  --bg-panel-strong: #fffdfa;
  --bg-code: #efeee8;
  --bg-chip: rgba(201, 87, 63, 0.1);
  --bg-overlay: rgba(32, 39, 37, 0.1);
  --border-color: #d9dfdb;
  --border-subtle: #e8edea;
  --text-primary: #202725;
  --text-secondary: #58615d;
  --text-muted: #7c8581;
  --accent-blue: #c9573f;
  --accent-blue-rgb: 201, 87, 63;
  --accent-green: #238896;
  --accent-green-rgb: 35, 136, 150;
  --accent-red: #c94843;
  --accent-red-rgb: 201, 72, 67;
  --accent-yellow: #ae6a18;
  --accent-yellow-rgb: 174, 106, 24;
  --accent-purple: #3f6fca;
  --app-gradient: #f4f3ee;
  --sidebar-bg: #faf8f4;
  --sidebar-hover: rgba(201, 87, 63, 0.055);
  --sidebar-active: rgba(201, 87, 63, 0.1);
  --sidebar-active-border: #c9573f;
  --terminal-bg: #eeece6;
  --scrollbar-track: #ecebe6;
  --scrollbar-thumb: #c5cbc7;
  --theme-toggle-bg: #fffdfa;
  --theme-toggle-border: #d9dfdb;
  --theme-toggle-color: #202725;
  --shadow-sm: 0 8px 20px rgba(32, 39, 37, 0.06);
  --shadow-md: 0 16px 32px rgba(32, 39, 37, 0.09);
  --shadow-lg: 0 24px 54px rgba(32, 39, 37, 0.12);
}

.app-layout {
  background-image:
    linear-gradient(rgba(var(--accent-blue-rgb), 0.028) 1px, transparent 1px),
    linear-gradient(90deg, rgba(var(--accent-blue-rgb), 0.028) 1px, transparent 1px);
}

.main-area::before {
  border-top-color: rgba(var(--accent-blue-rgb), 0.4);
}
</style>

<style>
/* Unified deployment-console visual language. */
:root {
  --content-padding-x: clamp(18px, 2.8vw, 34px);
  --content-padding-y: clamp(18px, 2.6vw, 30px);
  --grid-gap: clamp(16px, 1.8vw, 24px);
  --radius-xs: 3px;
  --radius-sm: 6px;
  --radius-md: 8px;
  --radius-lg: 10px;
  --radius-xl: 12px;
  --motion-fast: 180ms cubic-bezier(0.22, 1, 0.36, 1);
  --motion-base: 240ms cubic-bezier(0.22, 1, 0.36, 1);
}

:root[data-theme='dark'], .theme-dark {
  --bg-primary: #0e1410;
  --bg-secondary: #121a15;
  --bg-elevated: #151d18;
  --bg-card: #18211b;
  --bg-card-hover: #1c2921;
  --bg-soft: rgba(231, 244, 234, 0.045);
  --bg-panel: rgba(20, 29, 24, 0.93);
  --bg-panel-strong: rgba(18, 26, 21, 0.98);
  --bg-code: #0b110d;
  --bg-chip: rgba(23, 185, 120, 0.12);
  --bg-overlay: rgba(5, 10, 7, 0.76);
  --border-color: #2a3830;
  --border-subtle: #202c26;
  --text-primary: #edf4ee;
  --text-secondary: #bac8bc;
  --text-muted: #849388;
  --accent-blue: #17b978;
  --accent-blue-rgb: 23, 185, 120;
  --accent-green: #5bd89c;
  --accent-green-rgb: 91, 216, 156;
  --accent-red: #ff7770;
  --accent-red-rgb: 255, 119, 112;
  --accent-yellow: #f0b25f;
  --accent-yellow-rgb: 240, 178, 95;
  --accent-purple: #4dd4bd;
  --app-gradient: #0e1410;
  --sidebar-bg: #101711;
  --sidebar-hover: rgba(237, 244, 238, 0.045);
  --sidebar-active: rgba(23, 185, 120, 0.13);
  --sidebar-active-border: #17b978;
  --terminal-bg: #09100b;
  --scrollbar-track: #111913;
  --scrollbar-thumb: #3a4a3e;
  --theme-toggle-bg: #18211b;
  --theme-toggle-border: #2a3830;
  --theme-toggle-color: #c6d3c8;
  --shadow-sm: 0 8px 20px rgba(0, 0, 0, 0.18);
  --shadow-md: 0 16px 36px rgba(0, 0, 0, 0.24);
  --shadow-lg: 0 24px 56px rgba(0, 0, 0, 0.3);
}

:root[data-theme='light'], .theme-light {
  --bg-primary: #f4f7f3;
  --bg-secondary: #edf2ed;
  --bg-elevated: #ffffff;
  --bg-card: #ffffff;
  --bg-card-hover: #f6faf6;
  --bg-soft: rgba(24, 35, 30, 0.035);
  --bg-panel: rgba(255, 255, 255, 0.95);
  --bg-panel-strong: #ffffff;
  --bg-code: #f0f5f0;
  --bg-chip: rgba(8, 127, 91, 0.09);
  --bg-overlay: rgba(24, 35, 30, 0.1);
  --border-color: #d7dfd8;
  --border-subtle: #e5ebe6;
  --text-primary: #18231e;
  --text-secondary: #4c5d52;
  --text-muted: #718077;
  --accent-blue: #087f5b;
  --accent-blue-rgb: 8, 127, 91;
  --accent-green: #17875a;
  --accent-green-rgb: 23, 135, 90;
  --accent-red: #c43c36;
  --accent-red-rgb: 196, 60, 54;
  --accent-yellow: #a65d0a;
  --accent-yellow-rgb: 166, 93, 10;
  --accent-purple: #087f76;
  --app-gradient: #f4f7f3;
  --sidebar-bg: #f8faf7;
  --sidebar-hover: rgba(8, 127, 91, 0.055);
  --sidebar-active: rgba(8, 127, 91, 0.1);
  --sidebar-active-border: #087f5b;
  --terminal-bg: #f0f4f0;
  --scrollbar-track: #edf2ed;
  --scrollbar-thumb: #c3cec5;
  --theme-toggle-bg: #ffffff;
  --theme-toggle-border: #d7dfd8;
  --theme-toggle-color: #18231e;
  --shadow-sm: 0 8px 20px rgba(24, 35, 30, 0.06);
  --shadow-md: 0 16px 32px rgba(24, 35, 30, 0.09);
  --shadow-lg: 0 24px 54px rgba(24, 35, 30, 0.12);
}

html, body {
  font-family: 'Manrope', 'Noto Sans SC', sans-serif;
}

.app-layout {
  background-color: var(--bg-primary);
  background-image:
    linear-gradient(rgba(var(--accent-blue-rgb), 0.035) 1px, transparent 1px),
    linear-gradient(90deg, rgba(var(--accent-blue-rgb), 0.035) 1px, transparent 1px);
  background-size: 28px 28px;
}

.main-area {
  position: relative;
}

.main-area::before {
  content: '';
  position: absolute;
  inset: 0;
  pointer-events: none;
  border-top: 2px solid rgba(var(--accent-blue-rgb), 0.26);
}

.page-container {
  max-width: var(--content-max-width);
  width: 100%;
  margin: 0 auto;
}

.page-header h2 {
  font-size: 20px;
  font-weight: 750;
  letter-spacing: 0;
}

.panel-card {
  border-radius: var(--radius-lg);
  box-shadow: none;
  backdrop-filter: none;
}

.n-button {
  font-weight: 650;
  letter-spacing: 0;
}

.n-tag {
  font-weight: 650;
  border-radius: 4px;
}

.theme-toggle {
  width: 36px;
  height: 36px;
  right: 18px;
  bottom: 18px;
  border-radius: 7px;
  box-shadow: var(--shadow-sm);
}

.theme-toggle:hover {
  transform: none;
  border-color: var(--accent-blue);
}

.sidebar-resize-handle {
  width: 1px;
  background: var(--border-subtle);
}

.sidebar-resize-handle:hover,
.sidebar-resize-handle:active {
  opacity: 1;
  background: var(--accent-blue);
}

.updater-progress-track,
.updater-progress-track span {
  border-radius: 2px;
}

@media (max-width: 900px) {
  .page-container {
    padding: 16px;
  }
}
</style>

<style>
/* Final palette layer: graphite, cobalt, ice cyan, and amber. */
:root[data-theme='dark'], .theme-dark {
  --bg-primary: #151619;
  --bg-secondary: #1a1c21;
  --bg-elevated: #1e2229;
  --bg-card: #252a32;
  --bg-card-hover: #2d333d;
  --bg-soft: rgba(241, 243, 246, 0.048);
  --bg-panel: rgba(30, 34, 41, 0.94);
  --bg-panel-strong: rgba(24, 28, 34, 0.98);
  --bg-code: #0e1014;
  --bg-chip: rgba(91, 140, 255, 0.14);
  --bg-overlay: rgba(8, 10, 13, 0.76);
  --border-color: #343942;
  --border-subtle: #272c34;
  --text-primary: #f1f3f6;
  --text-secondary: #b8bec9;
  --text-muted: #89919e;
  --accent-blue: #5b8cff;
  --accent-blue-rgb: 91, 140, 255;
  --accent-green: #63cfe0;
  --accent-green-rgb: 99, 207, 224;
  --accent-red: #ff7b72;
  --accent-red-rgb: 255, 123, 114;
  --accent-yellow: #f1b95a;
  --accent-yellow-rgb: 241, 185, 90;
  --accent-purple: #7aa4ff;
  --app-gradient: #151619;
  --sidebar-bg: #191b20;
  --sidebar-hover: rgba(241, 243, 246, 0.05);
  --sidebar-active: rgba(91, 140, 255, 0.16);
  --sidebar-active-border: #5b8cff;
  --terminal-bg: #0d0f13;
  --scrollbar-track: #171a1f;
  --scrollbar-thumb: #444b57;
  --theme-toggle-bg: #252a32;
  --theme-toggle-border: #3b424e;
  --theme-toggle-color: #e1e5eb;
  --shadow-sm: 0 8px 20px rgba(0, 0, 0, 0.2);
  --shadow-md: 0 16px 36px rgba(0, 0, 0, 0.26);
  --shadow-lg: 0 24px 56px rgba(0, 0, 0, 0.32);
}

:root[data-theme='light'], .theme-light {
  --bg-primary: #f4f5f7;
  --bg-secondary: #edf0f4;
  --bg-elevated: #ffffff;
  --bg-card: #ffffff;
  --bg-card-hover: #f7f9fc;
  --bg-soft: rgba(32, 36, 43, 0.04);
  --bg-panel: rgba(255, 255, 255, 0.96);
  --bg-panel-strong: #ffffff;
  --bg-code: #eff2f6;
  --bg-chip: rgba(65, 111, 202, 0.1);
  --bg-overlay: rgba(32, 36, 43, 0.1);
  --border-color: #d9dfe7;
  --border-subtle: #e8ecf1;
  --text-primary: #20242b;
  --text-secondary: #59616d;
  --text-muted: #7d8590;
  --accent-blue: #416fca;
  --accent-blue-rgb: 65, 111, 202;
  --accent-green: #238da3;
  --accent-green-rgb: 35, 141, 163;
  --accent-red: #c94843;
  --accent-red-rgb: 201, 72, 67;
  --accent-yellow: #ad741d;
  --accent-yellow-rgb: 173, 116, 29;
  --accent-purple: #5c85de;
  --app-gradient: #f4f5f7;
  --sidebar-bg: #f8f9fb;
  --sidebar-hover: rgba(65, 111, 202, 0.055);
  --sidebar-active: rgba(65, 111, 202, 0.1);
  --sidebar-active-border: #416fca;
  --terminal-bg: #eef1f5;
  --scrollbar-track: #eceff3;
  --scrollbar-thumb: #c5ccd6;
  --theme-toggle-bg: #ffffff;
  --theme-toggle-border: #d9dfe7;
  --theme-toggle-color: #20242b;
  --shadow-sm: 0 8px 20px rgba(32, 36, 43, 0.06);
  --shadow-md: 0 16px 32px rgba(32, 36, 43, 0.09);
  --shadow-lg: 0 24px 54px rgba(32, 36, 43, 0.12);
}

.app-layout {
  background-image:
    linear-gradient(rgba(var(--accent-blue-rgb), 0.028) 1px, transparent 1px),
    linear-gradient(90deg, rgba(var(--accent-blue-rgb), 0.028) 1px, transparent 1px);
}

.main-area::before {
  border-top-color: rgba(var(--accent-blue-rgb), 0.4);
}
</style>
