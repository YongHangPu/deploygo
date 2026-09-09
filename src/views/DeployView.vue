<script setup lang="ts">
import { computed, ref, onMounted, watch } from 'vue'
import {
  NButton,
  NProgress,
  NTag,
} from 'naive-ui'
import { useProjectStore } from '../stores/projectStore'
import { useServerStore } from '../stores/serverStore'
import { useDeployStore } from '../stores/deployStore'
import { fmtErr } from '../utils/format'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-shell'
import type { LicenseInfo } from '../types'
import PasswordPrompt from '../components/PasswordPrompt.vue'
import { useNotify } from '../composables/useNotify'

const notify = useNotify()
const projectStore = useProjectStore()
const serverStore = useServerStore()
const deployStore = useDeployStore()

const testConnectionLoading = ref(false)
const serverStatus = computed(() => {
  if (!currentServer.value) return null
  return serverStore.testResults[currentServer.value.id] || null
})

// 部署密码提示
const showDeployPasswordPrompt = ref(false)
const deployPassword = ref('')
const deployPasswordLoading = ref(false)

// 测试连接密码提示
const showTestPasswordPrompt = ref(false)
const testPassword = ref('')
const testPasswordLoading = ref(false)

// 许可证购买弹窗
const showPurchaseModal = ref(false)
const purchaseModalReason = ref<'trial' | 'license' | 'shell'>('trial')

const currentProject = computed(() => projectStore.currentProject)
const currentServer = computed(() => {
  if (!currentProject.value) return null
  return serverStore.getServer(currentProject.value.server_id) || null
})
const derivedReleasesRoot = computed(() => {
  const liveRoot = currentProject.value?.live_root_path?.trim().replace(/\\/g, '/').replace(/\/+$/, '')
  if (!liveRoot) return ''

  const lastSlashIndex = liveRoot.lastIndexOf('/')
  if (lastSlashIndex <= 0) return '/releases'

  return `${liveRoot.slice(0, lastSlashIndex)}/releases`
})
const isCurrentServerKeyMode = computed(() => currentServer.value?.auth_method === 'key')

const deploySteps = [
  { step: 0, label: '准备' },
  { step: 1, label: '上传' },
  { step: 2, label: '发布' },
  { step: 3, label: '完成' },
]

onMounted(async () => {
  // 切回本页时清空上次遗留的部署状态（部署中不清，事件流会重建状态）
  if (!deployStore.isRunning) {
    deployStore.clearLogs()
  }
})

// 切换项目时重置部署状态。
watch(
  () => projectStore.currentProjectId,
  () => {
    deployStore.clearLogs()
    if (currentServer.value) {
      delete serverStore.testResults[currentServer.value.id]
    }
    testConnectionLoading.value = false
  }
)

const testServerConnection = async () => {
  if (!currentServer.value) return
  // 检查内存中是否缓存了密码。
  const cachedPwd = serverStore.getCachedPassword(currentServer.value.id)
  if (cachedPwd) {
    doTestConnection(cachedPwd)
    return
  }
  // 不传入密码测试，让后端从数据库解析已保存的密码或私钥口令。
  doTestConnection()
}

const doTestConnection = async (password?: string) => {
  if (!currentServer.value) return
  deployStore.addLog(`🔍 正在测试服务器 ${currentServer.value.name} 连接...`)
  serverStore.testResults[currentServer.value.id] = null as any
  testConnectionLoading.value = true
  try {
    const latency = await serverStore.testConnection(currentServer.value.id, password)
    if (password) {
      serverStore.cachePassword(currentServer.value.id, password)
      await serverStore.saveServer(currentServer.value, password)
    }
    serverStore.testResults[currentServer.value.id] = { connected: true, latency }
  } catch (e: any) {
    const errMsg = typeof e === 'string' ? e : (e?.message || '')
    if (!password && (errMsg.includes('密码未找到') || errMsg.includes('需要密码短语'))) {
      testPassword.value = ''
      showTestPasswordPrompt.value = true
      return
    }
    serverStore.testResults[currentServer.value.id] = { connected: false, error: fmtErr(e) }
  } finally {
    testConnectionLoading.value = false
  }
}

