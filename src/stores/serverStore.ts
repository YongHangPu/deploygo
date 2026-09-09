import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { invokeWithLoading } from '../utils/invoke'
import type { ServerConfig } from '../types'

export const useServerStore = defineStore('server', () => {
  const servers = ref<ServerConfig[]>([])
  const loading = ref(false)
  // 连接测试结果（切换页面后仍保留）。
  const testResults = ref<Record<string, { connected: boolean; latency?: number; error?: string }>>({})
  // 会话级密码缓存，避免重复读取数据库或再次提示用户输入。
  const passwordCache = new Map<string, string>()
  const jumpPasswordCache = new Map<string, string>()

  async function loadServers() {
    servers.value = await invokeWithLoading(loading, 'get_servers')
  }

  async function saveServer(server: ServerConfig, password?: string, jumpPassword?: string) {
    // 为新服务器自动分配 sort_order。
    if (!server.sort_order && server.sort_order !== 0) {
      const maxOrder = servers.value.reduce((max, s) => Math.max(max, s.sort_order ?? 0), -1)
      server.sort_order = maxOrder + 1
    }
    if (password !== undefined) {
      if (password) passwordCache.set(server.id, password)
      else passwordCache.delete(server.id)
    }
    if (jumpPassword !== undefined) {
      if (jumpPassword) jumpPasswordCache.set(server.id, jumpPassword)
      else jumpPasswordCache.delete(server.id)
    }
    await invoke('save_server', {
      server,
      password: password ?? null,
      jumpPassword: jumpPassword ?? null,
    })
    await loadServers()
  }

  async function deleteServer(id: string) {
    passwordCache.delete(id)
    jumpPasswordCache.delete(id)
    await invoke('delete_server', { serverId: id })
    await loadServers()
  }

  async function reorderServers(ids: string[]) {
    // 原地修改 sort_order，不要重新赋值 servers.value。
    // SortableJS 已经移动了 DOM，重新赋值会触发 Vue 重渲染并重建 n-card 内部节点，导致拖拽失效。
    for (let i = 0; i < ids.length; i++) {
      const s = servers.value.find(srv => srv.id === ids[i])
      if (s) s.sort_order = i
    }
    await invoke('reorder_servers', { ids })
  }

  /** 通过已保存的服务器测试，优先使用缓存密码，否则由后端解析已保存的凭据。 */
  async function testConnection(serverId: string, password?: string): Promise<number> {
    const pwd = password || passwordCache.get(serverId)
    return await invoke<number>('test_connection', { serverId, password: pwd || null })
  }

  /** 不保存服务器即可直接测试连接。 */
  async function testConnectionDirect(
    host: string, port: number, username: string, password: string,
    authMethod?: string, keyPath?: string,
    jumpHost?: string, jumpPort?: number, jumpUsername?: string,
    jumpPassword?: string, jumpAuthMethod?: string, jumpKeyPath?: string,
    serverId?: string,
  ): Promise<number> {
    return await invoke<number>('test_connection_direct', {
      host, port, username, password,
      authMethod: authMethod || null,
      keyPath: keyPath || null,
      jumpHost: jumpHost || null,
      jumpPort: jumpPort || null,
      jumpUsername: jumpUsername || null,
      jumpPassword: jumpPassword ?? null,
      jumpAuthMethod: jumpAuthMethod || null,
      jumpKeyPath: jumpKeyPath || null,
      serverId: serverId || null,
    })
  }

  async function hasPassword(serverId: string): Promise<boolean> {
    return passwordCache.has(serverId) || await invoke<boolean>('get_server_password', { serverId })
  }

  async function hasJumpPassword(serverId: string): Promise<boolean> {
    return jumpPasswordCache.has(serverId) || await invoke<boolean>('get_jump_password_exists', { serverId })
  }

  /** 获取缓存密码；未缓存时返回空字符串。 */
  function getCachedPassword(serverId: string): string | undefined {
    return passwordCache.get(serverId)
  }

  /** 将密码缓存到内存，供后续操作使用（例如卡片测试）。 */
  function cachePassword(serverId: string, password: string) {
    passwordCache.set(serverId, password)
  }

  function getServer(id: string): ServerConfig | undefined {
    return servers.value.find(s => s.id === id)
  }

  return {
    servers,
    loading,
    testResults,
    loadServers,
    saveServer,
    deleteServer,
    reorderServers,
    testConnection,
    testConnectionDirect,
    hasPassword,
    hasJumpPassword,
    getCachedPassword,
    getServer,
    cachePassword,
  }
})
