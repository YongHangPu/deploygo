/**
 * VersionUpdateNotification — 纯 JavaScript 版本更新通知
 *
 * 零依赖、框架无关，仅需约 4 KB。适用于任何 HTML/JS 项目。
 *
 * ## 快速接入
 *
 * ```html
 * <script src="VersionUpdateNotification.js"></script>
 * <script>
 *   VersionUpdateNotification.init({ basePath: '/' })
 * </script>
 * ```
 *
 * ## API
 *
 * | 方法                                   | 说明                       |
 * |----------------------------------------|----------------------------|
 * | `VersionUpdateNotification.init(opts)` | 启动检测，可传入配置项     |
 * | `VersionUpdateNotification.stop()`     | 停止轮询并移除 DOM         |
 * | `VersionUpdateNotification.checkNow()` | 立即执行一次检查           |
 *
 * ## 配置项 (opts)
 *
 * | 字段               | 类型     | 默认值    | 说明                     |
 * |--------------------|----------|-----------|--------------------------|
 * | `basePath`         | string   | `'/'`     | `version.json` 所在路径  |
 * | `checkInterval`    | number   | `60000`   | 轮询间隔 (ms)            |
 * | `laterRemindDelay` | number   | `1800000` | "稍后" 再次提醒间隔 (ms) |
 * | `closeRemindDelay` | number   | `3600000` | "关闭" 再次提醒间隔 (ms) |
 * | `onNewVersion`     | function | `null`    | 检测到新版本时的回调     |
 */
