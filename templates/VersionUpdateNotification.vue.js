<template>
  <Transition name="slide-up">
    <!-- 当检测到新版本时展示通知 UI -->
    <div v-if="showNotification" class="update-notification">
      <button class="close-btn" @click="handleClose" aria-label="关闭">&times;</button>
      <div class="content-wrapper">
        <div class="icon-wrapper">
          <svg viewBox="0 0 1024 1024" width="16" height="16" fill="currentColor">
            <path d="M136.533333 512a375.466667 375.466667 0 0 1 375.466667-375.466667v-85.333333a42.666667 42.666667 0 0 1 72.533333-30.293333l170.666667 170.666666a42.666667 42.666667 0 0 1 0 60.586667l-170.666667 170.666667A42.666667 42.666667 0 0 1 512 384v-85.333333a290.133333 290.133333 0 1 0 290.133333 290.133333 42.666667 42.666667 0 0 1 85.333334 0A375.466667 375.466667 0 1 1 136.533333 512z"></path>
          </svg>
        </div>
        <div class="text-content">
          <h4 class="title">发现新版本</h4>
          <p class="desc">应用已更新，刷新页面获得最佳体验</p>
          <div class="actions">
            <!-- 用户点击刷新时，重新加载页面获取最新资源 -->
            <button class="btn-primary" @click="handleRefresh">立即刷新</button>
            <button class="btn-default" @click="handleLater">稍后</button>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'

const showNotification = ref(false)

const UPDATE_CHECK_INTERVAL = 1 * 60 * 1000
const VISIBILITY_CHECK_DELAY = 1000
const LATER_REMIND_DELAY = 30 * 60 * 1000
const CLOSE_REMIND_DELAY = 60 * 60 * 1000

let reminderTimer = null
let visibilityCheckTimer = null
let pollTimer = null
let isPolling = false

const currentVersion = ref('')
const isChecking = ref(false)

// 拼接 version.json 的请求路径
const baseUrl = (() => {
  try {
    const viteBase = import.meta.env?.BASE_URL
    if (viteBase) return viteBase
} catch { /* 当前不是 Vite 构建 */ }
  if (typeof document !== 'undefined') {
    const baseEl = document.querySelector('base[href]')
    if (baseEl?.href) return baseEl.href
  }
  return '/'
})()
const versionJsonPath = `${baseUrl.endsWith('/') ? baseUrl : `${baseUrl}/`}version.json`

const handleRefresh = () => {
  window.location.reload()
}

const clearReminderTimer = () => {
  if (reminderTimer) {
    clearTimeout(reminderTimer)
    reminderTimer = null
  }
}

const scheduleReminder = (delay) => {
  clearReminderTimer()
  reminderTimer = setTimeout(() => {
    showNotification.value = true
    reminderTimer = null
  }, delay)
}

const handleLater = () => {
  showNotification.value = false
  scheduleReminder(LATER_REMIND_DELAY)
}

const handleClose = () => {
  showNotification.value = false
  scheduleReminder(CLOSE_REMIND_DELAY)
}

const getCurrentVersion = async () => {
  try {
    const response = await fetch(`${versionJsonPath}?t=${Date.now()}`, {
      method: 'GET',
      headers: {
        'Cache-Control': 'no-cache',
        'Pragma': 'no-cache'
      }
    })
    if (!response.ok) throw new Error(`HTTP ${response.status}`)
    const data = await response.json()
    return data.version || null
  } catch (error) {
    console.error('获取当前版本失败:', error)
    return null
  }
}

const checkUpdate = async () => {
  const version = await getCurrentVersion()
  if (version) currentVersion.value = version
}

const manualCheckUpdate = async () => {
  if (isChecking.value || showNotification.value) return
  isChecking.value = true

  try {
    if (!currentVersion.value) {
      const version = await getCurrentVersion()
      if (!version) return
      currentVersion.value = version
    }
    const versionInfo = await getCurrentVersion()
    if (versionInfo && versionInfo !== currentVersion.value) {
      showNotification.value = true
      stopUpdateCheck()
    }
  } catch (error) {
    console.error('检查更新失败:', error)
  } finally {
    isChecking.value = false
  }
}

