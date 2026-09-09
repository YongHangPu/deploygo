import React, { useState, useEffect, useRef, useCallback } from 'react'

// ---------------------------------------------------------------------------
// 常量
// ---------------------------------------------------------------------------
const UPDATE_CHECK_INTERVAL = 1 * 60 * 1000 // 定时检查间隔 1 分钟
const VISIBILITY_CHECK_DELAY = 1000 // 页面恢复可见时的延迟
const LATER_REMIND_DELAY = 30 * 60 * 1000 // "稍后" 30 分钟后再次提醒
const CLOSE_REMIND_DELAY = 60 * 60 * 1000 // "关闭" 60 分钟后再次提醒

/** 运行时解析 version.json 路径，兼容 Vite / CRA / Next.js 等构建工具 */
function resolveVersionJsonPath(): string {
  // Vite 构建环境
  let viteBase: string | undefined
  try { viteBase = (import.meta as any).env?.BASE_URL } catch { /* 当前不是 Vite 构建 */ }
  // CRA 构建环境
  const publicUrl = (typeof process !== 'undefined' && (process as any).env?.PUBLIC_URL) || ''
  // 通用回退方案：读取 <base> 标签
  const baseEl = typeof document !== 'undefined'
    ? document.querySelector<HTMLBaseElement>('base[href]')
    : null
  const base = (viteBase || publicUrl || baseEl?.href || '/') as string
  const normalized = base.endsWith('/') ? base : `${base}/`
  return `${normalized}version.json`
}

