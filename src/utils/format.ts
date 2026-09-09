/**
 * 格式化工具函数
 */

/** 从 Tauri invoke 错误中提取可读消息 */
export function fmtErr(e: unknown): string {
  if (typeof e === 'string') return e
  if (e && typeof e === 'object') {
    // Tauri 返回的序列化错误对象
    if ('message' in e) return (e as { message: string }).message
    // 原生 Error 实例
    if (e instanceof Error) return e.message
  }
  return String(e)
}

/** 格式化当前时间为 HH:MM:SS */
export function fmtTime(date?: Date): string {
  const d = date ?? new Date()
  const pad = (n: number) => n.toString().padStart(2, '0')
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
}
