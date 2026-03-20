import { ref, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { MemoryInfo } from '../types'

export function useMemory() {
  const memoryInfo = ref<MemoryInfo>({
    total: 0,
    used: 0,
    available: 0,
    free: 0,
    reclaimable: 0,
    usage: 0
  })
  const isLoading = ref(false)
  const isFreeing = ref(false)
  let unlisten: UnlistenFn | null = null

  const fetchMemoryInfo = async () => {
    isLoading.value = true
    try {
      const info = await invoke<MemoryInfo>('get_memory_info')
      memoryInfo.value = info
    } catch (error) {
      console.error('Failed to fetch memory info:', error)
    } finally {
      isLoading.value = false
    }
  }

  const freeMemory = async (): Promise<string> => {
    isFreeing.value = true
    try {
      const result = await invoke<string>('free_memory')
      await fetchMemoryInfo()
      return result
    } catch (error) {
      throw error
    } finally {
      isFreeing.value = false
    }
  }

  const formatSize = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  const startListening = async () => {
    if (unlisten) return
    unlisten = await listen<MemoryInfo>('memory-update', (event) => {
      memoryInfo.value = event.payload
    })
  }

  const stopListening = () => {
    if (unlisten) {
      unlisten()
      unlisten = null
    }
  }

  onUnmounted(() => {
    stopListening()
  })

  return {
    memoryInfo,
    isLoading,
    isFreeing,
    fetchMemoryInfo,
    freeMemory,
    formatSize,
    startListening,
    stopListening
  }
}