;(function (global) {
  'use strict'

  // =========================================================================
  // 默认配置
  // =========================================================================
  var DEFAULTS = {
    basePath: '/',
    checkInterval: 1 * 60 * 1000,
    laterRemindDelay: 30 * 60 * 1000,
    closeRemindDelay: 60 * 60 * 1000,
    onNewVersion: null,
  }

  var state = {
    currentVersion: '',
    isChecking: false,
    isActive: false,
    notificationVisible: false,
    pollTimer: null,
    reminderTimer: null,
    visibilityTimer: null,
    visibilityHandler: null,
    versionJsonPath: '',
    options: {},
    container: null,
  }

  // =========================================================================
  // 工具函数
  // =========================================================================

  function getCurrentVersion() {
    return fetch(state.versionJsonPath + '?t=' + Date.now(), {
      method: 'GET',
      headers: { 'Cache-Control': 'no-cache', Pragma: 'no-cache' },
    })
      .then(function (res) {
        if (!res.ok) throw new Error('HTTP ' + res.status)
        return res.json()
      })
      .then(function (data) {
        return data.version || null
      })
      .catch(function () {
        return null
      })
  }

  function clearReminder() {
    if (state.reminderTimer) {
      clearTimeout(state.reminderTimer)
      state.reminderTimer = null
    }
  }

  function scheduleReminder(delay) {
    clearReminder()
    state.reminderTimer = setTimeout(function () {
      showNotification()
      state.reminderTimer = null
    }, delay)
  }

  // =========================================================================
  // DOM 创建与销毁
  // =========================================================================

  function createDOM() {
    if (state.container) return

 // 注入 CSS 关键帧
    var styleEl = document.createElement('style')
    styleEl.textContent =
      '@keyframes vun-slideUp { from { opacity:0; transform:translateY(16px); } to { opacity:1; transform:translateY(0); } }' +
      '.vun-card { position:fixed; bottom:24px; right:24px; z-index:9999; max-width:360px; width:calc(100% - 48px);' +
      'background:#fff; border:1px solid #e5e7eb; border-radius:14px; box-shadow:0 12px 32px rgba(0,0,0,.12);' +
      'padding:20px; font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif;' +
      'animation:vun-slideUp .3s ease-out; }' +
      '.vun-close { position:absolute; top:12px; right:14px; background:none; border:none; font-size:20px;' +
      'cursor:pointer; color:#6b7280; line-height:1; }' +
      '.vun-body { display:flex; align-items:flex-start; gap:12px; }' +
      '.vun-icon { flex-shrink:0; width:36px; height:36px; border-radius:10px;' +
      'background:rgba(9,105,218,.1); color:#0969da; display:flex; align-items:center; justify-content:center; }' +
      '.vun-title { margin:0 0 4px; font-size:15px; font-weight:600; }' +
      '.vun-desc { margin:0 0 14px; font-size:13px; color:#4b5563; }' +
      '.vun-actions { display:flex; gap:8px; }' +
      '.vun-btn-primary { padding:7px 16px; font-size:13px; font-weight:500; border:none; border-radius:8px;' +
      'cursor:pointer; background:#0969da; color:#fff; }' +
      '.vun-btn-default { padding:7px 16px; font-size:13px; font-weight:500;' +
      'border:1px solid #d0d7de; border-radius:8px; cursor:pointer; background:#f6f8fa; color:#1f2328; }'
    document.head.appendChild(styleEl)

    // 图标 SVG
    var svgMarkup =
      '<svg viewBox="0 0 1024 1024" width="16" height="16" fill="currentColor">' +
      '<path d="M136.533333 512a375.466667 375.466667 0 0 1 375.466667-375.466667v-85.333333' +
      'a42.666667 42.666667 0 0 1 72.533333-30.293333l170.666667 170.666666a42.666667 42.666667 0 0 1 0 60.586667' +
      'l-170.666667 170.666667A42.666667 42.666667 0 0 1 512 384v-85.333333a290.133333 290.133333 0 1 0 290.133333 290.133333' +
      ' 42.666667 42.666667 0 0 1 85.333334 0A375.466667 375.466667 0 1 1 136.533333 512z"/>' +
      '</svg>'

    var container = document.createElement('div')
    container.className = 'vun-card'
    container.innerHTML =
      '<button class="vun-close" aria-label="关闭">&times;</button>' +
      '<div class="vun-body">' +
      '<div class="vun-icon">' + svgMarkup + '</div>' +
      '<div>' +
      '<h4 class="vun-title">发现新版本</h4>' +
      '<p class="vun-desc">应用已更新，刷新页面获得最佳体验</p>' +
      '<div class="vun-actions">' +
      '<button class="vun-btn-primary">立即刷新</button>' +
      '<button class="vun-btn-default">稍后</button>' +
      '</div>' +
      '</div>' +
      '</div>'

    // 事件绑定
    container.querySelector('.vun-close').addEventListener('click', handleClose)
    container.querySelector('.vun-btn-primary').addEventListener('click', handleRefresh)
    container.querySelector('.vun-btn-default').addEventListener('click', handleLater)

    state.container = container
  }

  function showNotification() {
    if (state.notificationVisible) return
    createDOM()
    document.body.appendChild(state.container)
    state.notificationVisible = true
    if (typeof state.options.onNewVersion === 'function') {
      state.options.onNewVersion()
    }
  }

  function hideNotification() {
    if (!state.notificationVisible || !state.container) return
    if (state.container.parentNode) {
      state.container.parentNode.removeChild(state.container)
    }
    state.notificationVisible = false
  }

  // =========================================================================
  // 事件处理
  // =========================================================================

  function handleRefresh() {
    window.location.reload()
  }

  function handleLater() {
    hideNotification()
    scheduleReminder(state.options.laterRemindDelay)
  }

  function handleClose() {
    hideNotification()
    scheduleReminder(state.options.closeRemindDelay)
  }

  // =========================================================================
  // 核心检查逻辑
  // =========================================================================

  function checkNow() {
    if (state.isChecking || state.notificationVisible) return
    state.isChecking = true

    var check = function () {
      if (!state.currentVersion) {
        return getCurrentVersion().then(function (v) {
          if (v) state.currentVersion = v
          return null
        })
      }
      return getCurrentVersion().then(function (v) {
        if (v && v !== state.currentVersion) {
          showNotification()
        }
        return null
      })
    }

    return check().finally(function () {
      state.isChecking = false
    })
  }

  // =========================================================================
  // 公开 API
  // =========================================================================

  function init(opts) {
    if (state.isActive) return

    state.options = {}
    var keys = Object.keys(DEFAULTS)
    for (var i = 0; i < keys.length; i++) {
      var k = keys[i]
      state.options[k] = (opts && opts[k] !== undefined) ? opts[k] : DEFAULTS[k]
    }

    var base = state.options.basePath
    state.versionJsonPath = (base.endsWith('/') ? base : base + '/') + 'version.json'

    // 初始化基准版本
    getCurrentVersion().then(function (v) {
      if (v) state.currentVersion = v
    })

    // 定时轮询
    state.pollTimer = setInterval(checkNow, state.options.checkInterval)

    // 页面可见性变化
    state.visibilityHandler = function () {
      if (state.visibilityTimer) clearTimeout(state.visibilityTimer)
      if (document.visibilityState === 'visible') {
        state.visibilityTimer = setTimeout(checkNow, 1000)
      }
    }
    document.addEventListener('visibilitychange', state.visibilityHandler)

    state.isActive = true
  }

  function stop() {
    if (state.pollTimer) clearInterval(state.pollTimer)
    if (state.reminderTimer) clearTimeout(state.reminderTimer)
    if (state.visibilityTimer) clearTimeout(state.visibilityTimer)
    if (state.visibilityHandler) {
      document.removeEventListener('visibilitychange', state.visibilityHandler)
    }
    hideNotification()
    state.isActive = false
    state.pollTimer = null
    state.reminderTimer = null
    state.visibilityTimer = null
    state.visibilityHandler = null
  }

  // =========================================================================
  // 导出
  // =========================================================================
  var api = {
    init: init,
    stop: stop,
    checkNow: checkNow,
  }

  if (typeof module !== 'undefined' && module.exports) {
    module.exports = api
  } else {
    global.VersionUpdateNotification = api
  }
})(typeof window !== 'undefined' ? window : this)
