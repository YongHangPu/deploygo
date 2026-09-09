<script setup lang="ts">
import { computed, h, ref, onMounted, reactive, watch } from 'vue'
import {
  NButton,
  NTag,
  NModal,
  NDataTable,
  NPagination,
  NSelect,
  NDatePicker,
  NSpace,
  useDialog,
} from 'naive-ui'
import type { DataTableColumns } from 'naive-ui'
import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'
import { useProjectStore } from '../stores/projectStore'
import { useServerStore } from '../stores/serverStore'
import { useDeployStore } from '../stores/deployStore'
import { useNotify } from '../composables/useNotify'
import { fmtErr } from '../utils/format'
import type { DeployRecord } from '../types'

const notify = useNotify()
const dialog = useDialog()
const projectStore = useProjectStore()
const serverStore = useServerStore()
const deployStore = useDeployStore()

const showLogModal = ref(false)
const selectedRecord = ref<DeployRecord | null>(null)

// 服务端筛选状态
const projectFilter = ref('')
const serverFilter = ref('')
const statusFilter = ref('')
const dateRange = ref<[number, number] | null>(null)
const page = ref(1)
const pageSize = ref(20)

onMounted(async () => {
  await fetchHistory()
})

/** 根据当前筛选状态构造请求参数。 */
const buildFilter = (extra?: { limit?: number; offset?: number }) => {
  const dateFrom = dateRange.value?.[0] ? new Date(dateRange.value[0]).toISOString().slice(0, 10) : undefined
  const dateTo = dateRange.value?.[1] ? new Date(dateRange.value[1]).toISOString().slice(0, 10) : undefined
  return {
    projectId: projectFilter.value || undefined,
    serverId: serverFilter.value || undefined,
    status: statusFilter.value || undefined,
    dateFrom,
    dateTo,
    limit: extra?.limit ?? pageSize.value,
    offset: extra?.offset ?? (page.value - 1) * pageSize.value,
  }
}

const fetchHistory = async () => {
  await deployStore.loadHistory(buildFilter())
  paginationConfig.page = page.value
  paginationConfig.pageSize = pageSize.value
  paginationConfig.itemCount = deployStore.historyTotal
}

const onFilterChange = () => {
  page.value = 1
  fetchHistory()
}

const hasActiveFilters = computed(() => {
  const hasDateRange = Array.isArray(dateRange.value) && dateRange.value.some(Boolean)
  return Boolean(projectFilter.value || serverFilter.value || statusFilter.value || hasDateRange)
})

const resetFilters = async () => {
  if (!hasActiveFilters.value) return
  projectFilter.value = ''
  serverFilter.value = ''
  statusFilter.value = ''
  dateRange.value = null
  page.value = 1
  await fetchHistory()
}

const paginationConfig = reactive({
  page: page.value,
  pageSize: pageSize.value,
  itemCount: deployStore.historyTotal,
  pageSizes: [10, 20, 30, 40, 50, 100],
  showSizePicker: true,
  onChange: async (nextPage: number) => {
    page.value = nextPage
    paginationConfig.page = nextPage
    await fetchHistory()
  },
  onUpdatePageSize: async (nextPageSize: number) => {
    pageSize.value = nextPageSize
    page.value = 1
    paginationConfig.pageSize = nextPageSize
    paginationConfig.page = 1
    await fetchHistory()
  }
})

watch(
  () => deployStore.historyTotal,
  (total) => {
    paginationConfig.itemCount = total
  }
)

const openLog = (record: DeployRecord) => {
  selectedRecord.value = record
  showLogModal.value = true
}

const exportHistoryLog = async () => {
  if (!selectedRecord.value) return
  const filePath = await save({
    defaultPath: `deploy-log-${selectedRecord.value.id}-${new Date(selectedRecord.value.created_at).toISOString().slice(0, 10)}.txt`,
    filters: [{ name: '文本文件', extensions: ['txt'] }],
  })
  if (!filePath) return
  await writeTextFile(filePath, selectedRecord.value.log)
  notify.success('日志已导出')
}

