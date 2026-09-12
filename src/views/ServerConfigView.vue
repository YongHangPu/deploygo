<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue'
import {
  NAlert,
  NButton,
  NForm,
  NFormItem,
  NInput,
  NInputNumber,
  NModal,
  NTag,
  NSelect,
  NSwitch,
  NTooltip,
  useDialog,
} from 'naive-ui'
import { useSortable } from '../composables/useSortable'
import { useServerStore } from '../stores/serverStore'
import { useProjectStore } from '../stores/projectStore'
import { fmtErr } from '../utils/format'
import { invoke } from '@tauri-apps/api/core'
import { save, open } from '@tauri-apps/plugin-dialog'
import { readTextFile, writeTextFile } from '@tauri-apps/plugin-fs'
import PasswordPrompt from '../components/PasswordPrompt.vue'
import { useNotify } from '../composables/useNotify'
import type { ServerConfig } from '../types'

const notify = useNotify()
const dialog = useDialog()
const serverStore = useServerStore()
const projectStore = useProjectStore()

const showModal = ref(false)
const isEditing = ref(false)
const editingServer = ref<Partial<ServerConfig>>({
  name: '',
  host: '',
  port: 22,
  username: '',
  auth_method: 'password',
  key_path: null,
  jump_host: null,
  jump_port: null,
  jump_username: null,
  jump_auth_method: null,
  jump_key_path: null,
})
const editingPassword = ref('')
const editingJumpPassword = ref('')
const showJumpConfig = ref(false)
const testingId = ref<string | null>(null)
const testResults = ref<Record<string, { success: boolean; latency?: number; error?: string }>>({})
const modalTesting = ref(false)
const modalTestResult = ref<{ success: boolean; latency?: number; error?: string } | null>(null)
const serverErrors = ref<Record<string, string>>({})

// 密码提示对话框（卡片测试且没有缓存密码时显示）。
const showPasswordPrompt = ref(false)
const passwordPromptServerId = ref('')
const passwordPromptValue = ref('')
const passwordPromptTesting = ref(false)
const passwordPromptIsKeyMode = computed(() =>
  serverStore.getServer(passwordPromptServerId.value)?.auth_method === 'key'
)

// ---- SortableJS 初始化 ----
const serverListEl = ref<HTMLElement | null>(null)
const serversRef = computed(() => serverStore.servers)

useSortable(serverListEl, {
  handle: '.server-drag-handle',
  items: serversRef,
  watchSource: () => serverStore.servers.length,
  getId: (s) => s.id,
  onReorder: (ids) => serverStore.reorderServers(ids),
})

onMounted(async () => {
  await serverStore.loadServers()
})

const authMethodOptions = [
  { label: '密码认证', value: 'password' },
  { label: 'SSH 密钥认证', value: 'key' },
]

const isPasswordMode = computed(() => editingServer.value.auth_method === 'password')
const isKeyMode = computed(() => editingServer.value.auth_method === 'key')
const isJumpPasswordMode = computed(() => (editingServer.value.jump_auth_method || 'password') === 'password')
const isJumpKeyMode = computed(() => editingServer.value.jump_auth_method === 'key')

const validateServerField = (field: string): boolean => {
  const server = editingServer.value
  let message = ''

  switch (field) {
    case 'name':
      if (!server.name?.trim()) message = '服务器名称不能为空'
      break
    case 'host':
      if (!server.host?.trim()) message = '主机地址不能为空'
      break
    case 'username':
      if (!server.username?.trim()) message = '用户名不能为空'
      break
    case 'password':
      if (isPasswordMode.value && !editingPassword.value) message = '请输入服务器密码'
      break
    case 'key_path':
      if (isKeyMode.value && !server.key_path) message = '请选择 SSH 私钥文件'
      break
    case 'jump_password':
      if (server.jump_host && isJumpPasswordMode.value && !editingJumpPassword.value) message = '请输入跳板机密码'
      break
    case 'jump_key_path':
      if (server.jump_host && isJumpKeyMode.value && !server.jump_key_path) message = '请选择跳板机 SSH 私钥'
      break
  }

  if (message) {
    serverErrors.value[field] = message
    return false
  }

  delete serverErrors.value[field]
  return true
}

const clearServerFieldError = (field: string) => {
  delete serverErrors.value[field]
}

// 切换认证方式时清空密码/密码短语，避免遗留值污染
watch(() => editingServer.value.auth_method, () => {
  editingPassword.value = ''
  clearServerFieldError('password')
  clearServerFieldError('key_path')
})
watch(() => editingServer.value.jump_auth_method, () => {
  editingJumpPassword.value = ''
  clearServerFieldError('jump_password')
  clearServerFieldError('jump_key_path')
})

