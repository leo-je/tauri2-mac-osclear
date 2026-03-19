<template>
  <n-card title="🗑️ 垃圾文件清理" size="large">
    <template #header-extra>
      <n-button
        type="info"
        :loading="isScanning"
        @click="startScan"
      >
        <template #icon>
          <n-icon><SearchIcon /></n-icon>
        </template>
        {{ isScanning ? '扫描中...' : '开始扫描' }}
      </n-button>
    </template>

    <div v-if="scanResult.categories.length > 0" class="category-section">
      <div class="section-title">
        <span>📂 垃圾类别</span>
        <n-tag type="warning" size="small">
          可清理: {{ formatSize(scanResult.total_size) }}
        </n-tag>
      </div>

      <div class="category-grid">
        <div
          v-for="[category, size] in scanResult.categories"
          :key="category"
          class="category-card"
          :class="{ active: selectedCategories.includes(category) }"
          @click="toggleCategory(category)"
        >
          <div class="category-name">{{ category }}</div>
          <div class="category-size">{{ formatSize(size) }}</div>
        </div>
      </div>
    </div>

    <div v-if="scanResult.items.length > 0" class="junk-section mt-6">
      <div class="section-header">
        <span>📋 找到 {{ scanResult.items.length }} 个垃圾文件</span>
        <n-space>
          <n-button
            size="small"
            @click="selectAll"
          >
            全选
          </n-button>
          <n-button
            size="small"
            @click="deselectAll"
          >
            取消全选
          </n-button>
          <n-button
            type="error"
            :loading="isCleaning"
            :disabled="selectedItems.length === 0"
            @click="handleClean"
          >
            <template #icon>
              <n-icon><DeleteIcon /></n-icon>
            </template>
            清理选中 ({{ formatSize(selectedSize) }})
          </n-button>
        </n-space>
      </div>

      <div class="junk-list">
        <div
          v-for="item in scanResult.items"
          :key="item.path"
          class="junk-item"
          :class="{ selected: selectedItems.includes(item.path) }"
        >
          <n-checkbox
            :checked="selectedItems.includes(item.path)"
            @update:checked="() => toggleItem(item.path)"
          />
          <div class="item-info">
            <span class="item-path" :title="item.path">{{ item.path }}</span>
            <n-tag size="tiny" type="info">{{ item.category }}</n-tag>
          </div>
          <span class="item-size">{{ formatSize(item.size) }}</span>
        </div>
      </div>
    </div>

    <div v-else-if="!isScanning && hasScanned" class="empty-state">
      <n-empty description="未发现可清理的垃圾文件" size="large">
        <template #icon>
          <n-icon size="60"><SuccessIcon /></n-icon>
        </template>
      </n-empty>
    </div>

    <div v-else-if="!isScanning" class="empty-state">
      <n-empty description="点击「开始扫描」查找系统垃圾文件" size="large">
        <template #icon>
          <n-icon size="60"><SearchIconLarge /></n-icon>
        </template>
      </n-empty>
    </div>

    <CleanResultPanel
      v-if="lastCleanResult"
      :result="lastCleanResult"
      :formatSize="formatSize"
    />
  </n-card>
</template>

<script setup lang="ts">
import { h } from 'vue'
import {
  NCard, NButton, NSpace, NTag, NCheckbox, NEmpty, NIcon, useMessage
} from 'naive-ui'
import { useJunkCleaner } from '../composables/useJunkCleaner'
import CleanResultPanel from './CleanResultPanel.vue'

const message = useMessage()

const SearchIcon = () => h('svg', {
  xmlns: 'http://www.w3.org/2000/svg',
  viewBox: '0 0 24 24',
  fill: 'none',
  stroke: 'currentColor',
  'stroke-width': '2',
  width: '20',
  height: '20'
}, [
  h('circle', { cx: '11', cy: '11', r: '8' }),
  h('path', { d: 'M21 21l-4.35-4.35' })
])

const SearchIconLarge = () => h('svg', {
  xmlns: 'http://www.w3.org/2000/svg',
  viewBox: '0 0 24 24',
  fill: 'none',
  stroke: 'currentColor',
  'stroke-width': '1.5',
  width: '60',
  height: '60'
}, [
  h('circle', { cx: '11', cy: '11', r: '8' }),
  h('path', { d: 'M21 21l-4.35-4.35' })
])

const DeleteIcon = () => h('svg', {
  xmlns: 'http://www.w3.org/2000/svg',
  viewBox: '0 0 24 24',
  fill: 'currentColor',
  width: '20',
  height: '20'
}, [
  h('path', { d: 'M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z' })
])

const SuccessIcon = () => h('svg', {
  xmlns: 'http://www.w3.org/2000/svg',
  viewBox: '0 0 24 24',
  fill: 'currentColor',
  width: '60',
  height: '60'
}, [
  h('path', { d: 'M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-2 15l-5-5 1.41-1.41L10 14.17l7.59-7.59L19 8l-9 9z' })
])

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
  selectAll,
  deselectAll,
  formatSize
} = useJunkCleaner()

const startScan = async () => {
  try {
    await scanJunkFiles()
  } catch (error) {
    message.error('扫描失败: ' + String(error))
  }
}

const handleClean = async () => {
  try {
    const result = await cleanSelected()
    message.success(
      `清理完成! 已清理 ${result.cleaned_count} 个项目，释放 ${formatSize(result.cleaned_size)}`
    )
  } catch (error) {
    message.error('清理失败: ' + String(error))
  }
}
</script>

<style scoped>
.category-section {
  margin-bottom: 20px;
}

.section-title {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  font-size: 1rem;
  color: #8892b0;
}

.category-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 12px;
}

.category-card {
  padding: 16px;
  background: rgba(255, 255, 255, 0.03);
  border: 2px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.25s ease;
}

.category-card:hover {
  background: rgba(255, 255, 255, 0.06);
  border-color: rgba(0, 217, 255, 0.3);
}

.category-card.active {
  background: rgba(255, 71, 87, 0.15);
  border-color: rgba(255, 71, 87, 0.5);
}

.category-name {
  font-size: 0.9rem;
  color: #e0e0e0;
  margin-bottom: 8px;
}

.category-size {
  font-size: 1.1rem;
  font-weight: 700;
  color: #00ff88;
}

.mt-6 {
  margin-top: 24px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  color: #8892b0;
}

.junk-list {
  max-height: 400px;
  overflow-y: auto;
  background: rgba(255, 255, 255, 0.02);
  border-radius: 12px;
  padding: 8px;
}

.junk-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border-radius: 8px;
  transition: background 0.2s;
}

.junk-item:hover {
  background: rgba(255, 255, 255, 0.05);
}

.junk-item.selected {
  background: rgba(255, 71, 87, 0.1);
}

.item-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.item-path {
  font-size: 0.85rem;
  color: #c0c0c0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-size {
  font-size: 0.9rem;
  font-weight: 600;
  color: #00ff88;
  white-space: nowrap;
}

.empty-state {
  padding: 60px 20px;
  text-align: center;
}

::-webkit-scrollbar {
  display: none;
}

::-webkit-scrollbar-track {
  display: none;
}

::-webkit-scrollbar-thumb {
  display: none;
}

::-webkit-scrollbar-thumb:hover {
  display: none;
}
</style>