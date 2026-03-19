<template>
  <div class="result-panel">
    <div class="result-header">
      <div class="result-icon">
        <CheckCircleIcon />
      </div>
      <div class="result-info">
        <h3 class="result-title">清理完成</h3>
        <p class="result-subtitle">已成功清理垃圾文件</p>
      </div>
    </div>

    <div class="result-stats">
      <div class="stat-item">
        <span class="stat-value">{{ result.cleaned_count }}</span>
        <span class="stat-label">已清理项目</span>
      </div>
      <div class="stat-divider"></div>
      <div class="stat-item">
        <span class="stat-value">{{ formatSize(result.cleaned_size) }}</span>
        <span class="stat-label">释放空间</span>
      </div>
    </div>

    <div v-if="result.errors.length > 0" class="result-errors">
      <div class="error-header">
        <span class="error-icon">⚠️</span>
        <span class="error-title">部分文件清理失败</span>
      </div>
      <ul class="error-list">
        <li v-for="error in result.errors" :key="error">{{ error }}</li>
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { CleanResult } from '../types'
import CheckCircleIcon from './icons/CheckCircleIcon.vue'

interface Props {
  result: CleanResult
  formatSize: (bytes: number) => string
}

defineProps<Props>()
</script>

<style scoped>
.result-panel {
  margin-top: 24px;
  background: rgba(52, 199, 89, 0.08);
  border: 1px solid rgba(52, 199, 89, 0.2);
  border-radius: 16px;
  padding: 24px;
  animation: slideIn 0.4s ease;
}

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.result-header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 20px;
}

.result-icon {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: rgba(52, 199, 89, 0.15);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #34C759;
}

.result-icon :deep(svg) {
  width: 24px;
  height: 24px;
}

.result-info {
  display: flex;
  flex-direction: column;
}

.result-title {
  font-size: 18px;
  font-weight: 600;
  color: #ffffff;
  margin-bottom: 4px;
}

.result-subtitle {
  font-size: 13px;
  color: #8892b0;
}

.result-stats {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 32px;
  padding: 20px;
  background: rgba(255, 255, 255, 0.04);
  border-radius: 12px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.stat-value {
  font-size: 28px;
  font-weight: 700;
  color: #34C759;
}

.stat-label {
  font-size: 12px;
  color: #8892b0;
  margin-top: 4px;
}

.stat-divider {
  width: 1px;
  height: 40px;
  background: rgba(255, 255, 255, 0.08);
}

.result-errors {
  margin-top: 20px;
  padding: 16px;
  background: rgba(255, 149, 0, 0.1);
  border: 1px solid rgba(255, 149, 0, 0.2);
  border-radius: 12px;
}

.error-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.error-icon {
  font-size: 16px;
}

.error-title {
  font-size: 14px;
  font-weight: 600;
  color: #FF9500;
}

.error-list {
  margin: 0;
  padding-left: 20px;
}

.error-list li {
  font-size: 13px;
  color: #FF9500;
  margin-bottom: 4px;
}

.error-list li:last-child {
  margin-bottom: 0;
}
</style>
