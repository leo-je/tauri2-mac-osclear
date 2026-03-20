<template>
  <div class="process-list">
    <div class="list-header">
      <h3 class="list-title">内存使用排行</h3>
      <n-button quaternary size="small" @click="$emit('refresh')">
        <template #icon>
          <n-icon><RefreshIcon /></n-icon>
        </template>
      </n-button>
    </div>

    <div class="list-content" v-if="processes.length > 0">
      <div 
        v-for="process in processes.slice(0, 20)" 
        :key="process.pid"
        class="process-item"
      >
        <div class="process-info">
          <span class="process-name" :title="process.name">{{ process.name }}</span>
          <span class="process-pid">PID: {{ process.pid }}</span>
        </div>
        <div class="process-stats">
          <span class="process-memory">{{ formatSize(process.memory_bytes) }}</span>
          <div class="memory-bar">
            <div 
              class="memory-bar-fill" 
              :style="{ width: getMemoryPercent(process.memory_bytes) + '%' }"
            ></div>
          </div>
        </div>
      </div>
    </div>

    <div class="empty-state" v-else>
      <span>加载中...</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { NButton, NIcon } from 'naive-ui'
import type { ProcessMemoryInfo } from '../types'
import RefreshIcon from './icons/RefreshIcon.vue'

interface Props {
  processes: ProcessMemoryInfo[]
  formatSize: (bytes: number) => string
}

const props = defineProps<Props>()
defineEmits(['refresh'])

const getMemoryPercent = (bytes: number): number => {
  if (props.processes.length === 0) return 0
  const maxMemory = props.processes[0].memory_bytes
  return (bytes / maxMemory) * 100
}
</script>

<style scoped>
.process-list {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 16px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  height: 100%;
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  flex-shrink: 0;
}

.list-title {
  font-size: 16px;
  font-weight: 600;
  color: #ffffff;
}

.list-content {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.process-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 12px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 8px;
  transition: background 0.15s ease;
}

.process-item:hover {
  background: rgba(255, 255, 255, 0.06);
}

.process-info {
  display: flex;
  flex-direction: column;
  min-width: 0;
  flex: 1;
  margin-right: 16px;
}

.process-name {
  font-size: 13px;
  font-weight: 500;
  color: #ffffff;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.process-pid {
  font-size: 11px;
  color: #666666;
}

.process-stats {
  display: flex;
  flex-direction: column;
  align-items: flex-end;
  min-width: 100px;
}

.process-memory {
  font-size: 13px;
  font-weight: 600;
  color: #007AFF;
  margin-bottom: 4px;
}

.memory-bar {
  width: 100px;
  height: 4px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 2px;
  overflow: hidden;
}

.memory-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, #007AFF, #5856D6);
  border-radius: 2px;
  transition: width 0.3s ease;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100px;
  color: #666666;
  font-size: 14px;
}
</style>
