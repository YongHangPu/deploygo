<script setup lang="ts">
import { computed, ref, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import {
  NModal,
  NForm,
  NFormItem,
  NInput,
  NInputNumber,
  NButton,
  NSelect,
  NTooltip,
  useDialog,
} from 'naive-ui'
import { open } from '@tauri-apps/plugin-dialog'
import { useSortable } from '../composables/useSortable'
import { useNotify } from '../composables/useNotify'
import { useProjectStore } from '../stores/projectStore'
import { useServerStore } from '../stores/serverStore'
import type { Project } from '../types'

const router = useRouter()
const route = useRoute()
const notify = useNotify()
const dialog = useDialog()
const projectStore = useProjectStore()
const serverStore = useServerStore()

const appVersion = __APP_VERSION__

const showAddModal = ref(false)
const editingProject = ref<Partial<Project>>({
  name: '',
  local_dist_path: '',
  server_id: '',
  keep_versions: 5,
  live_root_path: '',
  releases_root_path: '',
})
const isEditing = ref(false)
const projectSearch = ref('')
const projectListEl = ref<HTMLElement | null>(null)

const filteredProjects = computed(() => {
  const q = projectSearch.value.trim().toLowerCase()
  if (!q) return projectStore.projects
  return projectStore.projects.filter(p =>
    p.name.toLowerCase().includes(q) ||
    (serverStore.getServer(p.server_id)?.name || '').toLowerCase().includes(q)
  )
})

useSortable(projectListEl, {
  handle: '.project-item-drag-handle',
  items: filteredProjects,
  getId: (p) => p.id,
  onReorder: (ids) => projectStore.reorderProjects(ids),
})

onMounted(async () => {
  await projectStore.loadProjects()
  await serverStore.loadServers()
})

const derivedReleasesRoot = computed(() => deriveReleasesRoot(editingProject.value.live_root_path || ''))

  // 表单校验错误
const errors = ref<Record<string, string>>({})

const validateField = (field: string): boolean => {
  const val = editingProject.value
  switch (field) {
    case 'name':
      if (!val.name?.trim()) {
        errors.value['name'] = '项目名称不能为空'
        return false
      }
      break
    case 'local_dist_path':
      if (!val.local_dist_path?.trim()) {
        errors.value['local_dist_path'] = '本地路径不能为空'
        return false
      }
      break
    case 'server_id':
      if (!val.server_id) {
        errors.value['server_id'] = '请选择关联服务器'
        return false
      }
      break
    case 'live_root_path':
      if (!val.live_root_path?.trim()) {
        errors.value['live_root_path'] = '线上发布目录不能为空'
        return false
      }
      break
  }
  delete errors.value[field]
  return true
}

const navigateTo = (path: string) => {
  router.push(path)
}

const navigateToHome = () => {
  projectStore.selectProject(null)
  router.push('/')
}

const openAddModal = () => {
  isEditing.value = false
  errors.value = {}
  editingProject.value = {
    name: '',
    local_dist_path: '',
    server_id: serverStore.servers[0]?.id || '',
    keep_versions: 5,
    live_root_path: '',
    releases_root_path: '',
  }
  showAddModal.value = true
}

const openEditModal = (project: Project) => {
  isEditing.value = true
  errors.value = {}
  editingProject.value = { ...project }
  showAddModal.value = true
}

  /** 打开操作系统文件夹选择器，选择构建输出目录。 */
const pickDistDir = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择构建产物目录',
    })
    if (selected) {
      editingProject.value.local_dist_path = selected as string
      validateField('local_dist_path')
    }
  } catch (e: any) {
    notify.error(`选择目录失败: ${typeof e === 'string' ? e : (e?.message || '未知错误')}`)
  }
}