const openAddModal = () => {
  isEditing.value = false
  editingServer.value = {
    name: '',
    host: '',
    port: 22,
    username: '',
    auth_method: 'password',
    key_path: null,
    jump_host: null,
    jump_port: null,
    jump_username: null,
    jump_auth_method: null,
    jump_key_path: null,
  }
  editingPassword.value = ''
  editingJumpPassword.value = ''
  serverErrors.value = {}
  showJumpConfig.value = false
  showModal.value = true
}

const openEditModal = async (server: ServerConfig) => {
  isEditing.value = true
  editingServer.value = { ...server }
  const hasPwd = await serverStore.hasPassword(server.id)
  editingPassword.value = hasPwd ? '********' : ''
  const hasJumpPwd = await serverStore.hasJumpPassword(server.id)
  editingJumpPassword.value = hasJumpPwd ? '********' : ''
  serverErrors.value = {}
  showJumpConfig.value = !!server.jump_host
  showModal.value = true
}

const saveServer = async () => {
  const s = editingServer.value
  const valid = [
    validateServerField('name'),
    validateServerField('host'),
    validateServerField('username'),
    isPasswordMode.value ? validateServerField('password') : true,
    isKeyMode.value ? validateServerField('key_path') : true,
    s.jump_host && isJumpPasswordMode.value ? validateServerField('jump_password') : true,
    s.jump_host && isJumpKeyMode.value ? validateServerField('jump_key_path') : true,
  ].every(Boolean)
  if (!valid) return

  const server: ServerConfig = {
    id: s.id || crypto.randomUUID(),
    name: s.name!,
    host: s.host!,
    port: s.port ?? 22,
    username: s.username!,
    auth_method: s.auth_method || 'password',
    key_path: s.key_path || null,
    jump_host: s.jump_host || null,
    jump_port: s.jump_port ?? null,
    jump_username: s.jump_username || null,
    jump_auth_method: s.jump_auth_method || null,
    jump_key_path: s.jump_key_path || null,
    sort_order: s.sort_order ?? 0,
    created_at: s.created_at || new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }

  const password = editingPassword.value === '********' ? undefined : editingPassword.value
  const jumpPassword = editingJumpPassword.value === '********' ? undefined : editingJumpPassword.value

  await serverStore.saveServer(server, password, jumpPassword)
  notify.success(isEditing.value ? '服务器已更新' : '服务器已添加')
  showModal.value = false
}

const deleteServer = async (id: string, name: string) => {
  dialog.warning({
    title: '删除服务器',
    content: `确定要删除服务器「${name}」吗？所有关联此服务器的项目将无法继续使用。`,
    positiveText: '确定删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      await serverStore.deleteServer(id)
      notify.success('服务器已删除')
      delete testResults.value[id]
    },
  })
}

const testConnection = async (id: string) => {
  const server = serverStore.getServer(id)
  if (!server) {
    notify.error('服务器信息未找到')
    return
  }
  const cachedPwd = serverStore.getCachedPassword(id)
  if (cachedPwd) {
    doDirectTest(id, server, cachedPwd)
    return
  }
  testingId.value = id
  try {
    const latency = await serverStore.testConnection(id)
    testResults.value[id] = { success: true, latency }
    notify.success(`连接成功（测试耗时 ${latency}ms）`)
  } catch (e: any) {
    const errMsg = typeof e === 'string' ? e : (e?.message || '')
    if (errMsg.includes('密码未找到') || errMsg.includes('需要密码短语')) {
      passwordPromptServerId.value = id
      passwordPromptValue.value = ''
      showPasswordPrompt.value = true
    } else {
      testResults.value[id] = { success: false, error: fmtErr(e) }
      notify.error(`连接失败: ${fmtErr(e)}`, { log: false })
    }
  } finally {
    testingId.value = null
  }
}

const doDirectTest = async (id: string, server: ServerConfig, password: string) => {
  testingId.value = id
  try {
    const latency = await serverStore.testConnection(id, password)
    testResults.value[id] = { success: true, latency }
    serverStore.cachePassword(id, password)
    await serverStore.saveServer(server, password)
    notify.success(`连接成功（测试耗时 ${latency}ms）`)
  } catch (e: any) {
    testResults.value[id] = { success: false, error: fmtErr(e) }
    notify.error(`连接失败: ${fmtErr(e)}`, { log: false })
  } finally {
    testingId.value = null
  }
}