const testConnectionWithPassword = async () => {
  if (!testPassword.value) {
    notify.warning(isCurrentServerKeyMode.value ? '请输入密钥密码短语' : '请输入密码')
    return
  }
  testPasswordLoading.value = true
  showTestPasswordPrompt.value = false
  try {
    await doTestConnection(testPassword.value)
  } finally {
    testPasswordLoading.value = false
  }
}

const handleDeploy = async () => {
  if (deployStore.isRunning) {
    notify.warning('部署任务正在进行中，请等待完成')
    return
  }
  if (!currentProject.value) {
    notify.warning('请先选择项目')
    return
  }
  if (!currentServer.value) {
    notify.warning('项目未关联服务器，无法部署')
    return
  }

  // 许可证检查：试用期结束或许可证无效时阻止部署。
  try {
    const license = await invoke<LicenseInfo>('get_license_status')
    if (!license.can_deploy) {
      // 区分原因：开源纯壳、试用期结束还是正式许可证过期。
      if (license.engine === 'shell') {
        purchaseModalReason.value = 'shell'
      } else if (license.tier === 'free') {
        purchaseModalReason.value = 'trial'
      } else {
        purchaseModalReason.value = 'license'
      }
      showPurchaseModal.value = true
      return
    }
  } catch (e: any) {
    notify.error(`许可证状态检查失败: ${fmtErr(e)}`)
    return
  }

  // 部署前检查：确认本地构建输出目录存在。
  const distPath = currentProject.value.local_dist_path
  try {
    await invoke('validate_build_dir', { path: distPath })
  } catch (e: any) {
    notify.error(typeof e === 'string' ? e : (e?.message || '目录验证失败'))
    return
  }

  // 检查内存中是否缓存了密码。
  const cachedPwd = serverStore.getCachedPassword(currentServer.value.id)
  if (cachedPwd) {
    doDeploy(cachedPwd)
  } else {
    // 不传入密码尝试部署，让后端从数据库解析已保存的密码或私钥口令。
    try {
      await doDeploy()
    } catch {
      // doDeploy 已处理“找不到密码”的情况，会自动弹出输入提示。
    }
  }
}

const doDeploy = async (password?: string) => {
  if (!currentProject.value || !currentServer.value) return

  deployStore.clearLogs()
  deployStore.addLog('开始一键部署...')
  deployStore.addLog(`项目: ${currentProject.value.name}`)
  deployStore.addLog(`服务器: ${currentServer.value.name} (${currentServer.value.host})`)

  if (password) {
    serverStore.cachePassword(currentServer.value.id, password)
  }

  try {
    await deployStore.startDeploy(currentProject.value.id, password)
    if (deployStore.deployStepStatus === 'done') {
      notify.success('部署完成')
    } else if (deployStore.deployStepStatus === 'cancelled') {
      notify.warning('部署已取消')
    } else {
      notify.error('部署失败')
    }
  } catch (e: any) {
    const errMsg = typeof e === 'string' ? e : (e?.message || '')
    if (!password && (errMsg.includes('密码未找到') || errMsg.includes('需要密码短语'))) {
      // 没有保存的密码或密钥口令，提示用户输入。
      deployPassword.value = ''
      showDeployPasswordPrompt.value = true
      return
    }
    if (errMsg.includes('检测到重复发布')) {
      notify.warning(errMsg, { log: false })
      return
    }
    notify.error(`部署失败: ${fmtErr(e)}`)
  }
}

const startDeployWithPassword = async () => {
  if (!deployPassword.value) {
    notify.warning(isCurrentServerKeyMode.value ? '请输入密钥密码短语' : '请输入密码')
    return
  }
  deployPasswordLoading.value = true
  showDeployPasswordPrompt.value = false
  await doDeploy(deployPassword.value)
  deployPasswordLoading.value = false
}