const deleteHistoryRecord = async (record: DeployRecord) => {
  const ts = new Date(record.created_at).toLocaleString()
  const confirmed = await new Promise<boolean>((resolve) => {
    dialog.warning({
      title: '删除记录',
      content: `确定要删除 ${ts} 的部署记录吗？该操作仅删除本地历史记录，不影响服务器文件。`,
      positiveText: '确定删除',
      negativeText: '取消',
      onPositiveClick: () => { resolve(true) },
      onNegativeClick: () => { resolve(false) },
      onClose: () => { resolve(false) },
    })
  })
  if (!confirmed) return
  try {
    await deployStore.deleteRecord(record.id)
    notify.success('记录已删除')
    await fetchHistory()
  } catch {
    notify.error('删除失败')
  }
}

const rollbackTo = async (record: DeployRecord) => {
  if (deployStore.isRunning) {
    notify.warning('部署任务正在进行中，请等待完成')
    return
  }
  const confirmed = await new Promise<boolean>((resolve) => {
    dialog.warning({
      title: '回滚确认',
      content: `确定回滚到版本 ${record.release_dir_path} 吗？`,
      positiveText: '确定回滚',
      negativeText: '取消',
      onPositiveClick: () => { resolve(true) },
      onNegativeClick: () => { resolve(false) },
      onClose: () => { resolve(false) },
    })
  })
  if (!confirmed) return
  try {
    deployStore.clearLogs()
    deployStore.addLog(`回滚到: ${record.release_dir_path}`)
    const serverId = record.server_id
    // 优先尝试缓存密码（内存），再让后端解析数据库中已保存的密码。
    const cachedPwd = serverStore.getCachedPassword(serverId)
    if (cachedPwd) {
      await deployStore.rollbackToVersion(record.id, cachedPwd)
    } else {
      // 不传入密码尝试，让后端从数据库解析已保存的密码。
      await deployStore.rollbackToVersion(record.id)
    }
    notify.success('回滚完成')
    await fetchHistory()
  } catch (e: any) {
    const errMsg = typeof e === 'string' ? e : (e?.message || '')
    if (errMsg.includes('密码未找到')) {
      // 提示用户输入密码。
      const pwd = prompt('请输入服务器密码以执行回滚:')
      if (!pwd) return
      serverStore.cachePassword(record.server_id, pwd)
      try {
        deployStore.clearLogs()
        deployStore.addLog(`回滚到: ${record.release_dir_path}`)
        await deployStore.rollbackToVersion(record.id, pwd)
        notify.success('回滚完成')
        await fetchHistory()
      } catch (e2: any) {
        notify.error(`回滚失败: ${fmtErr(e2)}`)
      }
      return
    }
    notify.error(`回滚失败: ${fmtErr(e)}`)
  }
}

const columns: DataTableColumns<DeployRecord> = [
  {
    title: '项目',
    key: 'project_name',
    width: 150,
    ellipsis: { tooltip: true },
    resizable: true,
    minWidth: 110,
    render: (row: DeployRecord) => h('div', { class: 'history-identity' }, [
      h('span', { class: 'history-identity__marker' }),
      h('span', { class: 'history-identity__value', title: row.project_name }, row.project_name),
    ]),
  },
  {
    title: '服务器',
    key: 'server_name',
    width: 150,
    ellipsis: { tooltip: true },
    resizable: true,
    minWidth: 110,
    render: (row: DeployRecord) => h('div', { class: 'history-identity history-identity--server' }, [
      h('span', { class: 'history-identity__marker' }),
      h('span', { class: 'history-identity__value', title: row.server_name }, row.server_name),
    ]),
  },
  {
    title: '版本目录',
    key: 'release_dir_path',
    minWidth: 220,
    ellipsis: { tooltip: true },
    resizable: true,
    render: (row: DeployRecord) => h('code', { class: 'history-release', title: row.release_dir_path }, row.release_dir_path),
  },
  { title: '状态', key: 'status', width: 80, render: (row: DeployRecord) => {
    const typeMap: Record<string, 'success' | 'error' | 'warning' | 'info'> = {
      success: 'success', failed: 'error', cancelled: 'warning', rollback: 'info',
    }
    const labelMap: Record<string, string> = {
      success: '成功', failed: '失败', cancelled: '已取消', rollback: '回滚',
    }
    return h(NTag, { type: typeMap[row.status] || 'default', size: 'tiny' }, () => labelMap[row.status] || row.status)
  }},
  {
    title: '耗时',
    key: 'duration_ms',
    width: 96,
    render: (row: DeployRecord) => h('span', { class: 'history-duration' }, `${(row.duration_ms / 1000).toFixed(1)}s`),
  },
  {
    title: '时间',
    key: 'created_at',
    width: 172,
    render: (row: DeployRecord) => h('time', { class: 'history-time' }, new Date(row.created_at).toLocaleString()),
  },
  {
    title: '操作',
    key: 'actions',
    width: 160,
    fixed: 'right',
    render: (row: DeployRecord) => {
      return h(NSpace, { size: 'small', class: 'history-row-actions' }, () => [
        h(NButton, { size: 'tiny', onClick: () => openLog(row) }, () => '日志'),
        row.status === 'success' ? h(NButton, { size: 'tiny', type: 'warning', disabled: deployStore.isRunning, onClick: () => rollbackTo(row) }, () => '回滚') : null,
        h(NButton, { size: 'tiny', type: 'error', secondary: true, onClick: () => deleteHistoryRecord(row) }, () => '删除'),
      ])
    },
  },
]
</script>

