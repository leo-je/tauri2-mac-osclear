export type AppView = 'dashboard' | 'memory' | 'junk' | 'settings'
export type JunkScanTargetId =
  | 'user_cache'
  | 'logs'
  | 'tmp'
  | 'system_cache'
  | 'xcode_derived_data'
  | 'application_support'
  | 'downloads'
  | 'trash'

export interface MemoryInfo {
  total: number
  used: number
  available: number
  free: number
  reclaimable: number
  usage: number
}

export interface FreeMemoryResult {
  freed_bytes: number
  before_usage: number
  after_usage: number
}

export interface ProcessMemoryInfo {
  pid: number
  name: string
  memory_bytes: number
  cpu_usage: number
}

export interface JunkItem {
  path: string
  size: number
  category: string
}

export interface JunkScanResult {
  items: JunkItem[]
  total_size: number
  categories: [string, number][]
}

export interface UninstalledAppJunkItem {
  path: string
  size: number
  category: string
}

export interface UninstalledAppJunkApp {
  app_id: string
  app_name: string
  identifier: string | null
  total_size: number
  items: UninstalledAppJunkItem[]
}

export interface UninstalledAppJunkScanResult {
  apps: UninstalledAppJunkApp[]
  total_size: number
  total_items: number
}

export interface JunkScanRequest {
  target_ids: JunkScanTargetId[]
  downloads_min_age_days: number
  downloads_min_size_mb: number
}

export interface CleanRequest {
  paths: string[]
}

export interface CleanResult {
  cleaned_count: number
  cleaned_size: number
  errors: string[]
}

export interface SystemInfo {
  os_name: string
  os_version: string
  hostname: string
  cpu_count: number
  total_memory: number
}

export interface AppSettings {
  startupView: AppView
  showDashboardQuickActions: boolean
  reduceMotion: boolean
  autoSelectScanResults: boolean
  confirmBeforeCleaning: boolean
  enabledJunkTargets: JunkScanTargetId[]
  rescanAfterCleaning: boolean
  warningUsageThreshold: number
  criticalUsageThreshold: number
  downloadsMinAgeDays: number
  downloadsMinSizeMB: number
}

export interface JunkScanTargetOption {
  id: JunkScanTargetId
  label: string
  description: string
}