const handleCancel = async () => {
  const result = await deployStore.cancelDeploy()
  if (result === 'accepted') {
    notify.warning('正在取消部署')
  } else if (result === 'too_late') {
    notify.info('发布已进入提交阶段，无法取消')
  }
}

const isStepDone = (step: number) =>
  deployStore.deployStep > step ||
  (deployStore.deployStep === step && deployStore.deployStepStatus === 'done')

const isStepActive = (step: number) =>
  deployStore.deployStep === step &&
  ['running', 'committing', 'cancelling'].includes(deployStore.deployStepStatus)

const isStepError = (step: number) =>
  deployStore.deployStep === step && deployStore.deployStepStatus === 'error'

const isStepCancelled = (step: number) =>
  deployStore.deployStep === step && deployStore.deployStepStatus === 'cancelled'

</script>

<template>
  <div class="page-container deploy-view">
    <!-- 未选择项目 -->
    <div v-if="!currentProject" class="empty-state panel-card">
      <div class="empty-icon" aria-hidden="true">
        <svg viewBox="0 0 24 24" width="30" height="30" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">
          <path d="m4 7 8-4 8 4-8 4-8-4Z" />
          <path d="m4 12 8 4 8-4" />
          <path d="m4 17 8 4 8-4" />
        </svg>
      </div>
      <h2>欢迎使用零停机部署工具</h2>
      <p>请先在左侧添加项目并配置服务器</p>
      <div class="empty-steps">
        <div class="step-card animate-in animate-in-d1">
          <div class="step-num">1</div>
          <div>进入「服务器管理」添加服务器信息</div>
        </div>
        <div class="step-card animate-in animate-in-d3">
          <div class="step-num">2</div>
          <div>点击项目列表右侧的 + 图标创建项目</div>
        </div>
        <div class="step-card animate-in animate-in-d4">
          <div class="step-num">3</div>
          <div>选择项目，点击「一键部署」</div>
        </div>
      </div>
    </div>

    <!-- 已选择项目 -->
    <div v-else class="deploy-content">
      <!-- 信息栏：占满宽度 -->
      <div class="info-bar panel-card animate-in animate-in-d1">
        <div class="info-item">
          <span class="info-label">项目</span>
          <span class="info-value">{{ currentProject.name }}</span>
        </div>
        <div class="info-item" v-if="currentServer">
          <span class="info-label">服务器</span>
          <span class="info-value">
            {{ currentServer.name }}
            <n-tag :type="testConnectionLoading ? 'info' : serverStatus?.connected ? 'success' : serverStatus?.error ? 'error' : 'default'" size="tiny">
              {{
                testConnectionLoading
                  ? '测试中'
                  : serverStatus?.connected
                    ? `已连接，耗时 ${serverStatus.latency}ms`
                    : serverStatus?.error
                      ? '连接失败'
                      : '未测试'
              }}
            </n-tag>
          </span>
        </div>
        <n-button size="small" secondary :loading="testConnectionLoading" @click="testServerConnection" v-if="currentServer">
          测试连接
        </n-button>
      </div>

      <!-- 双栏网格：路径信息 + 构建信息 -->
      <div class="deploy-grid-2">
        <section class="path-card panel-card animate-in animate-in-d2" v-if="currentProject">
          <h3 class="surface-card-title">部署路径</h3>
          <div class="path-grid">
            <div class="path-row">
              <span class="path-label">线上站点目录:</span>
              <code class="path-value">{{ currentProject.live_root_path }}</code>
            </div>
            <div class="path-row">
              <span class="path-label">版本归档目录:</span>
              <code class="path-value">{{ derivedReleasesRoot || currentProject.releases_root_path }}</code>
            </div>
          </div>
        </section>

        <section class="build-card panel-card animate-in animate-in-d3">
          <h3 class="surface-card-title">本地发布产物</h3>
          <div class="build-info">
            <div class="build-path">
              <span>路径:</span>
              <code>{{ currentProject.local_dist_path }}</code>
            </div>
            <div class="build-tip">
              deploygo 会自动识别最终构建目录，并在 <code>dist/.deploygo</code> 下优先生成
              <code>&lt;release_name&gt;.tar.gz</code>。
              如果压缩失败，会自动回退生成 <code>release</code> 目录后再上传。
            </div>
          </div>
        </section>
      </div>

      <!-- 部署面板 -->
      <div class="deploy-panel animate-in animate-in-d4">
        <!-- 进度区域 -->
          <section class="progress-card panel-card" v-if="deployStore.uploadProgress">
            <h3 class="surface-card-title">上传进度</h3>
            <div class="progress-info">
            <n-progress
              :percentage="Math.round(deployStore.uploadProgress.percent)"
              :indicator-placement="'inside'"
              type="line"
              :height="20"
            />
            <div class="progress-text">
              正在上传: {{ deployStore.uploadProgress.filename }}
              ({{ deployStore.uploadProgress.current }} / {{ deployStore.uploadProgress.total }})
            </div>
            </div>
          </section>

        <!-- 部署步骤 -->
        <div class="deploy-steps">
          <div
            v-for="s in deploySteps"
            :key="s.step"
            class="deploy-step animate-in"
            :class="[`animate-in-d${s.step + 1}`, {
              active: isStepActive(s.step),
              done: isStepDone(s.step),
              error: isStepError(s.step),
              cancelled: isStepCancelled(s.step),
            }]"
          >
            <div class="step-indicator">
              <span v-if="isStepDone(s.step)">✓</span>
              <span v-else-if="isStepCancelled(s.step)">⚠</span>
              <span v-else-if="isStepError(s.step)">✕</span>
              <span v-else-if="isStepActive(s.step)">◉</span>
              <span v-else>{{ s.step + 1 }}</span>
            </div>
            <span class="step-label">{{ s.label }}</span>
          </div>
        </div>

        <!-- 操作按钮 -->
        <div class="action-buttons">
          <n-button
            type="primary"
            size="large"
            class="deploy-action-main"
            :loading="deployStore.isRunning"
            :disabled="deployStore.isRunning"
            @click="handleDeploy"
          >
            一键部署
          </n-button>
          <n-button
            type="warning"
            size="large"
            v-if="deployStore.isRunning && !['committing', 'cancelling'].includes(deployStore.deployStepStatus)"
            @click="handleCancel"
          >
            取消
          </n-button>
        </div>
      </div>
    </div>

    <!-- 许可证购买弹窗 -->
    <n-modal v-model:show="showPurchaseModal" title="功能受限" preset="dialog" type="warning"
      :positive-text="purchaseModalReason === 'shell' ? '下载完整版' : '前往购买'"
      negative-text="稍后再说"
      @positive-click="() => { showPurchaseModal = false; open(purchaseModalReason === 'shell' ? 'https://github.com/YongHangPu/deploygo/releases' : 'https://mbd.pub/o/deploygo') }"
      @negative-click="() => { showPurchaseModal = false }"
    >
      <template v-if="purchaseModalReason === 'shell'">
        <div style="line-height: 1.8;">
          <p>当前为开源展示版构建，不包含部署引擎。</p>
          <p>请从 GitHub Releases 下载官方完整版（提供 90 天免费试用）。</p>
        </div>
      </template>
      <template v-else-if="purchaseModalReason === 'trial'">
        <div style="line-height: 1.8;">
          <p>试用期已结束，部署功能已暂停。</p>
          <p>购买专业版即可继续使用，仅需 ¥99/年。</p>
        </div>
      </template>
      <template v-else>
        <div style="line-height: 1.8;">
          <p>授权已到期，部署功能已暂停。</p>
          <p>请续费专业版以继续使用，仅需 ¥99/年。</p>
        </div>
      </template>
    </n-modal>

    <!-- 部署密码提示 -->
    <PasswordPrompt
      v-model:visible="showDeployPasswordPrompt"
      v-model="deployPassword"
      :server-name="currentServer?.name || ''"
      description="以开始部署"
      :loading="deployPasswordLoading"
      confirm-label="开始部署"
      :is-key-mode="isCurrentServerKeyMode"
      @confirm="startDeployWithPassword"
    />

    <!-- 测试连接密码提示 -->
    <PasswordPrompt
      v-model:visible="showTestPasswordPrompt"
      v-model="testPassword"
      :server-name="currentServer?.name || ''"
      description="以测试连接"
      :loading="testPasswordLoading"
      confirm-label="测试连接"
      :is-key-mode="isCurrentServerKeyMode"
      @confirm="testConnectionWithPassword"
    />
  </div>
