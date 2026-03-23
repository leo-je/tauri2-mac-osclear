import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { JunkScanResult, CleanResult, CleanRequest, JunkScanRequest } from '../types'
import { useSettings } from './useSettings'

export function useJunkCleaner() {
  const { settings } = useSettings()
  const scanResult = ref<JunkScanResult>({
    items: [],
    total_size: 0,
    categories: []
  })
  const selectedItems = ref<string[]>([])
  const selectedCategories = ref<string[]>([])
  const isScanning = ref(false)
  const isCleaning = ref(false)
  const hasScanned = ref(false)
  const lastCleanResult = ref<CleanResult | null>(null)

  const selectedSize = computed(() => {
    return scanResult.value.items
      .filter(item => selectedItems.value.includes(item.path))
      .reduce((sum, item) => sum + item.size, 0)
  })

  const scanJunkFiles = async (options: { preserveLastCleanResult?: boolean } = {}) => {
    isScanning.value = true
    scanResult.value = { items: [], total_size: 0, categories: [] }
    selectedItems.value = []
    selectedCategories.value = []

    if (!options.preserveLastCleanResult) {
      lastCleanResult.value = null
    }

    try {
      const request: JunkScanRequest = {
        target_ids: settings.enabledJunkTargets,
        downloads_min_age_days: settings.downloadsMinAgeDays,
        downloads_min_size_mb: settings.downloadsMinSizeMB
      }
      const result = await invoke<JunkScanResult>('scan_junk_files', { request })
      scanResult.value = result
      if (settings.autoSelectScanResults) {
        selectedItems.value = result.items.map(item => item.path)
        selectedCategories.value = result.categories.map(([cat]) => cat)
      }
      hasScanned.value = true
    } catch (error) {
      console.error('Failed to scan junk files:', error)
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
          await scanJunkFiles({ preserveLastCleanResult: true })
        } catch (rescanError) {
          console.error('Failed to rescan junk files after cleaning:', rescanError)
        }
      }

      return result
    } catch (error) {
      throw error
    } finally {
      isCleaning.value = false
    }
  }

  const toggleItem = (path: string) => {
    const index = selectedItems.value.indexOf(path)
    if (index > -1) {
      selectedItems.value.splice(index, 1)
    } else {
      selectedItems.value.push(path)
    }
  }

  const toggleCategory = (category: string) => {
    const catIndex = selectedCategories.value.indexOf(category)
    if (catIndex > -1) {
      selectedCategories.value.splice(catIndex, 1)
    } else {
      selectedCategories.value.push(category)
    }

    const categoryItems = scanResult.value.items
      .filter(item => item.category === category)
      .map(item => item.path)

    const allSelected = categoryItems.every(path => selectedItems.value.includes(path))

    if (allSelected) {
      selectedItems.value = selectedItems.value.filter(path => !categoryItems.includes(path))
    } else {
      const newItems = categoryItems.filter(path => !selectedItems.value.includes(path))
      selectedItems.value.push(...newItems)
    }
  }

  const selectAll = () => {
    selectedItems.value = scanResult.value.items.map(item => item.path)
    selectedCategories.value = scanResult.value.categories.map(([cat]) => cat)
  }

  const deselectAll = () => {
    selectedItems.value = []
    selectedCategories.value = []
  }

  const formatSize = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  return {
    scanResult,
    selectedItems,
    selectedCategories,
    isScanning,
    isCleaning,
    hasScanned,
    lastCleanResult,
    selectedSize,
    scanJunkFiles,
    cleanSelected,
    toggleItem,
    toggleCategory,
    selectAll,
    deselectAll,
    formatSize
  }
}
