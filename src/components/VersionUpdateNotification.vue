<script setup lang="ts">
withDefaults(
  defineProps<{
    visible: boolean
    currentVersion: string
    latestVersion: string
    title?: string
    description?: string
    refreshText?: string
    laterText?: string
    closable?: boolean
  }>(),
  {
    title: '发现新版本',
    description: '应用已发布新版本，刷新页面后即可切换到最新资源。',
    refreshText: '立即刷新',
    laterText: '稍后',
    closable: true
  }
)

defineEmits<{
  refresh: []
  later: []
  close: []
}>()
</script>

<template>
  <Transition name="toast-fade">
    <article v-if="visible" class="version-toast">
      <button v-if="closable" type="button" class="toast-close" aria-label="关闭提示" @click="$emit('close')">&times;</button>
      <div class="toast-icon">更</div>
      <div class="toast-body">
        <p class="toast-title">{{ title }}</p>
        <p class="toast-desc">{{ description }}</p>
        <div class="toast-version-row">
          <span>当前 {{ currentVersion }}</span>
          <span>最新 {{ latestVersion }}</span>
        </div>
        <div class="toast-actions">
          <button type="button" class="toast-primary" @click="$emit('refresh')">{{ refreshText }}</button>
          <button type="button" class="toast-secondary" @click="$emit('later')">{{ laterText }}</button>
        </div>
      </div>
    </article>
  </Transition>
</template>

<style scoped>
.version-toast {
  position: fixed;
  bottom: 24px;
  right: 24px;
  z-index: 100;
  display: flex;
  align-items: flex-start;
  gap: 14px;
  width: 360px;
  padding: 18px 20px;
  border-radius: 14px;
  background: var(--bg-panel-strong);
  border: 1px solid rgba(var(--accent-blue-rgb), 0.24);
  box-shadow: 0 18px 46px rgba(16, 24, 38, 0.18);
  backdrop-filter: blur(12px);
}

.toast-close {
  position: absolute;
  top: 8px;
  right: 10px;
  background: none;
  border: none;
  color: var(--text-muted);
  font-size: 20px;
  cursor: pointer;
  padding: 0;
  line-height: 1;
}

.toast-close:hover { color: var(--text-primary); }

.toast-icon {
  flex-shrink: 0;
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: rgba(var(--accent-blue-rgb), 0.12);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 700;
  color: var(--accent-blue);
  letter-spacing: 0.5px;
}

.toast-body { flex: 1; }

.toast-title {
  margin: 0 0 4px;
  font-weight: 600;
  font-size: 15px;
  color: var(--text-primary);
}

.toast-desc {
  margin: 0 0 10px;
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.4;
}

.toast-version-row {
  display: flex;
  gap: 12px;
  font-size: 12px;
  color: var(--text-muted);
  margin-bottom: 12px;
}

.toast-actions {
  display: flex;
  gap: 8px;
}

.toast-primary,
.toast-secondary {
  padding: 5px 14px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border: none;
  transition:
    transform 180ms cubic-bezier(0.22, 1, 0.36, 1),
    background-color 180ms cubic-bezier(0.22, 1, 0.36, 1),
    color 180ms cubic-bezier(0.22, 1, 0.36, 1);
}

.toast-primary {
  background: var(--accent-blue);
  color: #fff;
}

.toast-secondary {
  background: var(--bg-soft);
  color: var(--text-secondary);
}

.toast-primary:hover { transform: translateY(-1px); }
.toast-secondary:hover { background: rgba(var(--accent-blue-rgb), 0.08); }

/* 进入/离开动画 */
.toast-fade-enter-active { transition: opacity 300ms cubic-bezier(0.22, 1, 0.36, 1), transform 300ms cubic-bezier(0.22, 1, 0.36, 1); }
.toast-fade-leave-active { transition: opacity 200ms cubic-bezier(0.4, 0, 1, 1), transform 200ms cubic-bezier(0.4, 0, 1, 1); }
.toast-fade-enter-from { opacity: 0; transform: translateY(16px) scale(0.96); }
.toast-fade-leave-to { opacity: 0; transform: translateY(8px) scale(0.98); }
</style>