const testWithPromptPassword = async () => {
  if (!passwordPromptValue.value) {
    notify.warning(passwordPromptIsKeyMode.value ? '请输入密钥密码短语' : '请输入密码')
    return
  }
  passwordPromptTesting.value = true
  try {
    const serverId = passwordPromptServerId.value
    const server = serverStore.getServer(serverId)
    if (!server) {
      notify.error('服务器未找到')
      return
    }
    await doDirectTest(serverId, server, passwordPromptValue.value)
    showPasswordPrompt.value = false
  } finally {
    passwordPromptTesting.value = false
  }
}

const testConnectionFromModal = async () => {
  const validHost = validateServerField('host')
  const validUsername = validateServerField('username')
  const validKeyPath = isKeyMode.value ? validateServerField('key_path') : true
  const validJumpKeyPath = editingServer.value.jump_host && isJumpKeyMode.value
    ? validateServerField('jump_key_path')
    : true
  if (!validHost || !validUsername || !validKeyPath || !validJumpKeyPath) return

  if (editingPassword.value === '********') {
    if (!editingServer.value.id) {
      notify.warning(isKeyMode.value ? '服务器未保存，请先填写密钥密码短语' : '服务器未保存，请先填写密码')
      return
    }
    const cachedPwd = serverStore.getCachedPassword(editingServer.value.id)
    if (cachedPwd) {
      await doModalDirectTest(cachedPwd)
      return
    }
    try {
      const latency = await serverStore.testConnection(editingServer.value.id)
      modalTestResult.value = { success: true, latency }
      notify.success(`连接成功（测试耗时 ${latency}ms）`)
      modalTesting.value = false
      return
    } catch (e: any) {
      const errMsg = typeof e === 'string' ? e : (e?.message || '')
      if (errMsg.includes('密码未找到') || errMsg.includes('需要密码短语')) {
        notify.warning(isKeyMode.value ? '请重新输入密钥密码短语后再测试' : '请重新输入密码后再测试')
      } else {
        modalTestResult.value = { success: false, error: fmtErr(e) }
        notify.error(`连接失败: ${fmtErr(e)}`, { log: false })
      }
      modalTesting.value = false
      return
    }
  }

  if (isPasswordMode.value && !editingPassword.value) {
    validateServerField('password')
    return
  }
  if (editingServer.value.jump_host && isJumpPasswordMode.value && !editingJumpPassword.value) {
    validateServerField('jump_password')
    return
  }

  await doModalDirectTest(editingPassword.value)
}

const doModalDirectTest = async (password: string) => {
  modalTesting.value = true
  modalTestResult.value = null
  try {
    const jumpPassword = editingJumpPassword.value === '********' ? undefined : editingJumpPassword.value
    const latency = await serverStore.testConnectionDirect(
      editingServer.value.host!,
      editingServer.value.port ?? 22,
      editingServer.value.username!,
      password,
      editingServer.value.auth_method,
      editingServer.value.key_path || undefined,
      editingServer.value.jump_host || undefined,
      editingServer.value.jump_port ?? undefined,
      editingServer.value.jump_username || undefined,
      jumpPassword,
      editingServer.value.jump_auth_method || undefined,
      editingServer.value.jump_key_path || undefined,
      editingServer.value.id,
    )
    modalTestResult.value = { success: true, latency }
    if (editingServer.value.id) {
      serverStore.cachePassword(editingServer.value.id, password)
    }
    notify.success(`连接成功（测试耗时 ${latency}ms）`)
  } catch (e: any) {
    modalTestResult.value = { success: false, error: fmtErr(e) }
    notify.error(`连接失败: ${fmtErr(e)}`, { log: false })
  } finally {
    modalTesting.value = false
  }
}

/** 打开操作系统文件选择器，选择 SSH 私钥。 */
const pickKeyFile = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const file = await open({
      multiple: false,
      title: '选择 SSH 私钥文件',
    })
    if (file) {
      const path = file as string
      editingServer.value.key_path = path
      validateServerField('key_path')

      // 友善检测：提示用户选择的文件可能不是有效的 SSH 私钥
      const filename = path.split(/[/\\]/).pop() || ''
      const isLikelySshKey =
        /^id_(rsa|ed25519|ecdsa|dsa)(\.pub)?$/.test(filename) ||
        /\.(pem|key|ppk)$/i.test(filename)
      if (!isLikelySshKey) {
        notify.warning(
          `提示：「${filename}」看起来不是标准的 SSH 私钥文件名。` +
          'SSH 私钥通常是 id_rsa、id_ed25519、*.pem 或 *.key 文件。'
        )
      }
    }
  } catch (e: any) {
    notify.error(`选择文件失败: ${typeof e === 'string' ? e : (e?.message || '未知错误')}`)
  }
}