<template>
  <div class="page-container history-view">
    <div class="page-header">
      <div class="history-heading">
        <h2>部署历史</h2>
      </div>
      <div class="filter-bar panel-card">
        <div class="filter-bar__actions">
          <n-button
            class="filter-reset"
            size="small"
            quaternary
            :disabled="!hasActiveFilters"
            @click="resetFilters"
          >
            清空筛选
          </n-button>
        </div>

        <div class="filter-grid">
          <div class="filter-item">
            <span class="filter-item__label">项目</span>
            <n-select
              v-model:value="projectFilter"
              class="filter-control"
              :options="[{ label: '全部项目', value: '' }, ...projectStore.projects.map(p => ({ label: p.name, value: p.id }))]"
              placeholder="选择项目"
              clearable
              @update:value="onFilterChange"
            />
          </div>

          <div class="filter-item">
            <span class="filter-item__label">服务器</span>
            <n-select
              v-model:value="serverFilter"
              class="filter-control"
              :options="[{ label: '全部服务器', value: '' }, ...serverStore.servers.map(s => ({ label: s.name, value: s.id }))]"
              placeholder="选择服务器"
              clearable
              @update:value="onFilterChange"
            />
          </div>

          <div class="filter-item">
            <span class="filter-item__label">状态</span>
            <n-select
              v-model:value="statusFilter"
              class="filter-control"
              :options="[
                { label: '全部状态', value: '' },
                { label: '成功', value: 'success' },
                { label: '失败', value: 'failed' },
                { label: '已取消', value: 'cancelled' },
                { label: '回滚', value: 'rollback' },
              ]"
              placeholder="选择状态"
              clearable
              @update:value="onFilterChange"
            />
          </div>

          <div class="filter-item filter-item--date">
            <span class="filter-item__label">日期范围</span>
            <n-date-picker
              v-model:value="dateRange"
              class="filter-control"
              type="daterange"
              placeholder="选择日期范围"
              clearable
              @update:value="onFilterChange"
            />
          </div>
        </div>
      </div>
    </div>

    <div class="history-table-shell panel-card">
      <div class="history-table-head">
        <span>发布记录</span>
        <span class="record-count">共 {{ deployStore.historyTotal }} 条记录</span>
      </div>
      <div class="table-wrapper">
      <n-data-table
        :columns="columns"
        :data="deployStore.history"
        :loading="deployStore.historyLoading"
        :scroll-x="1200"
        size="small"
        :row-key="(row: DeployRecord) => row.id"
      />
      </div>
    </div>

    <div class="table-footer">
      <n-pagination
        v-model:page="page"
        v-model:page-size="pageSize"
        :item-count="deployStore.historyTotal"
        :page-sizes="paginationConfig.pageSizes"
        show-size-picker
        @update:page="paginationConfig.onChange"
        @update:page-size="paginationConfig.onUpdatePageSize"
      />
    </div>

    <!-- 日志弹窗 -->
    <n-modal v-model:show="showLogModal" title="部署日志" preset="card" style="width: 700px">
      <div class="log-detail" v-if="selectedRecord">
        <div class="log-meta">
          <div class="meta-item">
            <span class="meta-label">项目:</span>
            <span>{{ selectedRecord.project_name }}</span>
          </div>
          <div class="meta-item">
            <span class="meta-label">服务器:</span>
            <span>{{ selectedRecord.server_name }}</span>
          </div>
          <div class="meta-item">
            <span class="meta-label">版本目录:</span>
            <code>{{ selectedRecord.release_dir_path }}</code>
          </div>
          <div class="meta-item" v-if="selectedRecord.exit_code !== null">
            <span class="meta-label">退出码:</span>
            <span>{{ selectedRecord.exit_code }}</span>
          </div>
          <div class="meta-item">
            <span class="meta-label">时间:</span>
            <span>{{ new Date(selectedRecord.created_at).toLocaleString() }}</span>
          </div>
          <div class="meta-item">
            <span class="meta-label">耗时:</span>
            <span>{{ (selectedRecord.duration_ms / 1000).toFixed(1) }}s</span>
          </div>
        </div>
        <div class="log-text">
          <pre>{{ selectedRecord.log }}</pre>
        </div>
        <div class="log-export-bar">
          <n-button size="tiny" @click="exportHistoryLog">📥 导出日志文件</n-button>
        </div>
      </div>
    </n-modal>
  </div>
