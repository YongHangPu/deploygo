<script setup lang="ts">
/**
 * 统一密码输入弹窗组件
 *
 * 用法:
 *   <PasswordPrompt
 *     v-model:visible="show"
 *     v-model="password"
 *     :server-name="server?.name || ''"
 *     description="以测试连接"
 *     :loading="loading"
 *     confirm-label="测试连接"
 *     @confirm="handleConfirm"
 *   />
 */
defineProps<{
  visible: boolean
  serverName: string
  description: string
  loading: boolean
  confirmLabel: string
  /** 为 true 时显示“SSH 密钥密码短语”，否则显示“SSH 密码”。 */
  isKeyMode?: boolean
}>()

const password = defineModel<string>({ required: true })
const emit = defineEmits<{
  'update:visible': [value: boolean]
  confirm: []
}>()

const handleConfirm = () => {
  if (!password.value) return
  emit('confirm')
}
</script>

<template>
  <n-modal
    :show="visible"
    title="输入密码"
    preset="card"
    class="password-prompt-modal"
    style="width: 380px"
    @update:show="(v: boolean) => emit('update:visible', v)"
  >
    <p class="password-prompt-desc">
      请输入服务器「{{ serverName }}」的{{ isKeyMode ? 'SSH 密钥密码短语' : 'SSH 密码' }}{{ description }}
    </p>
    <n-input
      v-model:value="password"
      type="password"
      show-password-on="click"
      :placeholder="isKeyMode ? '请输入密钥密码短语' : '请输入密码'"
      @keyup.enter="handleConfirm"
    />
    <template #footer>
      <div class="password-prompt-footer">
        <n-button @click="emit('update:visible', false)">取消</n-button>
        <n-button
          type="primary"
          :loading="loading"
          :disabled="!password"
          @click="handleConfirm"
        >
          {{ confirmLabel }}
        </n-button>
      </div>
    </template>
  </n-modal>
</template>

<style scoped>
.password-prompt-desc {
  margin: 0 0 12px;
  color: var(--text-secondary);
  font-size: 13px;
  line-height: 1.7;
}

.password-prompt-footer {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}
</style>
