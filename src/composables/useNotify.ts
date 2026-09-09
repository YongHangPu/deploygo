import { useMessage } from 'naive-ui'
import { useDeployStore } from '../stores/deployStore'

/**
 * 统一通知工具 —— 同时显示 Naive UI 的 toast 消息 和 写入终端日志。
 *
 * 替换所有 `message.error/warning/success/info(...)` 调用为:
 *   const notify = useNotify()
 *   notify.error('连接失败')
 *   notify.warning('请先选择项目')
 */
export function useNotify() {
  const message = useMessage()
  const deployStore = useDeployStore()

  function emit(
    type: 'error' | 'warning' | 'success' | 'info',
    text: string,
    options?: { log?: boolean },
  ) {
  // Naive UI 提示消息
    message[type](text)
    // 同步写入终端日志
    if (options?.log !== false) {
      const logMap: Record<string, string> = { error: '❌', warning: '⚠', success: '✓', info: 'ℹ' }
      deployStore.addLog(`${logMap[type]} ${text}`, type === 'success' ? 'info' : type)
    }
  }

  return {
    error: (text: string, options?: { log?: boolean }) => emit('error', text, options),
    warning: (text: string, options?: { log?: boolean }) => emit('warning', text, options),
    success: (text: string, options?: { log?: boolean }) => emit('success', text, options),
    info: (text: string, options?: { log?: boolean }) => emit('info', text, options),
  }
}
