<template>
  <n-card title="📊 清理结果" size="small" class="result-panel">
    <n-statistic label="已清理项目" :value="result.cleaned_count">
      <template #suffix>个</template>
    </n-statistic>
    <n-statistic label="释放空间" :value="formatSize(result.cleaned_size)" class="mt-4" />

    <n-divider v-if="result.errors.length > 0" />

    <div v-if="result.errors.length > 0" class="errors">
      <n-alert type="warning" title="部分文件清理失败">
        <ul class="error-list">
          <li v-for="error in result.errors" :key="error">{{ error }}</li>
        </ul>
      </n-alert>
    </div>
  </n-card>
</template>

<script setup lang="ts">
import { NCard, NStatistic, NDivider, NAlert } from 'naive-ui'
import type { CleanResult } from '../types'

interface Props {
  result: CleanResult
  formatSize: (bytes: number) => string
}

defineProps<Props>()
</script>

<style scoped>
.result-panel {
  margin-top: 20px;
  background: rgba(0, 255, 136, 0.08);
  border: 1px solid rgba(0, 255, 136, 0.2);
}

.mt-4 {
  margin-top: 16px;
}

.error-list {
  margin: 8px 0 0 0;
  padding-left: 20px;
}

.error-list li {
  font-size: 0.85rem;
  color: #f0ad4e;
}
</style>