// ---------------------------------------------------------------------------
// 组件
// ---------------------------------------------------------------------------
export default function VersionUpdateNotification() {
  const [showNotification, setShowNotification] = useState(false)
  const showNotificationRef = useRef(false)
  const currentVersionRef = useRef('')
  const isCheckingRef = useRef(false)
  const reminderTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const visibilityTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const pollTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  // ---- 工具函数 ----

  const setNotificationVisible = useCallback((visible: boolean) => {
    showNotificationRef.current = visible
    setShowNotification(visible)
  }, [])

  const getCurrentVersion = useCallback(async (): Promise<string | null> => {
    try {
      const response = await fetch(`${resolveVersionJsonPath()}?t=${Date.now()}`, {
        method: 'GET',
        headers: { 'Cache-Control': 'no-cache', Pragma: 'no-cache' },
      })
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      const data = await response.json()
      return data.version || null
    } catch {
      return null
    }
  }, [])

  const clearReminder = useCallback(() => {
    if (reminderTimerRef.current) {
      clearTimeout(reminderTimerRef.current)
      reminderTimerRef.current = null
    }
  }, [])

  const scheduleReminder = useCallback((delay: number) => {
    clearReminder()
    reminderTimerRef.current = setTimeout(() => {
      setNotificationVisible(true)
      reminderTimerRef.current = null
    }, delay)
  }, [clearReminder, setNotificationVisible])

  // ---- 事件处理 ----

  const handleRefresh = useCallback(() => {
    window.location.reload()
  }, [])

  const handleLater = useCallback(() => {
    setNotificationVisible(false)
    scheduleReminder(LATER_REMIND_DELAY)
  }, [scheduleReminder, setNotificationVisible])

  const handleClose = useCallback(() => {
    setNotificationVisible(false)
    scheduleReminder(CLOSE_REMIND_DELAY)
  }, [scheduleReminder, setNotificationVisible])

  const manualCheckUpdate = useCallback(async () => {
    if (isCheckingRef.current || showNotificationRef.current) return
    isCheckingRef.current = true

    try {
      if (!currentVersionRef.current) {
        const version = await getCurrentVersion()
        if (!version) return
        currentVersionRef.current = version
      }

      const versionInfo = await getCurrentVersion()
      if (versionInfo && versionInfo !== currentVersionRef.current) {
        setNotificationVisible(true)
      }
    } finally {
      isCheckingRef.current = false
    }
  }, [getCurrentVersion, setNotificationVisible])

  // ---- 生命周期 ----

  useEffect(() => {
    // 初始化：获取当前版本作为基准
    const init = async () => {
      const version = await getCurrentVersion()
      if (version) currentVersionRef.current = version
    }
    init()

    // 定时轮询
    pollTimerRef.current = setInterval(manualCheckUpdate, UPDATE_CHECK_INTERVAL)

    // 页面可见性变化时检查
    const handleVisibility = () => {
      if (visibilityTimerRef.current) clearTimeout(visibilityTimerRef.current)
      if (document.visibilityState === 'visible') {
        visibilityTimerRef.current = setTimeout(manualCheckUpdate, VISIBILITY_CHECK_DELAY)
      }
    }
    document.addEventListener('visibilitychange', handleVisibility)

    return () => {
      if (pollTimerRef.current) clearInterval(pollTimerRef.current)
      if (visibilityTimerRef.current) clearTimeout(visibilityTimerRef.current)
      clearReminder()
      document.removeEventListener('visibilitychange', handleVisibility)
    }
  }, [manualCheckUpdate, clearReminder, getCurrentVersion])

  // ---- 渲染 ----

  if (!showNotification) return null

  return (
    <div
      style={{
        position: 'fixed',
        bottom: '24px',
        right: '24px',
        zIndex: 9999,
        maxWidth: '360px',
        width: 'calc(100% - 48px)',
        background: 'var(--card-bg, #ffffff)',
        border: '1px solid var(--border-color, #e5e7eb)',
        borderRadius: '14px',
        boxShadow: '0 12px 32px rgba(0, 0, 0, 0.12)',
        padding: '20px',
        fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
        animation: 'slideUp 0.3s ease-out',
      }}
      className="version-update-notification"
    >
      <button
        onClick={handleClose}
        aria-label="关闭"
        style={{
          position: 'absolute',
          top: '12px',
          right: '14px',
          background: 'none',
          border: 'none',
          fontSize: '20px',
          cursor: 'pointer',
          color: 'var(--text-muted, #6b7280)',
          lineHeight: 1,
        }}
      >
        ×
      </button>

      <div style={{ display: 'flex', alignItems: 'flex-start', gap: '12px' }}>
        <div
          style={{
            flexShrink: 0,
            width: '36px',
            height: '36px',
            borderRadius: '10px',
            background: 'var(--accent-blue-bg, rgba(9, 105, 218, 0.1))',
            color: 'var(--accent-blue, #0969da)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}
        >
          <svg viewBox="0 0 1024 1024" width="16" height="16" fill="currentColor">
            <path d="M136.533333 512a375.466667 375.466667 0 0 1 375.466667-375.466667v-85.333333a42.666667 42.666667 0 0 1 72.533333-30.293333l170.666667 170.666666a42.666667 42.666667 0 0 1 0 60.586667l-170.666667 170.666667A42.666667 42.666667 0 0 1 512 384v-85.333333a290.133333 290.133333 0 1 0 290.133333 290.133333 42.666667 42.666667 0 0 1 85.333334 0A375.466667 375.466667 0 1 1 136.533333 512z" />
          </svg>
        </div>

        <div>
          <h4 style={{ margin: '0 0 4px', fontSize: '15px', fontWeight: 600 }}>
            发现新版本
          </h4>
          <p style={{ margin: '0 0 14px', fontSize: '13px', color: 'var(--text-secondary, #4b5563)' }}>
            应用已更新，刷新页面获得最佳体验
          </p>
          <div style={{ display: 'flex', gap: '8px' }}>
            <button
              onClick={handleRefresh}
              style={{
                padding: '7px 16px',
                fontSize: '13px',
                fontWeight: 500,
                border: 'none',
                borderRadius: '8px',
                cursor: 'pointer',
                background: 'var(--accent-blue, #0969da)',
                color: '#fff',
              }}
            >
              立即刷新
            </button>
            <button
              onClick={handleLater}
              style={{
                padding: '7px 16px',
                fontSize: '13px',
                fontWeight: 500,
                border: '1px solid var(--border-color, #d0d7de)',
                borderRadius: '8px',
                cursor: 'pointer',
                background: 'var(--btn-bg, #f6f8fa)',
                color: 'var(--text-primary, #1f2328)',
              }}
            >
              稍后
            </button>
          </div>
        </div>
      </div>

      <style>{`
        @keyframes slideUp {
          from { opacity: 0; transform: translateY(16px); }
          to   { opacity: 1; transform: translateY(0); }
        }
      `}</style>
    </div>
  )
}
