import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type {
  CleanRequest,
  CleanResult,
  UninstalledAppJunkApp,
  UninstalledAppJunkScanResult
} from '../types'
import { useSettings } from './useSettings'

function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

export function useUninstalledAppsCleaner() {
  const { settings } = useSettings()
  const scanResult = ref<UninstalledAppJunkScanResult>({
    apps: [],
    total_size: 0,
    total_items: 0
  })
  const selectedItems = ref<string[]>([])
  const isScanning = ref(false)
  const isCleaning = ref(false)
  const hasScanned = ref(false)
  const lastCleanResult = ref<CleanResult | null>(null)

  const flattenedItems = computed(() => scanResult.value.apps.flatMap(app => app.items))

  const selectedSize = computed(() => {
    return flattenedItems.value
      .filter(item => selectedItems.value.includes(item.path))
      .reduce((sum, item) => sum + item.size, 0)
  })

  const selectedAppCount = computed(() => {
    return scanResult.value.apps.filter(app =>
      app.items.some(item => selectedItems.value.includes(item.path))
    ).length
  })

  const isAppFullySelected = (app: UninstalledAppJunkApp) => {
    return app.items.length > 0 && app.items.every(item => selectedItems.value.includes(item.path))
  }

  const toggleItem = (path: string) => {
    const index = selectedItems.value.indexOf(path)
    if (index > -1) {
      selectedItems.value.splice(index, 1)
    } else {
      selectedItems.value.push(path)
    }
  }

  const toggleApp = (appId: string) => {
    const app = scanResult.value.apps.find(item => item.app_id === appId)
    if (!app) return

    const appPaths = app.items.map(item => item.path)
    const allSelected = appPaths.every(path => selectedItems.value.includes(path))

    if (allSelected) {
      selectedItems.value = selectedItems.value.filter(path => !appPaths.includes(path))
    } else {
      const nextPaths = appPaths.filter(path => !selectedItems.value.includes(path))
      selectedItems.value.push(...nextPaths)
    }
  }

  const scanUninstalledAppsJunk = async (options: { preserveLastCleanResult?: boolean } = {}) => {
    isScanning.value = true
    scanResult.value = { apps: [], total_size: 0, total_items: 0 }
    selectedItems.value = []

    if (!options.preserveLastCleanResult) {
      lastCleanResult.value = null
    }

    try {
      const result = await invoke<UninstalledAppJunkScanResult>('scan_uninstalled_app_junk')
      scanResult.value = result
      if (settings.autoSelectScanResults) {
        selectedItems.value = result.apps.flatMap(app => app.items.map(item => item.path))
      }
      hasScanned.value = true
    } catch (error) {
      console.error('Failed to scan uninstalled app junk:', error)
      throw error
    } finally {
      isScanning.value = false
    }
  }

  const cleanSelected = async (): Promise<CleanResult> => {
    if (selectedItems.value.length === 0) {
      throw new Error('No items selected')
    }

    isCleaning.value = true
    try {
      const request: CleanRequest = { paths: selectedItems.value }
      const result = await invoke<CleanResult>('clean_junk_files', { request })
      lastCleanResult.value = result

      if (settings.rescanAfterCleaning) {
        try {
          await scanUninstalledAppsJunk({ preserveLastCleanResult: true })
        } catch (rescanError) {
          console.error('Failed to rescan uninstalled app junk:', rescanError)
        }
      }

      return result
    } finally {
      isCleaning.value = false
    }
  }

  const selectAll = () => {
    selectedItems.value = flattenedItems.value.map(item => item.path)
  }

  const deselectAll = () => {
    selectedItems.value = []
  }

  return {
    scanResult,
    selectedItems,
    selectedSize,
    selectedAppCount,
    isScanning,
    isCleaning,
    hasScanned,
    lastCleanResult,
    isAppFullySelected,
    toggleItem,
    toggleApp,
    scanUninstalledAppsJunk,
    cleanSelected,
    selectAll,
    deselectAll,
    formatSize
  }
}
