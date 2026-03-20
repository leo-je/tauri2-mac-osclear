<template>
  <div class="junk-view">
    <div class="page-header">
      <div class="header-left">
        <h1 class="page-title">垃圾清理</h1>
        <p class="page-subtitle">清除系统垃圾文件，释放磁盘空间</p>
      </div>
      <n-button
        type="primary"
        :loading="isScanning"
        @click="startScan"
        class="scan-button"
      >
        <template #icon>
          <n-icon><SearchIcon /></n-icon>
        </template>
        {{ isScanning ? '扫描中...' : '开始扫描' }}
      </n-button>
    </div>

    <!-- Scanning State -->
    <div v-if="isScanning" class="scanning-state">
      <div class="scanning-animation">
        <div class="scanning-ring"></div>
        <SearchIcon class="scanning-icon" />
      </div>
      <p class="scanning-text">正在扫描系统垃圾文件...</p>
    </div>

    <!-- Scan Results -->
    <template v-else-if="scanResult.categories.length > 0">
      <div class="results-header">
        <div class="results-summary">
          <span class="results-count">{{ scanResult.categories.length }} 类垃圾文件</span>
          <span class="results-size">可清理 {{ formatSize(scanResult.total_size) }}</span>
        </div>
      </div>

      <div class="category-section">
        <h3 class="section-title">垃圾类别</h3>
        <div class="category-grid">
          <div
            v-for="[category, size] in scanResult.categories"
            :key="category"
            class="category-card"
            :class="{ active: selectedCategories.includes(category) }"
            @click="toggleCategory(category)"
          >
            <div class="category-checkbox">
              <div class="checkbox-inner" :class="{ checked: selectedCategories.includes(category) }">
                <CheckCircleIcon v-if="selectedCategories.includes(category)" />
              </div>
            </div>
            <div class="category-info">
              <span class="category-name">{{ category }}</span>
              <span class="category-size">{{ formatSize(size) }}</span>
            </div>
          </div>
        </div>
      </div>

      <div class="items-section">
        <div class="section-header">
          <h3 class="section-title">找到 {{ scanResult.items.length }} 个垃圾文件</h3>
          <div class="section-actions">
            <n-button size="small" quaternary @click="selectAll">全选</n-button>
            <n-button size="small" quaternary @click="deselectAll">取消全选</n-button>
          </div>
        </div>

        <div class="junk-list">
          <div
            v-for="item in scanResult.items"
            :key="item.path"
            class="junk-item"
            :class="{ selected: selectedItems.includes(item.path) }"
            @click="toggleItem(item.path)"
          >
            <div class="item-checkbox">
              <div class="checkbox-inner" :class="{ checked: selectedItems.includes(item.path) }">
                <CheckCircleIcon v-if="selectedItems.includes(item.path)" />
              </div>
            </div>
            <div class="item-info">
              <span class="item-path" :title="item.path">{{ item.path }}</span>
              <span class="item-category">{{ item.category }}</span>
            </div>
            <span class="item-size">{{ formatSize(item.size) }}</span>
          </div>
        </div>
      </div>

      <div class="action-bar">
        <div class="selection-info">
          <span v-if="selectedItems.length > 0">
            已选择 {{ selectedItems.length }} 个项目
          </span>
          <span v-else class="no-selection">未选择任何项目</span>
        </div>
        <n-button
          type="error"
          size="large"
          :loading="isCleaning"
          :disabled="selectedItems.length === 0"
          @click="handleClean"
          class="clean-button"
        >
          <template #icon>
            <n-icon><DeleteIcon /></n-icon>
          </template>
          清理选中 ({{ formatSize(selectedSize) }})
        </n-button>
      </div>
    </template>

    <!-- Empty State (not scanned) -->
    <div v-else-if="!hasScanned" class="empty-state">
      <div class="empty-icon">
        <SearchIcon />
      </div>
      <h3 class="empty-title">开始扫描垃圾文件</h3>
      <p class="empty-description">点击上方的「开始扫描」按钮，查找系统中的垃圾文件</p>
    </div>

    <!-- Empty State (no junk found) -->
    <div v-else class="empty-state">
      <div class="empty-icon success">
        <CheckCircleIcon />
      </div>
      <h3 class="empty-title">系统很干净</h3>
      <p class="empty-description">未发现可清理的垃圾文件</p>
    </div>

    <CleanResultPanel
      v-if="lastCleanResult"
      :result="lastCleanResult"
      :formatSize="formatSize"
    />
  </div>
</template>

<script setup lang="ts">
import { NButton, NIcon, useDialog, useMessage } from 'naive-ui'
import { useJunkCleaner } from '../composables/useJunkCleaner'
import { useSettings } from '../composables/useSettings'
import CleanResultPanel from './CleanResultPanel.vue'
import SearchIcon from './icons/SearchIcon.vue'
import DeleteIcon from './icons/DeleteIcon.vue'
import CheckCircleIcon from './icons/CheckCircleIcon.vue'

