<svelte:options runes={true} />

<script>
  import { onMount } from 'svelte'

  // -----------------------------------------------------------------------
  // 常量
  // -----------------------------------------------------------------------
  const UPDATE_CHECK_INTERVAL = 1 * 60 * 1000 // 定时检查间隔 1 分钟
  const VISIBILITY_CHECK_DELAY = 1000 // 页面恢复可见时的延迟
  const LATER_REMIND_DELAY = 30 * 60 * 1000 // "稍后" 30 分钟后再次提醒
  const CLOSE_REMIND_DELAY = 60 * 60 * 1000 // "关闭" 60 分钟后再次提醒

  /** 运行时解析 version.json 路径，兼容 Vite / SvelteKit 等构建工具 */
  function resolveVersionJsonPath() {
    let baseUrl = '/'
    // Vite 构建环境
    if (typeof import.meta !== 'undefined' && import.meta.env?.BASE_URL) {
      baseUrl = import.meta.env.BASE_URL
    }
  // 通用回退方案：读取 <base> 标签
    if (typeof document !== 'undefined') {
      const baseEl = document.querySelector('base[href]')
      if (baseEl) baseUrl = baseEl.href
    }
    const normalized = baseUrl.endsWith('/') ? baseUrl : `${baseUrl}/`
    return `${normalized}version.json`
  }

  const versionJsonPath = resolveVersionJsonPath()

  // -----------------------------------------------------------------------
  // 状态（Svelte 5 runes）
  // -----------------------------------------------------------------------
  let showNotification = $state(false)
  let currentVersion = $state('')
  let isChecking = $state(false)

  let pollTimer = null
  let reminderTimer = null
  let visibilityTimer = null

  // -----------------------------------------------------------------------
  // 工具函数
  // -----------------------------------------------------------------------
  async function getCurrentVersion() {
    try {
      const response = await fetch(`${versionJsonPath}?t=${Date.now()}`, {
        method: 'GET',
        headers: { 'Cache-Control': 'no-cache', Pragma: 'no-cache' },
      })
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      const data = await response.json()
      return data.version || null
    } catch {
      return null
    }
  }

  function clearReminder() {
    if (reminderTimer) {
      clearTimeout(reminderTimer)
      reminderTimer = null
    }
  }

  function scheduleReminder(delay) {
    clearReminder()
    reminderTimer = setTimeout(() => {
      showNotification = true
      reminderTimer = null
    }, delay)
  }

  // -----------------------------------------------------------------------
  // 核心检查逻辑
  // -----------------------------------------------------------------------
  async function manualCheckUpdate() {
    if (isChecking || showNotification) return
    isChecking = true

    try {
      if (!currentVersion) {
        const version = await getCurrentVersion()
        if (!version) return
        currentVersion = version
      }

      const versionInfo = await getCurrentVersion()
      if (versionInfo && versionInfo !== currentVersion) {
        showNotification = true
      }
    } finally {
      isChecking = false
    }
  }

  // -----------------------------------------------------------------------
  // 事件处理
  // -----------------------------------------------------------------------
  function handleRefresh() {
    window.location.reload()
  }

  function handleLater() {
    showNotification = false
    scheduleReminder(LATER_REMIND_DELAY)
  }

  function handleClose() {
    showNotification = false
    scheduleReminder(CLOSE_REMIND_DELAY)
  }

  function onVisibilityChange() {
    if (visibilityTimer) clearTimeout(visibilityTimer)
    if (document.visibilityState === 'visible') {
      visibilityTimer = setTimeout(manualCheckUpdate, VISIBILITY_CHECK_DELAY)
    }
  }

  // -----------------------------------------------------------------------
  // 生命周期
  // -----------------------------------------------------------------------
  onMount(async () => {
    // 初始化基准版本
    const version = await getCurrentVersion()
    if (version) currentVersion = version

    // 定时轮询
    pollTimer = setInterval(manualCheckUpdate, UPDATE_CHECK_INTERVAL)

    // 页面可见性变化
    document.addEventListener('visibilitychange', onVisibilityChange)

    return () => {
      if (pollTimer) clearInterval(pollTimer)
      if (reminderTimer) clearTimeout(reminderTimer)
      if (visibilityTimer) clearTimeout(visibilityTimer)
      document.removeEventListener('visibilitychange', onVisibilityChange)
    }
  })