</template>

<style scoped>
.history-view {
  overflow: hidden;
  gap: 18px;
}

.page-header {
  align-items: stretch;
  flex-direction: column;
  gap: 14px;
}

.history-heading {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  min-height: 42px;
}

.page-header .filter-bar {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 16px 18px 18px;
  position: relative;
  overflow: visible;
  background: var(--bg-panel);
  box-shadow: 0 1px 2px rgba(27, 34, 47, 0.035);
}

.page-header .filter-bar::before {
  content: '';
  position: absolute;
  inset: 0 auto 0 0;
  width: 3px;
  background: var(--accent-blue);
  pointer-events: none;
}

.filter-bar__actions {
  display: flex;
  justify-content: flex-start;
  gap: 12px;
  flex-wrap: wrap;
}

.filter-reset {
  align-self: flex-end;
}

.filter-grid {
  display: grid;
  grid-template-columns: repeat(12, minmax(0, 1fr));
  gap: 10px;
}

.filter-item {
  grid-column: span 2;
  min-width: 0;
}

.filter-item--date {
  grid-column: span 6;
}

.filter-item__label {
  display: block;
  margin-bottom: 6px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.06em;
  color: var(--text-muted);
}

.filter-control {
  width: 100%;
}

.filter-bar :deep(.n-base-selection) {
  min-height: 36px;
  border-radius: 6px;
  background: var(--bg-soft);
  transition: background var(--motion-fast), border-color var(--motion-fast), box-shadow var(--motion-fast), transform var(--motion-fast);
}

.filter-bar :deep(.n-base-selection:hover) {
  background: var(--bg-card-hover);
}

.filter-bar :deep(.n-base-selection.n-base-selection--active),
.filter-bar :deep(.n-base-selection:focus-within) {
  background: var(--bg-elevated);
  box-shadow: 0 0 0 3px rgba(var(--accent-blue-rgb), 0.12);
}

.filter-bar :deep(.n-base-selection__border),
.filter-bar :deep(.n-base-selection__state-border) {
  border-radius: 6px;
}

.filter-bar :deep(.n-base-selection-label) {
  font-size: 14px;
  color: var(--text-primary);
}

.filter-bar :deep(.n-base-selection-placeholder),
.filter-bar :deep(.n-base-selection-input__content) {
  color: var(--text-muted);
}

.filter-bar :deep(.n-base-selection__suffix) {
  color: var(--text-muted);
}

.filter-bar :deep(.n-button) {
  border-radius: 5px;
}

.history-table-shell {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 1px 2px rgba(27, 34, 47, 0.035);
}

.history-table-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-height: 54px;
  padding: 0 18px;
  border-bottom: 0;
  color: var(--text-primary);
  font-size: 13px;
  font-weight: 750;
}

.record-count {
  font: 600 11px/1.2 'JetBrains Mono', monospace;
  color: var(--text-secondary);
  padding: 5px 8px;
  border-radius: 4px;
  background: var(--bg-soft);
  border: 0;
}