const saveProject = async () => {
  // 校验所有字段
  const validName = validateField('name')
  const validPath = validateField('local_dist_path')
  const validServer = validateField('server_id')
  const validLiveRoot = validateField('live_root_path')
  if (!validName || !validPath || !validServer || !validLiveRoot) {
    return
  }

  const name = editingProject.value.name!.trim()
  const path = editingProject.value.local_dist_path!.trim()
  const currentId = editingProject.value.id

  // 检查重复项
  const duplicateName = projectStore.projects.find(p => p.name === name && p.id !== currentId)
  if (duplicateName) {
    errors.value['name'] = '项目名称已存在'
    notify.error('项目名称已存在')
    return
  }
  const duplicatePath = projectStore.projects.find(p => p.local_dist_path === path && p.id !== currentId)
  if (duplicatePath) {
    errors.value['local_dist_path'] = '该路径已被其他项目使用'
    notify.error('该路径已被其他项目使用')
    return
  }

  const project: Project = {
    id: currentId || crypto.randomUUID(),
    name,
    local_dist_path: path,
    build_command: null,
    server_id: editingProject.value.server_id!,
    keep_versions: editingProject.value.keep_versions ?? 5,
    live_root_path: editingProject.value.live_root_path!.trim(),
    releases_root_path: deriveReleasesRoot(editingProject.value.live_root_path!),
    sort_order: editingProject.value.sort_order ?? 0,
    created_at: editingProject.value.created_at || new Date().toISOString(),
    updated_at: new Date().toISOString(),
  }
  await projectStore.saveProject(project)
  notify.success(isEditing.value ? '项目已更新' : '项目已创建')
  showAddModal.value = false
}

const confirmDeleteProject = (id: string, name: string) => {
  dialog.warning({
    title: '删除项目',
    content: `确定要删除项目「${name}」吗？此操作仅删除本地配置，不影响服务器上的文件。`,
    positiveText: '确定删除',
    negativeText: '取消',
    onPositiveClick: async () => {
      await projectStore.deleteProject(id)
      notify.success('项目已删除')
    },
  })
}

const selectProject = (id: string) => {
  projectStore.selectProject(id)
  navigateTo('/')
}

const deriveReleasesRoot = (liveRootPath: string) => {
  const normalized = liveRootPath.trim().replace(/\\/g, '/').replace(/\/+$/, '')
  if (!normalized) return ''

  const lastSlashIndex = normalized.lastIndexOf('/')
  if (lastSlashIndex <= 0) return '/releases'

  const parent = normalized.slice(0, lastSlashIndex)
  return `${parent}/releases`
}
</script>

