<template>
  <div class="memory-view">
    <div class="page-header">
      <div class="header-left">
        <h1 class="page-title">内存清理</h1>
        <p class="page-subtitle">释放内存，提升系统性能</p>
      </div>
      <n-button quaternary circle @click="refresh" :loading="isLoading" class="refresh-btn">
        <template #icon>
          <n-icon><RefreshIcon /></n-icon>
        </template>
      </n-button>
    </div>

    <div class="memory-dashboard">
      <div class="main-card">
        <div class="card-header">
          <div class="card-icon">
            <MemoryIcon />
          </div>
          <div class="card-info">
            <span class="card-title">内存状态</span>
            <span class="card-subtitle">实时监控</span>
          </div>
          <span class="status-badge" :class="getPressureClass()">
            {{ getPressureStatus() }}
          </span>
        </div>

        <div class="memory-gauge">
          <div class="gauge-container">
            <svg class="gauge-svg" viewBox="0 0 200 200">
              <circle class="gauge-bg" cx="100" cy="100" r="85" />
              <circle 
                class="gauge-fill" 
                cx="100" 
                cy="100" 
                r="85"
                :style="{ 
                  strokeDasharray: `${memoryInfo.usage * 5.34} 534`,
                  stroke: getPressureColor()
                }"
              />
            </svg>
            <div class="gauge-center">
              <span class="gauge-value">{{ memoryInfo.usage.toFixed(1) }}</span>
              <span class="gauge-unit">%</span>
            </div>
          </div>
          <div class="gauge-label">内存使用率</div>
        </div>

        <div class="memory-stats">
          <div class="stat-item">
            <div class="stat-icon total">
              <MemoryIcon />
            </div>
            <div class="stat-info">
              <span class="stat-value">{{ formatSize(memoryInfo.total) }}</span>
              <span class="stat-label">总内存</span>
            </div>
          </div>
          <div class="stat-item">
            <div class="stat-icon used">
              <ZapIcon />
            </div>
            <div class="stat-info">
              <span class="stat-value">{{ formatSize(memoryInfo.used) }}</span>
              <span class="stat-label">已使用</span>
            </div>
          </div>
          <div class="stat-item">
            <div class="stat-icon available">
              <CheckCircleIcon />
            </div>
            <div class="stat-info">
              <span class="stat-value">{{ formatSize(memoryInfo.available) }}</span>
              <span class="stat-label">可用</span>
            </div>
          </div>
          <div class="stat-item">
            <div class="stat-icon free">
              <SparklesIcon />
            </div>
            <div class="stat-info">
              <span class="stat-value">{{ formatSize(memoryInfo.free) }}</span>
              <span class="stat-label">空闲</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="action-bar">
      <n-button
        type="primary"
        size="large"
        :loading="isFreeing"
        @click="handleFree"
        class="clean-button"
      >
        <template #icon>
          <n-icon><ZapIcon /></n-icon>
        </template>
        {{ isFreeing ? '清理中...' : '一键清理内存' }}
      </n-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { NButton, NIcon, useMessage } from 'naive-ui'
import { useMemory } from '../composables/useMemory'
import { useSettings } from '../composables/useSettings'
import RefreshIcon from './icons/RefreshIcon.vue'
import MemoryIcon from './icons/MemoryIcon.vue'
import ZapIcon from './icons/ZapIcon.vue'
import CheckCircleIcon from './icons/CheckCircleIcon.vue'
import SparklesIcon from './icons/SparklesIcon.vue'

const message = useMessage()
const { settings } = useSettings()

const {
  memoryInfo,
  isLoading,
  isFreeing,
  fetchMemoryInfo,
  freeMemory,
  formatSize,
  startListening
} = useMemory()

const refresh = async () => {
  await fetchMemoryInfo()
}

const handleFree = async () => {
  try {
    const result = await freeMemory()
    if (result.freed_bytes > 0) {
      message.success(`已释放 ${formatSize(result.freed_bytes)} 内存`)
    } else {
      message.info('当前内存状态良好，无需清理')
    }
  } catch (error) {
    message.error('内存清理失败: ' + String(error))
  }
}

const getPressureColor = () => {
  if (memoryInfo.value.usage < settings.warningUsageThreshold) return '#00ff88'
  if (memoryInfo.value.usage < settings.criticalUsageThreshold) return '#f0ad4e'
  return '#ff4757'
}

const getPressureClass = () => {
  if (memoryInfo.value.usage < settings.warningUsageThreshold) return 'success'
  if (memoryInfo.value.usage < settings.criticalUsageThreshold) return 'warning'
  return 'error'
}

