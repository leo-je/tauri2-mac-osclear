<template>
  <n-card title="💾 内存状态" size="large">
    <template #header-extra>
      <n-button quaternary circle @click="refresh" :loading="isLoading">
        <template #icon>
          <n-icon><RefreshIcon /></n-icon>
        </template>
      </n-button>
    </template>

    <n-grid :cols="4" :x-gap="16" :y-gap="16" responsive="screen">
      <n-gi>
        <div class="stat-card total">
          <div class="stat-label">总内存</div>
          <div class="stat-value">{{ formatSize(memoryInfo.total) }}</div>
        </div>
      </n-gi>
      <n-gi>
        <div class="stat-card used">
          <div class="stat-label">已用</div>
          <div class="stat-value">{{ formatSize(memoryInfo.used) }}</div>
        </div>
      </n-gi>
      <n-gi>
        <div class="stat-card free">
          <div class="stat-label">可用</div>
          <div class="stat-value">{{ formatSize(memoryInfo.free) }}</div>
        </div>
      </n-gi>
      <n-gi>
        <div class="stat-card cached">
          <div class="stat-label">缓存</div>
          <div class="stat-value">{{ formatSize(memoryInfo.cached) }}</div>
        </div>
      </n-gi>
    </n-grid>

    <div class="memory-visual mt-6">
      <div class="visual-header">
        <span>内存使用率</span>
        <span class="pressure-value">{{ memoryInfo.pressure.toFixed(1) }}%</span>
      </div>
      <n-progress
        type="line"
        :percentage="memoryInfo.pressure"
        :show-indicator="false"
        :height="12"
        :border-radius="6"
        :fill-border-radius="6"
        :color="getPressureColor()"
        rail-color="rgba(255,255,255,0.1)"
      />
    </div>

    <div class="action-bar mt-6">
      <n-space align="center">
        <n-tag :type="getPressureTagType()" size="large">
          {{ getPressureStatus() }}
        </n-tag>
        <n-button
          type="primary"
          size="large"
          :loading="isFreeing"
          @click="handleFree"
        >
          <template #icon>
            <n-icon><FlashIcon /></n-icon>
          </template>
          {{ isFreeing ? '清理中...' : '一键清理内存' }}
        </n-button>
      </n-space>
    </div>
  </n-card>
</template>

<script setup lang="ts">
import { onMounted, h } from 'vue'
import {
  NCard, NGrid, NGi, NProgress, NButton, NSpace, NTag, NIcon, useMessage
} from 'naive-ui'
import { useMemory } from '../composables/useMemory'
import RefreshIcon from './icons/RefreshIcon.vue'

const message = useMessage()

const FlashIcon = () => h('svg', {
  xmlns: 'http://www.w3.org/2000/svg',
  viewBox: '0 0 24 24',
  fill: 'currentColor',
  width: '20',
  height: '20'
}, [
  h('path', {
    d: 'M13 2L3 14h9l-1 8 10-12h-9l1-8z'
  })
])

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
    message.success(result)
  } catch (error) {
    message.error('内存清理失败: ' + String(error))
  }
}

const getPressureColor = () => {
  if (memoryInfo.value.pressure < 50) return '#00ff88'
  if (memoryInfo.value.pressure < 75) return '#f0ad4e'
  return '#ff4757'
}

const getPressureTagType = (): 'success' | 'warning' | 'error' => {
  if (memoryInfo.value.pressure < 50) return 'success'
  if (memoryInfo.value.pressure < 75) return 'warning'
  return 'error'
}

const getPressureStatus = () => {
  if (memoryInfo.value.pressure < 50) return '内存充裕'
  if (memoryInfo.value.pressure < 75) return '内存中等'
  return '内存紧张'
}

onMounted(() => {
  fetchMemoryInfo()
})
</script>

<style scoped>
.stat-card {
  padding: 20px;
  border-radius: 12px;
  text-align: center;
  transition: transform 0.2s;
}

.stat-card:hover {
  transform: translateY(-2px);
}

.stat-card.total {
  background: linear-gradient(135deg, rgba(0, 217, 255, 0.2), rgba(0, 136, 255, 0.2));
  border: 1px solid rgba(0, 217, 255, 0.3);
}

.stat-card.used {
  background: linear-gradient(135deg, rgba(255, 71, 87, 0.2), rgba(255, 107, 107, 0.2));
  border: 1px solid rgba(255, 71, 87, 0.3);
}

.stat-card.free {
  background: linear-gradient(135deg, rgba(0, 255, 136, 0.2), rgba(0, 200, 100, 0.2));
  border: 1px solid rgba(0, 255, 136, 0.3);
}

.stat-card.cached {
  background: linear-gradient(135deg, rgba(156, 89, 255, 0.2), rgba(89, 156, 255, 0.2));
  border: 1px solid rgba(156, 89, 255, 0.3);
}

.stat-label {
  font-size: 0.85rem;
  color: #8892b0;
  margin-bottom: 8px;
}

.stat-value {
  font-size: 1.5rem;
  font-weight: 700;
  color: #e0e0e0;
}

.mt-6 {
  margin-top: 24px;
}

.memory-visual {
  padding: 16px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 12px;
}

.visual-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 12px;
  color: #8892b0;
}

.pressure-value {
  font-weight: 700;
  color: #e0e0e0;
}

.action-bar {
  display: flex;
  justify-content: center;
}
</style>