const message = useMessage()
const dialog = useDialog()
const { settings } = useSettings()

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
  const runClean = async () => {
    try {
      const result = await cleanSelected()
      message.success(
        `清理完成! 已清理 ${result.cleaned_count} 个项目，释放 ${formatSize(result.cleaned_size)}`
      )
    } catch (error) {
      message.error('清理失败: ' + String(error))
    }
  }

  if (settings.confirmBeforeCleaning) {
    dialog.warning({
      title: '确认清理所选项目？',
      content: `即将把 ${selectedItems.value.length} 个项目移入废纸篓，预计释放 ${formatSize(selectedSize.value)}。`,
      positiveText: '确认清理',
      negativeText: '取消',
      onPositiveClick: runClean
    })
    return
  }

  await runClean()
}
</script>

<style scoped>
.junk-view {
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

.scan-button {
  height: 44px;
  font-weight: 600;
  border-radius: 12px;
}

.scanning-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 20px;
}

.scanning-animation {
  position: relative;
  width: 80px;
  height: 80px;
  margin-bottom: 24px;
}

.scanning-ring {
  position: absolute;
  inset: 0;
  border: 3px solid rgba(0, 122, 255, 0.2);
  border-top-color: #007AFF;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.scanning-icon {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  color: #007AFF;
}

.scanning-icon :deep(svg) {
  width: 32px;
  height: 32px;
}

.scanning-text {
  font-size: 16px;
  color: #8892b0;
}

.results-header {
  margin-bottom: 24px;
}

.results-summary {
  display: flex;
  align-items: center;
  gap: 16px;
}

.results-count {
  font-size: 18px;
  font-weight: 600;
  color: #ffffff;
}

.results-size {
  font-size: 14px;
  color: #34C759;
  font-weight: 500;
  padding: 4px 12px;
  background: rgba(52, 199, 89, 0.15);
  border-radius: 20px;
}

.category-section {
  margin-bottom: 32px;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: #ffffff;
  margin-bottom: 16px;
}

.category-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 12px;
}

.category-card {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.15s ease;
}

.category-card:hover {
  background: rgba(255, 255, 255, 0.06);
  border-color: rgba(255, 255, 255, 0.12);
}

.category-card.active {
  background: rgba(0, 122, 255, 0.1);
  border-color: rgba(0, 122, 255, 0.3);
}

.category-checkbox,
.item-checkbox {
  width: 24px;
  height: 24px;
  flex-shrink: 0;
}

.checkbox-inner {
  width: 24px;
  height: 24px;
  border: 2px solid rgba(255, 255, 255, 0.2);
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.checkbox-inner.checked {
  background: #007AFF;
  border-color: #007AFF;
  color: white;
}

.checkbox-inner :deep(svg) {
  width: 16px;
  height: 16px;
}

.category-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.category-name {
  font-size: 14px;
  font-weight: 500;
  color: #ffffff;
}

.category-size {
  font-size: 13px;
  color: #8892b0;
}

.items-section {
  margin-bottom: 32px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.section-actions {
  display: flex;
  gap: 8px;
}

.junk-list {
  max-height: 400px;
  overflow-y: auto;
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 12px;
}

.junk-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  cursor: pointer;
  transition: background 0.15s ease;
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.junk-item:last-child {
  border-bottom: none;
}

.junk-item:hover {
  background: rgba(255, 255, 255, 0.04);
}

.junk-item.selected {
  background: rgba(0, 122, 255, 0.08);
}

.item-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.item-path {
  font-size: 13px;
  color: #e0e0e0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-category {
  font-size: 11px;
  color: #8892b0;
}

.item-size {
  font-size: 13px;
  font-weight: 600;
  color: #34C759;
  white-space: nowrap;
}

.action-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 16px;
}

.selection-info {
  font-size: 14px;
  color: #8892b0;
}

.no-selection {
  color: #666666;
}

.clean-button {
  height: 44px;
  font-weight: 600;
  border-radius: 12px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 20px;
  text-align: center;
}

.empty-icon {
  width: 80px;
  height: 80px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.05);
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 24px;
  color: #8892b0;
}

.empty-icon.success {
  background: rgba(52, 199, 89, 0.1);
  color: #34C759;
}

.empty-icon :deep(svg) {
  width: 36px;
  height: 36px;
}

.empty-title {
  font-size: 20px;
  font-weight: 600;
  color: #ffffff;
  margin-bottom: 8px;
}

.empty-description {
  font-size: 14px;
  color: #8892b0;
  max-width: 300px;
}
</style>
