<template>
  <Transition name="slide-up">
    <!-- 当检测到新版本时展示通知 UI -->
    <div v-if="showNotification" class="update-notification">
      <button class="close-btn" @click="handleClose" aria-label="关闭">×</button>
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

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

// 控制更新弹窗的显示状态
const showNotification = ref(false)

// 定时检查的时间间隔，默认 1 分钟
const UPDATE_CHECK_INTERVAL = 1 * 60 * 1000
// 页面恢复可见时的延迟检查时间，避免频繁切换导致的性能问题
const VISIBILITY_CHECK_DELAY = 1000
// 点击“稍后”后的再次提醒时间，默认 30 分钟
const LATER_REMIND_DELAY = 30 * 60 * 1000
// 点击“关闭”后的再次提醒时间，默认 60 分钟
const CLOSE_REMIND_DELAY = 60 * 60 * 1000

let reminderTimer: ReturnType<typeof setTimeout> | null = null
let visibilityCheckTimer: ReturnType<typeof setTimeout> | null = null
let pollTimer: ReturnType<typeof setTimeout> | null = null
let isPolling = false

const currentVersion = ref('')
const isChecking = ref(false)

// 拼接 version.json 的请求路径，适配 base 路径配置
const baseUrl = (() => {
  // Vite 构建环境
  try {
    const viteBase = (import.meta as any).env?.BASE_URL
    if (viteBase) return viteBase
  } catch { /* 当前不是 Vite 构建 */ }
  // 通用回退方案：读取 <base> 标签
  if (typeof document !== 'undefined') {
    const baseEl = document.querySelector<HTMLBaseElement>('base[href]')
    if (baseEl?.href) return baseEl.href
  }
  return '/'
})()
const versionJsonPath = `${baseUrl.endsWith('/') ? baseUrl : `${baseUrl}/`}version.json`

/**
 * 刷新页面，加载最新资源
 */
const handleRefresh = () => {
  window.location.reload()
}

/**
 * 清除提醒定时器
 */
const clearReminderTimer = () => {
  if (reminderTimer) {
    clearTimeout(reminderTimer)
    reminderTimer = null
  }
}

/**
 * 安排下一次提醒
 * @param delay 延迟的毫秒数
 */
const scheduleReminder = (delay: number) => {
  clearReminderTimer()
  reminderTimer = setTimeout(() => {
    showNotification.value = true
    reminderTimer = null
  }, delay)
}

/**
 * 用户点击“稍后”按钮
 */
const handleLater = () => {
  showNotification.value = false
  scheduleReminder(LATER_REMIND_DELAY)
}

/**
 * 用户点击“关闭”按钮
 */
const handleClose = () => {
  showNotification.value = false
  scheduleReminder(CLOSE_REMIND_DELAY)
}

/**
 * 获取服务器上的最新版本号
 */