<template>
  <div class="sidebar">
    <!-- 应用图标 -->
    <div class="sidebar-header" @click="navigateToHome">
      <div class="sidebar-logo">
        <img src="/favicon.svg" alt="" />
      </div>
      <div class="sidebar-title">
        <span class="sidebar-title-name">零停机部署</span>
        <span class="sidebar-title-ver">v{{ appVersion }}</span>
      </div>
    </div>

    <!-- 项目区域 -->
    <div class="sidebar-section">
      <div class="sidebar-section-title">
        <span>项目列表</span>
        <div class="section-title-actions">
          <span class="project-count">{{ filteredProjects.length }}</span>
          <n-tooltip trigger="hover">
            <template #trigger>
              <button class="icon-btn add-icon" @click="openAddModal">
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" /></svg>
              </button>
            </template>
            添加项目
          </n-tooltip>
        </div>
      </div>
      <div class="project-search" v-if="projectStore.projects.length > 0">
        <input
          v-model="projectSearch"
          class="search-input"
          type="text"
          placeholder="搜索项目..."
        />
      </div>
      <div ref="projectListEl" class="project-list">
        <div
          v-for="project in filteredProjects"
          :key="project.id"
          class="project-item"
          :class="{ active: projectStore.currentProjectId === project.id }"
          @click="selectProject(project.id)"
        >
          <div class="project-item-drag-handle">
            <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <line x1="8" y1="6" x2="16" y2="6" /><line x1="8" y1="12" x2="16" y2="12" /><line x1="8" y1="18" x2="16" y2="18" />
            </svg>
          </div>
          <div class="project-item-icon">
            <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z" />
            </svg>
          </div>
          <div class="project-item-content">
            <span class="project-name">{{ project.name }}</span>
            <span class="project-server">{{ serverStore.getServer(project.server_id)?.name || '未关联' }}</span>
          </div>
          <div class="project-item-actions">
            <button class="icon-btn" title="编辑" @click.stop="openEditModal(project)">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 4H4a2 2 0 00-2 2v14a2 2 0 002 2h14a2 2 0 002-2v-7" /><path d="M18.5 2.5a2.121 2.121 0 013 3L12 15l-4 1 1-4 9.5-9.5z" /></svg>
            </button>
            <button class="icon-btn danger" title="删除" @click.stop="confirmDeleteProject(project.id, project.name)">
              <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6" /><path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2" /></svg>
            </button>
          </div>
        </div>

      </div>
    </div>

    <!-- 导航 -->
    <div class="sidebar-nav">
      <div
        class="nav-item"
        :class="{ active: route.path === '/servers' }"
        @click="navigateTo('/servers')"
      >
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="2" width="20" height="8" rx="2" ry="2" /><rect x="2" y="14" width="20" height="8" rx="2" ry="2" /><line x1="6" y1="6" x2="6.01" y2="6" /><line x1="6" y1="18" x2="6.01" y2="18" /></svg>
        服务器管理
      </div>
      <div
        class="nav-item"
        :class="{ active: route.path === '/history' }"
        @click="navigateTo('/history')"
      >
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10" /><polyline points="12 6 12 12 16 14" /></svg>
        部署历史
      </div>
      <div
        class="nav-item"
        :class="{ active: route.path === '/guide' }"
        @click="navigateTo('/guide')"
      >
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10" /><polyline points="12 16 12 12 12 8" /><line x1="12" y1="3" x2="12" y2="4" /><line x1="12" y1="20" x2="12" y2="21" /></svg>
        使用指南
      </div>
      <div
        class="nav-item"
        :class="{ active: route.path === '/activate' }"
        @click="navigateTo('/activate')"
      >
        <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="11" width="18" height="11" rx="2" ry="2" /><path d="M7 11V7a5 5 0 0110 0v4" /></svg>
        授权管理
      </div>
    </div>

    <!-- 添加/编辑项目弹窗 -->
    <n-modal v-model:show="showAddModal" :title="isEditing ? '编辑项目' : '添加项目'" preset="card" style="width: 550px">
      <n-form label-placement="top" label-width="100px" :show-feedback="true">
        <n-form-item
          label="项目名称"
          required
          :feedback="errors.name"
          :validation-status="errors.name ? 'error' : undefined"
        >
          <n-input v-model:value="editingProject.name" placeholder="如: 官网前端" @blur="validateField('name')" />
        </n-form-item>
        <n-form-item
          label="本地构建产物目录"
          required
          :feedback="errors.local_dist_path"
          :validation-status="errors.local_dist_path ? 'error' : undefined"
        >
          <div class="path-picker-row">
            <n-input
              v-model:value="editingProject.local_dist_path"
              placeholder="如: D:/projects/website/dist"
              @blur="validateField('local_dist_path')"
              class="path-picker-input"
            />
            <n-button secondary @click="pickDistDir">选择目录</n-button>
          </div>
        </n-form-item>
        <n-form-item
          label="关联服务器"
          required
          :feedback="errors.server_id"
          :validation-status="errors.server_id ? 'error' : undefined"
        >
          <n-select
            v-model:value="editingProject.server_id"
            :options="serverStore.servers.map(s => ({ label: s.name, value: s.id }))"
            placeholder="选择服务器"
            @blur="validateField('server_id')"
          />
          <div v-if="serverStore.servers.length === 0" class="no-server-hint">
            暂无服务器，请先到
            <a class="hint-link" @click="navigateTo('/servers'); showAddModal = false">服务器管理</a>
            添加服务器
          </div>
        </n-form-item>
        <n-form-item
          label="线上站点目录"
          required
          :feedback="errors.live_root_path"
          :validation-status="errors.live_root_path ? 'error' : undefined"
        >
          <template #label>
            <div style="display: flex; align-items: center; gap: 4px;">
              <span>线上站点目录</span>
              <n-tooltip trigger="hover" placement="right">
                <template #trigger>
                  <span style="cursor: help; color: var(--text-muted); font-size: 14px;">ⓘ</span>
                </template>
                <span style="font-size: 12px;">Nginx 当前实际对外服务的目录。<br/>部署时会把新版本内容零停机同步到此目录。<br/>例: /mnt/data/app/dist</span>
              </n-tooltip>
            </div>
          </template>
          <n-input v-model:value="editingProject.live_root_path" placeholder="如: /mnt/data/app/dist" @blur="validateField('live_root_path')" />
        </n-form-item>
        <n-form-item label="自动推导的版本归档目录">
          <n-input :value="derivedReleasesRoot || '请先填写线上发布目录'" readonly />
        </n-form-item>
        <n-form-item label="保留版本数">
          <n-input-number v-model:value="editingProject.keep_versions" :min="0" :max="20" />
        </n-form-item>
      </n-form>
      <template #footer>
        <div style="display: flex; gap: 8px; justify-content: flex-end">
          <n-button @click="showAddModal = false">取消</n-button>
          <n-button type="primary" @click="saveProject">{{ isEditing ? '更新' : '创建' }}</n-button>
        </div>
      </template>
    </n-modal>
  </div>