/** 打开操作系统文件选择器，选择跳板机 SSH 私钥。 */
const pickJumpKeyFile = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const file = await open({
      multiple: false,
      title: '选择跳板机 SSH 私钥文件',
    })
    if (file) {
      editingServer.value.jump_key_path = file as string
      validateServerField('jump_key_path')
    }
  } catch (e: any) {
    notify.error(`选择文件失败: ${typeof e === 'string' ? e : (e?.message || '未知错误')}`)
  }
}

// ---- 导入/导出 ----

const showImportDialog = ref(false)
const importOverwrite = ref(false)
const importFileContent = ref('')
const importBackupPassword = ref('')
const importHasEncryptedCredentials = computed(() => {
  if (!importFileContent.value) return false
  try {
    const data = JSON.parse(importFileContent.value)
    return data.version === 3 && Boolean(data.credential_payload)
  } catch {
    return false
  }
})
const importIsLegacyV2 = computed(() => {
  if (!importFileContent.value) return false
  try {
    return JSON.parse(importFileContent.value).version === 2
  } catch {
    return false
  }
})

const showExportDialog = ref(false)
const exportIncludeCredentials = ref(false)
const exportBackupPassword = ref('')
const exportBackupPasswordConfirm = ref('')
const exporting = ref(false)

const handleExport = async () => {
  exportIncludeCredentials.value = false
  exportBackupPassword.value = ''
  exportBackupPasswordConfirm.value = ''
  showExportDialog.value = true
}

const handleExportConfirm = async () => {
  if (exportIncludeCredentials.value) {
    if (exportBackupPassword.value.length < 8) {
      notify.warning('备份密码至少需要 8 个字符')
      return
    }
    if (exportBackupPassword.value !== exportBackupPasswordConfirm.value) {
      notify.warning('两次输入的备份密码不一致')
      return
    }
  }

  try {
    const path = await save({
      title: '导出配置',
      defaultPath: `deploygo-config-${new Date().toISOString().slice(0, 10)}.json`,
      filters: [{ name: 'JSON', extensions: ['json'] }],
    })
    if (!path) return
    exporting.value = true
    const data = await invoke<any>('export_config', {
      includeCredentials: exportIncludeCredentials.value,
      backupPassword: exportIncludeCredentials.value ? exportBackupPassword.value : null,
    })
    await writeTextFile(path, JSON.stringify(data, null, 2))
    showExportDialog.value = false
    notify.success(exportIncludeCredentials.value ? '配置与加密凭据已导出' : '配置已导出（未包含凭据）')
  } catch (e: any) {
    notify.error(`导出失败: ${fmtErr(e)}`)
  } finally {
    exporting.value = false
  }
}

const handleImportClick = async () => {
  try {
    const file = await open({
      multiple: false,
      title: '导入配置',
      filters: [{ name: 'JSON', extensions: ['json'] }],
    })
    if (!file) return
    const content = await readTextFile(file as string)
    // 校验 JSON。
    JSON.parse(content)
    importFileContent.value = content
    importOverwrite.value = false
    importBackupPassword.value = ''
    showImportDialog.value = true
  } catch (e: any) {
    notify.error(`读取文件失败: ${typeof e === 'string' ? e : (e?.message || '')}`)
  }
}

const handleImportConfirm = async () => {
  try {
    const data = JSON.parse(importFileContent.value)
    const summary = await invoke<string>('import_config', {
      data,
      overwrite: importOverwrite.value,
      backupPassword: importHasEncryptedCredentials.value ? importBackupPassword.value : null,
    })
    notify.success(summary)
    showImportDialog.value = false
    await serverStore.loadServers()
    await projectStore.loadProjects()
  } catch (e: any) {
    notify.error(`导入失败: ${fmtErr(e)}`)
  }
}
</script>

