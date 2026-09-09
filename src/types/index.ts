export interface Project {
  id: string
  name: string
  local_dist_path: string
  build_command: string | null
  server_id: string
  keep_versions: number
  live_root_path: string
  releases_root_path: string
  sort_order: number
  created_at: string
  updated_at: string
}

export interface ServerConfig {
  id: string
  name: string
  host: string
  port: number
  username: string
  auth_method: string
  key_path: string | null
  // 跳板机
  jump_host: string | null
  jump_port: number | null
  jump_username: string | null
  jump_auth_method: string | null
  jump_key_path: string | null
  sort_order: number
  created_at: string
  updated_at: string
}

export interface DeployRecord {
  id: number
  project_id: string
  project_name: string
  server_id: string
  server_name: string
  release_dir_path: string
  live_root_path: string
  version: string | null
  artifact_fingerprint?: string | null
  status: 'success' | 'failed' | 'cancelled' | 'rollback'
  file_count: number
  total_size: number
  exit_code: number | null
  duration_ms: number
  log: string
  created_at: string
}

export interface UploadProgress {
  current: number
  total: number
  filename: string
  percent: number
  release_dir_path?: string
}

export interface UploadComplete {
  file_count: number
  total_size: number
  release_dir_path: string
}

export interface DeployStepEvent {
  step: number
  status: 'running' | 'cancelling' | 'committing' | 'done' | 'error' | 'cancelled'
  message?: string
}

export interface LicenseInfo {
  license_id: string
  tier: string
  expires_at: string
  is_valid: boolean
  reason: string | null
  trial_remaining_days: number
  can_deploy: boolean
  /** 构建类型：'full'（官方完整版）或 'shell'（开源纯壳，不含部署引擎） */
  engine: 'full' | 'shell' | string
}
