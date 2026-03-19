export interface MemoryInfo {
  total: number
  used: number
  free: number
  cached: number
  pressure: number
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