</template>

<style scoped>
.deploy-view {
}

.deploy-content {
  display: grid;
  gap: 20px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 100%;
  padding: 40px 24px;
  text-align: center;
  color: var(--text-secondary);
  gap: 12px;
}

.empty-icon {
  font-size: 56px;
  margin-bottom: 8px;
  opacity: 0.85;
}

.empty-state h2 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
}

.empty-state p {
  margin: 0;
  font-size: 14px;
}

.empty-steps {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 200px), 1fr));
  gap: 16px;
  margin-top: 16px;
  width: 100%;
  max-width: 700px;
}

.step-card {
  background: var(--bg-card);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: 20px;
  text-align: left;
  font-size: 13px;
  line-height: 1.6;
  transition: border-color var(--motion-fast), box-shadow var(--motion-fast), transform var(--motion-fast), background var(--motion-fast);
}
.step-card:hover {
  border-color: rgba(var(--accent-blue-rgb), 0.35);
  box-shadow: var(--shadow-sm);
  transform: translateY(-2px);
}

.step-num {
  width: 28px;
  height: 28px;
  background: var(--accent-blue);
  color: #fff;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 10px;
  font-weight: 700;
  font-size: 13px;
}

/* 信息栏 */
.info-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 24px;
  padding: 16px 20px;
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}

