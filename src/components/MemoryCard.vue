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

    <div class="stats-grid">
      <div class="stat-card total">
        <div class="stat-icon">
          <MemoryIcon />
        </div>
        <div class="stat-content">
          <div class="stat-label">总内存</div>
          <div class="stat-value">{{ formatSize(memoryInfo.total) }}</div>
        </div>
      </div>
      <div class="stat-card used">
        <div class="stat-icon">
          <ZapIcon />
        </div>
        <div class="stat-content">
          <div class="stat-label">已使用</div>
          <div class="stat-value">{{ formatSize(memoryInfo.used) }}</div>
        </div>
      </div>
      <div class="stat-card free">
        <div class="stat-icon">
          <CheckCircleIcon />
        </div>
        <div class="stat-content">
          <div class="stat-label">可用</div>
          <div class="stat-value">{{ formatSize(memoryInfo.free) }}</div>
        </div>
      </div>
      <div class="stat-card cached">
        <div class="stat-icon">
          <SparklesIcon />
        </div>
        <div class="stat-content">
          <div class="stat-label">缓存</div>
          <div class="stat-value">{{ formatSize(memoryInfo.cached) }}</div>
        </div>
      </div>
    </div>

    <div class="pressure-section">
      <div class="section-header">
        <h3 class="section-title">内存使用率</h3>
        <span class="pressure-badge" :class="getPressureClass()">
          {{ getPressureStatus() }}
        </span>
      </div>
      
      <div class="pressure-display">
        <div class="pressure-value-container">
          <span class="pressure-value">{{ memoryInfo.pressure.toFixed(1) }}</span>
          <span class="pressure-unit">%</span>
        </div>
        <n-progress
          type="line"
          :percentage="memoryInfo.pressure"
          :show-indicator="false"
          :height="10"
          :border-radius="5"
          :color="getPressureColor()"
          rail-color="rgba(255,255,255,0.08)"
        />
      </div>

      <div class="pressure-stats">
        <div class="pressure-stat">
          <span class="pressure-stat-value">{{ formatSize(memoryInfo.used) }}</span>
          <span class="pressure-stat-label">已使用</span>
        </div>
        <div class="pressure-divider"></div>
        <div class="pressure-stat">
          <span class="pressure-stat-value">{{ formatSize(memoryInfo.free) }}</span>
          <span class="pressure-stat-label">可用</span>
        </div>
      </div>
    </div>

    <div class="action-section">
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
      <p class="action-hint">清理后将释放系统缓存，提升运行速度</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { NButton, NProgress, NIcon, useMessage } from 'naive-ui'
import { useMemory } from '../composables/useMemory'
import RefreshIcon from './icons/RefreshIcon.vue'
import MemoryIcon from './icons/MemoryIcon.vue'
import ZapIcon from './icons/ZapIcon.vue'
import CheckCircleIcon from './icons/CheckCircleIcon.vue'
import SparklesIcon from './icons/SparklesIcon.vue'

const message = useMessage()

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

const getPressureClass = () => {
  if (memoryInfo.value.pressure < 50) return 'success'
  if (memoryInfo.value.pressure < 75) return 'warning'
  return 'error'
}

const getPressureStatus = () => {
  if (memoryInfo.value.pressure < 50) return '状态良好'
  if (memoryInfo.value.pressure < 75) return '状态一般'
  return '需要清理'
}

onMounted(() => {
  fetchMemoryInfo()
})
</script>

<style scoped>
.memory-view {
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
  margin-bottom: 32px;
}

.header-left {
  display: flex;
  flex-direction: column;
}

.page-title {
  font-size: 28px;
  font-weight: 700;
  color: #ffffff;
  margin-bottom: 8px;
  letter-spacing: -0.5px;
}

.page-subtitle {
  font-size: 14px;
  color: #8892b0;
}

.refresh-btn {
  color: #8892b0;
}

.refresh-btn:hover {
  color: #ffffff;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 32px;
}

.stat-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 16px;
  transition: all 0.2s ease;
}

.stat-card:hover {
  background: rgba(255, 255, 255, 0.06);
  transform: translateY(-2px);
}

.stat-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  flex-shrink: 0;
}

.stat-card.total .stat-icon {
  background: linear-gradient(135deg, rgba(0, 122, 255, 0.2), rgba(88, 86, 214, 0.2));
}

.stat-card.used .stat-icon {
  background: linear-gradient(135deg, rgba(255, 59, 48, 0.2), rgba(255, 149, 0, 0.2));
}

.stat-card.free .stat-icon {
  background: linear-gradient(135deg, rgba(52, 199, 89, 0.2), rgba(48, 219, 91, 0.2));
}

.stat-card.cached .stat-icon {
  background: linear-gradient(135deg, rgba(175, 82, 222, 0.2), rgba(90, 200, 250, 0.2));
}

.stat-icon :deep(svg) {
  width: 24px;
  height: 24px;
}

.stat-content {
  display: flex;
  flex-direction: column;
}

.stat-label {
  font-size: 13px;
  color: #8892b0;
  margin-bottom: 4px;
}

.stat-value {
  font-size: 20px;
  font-weight: 700;
  color: #ffffff;
}

.pressure-section {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 16px;
  padding: 24px;
  margin-bottom: 32px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: #ffffff;
}

.pressure-badge {
  font-size: 12px;
  font-weight: 600;
  padding: 4px 12px;
  border-radius: 20px;
}

.pressure-badge.success {
  background: rgba(52, 199, 89, 0.15);
  color: #34C759;
}

.pressure-badge.warning {
  background: rgba(255, 149, 0, 0.15);
  color: #FF9500;
}

.pressure-badge.error {
  background: rgba(255, 59, 48, 0.15);
  color: #FF3B30;
}

.pressure-display {
  margin-bottom: 20px;
}

.pressure-value-container {
  display: flex;
  align-items: baseline;
  margin-bottom: 12px;
}

.pressure-value {
  font-size: 48px;
  font-weight: 700;
  color: #ffffff;
  line-height: 1;
}

.pressure-unit {
  font-size: 24px;
  font-weight: 500;
  color: #8892b0;
  margin-left: 4px;
}

.pressure-stats {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 32px;
  padding-top: 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.pressure-stat {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.pressure-stat-value {
  font-size: 18px;
  font-weight: 700;
  color: #ffffff;
}

.pressure-stat-label {
  font-size: 12px;
  color: #8892b0;
  margin-top: 4px;
}

.pressure-divider {
  width: 1px;
  height: 40px;
  background: rgba(255, 255, 255, 0.08);
}

.action-section {
  text-align: center;
}

.clean-button {
  min-width: 200px;
  height: 48px;
  font-size: 16px;
  font-weight: 600;
  border-radius: 12px;
}

.action-hint {
  margin-top: 12px;
  font-size: 13px;
  color: #8892b0;
}
</style>