</template>

<style scoped>
.sidebar {
  width: 240px;
  min-width: 240px;
  background: var(--sidebar-bg);
  color: var(--text-primary);
  display: flex;
  flex-direction: column;
  height: 100%;
  border-right: 1px solid var(--border-subtle);
  box-shadow: 12px 0 28px rgba(15, 23, 42, 0.04);
  backdrop-filter: blur(18px);
}

/* 头部 */
.sidebar-header {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 16px 16px 12px;
  cursor: pointer;
  transition: opacity var(--motion-fast), transform var(--motion-fast);
}
.sidebar-header:hover {
  opacity: 0.92;
  transform: translateY(-1px);
}

.sidebar-logo {
  width: 32px;
  height: 32px;
  flex: 0 0 auto;
  overflow: hidden;
  border-radius: 8px;
  box-shadow: 0 10px 24px rgba(var(--accent-blue-rgb), 0.22);
}

.sidebar-logo img {
  display: block;
  width: 100%;
  height: 100%;
}

.sidebar-title {
  display: flex;
  flex-direction: column;
  gap: 1px;
}
.sidebar-title-name { font-size: 14px; font-weight: 600; }
.sidebar-title-ver { font-size: 11px; color: var(--text-muted); }

/* 区域 */
.sidebar-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  min-height: 0;
  padding: 4px 0;
}

.sidebar-section-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px 6px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  color: var(--text-muted);
  letter-spacing: 0.5px;
}

.project-count {
  font-size: 11px;
  background: var(--bg-soft);
  padding: 2px 8px;
  border-radius: 999px;
  font-weight: 500;
  border: 1px solid var(--border-subtle);
}

.section-title-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.add-icon {
  padding: 4px;
  border-radius: 6px;
}
.add-icon:hover {
  color: var(--accent-blue);
  background: rgba(var(--accent-blue-rgb), 0.1);
}

.project-search {
  padding: 4px 12px 6px;
}

.search-input {
  width: 100%;
  padding: 6px 10px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-card);
  color: var(--text-primary);
  font-size: 12px;
  outline: none;
  transition: border-color var(--motion-fast);
  font-family: inherit;
}
.search-input::placeholder {
  color: var(--text-muted);
}
.search-input:focus {
  border-color: var(--accent-blue);
}

/* 项目列表 */
.project-list {
  flex: 1;
  overflow-y: auto;
  padding: 2px 8px;
}