.info-label {
  font-size: 11px;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  font-weight: 600;
}

.info-value {
  font-size: 14px;
  font-weight: 500;
  display: flex;
  align-items: center;
  gap: 8px;
}

/* 路径卡片 */
.path-card {
  border-radius: var(--radius-lg);
  margin-bottom: 0;
}

.build-card {
  border-radius: var(--radius-lg);
  margin-bottom: 0;
}

.deploy-grid-2 {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--grid-gap);
}

.deploy-grid-2 > :deep(.n-card) {
  height: 100%;
}

@media (max-width: 768px) {
  .deploy-grid-2 {
    grid-template-columns: 1fr;
  }
}

.path-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.path-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 12px;
  padding: 10px 12px;
  border-radius: var(--radius-md);
  background: var(--bg-soft);
}

.path-label {
  font-size: 12px;
  color: var(--text-muted);
  min-width: 100px;
  font-weight: 500;
  flex-shrink: 0;
  white-space: nowrap;
}

.path-value {
  font-size: 12px;
  background: var(--bg-code);
  padding: 4px 10px;
  border-radius: var(--radius-sm);
  color: var(--accent-blue);
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  border: 1px solid var(--border-subtle);
  word-break: break-all;
}

/* 部署面板 */
.deploy-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.deploy-panel :deep(.n-card) {
  border-radius: var(--radius-lg);
}

.build-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.build-path {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
}

.build-path span {
  white-space: nowrap;
  flex-shrink: 0;
  color: var(--text-muted);
  font-weight: 500;
  padding-top: 2px;
}

.build-path code {
  background: var(--bg-code);
  padding: 4px 10px;
  border-radius: var(--radius-sm);
  color: var(--accent-blue);
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  font-size: 12px;
  border: 1px solid var(--border-subtle);
  word-break: break-all;
}

.build-tip {
  font-size: 12px;
  line-height: 1.6;
  color: var(--text-muted);
}

.progress-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.progress-text {
  font-size: 12px;
  color: var(--text-muted);
}

/* 部署步骤 */
.deploy-steps {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
}

@media (max-width: 640px) {
  .deploy-steps {
    grid-template-columns: repeat(2, 1fr);
  }
}

