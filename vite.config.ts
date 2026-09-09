import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { readFileSync } from 'node:fs'
import Components from 'unplugin-vue-components/vite'
import { NaiveUiResolver } from 'unplugin-vue-components/resolvers'

const pkg = JSON.parse(readFileSync(new URL('./package.json', import.meta.url), 'utf-8'))

// GITHUB_ACTIONS 会在所有 CI 任务中设置，但只有 GitHub Pages 需要 /deploygo/ 基础路径。
// Tauri 构建（release.yml）始终需要 '/'；可使用 VITE_BASE 覆盖。
const base = process.env.VITE_BASE || (process.env.GITHUB_ACTIONS === 'true' ? '/deploygo/' : '/')

// Vite 配置文档：https://vite.dev/config/
export default defineConfig({
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
  },
  base,
  plugins: [
    vue(),
    Components({
      resolvers: [NaiveUiResolver()],
      dts: 'src/components.d.ts',
    }),
  ],

  // 避免 Vite 输出遮蔽 Rust 错误信息。
  clearScreen: false,

  server: {
    strictPort: true,
    // Tauri 需要固定端口；端口不可用时直接失败。
    watch: {
      // 让 Vite 忽略监听 `src-tauri`。
      ignored: ['**/src-tauri/**'],
    },
  },
})
