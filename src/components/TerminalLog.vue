<script setup lang="ts">
import { ref, nextTick, watch, onMounted } from 'vue'
import { useDeployStore } from '../stores/deployStore'
import { useNotify } from '../composables/useNotify'
import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'

const deployStore = useDeployStore()
const notify = useNotify()
const logContainer = ref<HTMLElement | null>(null)
const autoScroll = ref(true)
const containerHeight = ref(200)

onMounted(() => {
  deployStore.setupListeners()
})

watch(
  () => deployStore.logs.length,
  async () => {
    if (autoScroll.value) {
      await nextTick()
      if (logContainer.value) {
        logContainer.value.scrollTop = logContainer.value.scrollHeight
      }
    }
  }
)

const handleScroll = () => {
  if (!logContainer.value) return
  const el = logContainer.value
  const isAtBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 50
  autoScroll.value = isAtBottom
}

const clearLogs = () => {
  deployStore.clearLogs()
}

const getLogClass = (type: string): string => {
  switch (type) {
    case 'stderr': return 'log-stderr'
    case 'error': return 'log-error'
    case 'warn': return 'log-warn'
    default: return 'log-info'
  }
}

const exportLogs = async () => {
  const text = deployStore.logs
    .map(l => `[${l.time}] ${l.text}`)
    .join('\n')

  const filePath = await save({
    defaultPath: `deploy-log-${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.txt`,
    filters: [{ name: '文本文件', extensions: ['txt'] }],
  })
  if (!filePath) return

  await writeTextFile(filePath, text)
  notify.success('日志已导出')
}

const getResizeHandle = () => {
  return {
    onmousedown: (e: MouseEvent) => {
      e.preventDefault()
      const startY = e.clientY
      const startH = containerHeight.value
      const onMove = (ev: MouseEvent) => {
        const delta = startY - ev.clientY
        containerHeight.value = Math.max(120, Math.min(500, startH + delta))
      }
      const onUp = () => {
        document.removeEventListener('mousemove', onMove)
        document.removeEventListener('mouseup', onUp)
      }
      document.addEventListener('mousemove', onMove)
      document.addEventListener('mouseup', onUp)
    }
  }
}
</script>

<template>
  <div class="terminal-log" :style="{ height: containerHeight + 'px' }">
    <div class="terminal-resize-handle" v-bind="getResizeHandle()" />
    <div class="terminal-header">
      <div class="terminal-header-left">
        <div class="terminal-dot terminal-dot-red" />
        <div class="terminal-dot terminal-dot-yellow" />
        <div class="terminal-dot terminal-dot-green" />
        <span class="terminal-title">终端</span>
      </div>
      <div class="terminal-header-right">
        <span v-if="!autoScroll" class="scroll-hint" @click="autoScroll = true">▼ 自动滚动</span>
        <span v-if="deployStore.isRunning" class="running-badge">
          <span class="running-dot" /> 运行中
        </span>
        <button class="term-btn" @click="clearLogs">清屏</button>
        <button class="term-btn" @click="exportLogs">导出</button>
      </div>
    </div>
    <div ref="logContainer" class="log-content" @scroll="handleScroll">
      <div v-if="deployStore.logs.length === 0" class="log-empty">
        <span class="log-prompt">$</span> 等待操作输出...
      </div>
      <div v-for="(log, index) in deployStore.logs" :key="index" class="log-line" :class="getLogClass(log.type)">
        <span class="log-time">{{ log.time }}</span>
        <span v-if="log.type === 'stderr'" class="log-stderr-icon">✘</span>
        <span v-else-if="log.type === 'error'" class="log-error-icon">✘</span>
        <span v-else-if="log.type === 'warn'" class="log-warn-icon">⚠</span>
        <span v-else class="log-arrow">›</span>
        <span class="log-text">{{ log.text }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.terminal-log {
  background: var(--terminal-bg);
  border-top: 1px solid var(--border-subtle);
  display: flex;
  flex-direction: column;
  font-family: 'JetBrains Mono', 'Cascadia Code', 'Consolas', 'SF Mono', monospace;
  position: relative;
  box-shadow: 0 -10px 28px rgba(15, 23, 42, 0.07);
}

.terminal-resize-handle {
  position: absolute;
  top: -3px;
  left: 0;
  right: 0;
  height: 6px;
  cursor: ns-resize;
  z-index: 10;
}

.terminal-resize-handle:hover {
  background: var(--accent-blue);
  opacity: 0.3;
}

.terminal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: var(--bg-panel-strong);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
  user-select: none;
  backdrop-filter: blur(14px);
}

