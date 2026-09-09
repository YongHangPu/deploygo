import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { invokeWithLoading } from '../utils/invoke'
import { listen } from '@tauri-apps/api/event'
import { fmtTime } from '../utils/format'
import type { DeployRecord, UploadProgress, DeployStepEvent } from '../types'

export const useDeployStore = defineStore('deploy', () => {
  const isRunning = ref(false)
  const deployStep = ref<number>(0)
  const deployStepStatus = ref<string>('')
  const uploadProgress = ref<UploadProgress | null>(null)
  const logs = ref<Array<{ time: string; type: string; text: string }>>([])
  const history = ref<DeployRecord[]>([])
  const historyLoading = ref(false)
  const historyTotal = ref(0)

  let unlistenList: Array<() => void> = []

  async function setupListeners() {
  // 清理之前注册的监听器。
    await cleanupListeners()

    unlistenList.push(
      await listen<UploadProgress>('upload:progress', (event) => {
        uploadProgress.value = event.payload
      })
    )

    unlistenList.push(
      await listen<string>('deploy:log', (event) => {
        appendLogLines(event.payload, 'info')
      })
    )

    unlistenList.push(
      await listen<DeployStepEvent>('deploy:step', (event) => {
        deployStep.value = event.payload.step
        deployStepStatus.value = event.payload.status
      })
    )

    unlistenList.push(
      await listen<string>('script:stdout', (event) => {
        appendLogLines(event.payload, 'stdout')
      })
    )

    unlistenList.push(
      await listen<string>('script:stderr', (event) => {
        appendLogLines(event.payload, 'stderr')
      })
    )
  }

  async function cleanupListeners() {
    for (const unlisten of unlistenList) {
      unlisten()
    }
    unlistenList = []
  }

  async function startDeploy(projectId: string, password?: string) {
    uploadProgress.value = null
    deployStep.value = 0
    deployStepStatus.value = 'running'
    await invokeWithLoading(isRunning, 'start_deploy', { projectId, password: password || null })
  }

  async function cancelDeploy(): Promise<'accepted' | 'too_late' | 'not_running'> {
    const result = await invoke<'accepted' | 'too_late' | 'not_running'>('cancel_deploy')
    if (result === 'accepted') {
      deployStepStatus.value = 'cancelling'
      addLog('⚠ 正在取消部署...', 'warn')
    } else if (result === 'too_late') {
      addLog('ℹ 发布已进入提交阶段，当前操作将继续完成', 'info')
    }
    return result
  }

  async function loadHistory(params?: {
    projectId?: string
    serverId?: string
    status?: string
    dateFrom?: string
    dateTo?: string
    limit?: number
    offset?: number
  }) {
    const result = await invokeWithLoading<{ data: DeployRecord[]; total: number }>(
      historyLoading,
      'get_deploy_history',
      {
        projectId: params?.projectId || null,
        serverId: params?.serverId || null,
        status: params?.status || null,
        dateFrom: params?.dateFrom || null,
        dateTo: params?.dateTo || null,
        limit: params?.limit ?? 20,
        offset: params?.offset ?? 0,
      },
    )
    history.value = result.data
    historyTotal.value = result.total
  }

  async function rollbackToVersion(recordId: number, password?: string) {
    uploadProgress.value = null
    deployStep.value = 0
    deployStepStatus.value = 'running'
    await invokeWithLoading(isRunning, 'rollback_to_version', { recordId, password: password || null })
  }

  async function exportLog(recordId: number): Promise<string> {
    return await invoke<string>('export_log', { recordId })
  }

  async function deleteRecord(recordId: number): Promise<void> {
    await invoke('delete_deploy_record', { recordId })
  }

  function addLog(text: string, type: string = 'info') {
    appendLogLines(text, type)
  }

  function clearLogs() {
    logs.value = []
    uploadProgress.value = null
    deployStep.value = 0
    deployStepStatus.value = ''
  }

  function hasDeployStep(step: number): boolean {
    return deployStepStatus.value !== '' && deployStep.value >= step
  }

  function appendLogLines(text: string, type: string = 'info') {
    const lines = text
      .split(/\r?\n/)
      .map(line => line.trim())
      .filter(Boolean)

    for (const line of lines) {
      const previous = logs.value[logs.value.length - 1]
      if (previous?.type === type && previous.text === line) {
        continue
      }

      logs.value.push({
        time: fmtTime(),
        type,
        text: line,
      })
    }
  }

  return {
    isRunning,
    deployStep,
    deployStepStatus,
    uploadProgress,
    logs,
    history,
    historyTotal,
    historyLoading,
    setupListeners,
    cleanupListeners,
    startDeploy,
    cancelDeploy,
    loadHistory,
    rollbackToVersion,
    exportLog,
    addLog,
    clearLogs,
    hasDeployStep,
    deleteRecord,
  }
})