.deploy-step {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 10px 12px;
  background: var(--bg-panel);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  font-size: 13px;
  color: var(--text-muted);
  transition: background var(--motion-fast), color var(--motion-fast), border-color var(--motion-fast), transform var(--motion-fast), box-shadow var(--motion-fast);
}

.deploy-step.active {
  border-color: var(--accent-blue);
  color: var(--accent-blue);
  background: rgba(var(--accent-blue-rgb), 0.08);
  box-shadow: 0 0 0 1px rgba(var(--accent-blue-rgb), 0.24);
}

.deploy-step.done {
  border-color: var(--accent-green);
  color: var(--accent-green);
  background: rgba(var(--accent-green-rgb), 0.08);
}

.deploy-step.error {
  border-color: var(--accent-red);
  color: var(--accent-red);
  background: rgba(var(--accent-red-rgb), 0.08);
}

.deploy-step.cancelled {
  border-color: var(--accent-yellow);
  color: var(--accent-yellow);
  background: rgba(var(--accent-yellow-rgb), 0.08);
}

.step-indicator {
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  font-weight: 700;
  border-radius: 50%;
  background: var(--bg-soft);
}

.deploy-step.active .step-indicator,
.deploy-step.done .step-indicator,
.deploy-step.cancelled .step-indicator {
  background: transparent;
}

/* 摘要卡片 */
.summary-card {
  border-color: var(--accent-green);
}
.summary-grid {
  display: flex;
  gap: 24px;
  flex-wrap: wrap;
}
.summary-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.summary-label {
  font-size: 11px;
  color: var(--text-muted);
  font-weight: 500;
}
.summary-value {
  font-size: 16px;
  font-weight: 600;
}

/* 操作按钮 */
.action-buttons {
  display: flex;
  gap: 12px;
  margin-top: 4px;
  flex-wrap: wrap;
}

.deploy-action-main {
  flex: 1;
}

.action-buttons :deep(.n-button) {
  min-width: 120px;
  border-radius: var(--radius-md);
}

.action-buttons :deep(.n-button--primary-type) {
  font-size: 15px;
  font-weight: 600;
  padding: 12px 24px;
}

.status-loading, .status-success, .status-error {
  padding: 20px;
  text-align: center;
  font-size: 14px;
}

.status-success { color: var(--accent-green); }
.status-error { color: var(--accent-red); }

@media (max-width: 900px) {
  .info-bar {
    gap: 14px;
  }
}

@media (max-width: 640px) {
  .action-buttons {
    flex-direction: column;
  }

  .deploy-action-main {
    width: 100%;
  }
}
</style>

<style scoped>
.deploy-content {
  gap: 18px;
}

.empty-state {
  min-height: 100%;
  border-style: dashed;
  box-shadow: none;
}

.empty-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 64px;
  height: 64px;
  margin-bottom: 4px;
  border: 1px solid rgba(var(--accent-blue-rgb), 0.36);
  border-radius: 8px;
  background: var(--bg-chip);
  color: var(--accent-blue);
}

.empty-state h2 {
  font-size: 19px;
  font-weight: 750;
}

.empty-steps {
  max-width: 760px;
  gap: 10px;
}

.step-card {
  padding: 8px 14px;
  border: 0;
  border-left: 2px solid rgba(var(--accent-blue-rgb), 0.28);
  border-radius: 0;
  background: transparent;
  box-shadow: none;
}

.step-card:hover {
  transform: none;
  box-shadow: none;
  border-left-color: var(--accent-blue);
  background: var(--bg-soft);
}

.step-num {
  width: 24px;
  height: 24px;
  margin-bottom: 8px;
  border-radius: 4px;
  background: var(--bg-chip);
  border: 1px solid rgba(var(--accent-blue-rgb), 0.24);
  color: var(--accent-blue);
  font: 700 11px/1 'JetBrains Mono', monospace;
}

.info-bar {
  position: relative;
  min-height: 74px;
  gap: 18px;
  padding: 14px 18px 14px 21px;
  border-left: 3px solid var(--accent-blue);
}

.info-item {
  gap: 5px;
}

