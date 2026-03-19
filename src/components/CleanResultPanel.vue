<template>
  <div class="result-panel">
    <h3>清理结果</h3>
    <p>已清理: {{ result.cleaned_count }} 个项目</p>
    <p>释放空间: {{ formatSize(result.cleaned_size) }}</p>
    <div v-if="result.errors.length > 0" class="errors">
      <p>错误信息:</p>
      <ul>
        <li v-for="error in result.errors" :key="error">{{ error }}</li>
      </ul>
    </div>
  </div>
</template>

<script setup lang="ts">
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
  padding: 16px;
  background: linear-gradient(135deg, #d4edda 0%, #c3e6cb 100%);
  border-radius: 12px;
}

.result-panel h3 {
  color: #155724;
  margin-bottom: 12px;
}

.result-panel p {
  color: #155724;
  margin-bottom: 8px;
}

.errors {
  margin-top: 12px;
  padding: 12px;
  background: #f8d7da;
  border-radius: 8px;
}

.errors p {
  color: #721c24;
  font-weight: 600;
}

.errors ul {
  margin-top: 8px;
  padding-left: 20px;
}

.errors li {
  color: #721c24;
  font-size: 0.85rem;
}
</style>