.project-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px;
  border-radius: var(--radius-md);
  cursor: pointer;
  margin-bottom: 2px;
  transition: background var(--motion-fast), box-shadow var(--motion-fast), transform var(--motion-fast), border-color var(--motion-fast);
  position: relative;
  border: 1px solid transparent;
}

/* SortableJS ghost：拖拽项目即将放置的位置占位项 */
.project-item.sortable-ghost {
  opacity: 0.35;
  background: rgba(var(--accent-blue-rgb), 0.08);
  border: 1px dashed var(--accent-blue);
}

/* SortableJS chosen：拖拽过程中的原始项目 */
.project-item.sortable-chosen {
  opacity: 0.5;
}

/* SortableJS drag：跟随光标移动的浮动副本 */
.sortable-drag {
  opacity: 0.9 !important;
  box-shadow: var(--shadow-lg);
  border-color: rgba(var(--accent-blue-rgb), 0.3);
}

.project-item-drag-handle {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  color: var(--text-muted);
  opacity: 0;
  transition: opacity var(--motion-fast);
  cursor: grab;
}

.project-item-drag-handle:active {
  cursor: grabbing;
}

.project-item:hover .project-item-drag-handle {
  opacity: 0.6;
}

.project-item:not(:last-child)::after {
  content: '';
  position: absolute;
  bottom: -2px;
  left: 46px;
  right: 12px;
  height: 1px;
  background: var(--border-subtle);
  opacity: 0.4;
}

.project-item:hover {
  background: var(--sidebar-hover);
  border-color: rgba(var(--accent-blue-rgb), 0.1);
  transform: translateX(1px);
}

.project-item.active {
  background: var(--sidebar-active);
  box-shadow: inset 2px 0 0 var(--sidebar-active-border);
  border-color: rgba(var(--accent-blue-rgb), 0.16);
}

.project-item-icon {
  flex-shrink: 0;
  width: 28px;
  height: 28px;
  border-radius: 6px;
  background: var(--bg-soft);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--accent-blue);
  opacity: 0.82;
  transition:
    opacity var(--motion-fast),
    background-color var(--motion-fast),
    color var(--motion-fast),
    transform var(--motion-fast);
}

.project-item.active .project-item-icon {
  opacity: 1;
  background: rgba(var(--accent-blue-rgb), 0.12);
}

.project-item-content {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.project-name {
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.3;
}

.project-server {
  font-size: 10px;
  color: var(--text-muted);
  display: inline-flex;
  align-items: center;
  gap: 4px;
  line-height: 1.2;
}

.project-server::before {
  content: '';
  display: inline-block;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--accent-green);
  opacity: 0.6;
}

.project-item-actions {
  display: flex;
  gap: 2px;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.15s;
}

.project-item:hover .project-item-actions { opacity: 1; }

.icon-btn {
  background: none;
  border: none;
  color: var(--text-muted);
  cursor: pointer;
  padding: 3px;
  border-radius: var(--radius-xs);
  transition:
    color var(--motion-fast),
    background-color var(--motion-fast),
    transform var(--motion-fast);
  display: flex;
  align-items: center;
  justify-content: center;
}

.icon-btn:hover {
  color: var(--text-primary);
  background: var(--bg-soft);
}

.icon-btn.danger:hover {
  color: var(--accent-red);
  background: rgba(var(--accent-red-rgb), 0.1);
}

/* 导航 */
.sidebar-nav {
  border-top: 1px solid var(--border-subtle);
  padding: 8px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border-radius: var(--radius-md);
  cursor: pointer;
  font-size: 13px;
  transition:
    color var(--motion-fast),
    background-color var(--motion-fast),
    border-color var(--motion-fast),
    transform var(--motion-fast);
  margin-bottom: 2px;
  color: var(--text-secondary);
  border: 1px solid transparent;
}

.nav-item:hover {
  background: var(--sidebar-hover);
  color: var(--text-primary);
  border-color: rgba(var(--accent-blue-rgb), 0.1);
}

.nav-item.active {
  background: var(--sidebar-active);
  color: var(--accent-blue);
  font-weight: 500;
  border-color: rgba(var(--accent-blue-rgb), 0.16);
}