.table-wrapper {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 4px 10px 10px;
}

.table-wrapper :deep(.n-data-table) {
  border-radius: 0;
  overflow: hidden;
}

.table-wrapper :deep(.n-data-table-th) {
  height: 40px;
  font-size: 11px;
  font-weight: 750;
  letter-spacing: 0.055em;
  text-transform: uppercase;
  color: var(--text-muted);
}

.table-wrapper :deep(.n-data-table-td) {
  height: 60px;
  transition: background-color 220ms cubic-bezier(0.22, 1, 0.36, 1);
}

.table-wrapper :deep(.n-data-table-th--fixed-right),
.table-wrapper :deep(.n-data-table-td--fixed-right) {
  background: var(--bg-elevated);
}

.table-wrapper :deep(.n-data-table-th--fixed-right) {
  z-index: 3;
}

.table-wrapper :deep(.n-data-table-td--fixed-right) {
  z-index: 2;
}

.table-wrapper :deep(.n-data-table-th--fixed-right::before),
.table-wrapper :deep(.n-data-table-td--fixed-right::before) {
  box-shadow: inset -12px 0 12px -12px rgba(15, 23, 42, 0.14);
}

.table-wrapper :deep(.n-data-table-tr:hover > .n-data-table-td--fixed-right),
.table-wrapper :deep(.n-data-table-td--fixed-right.n-data-table-td--hover) {
  background: var(--bg-card-hover);
}

.history-identity {
  display: flex;
  align-items: center;
  min-width: 0;
  gap: 8px;
}

.history-identity__marker {
  display: block;
  width: 7px;
  height: 7px;
  flex: 0 0 auto;
  border: 2px solid var(--accent-blue);
  border-radius: 50%;
}

.history-identity--server .history-identity__marker {
  border-color: var(--text-muted);
  border-radius: 2px;
}

.history-identity__value {
  overflow: hidden;
  color: var(--text-primary);
  font-weight: 650;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.history-release,
.history-duration,
.history-time {
  color: var(--text-secondary);
  font: 500 11px/1.4 'JetBrains Mono', monospace;
}

.history-release {
  display: block;
  overflow: hidden;
  color: var(--accent-blue);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.history-row-actions :deep(.n-button) {
  min-width: 38px;
  padding-inline: 8px;
  border-radius: 4px;
}

.table-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}

.table-footer :deep(.n-pagination) {
  margin-left: auto;
}

.log-detail {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.log-meta {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 16px;
  background: var(--bg-panel);
  border-radius: var(--radius-lg);
  font-size: 13px;
  border: 1px solid var(--border-color);
}

.meta-item {
  display: flex;
  gap: 12px;
}

.meta-label {
  color: var(--text-muted);
  min-width: 60px;
  font-weight: 500;
}

.meta-item code {
  background: var(--bg-code);
  padding: 1px 8px;
  border-radius: var(--radius-sm);
  color: var(--accent-blue);
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  font-size: 12px;
  border: 1px solid var(--border-subtle);
}

.log-text {
  background: var(--bg-code);
  border-radius: var(--radius-lg);
  padding: 16px;
  max-height: 350px;
  overflow-y: auto;
  border: 1px solid var(--border-color);
}

.log-text pre {
  margin: 0;
  font-size: 12px;
  color: var(--text-primary);
  white-space: pre-wrap;
  word-break: break-all;
  font-family: 'JetBrains Mono', 'Cascadia Code', 'Consolas', monospace;
  line-height: 1.6;
}

.log-export-bar {
  display: flex;
  justify-content: flex-end;
  margin-top: 12px;
}

@media (max-width: 1080px) {
  .filter-grid {
    grid-template-columns: repeat(6, minmax(0, 1fr));
  }

  .filter-item {
    grid-column: span 2;
  }

  .filter-item--date {
    grid-column: span 6;
  }
}

@media (max-width: 720px) {
  .filter-reset {
    align-self: flex-start;
  }

  .filter-grid {
    grid-template-columns: 1fr;
  }

  .filter-item,
  .filter-item--date {
    grid-column: auto;
  }

  .table-footer {
    justify-content: flex-end;
  }

  .table-footer .record-count {
    margin-right: auto;
  }
}
</style>