<template>
  <div class="page-container server-view">
    <div class="page-header">
      <h2>服务器管理</h2>
      <div style="display: flex; gap: 8px;">
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button size="small" secondary @click="handleExport">导出配置</n-button>
          </template>
          默认导出服务器和项目，可选择用备份密码加密凭据
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button size="small" secondary @click="handleImportClick">导入配置</n-button>
          </template>
          从 JSON 文件导入服务器和项目配置
        </n-tooltip>
        <n-button size="small" type="primary" @click="openAddModal">+ 添加服务器</n-button>
      </div>
    </div>

    <div ref="serverListEl" class="server-list">
      <div
        v-for="server in serverStore.servers"
        :key="server.id"
        class="server-card-wrapper"
      >
        <article class="panel-card server-card">
          <header class="server-card-header">
            <div class="server-drag-handle" title="拖动排序">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="8" y1="6" x2="16" y2="6" /><line x1="8" y1="12" x2="16" y2="12" /><line x1="8" y1="18" x2="16" y2="18" />
              </svg>
            </div>
            <div class="server-heading">
              <div class="server-icon">
                <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="2" width="20" height="8" rx="2" ry="2" /><rect x="2" y="14" width="20" height="8" rx="2" ry="2" /><line x1="6" y1="6" x2="6.01" y2="6" /><line x1="6" y1="18" x2="6.01" y2="18" /></svg>
              </div>
              <span class="server-name" :title="server.name">{{ server.name }}</span>
            </div>
          </header>
          <div class="server-info">
            <div class="address-block info-row">
              <span class="label">地址</span>
              <code class="address-value">{{ server.host }}:{{ server.port }}</code>
            </div>
            <div class="info-grid">
              <div class="info-row">
                <span class="label">用户名</span>
                <strong class="info-value">{{ server.username }}</strong>
              </div>
              <div class="info-row">
                <span class="label">认证方式</span>
                <n-tag :type="server.auth_method === 'key' ? 'info' : 'default'" size="tiny">
                  {{ server.auth_method === 'key' ? '密钥' : '密码' }}
                </n-tag>
              </div>
              <div v-if="server.jump_host" class="info-row jump-row">
                <span class="label">跳板机</span>
                <strong class="info-value">{{ server.jump_host }}{{ server.jump_port ? ':' + server.jump_port : '' }}</strong>
              </div>
            </div>
          </div>
          <footer class="card-actions">
              <n-tag
                :type="testResults[server.id]?.success ? 'success' : 'default'"
                v-if="testResults[server.id]"
                size="tiny"
                class="status-tag"
              >
                {{ testResults[server.id]?.success ? `耗时 ${testResults[server.id].latency}ms` : '失败' }}
              </n-tag>
              <div class="spacer"></div>
              <n-button
                size="tiny"
                secondary
                :loading="testingId === server.id"
                @click="testConnection(server.id)"
              >
                测试连接
              </n-button>
              <n-button size="tiny" secondary @click="openEditModal(server)">
                编辑
              </n-button>
              <n-button size="tiny" type="error" secondary @click="deleteServer(server.id, server.name)">
                删除
              </n-button>
          </footer>
        </article>
      </div>

      <div v-if="serverStore.servers.length === 0" class="empty-servers">
        暂无服务器配置，请点击「+ 添加服务器」开始配置
      </div>
    </div>

    <!-- 添加/编辑服务器弹窗 -->
    <n-modal v-model:show="showModal" :title="isEditing ? '编辑服务器' : '添加服务器'" preset="card" style="width: 550px">
      <n-form class="server-form" label-placement="left" label-width="100px" :show-feedback="true">
        <n-form-item
          label="服务器名称"
          required
          :feedback="serverErrors.name"
          :validation-status="serverErrors.name ? 'error' : undefined"
        >
          <n-input v-model:value="editingServer.name" placeholder="如: 生产服务器-01" @blur="validateServerField('name')" />
        </n-form-item>
        <n-form-item
          label="主机地址"
          required
          :feedback="serverErrors.host"
          :validation-status="serverErrors.host ? 'error' : undefined"
        >
          <n-input v-model:value="editingServer.host" placeholder="如: 192.168.1.100" @blur="validateServerField('host')" />
        </n-form-item>
        <n-form-item label="端口">
          <n-input-number v-model:value="editingServer.port" :min="1" :max="65535" />
        </n-form-item>
        <n-form-item
          label="用户名"
          required
          :feedback="serverErrors.username"
          :validation-status="serverErrors.username ? 'error' : undefined"
        >
          <n-input v-model:value="editingServer.username" placeholder="如: root" @blur="validateServerField('username')" />
        </n-form-item>
        <n-form-item label="认证方式">
          <n-select v-model:value="editingServer.auth_method" :options="authMethodOptions" />
        </n-form-item>

        <template v-if="isPasswordMode">
          <n-form-item
            label="密码"
            :required="editingPassword !== '********'"
            :feedback="serverErrors.password"
            :validation-status="serverErrors.password ? 'error' : undefined"
          >
            <n-input
              v-model:value="editingPassword"
              type="password"
              show-password-on="click"
              :placeholder="isEditing ? '留空保持原密码' : '请输入密码'"
              @blur="validateServerField('password')"
            />
          </n-form-item>
        </template>

        <template v-if="isKeyMode">
          <n-form-item
            label="密钥文件"
            required
            :feedback="serverErrors.key_path"
            :validation-status="serverErrors.key_path ? 'error' : undefined"
          >
            <div class="key-path-row">
              <n-input
                v-model:value="editingServer.key_path"
                placeholder="如: C:\Users\me\.ssh\id_ed25519 或 ~/.ssh/id_rsa"
                class="key-path-input"
                @blur="validateServerField('key_path')"
              />
              <n-button secondary @click="pickKeyFile">选择文件</n-button>
            </div>
          </n-form-item>
          <n-form-item label="密钥密码">
            <n-input
              v-model:value="editingPassword"
              type="password"
              show-password-on="click"
              :placeholder="isEditing ? '留空保持原密码' : '密钥有密码则输入（可选）'"
            />
          </n-form-item>
        </template>

        <!-- 凭证安全说明：在用户输入敏感凭证的时刻给出明确的安全交代 -->
        <n-alert type="info" :show-icon="false" class="security-note">
          服务器凭证仅保存在你的电脑上：密码与口令以 AES-256-GCM 加密存储，部署时通过 SSH
          直连你的服务器，不经过任何第三方服务器或云端。
        </n-alert>

        <!-- 跳板机 -->
        <n-form-item label="跳板机">
          <n-button
            size="small"
            secondary
            :type="showJumpConfig || editingServer.jump_host ? 'info' : 'default'"
            @click="showJumpConfig = !showJumpConfig"
          >
            {{ showJumpConfig ? '收起跳板机配置' : '配置跳板机（堡垒机）' }}
          </n-button>
          <span v-if="editingServer.jump_host && !showJumpConfig" style="margin-left: 8px; font-size: 12px; color: var(--text-muted);">
            {{ editingServer.jump_host }}{{ editingServer.jump_port ? ':' + editingServer.jump_port : '' }}
          </span>
        </n-form-item>

        <template v-if="showJumpConfig">
          <n-form-item label="跳板机地址">
            <n-input v-model:value="editingServer.jump_host" placeholder="如: 10.0.0.1" />
          </n-form-item>
          <n-form-item label="跳板机端口">
            <n-input-number v-model:value="editingServer.jump_port" :min="1" :max="65535" :default-value="22" />
          </n-form-item>
          <n-form-item label="跳板机用户名">
            <n-input v-model:value="editingServer.jump_username" placeholder="留空则使用主服务器用户名" />
          </n-form-item>
          <n-form-item label="跳板机认证">
            <n-select v-model:value="editingServer.jump_auth_method" :options="authMethodOptions" :default-value="'password'" />
          </n-form-item>

          <template v-if="isJumpPasswordMode">
            <n-form-item
              label="跳板机密码"
              :required="editingJumpPassword !== '********'"
              :feedback="serverErrors.jump_password"
              :validation-status="serverErrors.jump_password ? 'error' : undefined"
            >
              <n-input
                v-model:value="editingJumpPassword"
                type="password"
                show-password-on="click"
                :placeholder="isEditing ? '留空保持原密码' : '请输入跳板机密码'"
                @blur="validateServerField('jump_password')"
              />
            </n-form-item>
          </template>

          <template v-if="isJumpKeyMode">
            <n-form-item
              label="跳板机密钥"
              required
              :feedback="serverErrors.jump_key_path"
              :validation-status="serverErrors.jump_key_path ? 'error' : undefined"
            >
              <div class="key-path-row">
                <n-input
                  v-model:value="editingServer.jump_key_path"
                  placeholder="如: ~/.ssh/id_ed25519"
                  class="key-path-input"
                  @blur="validateServerField('jump_key_path')"
                />
                <n-button secondary @click="pickJumpKeyFile">选择文件</n-button>
              </div>
            </n-form-item>
            <n-form-item label="密钥密码">
              <n-input
                v-model:value="editingJumpPassword"
                type="password"
                show-password-on="click"
                :placeholder="isEditing ? '留空保持原密码' : '密钥有密码则输入（可选）'"
              />
            </n-form-item>
          </template>
        </template>

        <n-form-item :show-label="false">
          <div class="modal-test-area">
            <n-button
              size="small"
              :loading="modalTesting"
              :disabled="!editingServer.host || !editingServer.username || (isPasswordMode && !editingPassword)"
              @click="testConnectionFromModal"
            >
              测试连接
            </n-button>
            <span v-if="modalTestResult" :class="modalTestResult.success ? 'test-success' : 'test-fail'">
              {{ modalTestResult.success ? `✅ 连接成功（测试耗时 ${modalTestResult.latency}ms）` : `❌ 失败: ${modalTestResult.error}` }}
            </span>
          </div>
        </n-form-item>
      </n-form>
      <template #footer>
        <div class="server-modal-footer">
          <n-button @click="showModal = false">取消</n-button>
          <n-button type="primary" @click="saveServer">{{ isEditing ? '更新' : '保存' }}</n-button>
        </div>
      </template>
    </n-modal>

    <PasswordPrompt
      v-model:visible="showPasswordPrompt"
      v-model="passwordPromptValue"
      :server-name="serverStore.getServer(passwordPromptServerId)?.name || ''"
      description="以测试连接"
      :loading="passwordPromptTesting"
      confirm-label="测试连接"
      :is-key-mode="passwordPromptIsKeyMode"
      @confirm="testWithPromptPassword"
    />

    <n-modal v-model:show="showExportDialog" title="导出配置" preset="card" style="width: 480px">
      <p class="backup-note">
        默认只导出服务器和项目配置。SSH 私钥文件不会写入备份，换机器后需要重新选择私钥。
      </p>
      <n-form-item label="包含密码凭据">
        <n-switch v-model:value="exportIncludeCredentials" />
      </n-form-item>
      <template v-if="exportIncludeCredentials">
        <n-form-item label="备份密码">
          <n-input
            v-model:value="exportBackupPassword"
            type="password"
            show-password-on="click"
            placeholder="至少 8 个字符"
          />
        </n-form-item>
        <n-form-item label="确认备份密码">
          <n-input
            v-model:value="exportBackupPasswordConfirm"
            type="password"
            show-password-on="click"
          />
        </n-form-item>
        <p class="backup-warning">备份密码无法找回。导入到其他机器时必须使用相同密码。</p>
      </template>
      <template #footer>
        <div style="display: flex; gap: 8px; justify-content: flex-end">
          <n-button @click="showExportDialog = false">取消</n-button>
          <n-button type="primary" :loading="exporting" @click="handleExportConfirm">导出</n-button>
        </div>
      </template>
    </n-modal>

    <!-- 导入配置对话框 -->
    <n-modal v-model:show="showImportDialog" title="导入配置" preset="card" style="width: 480px">
      <p class="backup-note">
        即将导入服务器和项目配置。已存在的条目默认跳过，勾选「覆盖已有」将替换。
      </p>
      <p v-if="importIsLegacyV2" class="backup-warning">
        检测到 v2 备份。旧凭据会先解密，再使用本机密钥重新加密；导入完成后建议重新导出 v3 备份。
      </p>
      <n-form-item v-if="importHasEncryptedCredentials" label="备份密码">
        <n-input
          v-model:value="importBackupPassword"
          type="password"
          show-password-on="click"
          placeholder="请输入导出时设置的备份密码"
        />
      </n-form-item>
      <n-form-item label="覆盖已有">
        <n-switch v-model:value="importOverwrite" />
      </n-form-item>
      <template #footer>
        <div style="display: flex; gap: 8px; justify-content: flex-end">
          <n-button @click="showImportDialog = false">取消</n-button>
          <n-button
            type="primary"
            :disabled="importHasEncryptedCredentials && !importBackupPassword"
            @click="handleImportConfirm"
          >确认导入</n-button>
        </div>
      </template>
    </n-modal>
  </div>