const getCurrentVersion = async (): Promise<string | null> => {
  try {
    // 加上时间戳防止缓存
    const response = await fetch(`${versionJsonPath}?t=${Date.now()}`, {
      method: 'GET',
      headers: {
        'Cache-Control': 'no-cache',
        'Pragma': 'no-cache'
      }
    })
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`)
    }
    const data = await response.json()
    return data.version || null
  } catch (error) {
    console.error('获取当前版本失败:', error)
    return null
  }
}

/**
 * 初始化时获取最新版本号并记录为当前版本
 */
const checkUpdate = async () => {
  const version = await getCurrentVersion()
  if (version) {
    currentVersion.value = version
  }
}

/**
 * 手动/自动检查更新核心逻辑
 */
const manualCheckUpdate = async () => {
  // 正在检查中或弹窗已经显示，直接跳过
  if (isChecking.value || showNotification.value) {
    return
  }

  isChecking.value = true

  try {
    // 容错：如果初始化时没有成功获取到当前版本，则重新获取一次作为基准
    if (!currentVersion.value) {
      const version = await getCurrentVersion()
      if (!version) {
        return
      }
      currentVersion.value = version
    }

    const versionInfo = await getCurrentVersion()

    // 发现版本号不一致，说明有新版本
    if (versionInfo && versionInfo !== currentVersion.value) {
      showNotification.value = true
      stopUpdateCheck() // 停止轮询，等待用户操作
    }
  } catch (error) {
    console.error('检查更新失败:', error)
  } finally {
    isChecking.value = false
  }
}

/**
 * 安排下一次轮询检查
 */
const scheduleNextCheck = () => {
  if (!isPolling) return

  if (pollTimer) {
    clearTimeout(pollTimer)
    pollTimer = null
  }

  // 页面隐藏时不执行定时任务，节省性能
  if (document.hidden) return

  pollTimer = setTimeout(async () => {
    await manualCheckUpdate()
    scheduleNextCheck()
  }, UPDATE_CHECK_INTERVAL)
}

/**
 * 页面可见性改变时的回调
 */
const handleVisibilityChange = () => {
  if (!document.hidden) {
    // 页面从后台回到前台，延迟检查一次
    if (visibilityCheckTimer) {
      clearTimeout(visibilityCheckTimer)
    }

    visibilityCheckTimer = setTimeout(async () => {
      visibilityCheckTimer = null
      await manualCheckUpdate()
      scheduleNextCheck()
    }, VISIBILITY_CHECK_DELAY)
  } else {
    // 页面进入后台，暂停轮询
    if (pollTimer) {
      clearTimeout(pollTimer)
      pollTimer = null
    }
  }
}

/**
 * 启动更新检查机制
 */
const startUpdateCheck = () => {
  isPolling = true
  scheduleNextCheck()

  // 监听页面可见性及焦点变化，及时感知更新
  document.addEventListener('visibilitychange', handleVisibilityChange)
  window.addEventListener('focus', handleVisibilityChange)
}

/**
 * 停止更新检查机制
 */
const stopUpdateCheck = () => {
  isPolling = false
  if (pollTimer) {
    clearTimeout(pollTimer)
    pollTimer = null
  }

  if (visibilityCheckTimer) {
    clearTimeout(visibilityCheckTimer)
    visibilityCheckTimer = null
  }

  document.removeEventListener('visibilitychange', handleVisibilityChange)
  window.removeEventListener('focus', handleVisibilityChange)
}

onMounted(() => {
  // 只在生产环境中开启无感更新检测
  if (import.meta.env.PROD) {
    checkUpdate()
    startUpdateCheck()
  }
})

onUnmounted(() => {
  if (import.meta.env.PROD) {
    stopUpdateCheck()
  }
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
  .update-notification {
    background: #1e1e1e;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }
}

.close-btn {
  position: absolute;
  top: 8px;
  right: 8px;
  background: transparent;
  border: none;
  font-size: 18px;
  line-height: 1;
  color: #909399;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
}

.close-btn:hover {
  color: #606266;
}

@media (prefers-color-scheme: dark) {
  .close-btn {
    color: #a3a6ad;
  }
  .close-btn:hover {
    color: #cfd3dc;
  }
}

.content-wrapper {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.icon-wrapper {
  flex-shrink: 0;
  width: 32px;
  height: 32px;
  background: #eff6ff;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #3b82f6;
}

@media (prefers-color-scheme: dark) {
  .icon-wrapper {
    background: rgba(30, 58, 138, 0.3);
    color: #60a5fa;
  }
}

.text-content {
  flex: 1;
  min-width: 0;
}

.title {
  font-size: 14px;
  font-weight: 500;
  color: #111827;
  margin: 0 0 4px 0;
}

.desc {
  font-size: 12px;
  color: #4b5563;
  margin: 0 0 12px 0;
  line-height: 1.5;
}

@media (prefers-color-scheme: dark) {
  .title {
    color: #f3f4f6;
  }
  .desc {
    color: #d1d5db;
  }
}

.actions {
  display: flex;
  gap: 8px;
}

.btn-primary, .btn-default {
  font-size: 12px;
  padding: 6px 12px;
  border-radius: 4px;
  cursor: pointer;
  border: 1px solid transparent;
  transition: all 0.2s;
}

.btn-primary {
  background: #409eff;
  color: #ffffff;
}

.btn-primary:hover {
  background: #79bbff;
}

.btn-default {
  background: #ffffff;
  border-color: #dcdfe6;
  color: #606266;
}

.btn-default:hover {
  color: #409eff;
  border-color: #c6e2ff;
  background-color: #ecf5ff;
}

@media (prefers-color-scheme: dark) {
  .btn-default {
    background: transparent;
    border-color: #4c4d4f;
    color: #cfd3dc;
  }
  .btn-default:hover {
    color: #409eff;
    border-color: #409eff;
    background-color: #18222c;
  }
}

.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.3s ease-out;
}

.slide-up-enter-from,
.slide-up-leave-to {
  transform: translateY(100%);
  opacity: 0;
}
</style>
