<template>
  <transition name="slide-up">
    <div v-if="showNotification" class="update-notification">
      <button class="close-btn" @click="handleClose" aria-label="关闭">&times;</button>
      <div class="content-wrapper">
        <div class="icon-wrapper">
          <svg viewBox="0 0 1024 1024" width="16" height="16" fill="currentColor">
            <path d="M136.533333 512a375.466667 375.466667 0 0 1 375.466667-375.466667v-85.333333a42.666667 42.666667 0 0 1 72.533333-30.293333l170.666667 170.666666a42.666667 42.666667 0 0 1 0 60.586667l-170.666667 170.666667A42.666667 42.666667 0 0 1 512 384v-85.333333a290.133333 290.133333 0 1 0 290.133333 290.133333 42.666667 42.666667 0 0 1 85.333334 0A375.466667 375.466667 0 1 1 136.533333 512z" />
          </svg>
        </div>
        <div class="text-content">
          <h4 class="title">发现新版本</h4>
          <p class="desc">应用已更新，刷新页面获得最佳体验</p>
          <div class="actions">
            <button class="btn-primary" @click="handleRefresh">立即刷新</button>
            <button class="btn-default" @click="handleLater">稍后</button>
          </div>
        </div>
      </div>
    </div>
  </transition>
</template>

<script lang="ts">
import Vue from 'vue'

const UPDATE_CHECK_INTERVAL = 1 * 60 * 1000
const VISIBILITY_CHECK_DELAY = 1000
const LATER_REMIND_DELAY = 30 * 60 * 1000
const CLOSE_REMIND_DELAY = 60 * 60 * 1000

function resolveVersionJsonPath(): string {
  let base: string = '/'
  try { base = (import.meta as any).env?.BASE_URL || '/' } catch { /* 当前不是 Vite 构建 */ }
  if (typeof process !== 'undefined' && (process as any).env?.BASE_URL) {
    base = (process as any).env.BASE_URL
  }
  if (typeof document !== 'undefined') {
    const baseEl = document.querySelector<HTMLBaseElement>('base[href]')
    if (baseEl) base = baseEl.href
  }
  return (base.endsWith('/') ? base : base + '/') + 'version.json'
}

interface Data {
  showNotification: boolean
  currentVersion: string
  isChecking: boolean
  versionJsonPath: string
}

export default Vue.extend({
  name: 'VersionUpdateNotification',
  data(): Data {
    return {
      showNotification: false,
      currentVersion: '',
      isChecking: false,
      versionJsonPath: resolveVersionJsonPath(),
    }
  },
  mounted(): void {
    this.init()
    ;(this as any).pollTimer = setInterval(() => { this.manualCheckUpdate() }, UPDATE_CHECK_INTERVAL)
    document.addEventListener('visibilitychange', this.onVisibilityChange)
  },
  beforeDestroy(): void {
    if ((this as any).pollTimer) clearInterval((this as any).pollTimer)
    if ((this as any).reminderTimer) clearTimeout((this as any).reminderTimer)
    if ((this as any).visibilityTimer) clearTimeout((this as any).visibilityTimer)
    document.removeEventListener('visibilitychange', this.onVisibilityChange)
  },
  methods: {
    getCurrentVersion(): Promise<string | null> {
      return fetch(this.versionJsonPath + '?t=' + Date.now(), {
        method: 'GET',
        headers: { 'Cache-Control': 'no-cache', Pragma: 'no-cache' },
      }).then((res) => {
        if (!res.ok) throw new Error('HTTP ' + res.status)
        return res.json()
      }).then((data: any) => data.version || null)
        .catch(() => null)
    },
    init(): void {
      this.getCurrentVersion().then((v) => { if (v) this.currentVersion = v })
    },
    manualCheckUpdate(): void {
      if (this.isChecking || this.showNotification) return
      this.isChecking = true
      const check = this.currentVersion
        ? this.getCurrentVersion()
        : this.getCurrentVersion().then((v) => { if (v) this.currentVersion = v; return null })
      check.then((v) => {
        if (v && v !== this.currentVersion) this.showNotification = true
      }).finally(() => { this.isChecking = false })
    },
    handleRefresh(): void { window.location.reload() },
    handleLater(): void { this.showNotification = false; this.scheduleReminder(LATER_REMIND_DELAY) },
    handleClose(): void { this.showNotification = false; this.scheduleReminder(CLOSE_REMIND_DELAY) },
    clearReminder(): void {
      if ((this as any).reminderTimer) { clearTimeout((this as any).reminderTimer); (this as any).reminderTimer = null }
    },
    scheduleReminder(delay: number): void {
      this.clearReminder()
      ;(this as any).reminderTimer = setTimeout(() => { this.showNotification = true; (this as any).reminderTimer = null }, delay)
    },
    onVisibilityChange(): void {
      if ((this as any).visibilityTimer) clearTimeout((this as any).visibilityTimer)
      if (document.visibilityState === 'visible') {
        (this as any).visibilityTimer = setTimeout(() => { this.manualCheckUpdate() }, VISIBILITY_CHECK_DELAY)
      }
    },
  },
})
</script>

<style scoped>
.update-notification {
  position: fixed; bottom: 24px; right: 24px; z-index: 9999; width: 320px;
  background: #ffffff; border-radius: 8px; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  padding: 16px; box-sizing: border-box; font-family: system-ui, -apple-system, sans-serif;
}
@media (prefers-color-scheme: dark) {
  .update-notification { background: #1e1e1e; box-shadow: 0 4px 12px rgba(0,0,0,0.3); }
}
.close-btn { position: absolute; top: 8px; right: 8px; background: transparent; border: none; font-size: 18px; line-height: 1; color: #909399; cursor: pointer; padding: 4px; border-radius: 4px; }
.close-btn:hover { color: #606266; }
@media (prefers-color-scheme: dark) { .close-btn { color: #a3a6ad; } .close-btn:hover { color: #cfd3dc; } }
.content-wrapper { display: flex; align-items: flex-start; gap: 12px; }
.icon-wrapper { flex-shrink: 0; width: 32px; height: 32px; background: #eff6ff; border-radius: 6px; display: flex; align-items: center; justify-content: center; color: #3b82f6; }
@media (prefers-color-scheme: dark) { .icon-wrapper { background: rgba(30,58,138,0.3); color: #60a5fa; } }
.text-content { flex: 1; min-width: 0; }
.title { font-size: 14px; font-weight: 500; color: #111827; margin: 0 0 4px 0; }
.desc { font-size: 12px; color: #4b5563; margin: 0 0 12px 0; line-height: 1.5; }
@media (prefers-color-scheme: dark) { .title { color: #f3f4f6; } .desc { color: #d1d5db; } }
.actions { display: flex; gap: 8px; }
.btn-primary, .btn-default { font-size: 12px; padding: 6px 12px; border-radius: 4px; cursor: pointer; border: 1px solid transparent; transition: all 0.2s; }
.btn-primary { background: #409eff; color: #ffffff; }
.btn-primary:hover { background: #79bbff; }
.btn-default { background: #ffffff; border-color: #dcdfe6; color: #606266; }
.btn-default:hover { color: #409eff; border-color: #c6e2ff; background-color: #ecf5ff; }
@media (prefers-color-scheme: dark) { .btn-default { background: transparent; border-color: #4c4d4f; color: #cfd3dc; } .btn-default:hover { color: #409eff; border-color: #409eff; background-color: #18222c; } }
.slide-up-enter-active, .slide-up-leave-active { transition: all 0.3s ease-out; }
.slide-up-enter-from, .slide-up-leave-to { transform: translateY(100%); opacity: 0; }
</style>