.info-item:first-child {
  min-width: min(260px, 100%);
}

.info-label {
  font: 700 10px/1.2 'JetBrains Mono', monospace;
  letter-spacing: 0.08em;
}

.info-value {
  font-size: 13px;
  font-weight: 750;
}

.deploy-grid-2 {
  gap: 14px;
}

.path-card,
.build-card,
.progress-card {
  display: flex;
  flex-direction: column;
  gap: 0;
  padding: 18px;
  border-radius: 8px;
  box-shadow: 0 1px 2px rgba(27, 34, 47, 0.035);
  transition:
    transform 260ms cubic-bezier(0.22, 1, 0.36, 1),
    border-color 260ms cubic-bezier(0.22, 1, 0.36, 1),
    box-shadow 260ms cubic-bezier(0.22, 1, 0.36, 1);
}

.path-card:hover,
.build-card:hover,
.progress-card:hover {
  transform: translateY(-1px);
  border-color: rgba(var(--accent-blue-rgb), 0.32);
  box-shadow: 0 10px 24px rgba(27, 34, 47, 0.065);
}

.surface-card-title {
  margin: 0 0 14px;
  color: var(--text-primary);
  font-size: 12px;
  font-weight: 750;
  letter-spacing: 0.01em;
}

.progress-card {
  padding-bottom: 16px;
}

.path-row {
  gap: 10px;
  padding: 9px 10px;
  border-radius: 5px;
  border: 1px solid transparent;
}

.path-row + .path-row {
  margin-top: 3px;
}

.path-label {
  min-width: 106px;
  font: 650 10px/1.5 'JetBrains Mono', monospace;
  letter-spacing: 0.025em;
}

.path-value,
.build-path code {
  padding: 3px 7px;
  border-radius: 3px;
  color: var(--accent-blue);
}

.build-info {
  gap: 11px;
}

.build-path {
  font-size: 12px;
}

.build-tip {
  padding-top: 0;
  line-height: 1.7;
}

.deploy-panel {
  gap: 12px;
}

.deploy-steps {
  position: relative;
  gap: 0;
  padding: 0 2px;
}

.deploy-steps::before {
  content: '';
  position: absolute;
  top: 18px;
  right: 12%;
  left: 12%;
  height: 1px;
  background: var(--border-color);
}

.deploy-step {
  position: relative;
  z-index: 1;
  flex-direction: column;
  min-height: 60px;
  gap: 6px;
  padding: 0 8px;
  border: 0;
  border-radius: 0;
  background: transparent;
  font-size: 11px;
  font-weight: 700;
}

.deploy-step.active,
.deploy-step.done,
.deploy-step.error,
.deploy-step.cancelled {
  border: 0;
  box-shadow: none;
}

.step-indicator {
  width: 36px;
  height: 36px;
  border: 1px solid var(--border-color);
  border-radius: 50%;
  background: var(--bg-elevated);
  font: 700 11px/1 'JetBrains Mono', monospace;
}

.deploy-step.active .step-indicator {
  border-color: var(--accent-blue);
  box-shadow: 0 0 0 4px rgba(var(--accent-blue-rgb), 0.12);
}

.deploy-step.done .step-indicator {
  border-color: var(--accent-green);
}

.deploy-step.error .step-indicator {
  border-color: var(--accent-red);
}

.deploy-step.cancelled .step-indicator {
  border-color: var(--accent-yellow);
}

.action-buttons {
  align-items: center;
  justify-content: flex-start;
  min-height: 48px;
  padding-top: 6px;
  border-top: 0;
}

.deploy-action-main {
  min-width: 156px;
  border-radius: 6px;
  box-shadow: none;
}

@media (max-width: 640px) {
  .path-card,
  .build-card,
  .progress-card {
    padding: 16px;
  }
}

@media (max-width: 640px) {
  .deploy-steps::before {
    display: none;
  }

  .deploy-step {
    min-height: 80px;
    border: 1px solid var(--border-subtle);
    border-radius: 6px;
  }
}
</style>
