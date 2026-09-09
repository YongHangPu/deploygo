import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { invokeWithLoading } from '../utils/invoke'
import type { Project } from '../types'

export const useProjectStore = defineStore('project', () => {
  const projects = ref<Project[]>([])
  const currentProjectId = ref<string | null>(null)
  const loading = ref(false)

  const currentProject = computed(() => {
    return projects.value.find(p => p.id === currentProjectId.value) || null
  })

  async function loadProjects() {
    projects.value = await invokeWithLoading(loading, 'get_projects')
  }

  async function saveProject(project: Project) {
    // 为新项目自动分配 sort_order。
    if (!project.sort_order && project.sort_order !== 0) {
      const maxOrder = projects.value.reduce((max, p) => Math.max(max, p.sort_order ?? 0), -1)
      project.sort_order = maxOrder + 1
    }
    await invoke('save_project', { project })
    await loadProjects()
  }

  async function deleteProject(id: string) {
    await invoke('delete_project', { projectId: id })
    if (currentProjectId.value === id) {
      currentProjectId.value = null
    }
    await loadProjects()
  }

  async function reorderProjects(ids: string[]) {
    // 更新 sort_order 并重新排序数组，使数据顺序保持同步。
    // ProjectSidebar 通过 filteredProjects 计算属性进行搜索筛选，因此需要这样处理。
    const idToOrder = new Map(ids.map((id, i) => [id, i]))
    for (const p of projects.value) {
      const idx = idToOrder.get(p.id)
      if (idx !== undefined) p.sort_order = idx
    }
    projects.value = [...projects.value].sort((a, b) => a.sort_order - b.sort_order)
    await invoke('reorder_projects', { ids })
  }

  function selectProject(id: string | null) {
    currentProjectId.value = id
  }

  return {
    projects,
    currentProjectId,
    currentProject,
    loading,
    loadProjects,
    saveProject,
    deleteProject,
    reorderProjects,
    selectProject,
  }
})
