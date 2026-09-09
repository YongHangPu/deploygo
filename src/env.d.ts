/// <reference types="vite/client" />

/** 注入的编译时常量，值为 package.json 中的 version 字段 */
declare const __APP_VERSION__: string

declare module 'sortablejs'

interface Window {
  __TAURI_INTERNALS__?: Record<string, unknown>
}