/* 路径选择行（与服务器表单中的 key-path-row 使用相同样式） */
.path-picker-row {
  display: flex;
  gap: 8px;
  width: 100%;
}
.path-picker-input {
  flex: 1;
}

.no-server-hint {
  margin-top: 6px;
  font-size: 12px;
  color: var(--text-muted);
}
.hint-link {
  color: var(--accent-blue);
  cursor: pointer;
  text-decoration: underline;
}
.hint-link:hover {
  color: var(--accent-blue);
  opacity: 0.8;
}
</style>

<style scoped>
.sidebar {
  background: var(--sidebar-bg);
  border-right-color: var(--border-color);
  box-shadow: none;
  backdrop-filter: none;
}

.sidebar-header {
  gap: 9px;
  min-height: 64px;
  padding: 15px 16px;
  border-bottom: 1px solid var(--border-subtle);
}

.sidebar-header:hover {
  opacity: 1;
  transform: none;
  background: var(--sidebar-hover);
}

.sidebar-logo {
  width: 28px;
  height: 28px;
  border-radius: 5px;
  box-shadow: none;
}

.sidebar-title-name {
  font-size: 13px;
  font-weight: 750;
  letter-spacing: 0;
}

.sidebar-title-ver {
  font: 600 10px/1.2 'JetBrains Mono', monospace;
}

.sidebar-section {
  padding-top: 8px;
}

.sidebar-section-title {
  padding: 7px 14px 8px;
  font-size: 10px;
  letter-spacing: 0.08em;
}

.project-count {
  min-width: 22px;
  padding: 3px 6px;
  border-radius: 4px;
  background: var(--bg-chip);
  border-color: rgba(var(--accent-blue-rgb), 0.2);
  color: var(--accent-blue);
  font: 650 10px/1 'JetBrains Mono', monospace;
  text-align: center;
}

.add-icon {
  width: 26px;
  height: 26px;
  padding: 0;
  border: 1px solid var(--border-subtle);
  border-radius: 4px;
}

.project-search {
  padding: 0 10px 8px;
}

.search-input {
  min-height: 30px;
  padding: 5px 9px;
  border-radius: 5px;
  background: var(--bg-primary);
  font-size: 11px;
}

.search-input:focus {
  border-color: var(--accent-blue);
  box-shadow: 0 0 0 2px rgba(var(--accent-blue-rgb), 0.11);
}

.project-list {
  padding: 2px 8px;
}

.project-item {
  gap: 7px;
  min-height: 46px;
  padding: 7px 8px;
  margin-bottom: 3px;
  border-radius: 6px;
}

.project-item:not(:last-child)::after {
  display: none;
}

.project-item:hover {
  transform: none;
  background: var(--sidebar-hover);
  border-color: transparent;
}

.project-item.active {
  background: var(--sidebar-active);
  border-color: rgba(var(--accent-blue-rgb), 0.28);
  box-shadow: inset 3px 0 0 var(--sidebar-active-border);
}

.project-item-icon {
  width: 26px;
  height: 26px;
  border-radius: 4px;
  border: 1px solid var(--border-subtle);
  background: var(--bg-soft);
}

.project-name {
  font-size: 12px;
  font-weight: 700;
}

.project-server {
  font: 500 10px/1.2 'JetBrains Mono', monospace;
}

.project-server::before {
  width: 5px;
  height: 5px;
  background: var(--accent-blue);
}

.icon-btn {
  border-radius: 4px;
}

.sidebar-nav {
  padding: 10px 8px 12px;
  border-top-color: var(--border-color);
}

.nav-item {
  min-height: 34px;
  padding: 7px 9px;
  margin-bottom: 3px;
  border-radius: 5px;
  font-size: 12px;
}

.nav-item:hover {
  border-color: transparent;
}

.nav-item.active {
  border-color: rgba(var(--accent-blue-rgb), 0.24);
  color: var(--accent-blue);
  font-weight: 700;
}
</style>