const scheduleNextCheck = () => {
  if (!isPolling) return
  if (pollTimer) {
    clearTimeout(pollTimer)
    pollTimer = null
  }
  if (document.hidden) return

  pollTimer = setTimeout(async () => {
    await manualCheckUpdate()
    scheduleNextCheck()
  }, UPDATE_CHECK_INTERVAL)
}

const handleVisibilityChange = () => {
  if (!document.hidden) {
    if (visibilityCheckTimer) clearTimeout(visibilityCheckTimer)
    visibilityCheckTimer = setTimeout(async () => {
      visibilityCheckTimer = null
      await manualCheckUpdate()
      scheduleNextCheck()
    }, VISIBILITY_CHECK_DELAY)
  } else {
    if (pollTimer) {
      clearTimeout(pollTimer)
      pollTimer = null
    }
  }
}

const startUpdateCheck = () => {
  isPolling = true
  scheduleNextCheck()
  document.addEventListener('visibilitychange', handleVisibilityChange)
  window.addEventListener('focus', handleVisibilityChange)
}

const stopUpdateCheck = () => {
  isPolling = false
  if (pollTimer) { clearTimeout(pollTimer); pollTimer = null }
  if (visibilityCheckTimer) { clearTimeout(visibilityCheckTimer); visibilityCheckTimer = null }
  document.removeEventListener('visibilitychange', handleVisibilityChange)
  window.removeEventListener('focus', handleVisibilityChange)
}

onMounted(() => {
  if (import.meta.env.PROD) {
    checkUpdate()
    startUpdateCheck()
  }
})

onUnmounted(() => {
  if (import.meta.env.PROD) stopUpdateCheck()
  clearReminderTimer()
})
</script>

<style scoped>
.update-notification {
  position: fixed;
  bottom: 24px;
  right: 24px;
  z-index: 9999;
  width: 320px;
  background: #ffffff;
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  padding: 16px;
  box-sizing: border-box;
  font-family: system-ui, -apple-system, sans-serif;
}
@media (prefers-color-scheme: dark) {
  .update-notification { background: #1e1e1e; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3); }
}
.close-btn {
  position: absolute; top: 8px; right: 8px;
  background: transparent; border: none; font-size: 18px; line-height: 1;
  color: #909399; cursor: pointer; padding: 4px; border-radius: 4px;
}
.close-btn:hover { color: #606266; }
@media (prefers-color-scheme: dark) {
  .close-btn { color: #a3a6ad; }
  .close-btn:hover { color: #cfd3dc; }
}
.content-wrapper { display: flex; align-items: flex-start; gap: 12px; }
.icon-wrapper {
  flex-shrink: 0; width: 32px; height: 32px; background: #eff6ff;
  border-radius: 6px; display: flex; align-items: center; justify-content: center;
  color: #3b82f6;
}
@media (prefers-color-scheme: dark) {
  .icon-wrapper { background: rgba(30, 58, 138, 0.3); color: #60a5fa; }
}
.text-content { flex: 1; min-width: 0; }
.title { font-size: 14px; font-weight: 500; color: #111827; margin: 0 0 4px 0; }
.desc { font-size: 12px; color: #4b5563; margin: 0 0 12px 0; line-height: 1.5; }
@media (prefers-color-scheme: dark) {
  .title { color: #f3f4f6; }
  .desc { color: #d1d5db; }
}
.actions { display: flex; gap: 8px; }
.btn-primary, .btn-default {
  font-size: 12px; padding: 6px 12px; border-radius: 4px;
  cursor: pointer; border: 1px solid transparent; transition: all 0.2s;
}
.btn-primary { background: #409eff; color: #ffffff; }
.btn-primary:hover { background: #79bbff; }
.btn-default { background: #ffffff; border-color: #dcdfe6; color: #606266; }
.btn-default:hover { color: #409eff; border-color: #c6e2ff; background-color: #ecf5ff; }
@media (prefers-color-scheme: dark) {
  .btn-default { background: transparent; border-color: #4c4d4f; color: #cfd3dc; }
  .btn-default:hover { color: #409eff; border-color: #409eff; background-color: #18222c; }
}
.slide-up-enter-active, .slide-up-leave-active { transition: all 0.3s ease-out; }
.slide-up-enter-from, .slide-up-leave-to { transform: translateY(100%); opacity: 0; }
</style>