.terminal-header-left {
  display: flex;
  align-items: center;
  gap: 6px;
}

.terminal-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
}
.terminal-dot-red { background: #ff5f56; }
.terminal-dot-yellow { background: #ffbd2e; }
.terminal-dot-green { background: #63cfe0; }

.terminal-title {
  font-size: 12px;
  color: var(--text-secondary);
  margin-left: 6px;
  font-family: 'Manrope', 'Noto Sans SC', sans-serif;
  font-weight: 600;
}

.terminal-header-right {
  display: flex;
  gap: 8px;
  align-items: center;
  font-family: 'Manrope', 'Noto Sans SC', sans-serif;
}

.scroll-hint {
  font-size: 11px;
  color: var(--accent-blue);
  cursor: pointer;
  padding: 2px 8px;
  border-radius: 999px;
  background: rgba(var(--accent-blue-rgb), 0.08);
}

.running-badge {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--accent-green);
}

.running-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--accent-green);
  animation: pulse 1.4s cubic-bezier(0.22, 1, 0.36, 1) infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}

.term-btn {
  background: var(--bg-soft);
  border: 1px solid var(--border-color);
  color: var(--text-secondary);
  padding: 3px 10px;
  font-size: 11px;
  border-radius: var(--radius-sm);
  cursor: pointer;
  font-family: inherit;
  transition:
    color 180ms cubic-bezier(0.22, 1, 0.36, 1),
    border-color 180ms cubic-bezier(0.22, 1, 0.36, 1),
    background-color 180ms cubic-bezier(0.22, 1, 0.36, 1),
    transform 180ms cubic-bezier(0.22, 1, 0.36, 1);
}

.term-btn:hover {
  color: var(--text-primary);
  border-color: rgba(var(--accent-blue-rgb), 0.22);
  background: rgba(var(--accent-blue-rgb), 0.06);
  transform: translateY(-1px);
}

.log-content {
  flex: 1;
  overflow-y: auto;
  padding: 10px 12px 12px;
  font-size: 12px;
  line-height: 1.7;
}

.log-empty {
  color: var(--text-muted);
  font-style: italic;
  display: flex;
  gap: 8px;
}

.log-prompt {
  color: var(--accent-green);
  font-style: normal;
}

.log-line {
  white-space: pre-wrap;
  word-break: break-all;
  display: flex;
  gap: 8px;
  animation: log-in 0.2s ease both;
  padding: 2px 0;
}

.log-time {
  color: var(--text-muted);
  user-select: none;
  flex-shrink: 0;
}

.log-arrow {
  color: var(--accent-green);
  flex-shrink: 0;
}

.log-stderr-icon {
  color: var(--accent-red);
  flex-shrink: 0;
}

.log-warn-icon {
  color: var(--accent-yellow);
  flex-shrink: 0;
}

.log-error-icon {
  color: var(--accent-red);
  flex-shrink: 0;
}

.log-info .log-text { color: var(--text-primary); }
.log-stderr .log-text { color: var(--accent-red); }
.log-error .log-text { color: var(--accent-red); font-weight: 500; }
.log-warn .log-text { color: var(--accent-yellow); }
</style>

<style scoped>
.terminal-log {
  border-top-color: var(--border-color);
  box-shadow: none;
}

.terminal-header {
  min-height: 38px;
  padding: 7px 12px;
  background: var(--bg-panel-strong);
  backdrop-filter: none;
}

.terminal-dot {
  width: 7px;
  height: 7px;
}

.terminal-title {
  margin-left: 5px;
  font: 700 10px/1 'JetBrains Mono', monospace;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.terminal-header-right {
  gap: 6px;
}

.scroll-hint {
  border-radius: 3px;
  font: 650 10px/1.2 'JetBrains Mono', monospace;
}

.running-badge {
  font: 650 10px/1.2 'JetBrains Mono', monospace;
}

.term-btn {
  min-height: 24px;
  padding: 3px 7px;
  border-radius: 3px;
  font: 650 10px/1.1 'Manrope', 'Noto Sans SC', sans-serif;
}

.log-content {
  padding: 9px 12px 12px;
  font-size: 11px;
  line-height: 1.75;
}

.log-line {
  gap: 7px;
  padding: 1px 0;
}

.log-time {
  min-width: 52px;
  color: color-mix(in srgb, var(--text-muted) 82%, transparent);
}
</style>