</template>

<style scoped>
.server-view {
  display: grid;
  align-content: start;
}

.backup-note,
.backup-warning {
  margin: 0 0 12px;
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-secondary);
}

.backup-warning {
  color: var(--accent-yellow);
}

.server-card-wrapper {
  transition: opacity var(--motion-fast);
}

/* SortableJS ghost：卡片即将放置的位置占位项 */
.server-card-wrapper.sortable-ghost {
  opacity: 0.35;
}

.server-card-wrapper.sortable-ghost .server-card {
  border: 1px dashed var(--accent-blue);
  background: rgba(var(--accent-blue-rgb), 0.04);
}

/* SortableJS chosen：拖拽过程中的原始卡片 */
.server-card-wrapper.sortable-chosen {
  opacity: 0.5;
}

/* SortableJS drag：跟随光标移动的浮动副本 */
.sortable-drag {
  opacity: 0.9 !important;
  box-shadow: var(--shadow-lg);
}

.sortable-drag .server-card {
  border-color: rgba(var(--accent-blue-rgb), 0.3);
}

.empty-servers {
  text-align: center;
  color: var(--text-secondary);
  padding: 60px 40px;
  font-size: 14px;
  line-height: 1.8;
  background: var(--bg-panel);
  border: 1px dashed var(--border-color);
  border-radius: var(--radius-xl);
  box-shadow: var(--shadow-sm);
}

