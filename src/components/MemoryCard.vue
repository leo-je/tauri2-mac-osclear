<template>
  <section class="card memory-card">
    <div class="card-header">
      <h2>内存状态</h2>
      <button @click="refresh" class="icon-btn" title="刷新" :disabled="isLoading">
        <RefreshIcon />
      </button>
    </div>

    <div class="memory-stats">
      <div class="stat-item">
        <span class="stat-label">总内存</span>
        <span class="stat-value">{{ formatSize(memoryInfo.total) }}</span>
      </div>
      <div class="stat-item">
        <span class="stat-label">已用</span>
        <span class="stat-value used">{{ formatSize(memoryInfo.used) }}</span>
      </div>
      <div class="stat-item">
        <span class="stat-label">可用</span>
        <span class="stat-value free">{{ formatSize(memoryInfo.free) }}</span>
      </div>
      <div class="stat-item">
        <span class="stat-label">缓存</span>
        <span class="stat-value cached">{{ formatSize(memoryInfo.cached) }}</span>
      </div>
    </div>

    <div class="memory-bar">
      <div
        class="memory-bar-fill"
        :style="{ width: memoryInfo.pressure + '%' }"
      ></div>
    </div>

    <div class="memory-pressure">
      <span>内存使用率: {{ memoryInfo.pressure.toFixed(1) }}%</span>
      <button
        @click="handleFree"
        :disabled="isFreeing"
        class="btn btn-primary"
      >
        {{ isFreeing ? '清理中...' : '清理内存' }}
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useMemory } from '../composables/useMemory'
import RefreshIcon from './icons/RefreshIcon.vue'

const emit = defineEmits<{
  (e: 'success', message: string): void
  (e: 'error', message: string): void
}>()

const {
  memoryInfo,
  isLoading,
  isFreeing,
  fetchMemoryInfo,
  freeMemory,
  formatSize
} = useMemory()

const refresh = async () => {
  await fetchMemoryInfo()
}

const handleFree = async () => {
  try {
    const result = await freeMemory()
    emit('success', result)
  } catch (error) {
    emit('error', '内存清理失败: ' + String(error))
  }
}

onMounted(() => {
  fetchMemoryInfo()
})
</script>

<style scoped>
.memory-card {
  background: rgba(255, 255, 255, 0.95);
}

.memory-stats {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 20px;
}

.stat-item {
  text-align: center;
  padding: 16px;
  background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
  border-radius: 12px;
}

.stat-label {
  display: block;
  font-size: 0.85rem;
  color: #666;
  margin-bottom: 4px;
}

.stat-value {
  display: block;
  font-size: 1.4rem;
  font-weight: 700;
  color: #1a1a2e;
}

.stat-value.used { color: #e74c3c; }
.stat-value.free { color: #27ae60; }
.stat-value.cached { color: #3498db; }

.memory-bar {
  height: 8px;
  background: #e0e0e0;
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 12px;
}

.memory-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, #27ae60, #f39c12, #e74c3c);
  border-radius: 4px;
  transition: width 0.5s ease;
}

.memory-pressure {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

@media (max-width: 768px) {
  .memory-stats {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
