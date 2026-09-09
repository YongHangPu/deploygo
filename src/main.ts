import { createApp } from 'vue'
import { createPinia } from 'pinia'
import './fonts.css'

// Tauri 环境检测：在 Tauri 中有 __TAURI_INTERNALS__ 全局变量
const isTauri = typeof window !== 'undefined' && window.__TAURI_INTERNALS__

if (isTauri) {
  // ── 桌面端应用 ──
  import('./App.vue').then(({ default: DesktopApp }) => {
    import('./router').then(({ default: router }) => {
      const app = createApp(DesktopApp)
      const pinia = createPinia()
      app.use(pinia)
      app.use(router)
      app.mount('#app')
    })
  })
} else {
  // ── 网页演示站 ──
  import('./DemoApp.vue').then(({ default: DemoApp }) => {
    const app = createApp(DemoApp)
    app.mount('#app')
  })
}