.modal-test-area {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.test-success {
  color: var(--accent-green);
  font-size: 13px;
}

.test-fail {
  color: var(--accent-red);
  font-size: 13px;
}

.security-note {
  margin: 4px 0 16px;
  font-size: 13px;
}

.key-path-row {
  display: flex;
  gap: 8px;
  width: 100%;
}
.key-path-input {
  flex: 1;
}

@media (max-width: 720px) {
  .page-header {
    align-items: stretch;
    flex-direction: column;
  }
}
</style>

<style scoped>
.server-view {
  gap: 18px;
}

.server-form :deep(.n-form-item-label__asterisk) {
  color: var(--accent-red);
}

.server-form :deep(.n-form-item-label__asterisk-placeholder) {
  display: none;
}

.server-modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.server-modal-footer :deep(.n-button) {
  min-width: 76px;
  border-radius: 6px;
}

.server-view .page-header {
  min-height: 42px;
  padding-bottom: 10px;
}

.server-view .page-header h2::before {
  content: none;
}

.server-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(270px, 1fr));
  gap: 16px;
  align-items: start;
}

.server-card {
  display: grid;
  grid-template-rows: auto auto auto;
  align-content: start;
  gap: 18px;
  min-height: 0;
  padding: 18px 20px 17px;
  background: var(--bg-panel);
  border: 1px solid rgba(var(--accent-blue-rgb), 0.16);
  border-radius: 8px;
  box-shadow: 0 1px 2px rgba(27, 34, 47, 0.035);
  transition:
    transform 260ms cubic-bezier(0.22, 1, 0.36, 1),
    border-color 260ms cubic-bezier(0.22, 1, 0.36, 1),
    box-shadow 260ms cubic-bezier(0.22, 1, 0.36, 1);
}

