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

export interface JunkScanRequest {
  target_ids: JunkScanTargetId[]
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
}

export interface JunkScanTargetOption {
  id: JunkScanTargetId
  label: string
  description: string
}
