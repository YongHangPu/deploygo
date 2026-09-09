/**
 * Tauri invoke 工具函数
 */
import { invoke } from '@tauri-apps/api/core'
import type { Ref } from 'vue'

/**
 * 带 loading 状态的 invoke 包装
 * 自动设置/清除 loading ref，减少 store 中的重复代码
 */
export async function invokeWithLoading<T>(
  loadingRef: Ref<boolean>,
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  loadingRef.value = true
  try {
    return await invoke<T>(cmd, args)
  } finally {
    loadingRef.value = false
  }
}
