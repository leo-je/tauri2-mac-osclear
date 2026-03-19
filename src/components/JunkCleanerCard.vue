<template>
  <section class="card junk-card">
    <div class="card-header">
      <h2>垃圾文件清理</h2>
      <button
        @click="startScan"
        :disabled="isScanning"
        class="btn btn-secondary"
      >
        {{ isScanning ? '扫描中...' : '开始扫描' }}
      </button>
    </div>

    <div v-if="scanResult.categories.length > 0" class="category-summary">
      <div
        v-for="[category, size] in scanResult.categories"
        :key="category"
        class="category-item"
        @click="toggleCategory(category)"
        :class="{ active: selectedCategories.includes(category) }"
      >
        <span class="category-name">{{ category }}</span>
        <span class="category-size">{{ formatSize(size) }}</span>
      </div>
      <div class="total-size">
        总计可清理: <strong>{{ formatSize(scanResult.total_size) }}</strong>
      </div>
    </div>

    <div v-if="scanResult.items.length > 0" class="junk-list">
      <div class="list-header">
        <span>找到 {{ scanResult.items.length }} 个垃圾文件/目录</span>
        <button
          @click="handleClean"
          :disabled="isCleaning || selectedItems.length === 0"
          class="btn btn-danger"
        >
          {{ isCleaning ? '清理中...' : `清理选中 (${formatSize(selectedSize)})` }}
        </button>
      </div>

      <div class="junk-items">
        <div
          v-for="item in scanResult.items"
          :key="item.path"
          class="junk-item"
          :class="{ selected: selectedItems.includes(item.path) }"
          @click="toggleItem(item.path)"
        >
          <div class="item-checkbox">
            <input
              type="checkbox"
              :checked="selectedItems.includes(item.path)"
              @click.stop
              @change="toggleItem(item.path)"
            />
          </div>
          <div class="item-info">
            <span class="item-path">{{ item.path }}</span>
            <span class="item-category">{{ item.category }}</span>
          </div>
          <span class="item-size">{{ formatSize(item.size) }}</span>
        </div>
      </div>
    </div>

    <div v-else-if="!isScanning && hasScanned" class="empty-state">
      <p>未发现可清理的垃圾文件</p>
    </div>

    <div v-else-if="!isScanning" class="empty-state">
      <p>点击"开始扫描"查找系统中的垃圾文件</p>
    </div>

    <CleanResultPanel
      v-if="lastCleanResult"
      :result="lastCleanResult"
      :formatSize="formatSize"
    />
  </section>
</template>

<script setup lang="ts">
import { useJunkCleaner } from '../composables/useJunkCleaner'
import CleanResultPanel from './CleanResultPanel.vue'

const emit = defineEmits<{
  (e: 'success', message: string): void
  (e: 'error', message: string): void
}>()

const {
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
  formatSize
} = useJunkCleaner()

const startScan = async () => {
  try {
    await scanJunkFiles()
  } catch (error) {
    emit('error', '扫描失败: ' + String(error))
  }
}

const handleClean = async () => {
  try {
    const result = await cleanSelected()
    emit(
      'success',
      `清理完成! 已清理 ${result.cleaned_count} 个项目，释放 ${formatSize(result.cleaned_size)}`
    )
  } catch (error) {
    emit('error', '清理失败: ' + String(error))
  }
}
</script>

<style scoped>
.junk-card {
  background: rgba(255, 255, 255, 0.95);
}

.category-summary {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-bottom: 20px;
}

.category-item {
  display: flex;
  flex-direction: column;
  padding: 12px 16px;
  background: #f5f7fa;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s ease;
  border: 2px solid transparent;
  min-width: 120px;
}

.category-item:hover {
  background: #e8eaf0;
}

.category-item.active {
  border-color: #667eea;
  background: #f0f3ff;
}

.category-name {
  font-size: 0.9rem;
  color: #666;
  margin-bottom: 4px;
}

.category-size {
  font-size: 1.1rem;
  font-weight: 700;
  color: #1a1a2e;
}

.total-size {
  width: 100%;
  margin-top: 8px;
  padding: 12px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border-radius: 10px;
  text-align: center;
  font-size: 1.1rem;
}

.junk-list {
  border-top: 1px solid #e0e0e0;
  padding-top: 20px;
}

.list-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.junk-items {
  max-height: 400px;
  overflow-y: auto;
  border: 1px solid #e0e0e0;
  border-radius: 10px;
}

.junk-item {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid #f0f0f0;
  cursor: pointer;
  transition: all 0.2s ease;
}

.junk-item:last-child {
  border-bottom: none;
}

.junk-item:hover {
  background: #f9f9f9;
}

.junk-item.selected {
  background: #f0f3ff;
}

.item-checkbox {
  margin-right: 12px;
}

.item-checkbox input {
  width: 18px;
  height: 18px;
  cursor: pointer;
}

.item-info {
  flex: 1;
  min-width: 0;
}

.item-path {
  display: block;
  font-size: 0.85rem;
  color: #333;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-category {
  display: inline-block;
  font-size: 0.75rem;
  color: #667eea;
  background: #f0f3ff;
  padding: 2px 8px;
  border-radius: 4px;
  margin-top: 4px;
}

.item-size {
  font-weight: 600;
  color: #e74c3c;
  margin-left: 12px;
  flex-shrink: 0;
}

.empty-state {
  text-align: center;
  padding: 40px;
  color: #999;
}

@media (max-width: 768px) {
  .category-summary {
    justify-content: center;
  }

  .category-item {
    min-width: 100px;
  }

  .list-header {
    flex-direction: column;
    gap: 12px;
    align-items: stretch;
  }
}
</style>