const getPressureStatus = () => {
  if (memoryInfo.value.usage < settings.warningUsageThreshold) return '状态良好'
  if (memoryInfo.value.usage < settings.criticalUsageThreshold) return '状态一般'
  return '需要清理'
}

onMounted(() => {
  fetchMemoryInfo()
  startListening()
})
</script>

<style scoped>
.memory-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  animation: slideIn 0.3s ease;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateX(20px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 20px;
  flex-shrink: 0;
}

.header-left {
  display: flex;
  flex-direction: column;
}

.page-title {
  font-size: 22px;
  font-weight: 700;
  color: #ffffff;
  margin-bottom: 4px;
  letter-spacing: -0.5px;
}

.page-subtitle {
  font-size: 12px;
  color: #8892b0;
}

.refresh-btn {
  color: #8892b0;
}

.refresh-btn:hover {
  color: #ffffff;
}

.memory-dashboard {
  flex: 1;
  min-height: 0;
}

.main-card {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 16px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  height: 100%;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
}

.card-icon {
  width: 40px;
  height: 40px;
  background: linear-gradient(135deg, rgba(0, 122, 255, 0.2), rgba(88, 86, 214, 0.2));
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #007AFF;
}

.card-icon :deep(svg) {
  width: 20px;
  height: 20px;
}

.card-info {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.card-title {
  font-size: 18px;
  font-weight: 600;
  color: #ffffff;
}

.card-subtitle {
  font-size: 13px;
  color: #8892b0;
}

.status-badge {
  font-size: 12px;
  font-weight: 600;
  padding: 6px 12px;
  border-radius: 20px;
}

.status-badge.success {
  background: rgba(52, 199, 89, 0.15);
  color: #34C759;
}

.status-badge.warning {
  background: rgba(255, 149, 0, 0.15);
  color: #FF9500;
}

.status-badge.error {
  background: rgba(255, 59, 48, 0.15);
  color: #FF3B30;
}

.memory-gauge {
  display: flex;
  flex-direction: column;
  align-items: center;
  margin-bottom: 16px;
  flex: 1;
  justify-content: center;
}

.gauge-container {
  position: relative;
  width: 140px;
  height: 140px;
}

.gauge-svg {
  transform: rotate(-90deg);
  width: 100%;
  height: 100%;
}

.gauge-bg {
  fill: none;
  stroke: rgba(255, 255, 255, 0.08);
  stroke-width: 10;
}

.gauge-fill {
  fill: none;
  stroke-width: 10;
  stroke-linecap: round;
  transition: stroke-dasharray 0.6s ease;
}

.gauge-center {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  align-items: baseline;
}

.gauge-value {
  font-size: 32px;
  font-weight: 700;
  color: #ffffff;
  line-height: 1;
}

.gauge-unit {
  font-size: 16px;
  font-weight: 500;
  color: #8892b0;
  margin-left: 2px;
}

.gauge-label {
  font-size: 12px;
  color: #8892b0;
  margin-top: 8px;
}

.memory-stats {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 10px;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 10px;
  transition: all 0.2s ease;
}

.stat-item:hover {
  background: rgba(255, 255, 255, 0.06);
}

.stat-icon {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.stat-icon.total {
  background: linear-gradient(135deg, rgba(0, 122, 255, 0.2), rgba(88, 86, 214, 0.2));
  color: #007AFF;
}

.stat-icon.used {
  background: linear-gradient(135deg, rgba(255, 59, 48, 0.2), rgba(255, 149, 0, 0.2));
  color: #FF3B30;
}

.stat-icon.available {
  background: linear-gradient(135deg, rgba(52, 199, 89, 0.2), rgba(48, 219, 91, 0.2));
  color: #34C759;
}

.stat-icon.free {
  background: linear-gradient(135deg, rgba(175, 82, 222, 0.2), rgba(90, 200, 250, 0.2));
  color: #AF52DE;
}

.stat-icon :deep(svg) {
  width: 20px;
  height: 20px;
}

.stat-info {
  display: flex;
  flex-direction: column;
}

.stat-value {
  font-size: 16px;
  font-weight: 700;
  color: #ffffff;
}

.stat-label {
  font-size: 12px;
  color: #8892b0;
}

.action-bar {
  padding: 20px 0;
  display: flex;
  justify-content: center;
  flex-shrink: 0;
}

.clean-button {
  min-width: 200px;
  height: 48px;
  font-size: 15px;
  font-weight: 600;
  border-radius: 12px;
}
</style>