</script>

{#if showNotification}
  <div class="vun-card">
    <button class="vun-close" onclick={handleClose} aria-label="关闭">&times;</button>
    <div class="vun-body">
      <div class="vun-icon">
        <svg viewBox="0 0 1024 1024" width="16" height="16" fill="currentColor">
          <path d="M136.533333 512a375.466667 375.466667 0 0 1 375.466667-375.466667v-85.333333a42.666667 42.666667 0 0 1 72.533333-30.293333l170.666667 170.666666a42.666667 42.666667 0 0 1 0 60.586667l-170.666667 170.666667A42.666667 42.666667 0 0 1 512 384v-85.333333a290.133333 290.133333 0 1 0 290.133333 290.133333 42.666667 42.666667 0 0 1 85.333334 0A375.466667 375.466667 0 1 1 136.533333 512z" />
        </svg>
      </div>
      <div>
        <h4 class="vun-title">发现新版本</h4>
        <p class="vun-desc">应用已更新，刷新页面获得最佳体验</p>
        <div class="vun-actions">
          <button class="vun-btn-primary" onclick={handleRefresh}>立即刷新</button>
          <button class="vun-btn-default" onclick={handleLater}>稍后</button>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .vun-card {
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
    animation: vun-slideUp 0.3s ease-out;
  }
  @media (prefers-color-scheme: dark) {
    .vun-card {
      background: #1e1e1e;
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    }
  }
  @keyframes vun-slideUp {
    from { opacity: 0; transform: translateY(16px); }
    to   { opacity: 1; transform: translateY(0); }
  }
  .vun-close {
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
  .vun-close:hover {
    color: #606266;
  }
  @media (prefers-color-scheme: dark) {
    .vun-close {
      color: #a3a6ad;
    }
    .vun-close:hover {
      color: #cfd3dc;
    }
  }
  .vun-body {
    display: flex;
    align-items: flex-start;
    gap: 12px;
  }
  .vun-icon {
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
    .vun-icon {
      background: rgba(30, 58, 138, 0.3);
      color: #60a5fa;
    }
  }
  .vun-title {
    font-size: 14px;
    font-weight: 500;
    color: #111827;
    margin: 0 0 4px 0;
  }
  .vun-desc {
    font-size: 12px;
    color: #4b5563;
    margin: 0 0 12px 0;
    line-height: 1.5;
  }
  @media (prefers-color-scheme: dark) {
    .vun-title { color: #f3f4f6; }
    .vun-desc  { color: #d1d5db; }
  }
  .vun-actions {
    display: flex;
    gap: 8px;
  }
  .vun-btn-primary,
  .vun-btn-default {
    font-size: 12px;
    padding: 6px 12px;
    border-radius: 4px;
    cursor: pointer;
    border: 1px solid transparent;
    transition: all 0.2s;
  }
  .vun-btn-primary {
    background: #409eff;
    color: #ffffff;
  }
  .vun-btn-primary:hover {
    background: #79bbff;
  }
  .vun-btn-default {
    background: #ffffff;
    border-color: #dcdfe6;
    color: #606266;
  }
  .vun-btn-default:hover {
    color: #409eff;
    border-color: #c6e2ff;
    background-color: #ecf5ff;
  }
  @media (prefers-color-scheme: dark) {
    .vun-btn-default {
      background: transparent;
      border-color: #4c4d4f;
      color: #cfd3dc;
    }
    .vun-btn-default:hover {
      color: #409eff;
      border-color: #409eff;
      background-color: #18222c;
    }
  }
</style>