.server-card:hover {
  transform: translateY(-2px);
  border-color: rgba(var(--accent-blue-rgb), 0.36);
  box-shadow: 0 12px 28px rgba(27, 34, 47, 0.075);
}

.server-card > .server-card-header {
  min-height: 36px;
  padding: 0;
}

.server-card > .server-info {
  display: grid;
  gap: 14px;
  padding: 0;
}

.server-card > .card-actions {
  min-height: 32px;
  padding-top: 2px;
}

.server-card-header {
  display: flex;
  align-items: center;
}

.server-heading {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-left: 12px;
  min-width: 0;
}

.server-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  flex: 0 0 auto;
  border-radius: 7px;
  background: rgba(var(--accent-blue-rgb), 0.1);
  border: 1px solid rgba(var(--accent-blue-rgb), 0.12);
  color: var(--accent-blue);
}

.server-name {
  min-width: 0;
  color: var(--text-primary);
  font-size: 16px;
  font-weight: 720;
  line-height: 1.3;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.server-drag-handle {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 24px;
  flex: 0 0 auto;
  border-radius: 5px;
  cursor: grab;
  opacity: 0.32;
  color: var(--text-muted);
  transition: opacity var(--motion-fast), background-color var(--motion-fast);
}

.server-drag-handle:active {
  cursor: grabbing;
}

.server-card-wrapper:hover .server-drag-handle {
  opacity: 0.68;
}

.server-drag-handle:hover {
  opacity: 0.9;
  background: var(--bg-soft);
}

.server-info > .address-block {
  grid-template-columns: 1fr;
  align-items: start;
  gap: 5px;
  min-height: 0;
  padding: 0;
  background: transparent;
  border: 0;
  border-radius: 0;
}

.address-value {
  min-width: 0;
  overflow: hidden;
  color: var(--text-primary);
  font-size: 14px;
  font-weight: 600;
  line-height: 1.45;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 14px 34px;
}

.info-row {
  display: grid;
  align-content: start;
  gap: 5px;
  min-width: 0;
  min-height: 0;
  padding: 0;
  background: transparent;
  border: 0;
  border-radius: 0;
  font-size: 12px;
}

.info-grid .jump-row {
  grid-column: 1 / -1;
}

.info-row .label {
  min-width: 0;
  color: var(--text-muted);
  font-size: 11px;
  font-weight: 650;
}

.info-value {
  min-width: 0;
  overflow: hidden;
  color: var(--text-primary);
  font-size: 13px;
  font-weight: 600;
  line-height: 1.45;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.info-row :deep(.n-tag) {
  justify-self: start;
  width: auto;
  margin: 0;
  padding-inline: 7px;
  border-radius: 4px;
  font-size: 11px;
}

.card-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 32px;
  flex-wrap: wrap;
}

.card-actions .spacer {
  flex: 1;
}

.card-actions :deep(.n-button) {
  min-height: 32px;
  padding-inline: 10px;
  border-radius: 6px;
  font-size: 12px;
}

.status-tag {
  border-radius: 4px;
  font-size: 11px;
}

.empty-servers {
  border-radius: 8px;
  box-shadow: none;
}

@media (max-width: 680px) {
  .info-grid {
    grid-template-columns: 1fr;
    gap: 12px;
  }

  .info-grid .jump-row {
    grid-column: auto;
  }
}

@media (max-width: 520px) {
  .server-card {
    padding: 16px 14px 14px;
  }

  .card-actions :deep(.n-button) {
    padding-inline: 8px;
  }
}
</style>
