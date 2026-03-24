<template>
  <div>
    <div class="panel-toolbar">
      <n-button
        type="primary"
        :loading="isScanning"
        @click="startScan"
        class="scan-button"
      >
        <template #icon>
          <n-icon><SearchIcon /></n-icon>
        </template>
        {{ isScanning ? '扫描中...' : '扫描卸载残留' }}
      </n-button>
    </div>

    <div v-if="isScanning" class="scanning-state">
      <div class="scanning-animation">
        <div class="scanning-ring"></div>
        <SearchIcon class="scanning-icon" />
      </div>
      <p class="scanning-text">正在识别已卸载程序遗留的缓存和配置...</p>
    </div>

    <template v-else-if="scanResult.apps.length > 0">
      <div class="results-header">
        <div class="results-summary">
          <span class="results-count">{{ scanResult.apps.length }} 个已卸载程序残留</span>
          <span class="results-count sub">{{ scanResult.total_items }} 个残留项</span>
          <span class="results-size">可清理 {{ formatSize(scanResult.total_size) }}</span>
        </div>
      </div>

      <div class="category-section">
        <h3 class="section-title">已卸载程序</h3>
        <div class="category-grid">
          <div
            v-for="app in scanResult.apps"
            :key="app.app_id"
            class="category-card"
            :class="{ active: expandedAppId === app.app_id }"
            @click="expandedAppId = app.app_id"
          >
            <div class="category-checkbox">
              <div class="checkbox-inner" :class="{ checked: isAppFullySelected(app) }" @click.stop="toggleApp(app.app_id)">
                <CheckCircleIcon v-if="isAppFullySelected(app)" />
              </div>
            </div>
            <div class="category-info">
              <span class="category-name">{{ app.app_name }}</span>
              <span v-if="app.identifier" class="category-subtitle">{{ app.identifier }}</span>
              <span class="category-size">
                {{ app.items.length }} 项 · {{ formatSize(app.total_size) }}
              </span>
            </div>
            <span class="category-action">查看详情</span>
          </div>
        </div>
      </div>

      <div v-if="activeApp" class="items-section">
        <div class="section-header">
          <div class="detail-title-group">
            <h3 class="section-title">{{ activeApp.app_name }} 的残留详情</h3>
            <span v-if="activeApp.identifier" class="detail-subtitle">{{ activeApp.identifier }}</span>
          </div>
          <div class="section-actions">
            <n-button size="small" quaternary @click="toggleApp(activeApp.app_id)">选择此程序</n-button>
            <n-button size="small" quaternary @click="selectAll">全选</n-button>
            <n-button size="small" quaternary @click="deselectAll">取消全选</n-button>
          </div>
        </div>

        <div class="app-groups">
          <div class="app-group">
            <div class="app-group-header">
              <div class="item-checkbox">
                <div class="checkbox-inner" :class="{ checked: isAppFullySelected(activeApp) }" @click.stop="toggleApp(activeApp.app_id)">
                  <CheckCircleIcon v-if="isAppFullySelected(activeApp)" />
                </div>
              </div>
              <div class="app-group-info">
                <span class="app-group-name">{{ activeApp.app_name }}</span>
                <span v-if="activeApp.identifier" class="app-group-identifier">{{ activeApp.identifier }}</span>
              </div>
              <span class="app-group-size">{{ formatSize(activeApp.total_size) }}</span>
            </div>

            <div class="junk-list">
              <div
                v-for="item in activeApp.items"
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
        </div>
      </div>

      <div class="action-bar">
        <div class="selection-info">
          <span v-if="selectedItems.length > 0">
            已选择 {{ selectedItems.length }} 个项目，涉及 {{ selectedAppCount }} 个程序
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

    <div v-else-if="!hasScanned" class="empty-state">
      <div class="empty-icon">
        <SearchIcon />
      </div>
      <h3 class="empty-title">开始扫描卸载残留</h3>
      <p class="empty-description">扫描已卸载程序遗留的缓存、配置和容器目录。</p>
    </div>

    <div v-else class="empty-state">
      <div class="empty-icon success">
        <CheckCircleIcon />
      </div>
      <h3 class="empty-title">未发现卸载残留</h3>
      <p class="empty-description">当前未识别到明显的已卸载程序垃圾。</p>
    </div>

    <CleanResultPanel
      v-if="lastCleanResult"
      :result="lastCleanResult"
      :formatSize="formatSize"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { NButton, NIcon, useDialog, useMessage } from 'naive-ui'
import { useSettings } from '../composables/useSettings'
import { useUninstalledAppsCleaner } from '../composables/useUninstalledAppsCleaner'
import CleanResultPanel from './CleanResultPanel.vue'
import SearchIcon from './icons/SearchIcon.vue'
import DeleteIcon from './icons/DeleteIcon.vue'
import CheckCircleIcon from './icons/CheckCircleIcon.vue'

const message = useMessage()
const dialog = useDialog()
const { settings } = useSettings()
const expandedAppId = ref<string | null>(null)

const {
  scanResult,
  selectedItems,
  selectedSize,
  selectedAppCount,
  isScanning,
  isCleaning,
  hasScanned,
  lastCleanResult,
  isAppFullySelected,
  toggleItem,
  toggleApp,
  scanUninstalledAppsJunk,
  cleanSelected,
  selectAll,
  deselectAll,
  formatSize
} = useUninstalledAppsCleaner()

const activeApp = computed(() => {
  if (!scanResult.value.apps.length) return null
  if (!expandedAppId.value) return scanResult.value.apps[0]
  return scanResult.value.apps.find(app => app.app_id === expandedAppId.value) ?? scanResult.value.apps[0]
})

watch(
  () => scanResult.value.apps,
  apps => {
    if (!apps.length) {
      expandedAppId.value = null
      return
    }

    if (!expandedAppId.value || !apps.some(app => app.app_id === expandedAppId.value)) {
      expandedAppId.value = apps[0].app_id
    }
  },
  { immediate: true }
)

const startScan = async () => {
  try {
    await scanUninstalledAppsJunk()
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
      title: '确认清理所选卸载残留？',
      content: `即将把 ${selectedItems.value.length} 个残留项目移入废纸篓，预计释放 ${formatSize(selectedSize.value)}。`,
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
.panel-toolbar {
  display: flex;
  justify-content: flex-end;
  margin-bottom: 32px;
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
  border-top-color: #007aff;
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
  color: #007aff;
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
  flex-wrap: wrap;
}

.results-count {
  font-size: 18px;
  font-weight: 600;
  color: #ffffff;
}

.results-count.sub {
  font-size: 14px;
  color: #8892b0;
}

.results-size {
  font-size: 14px;
  color: #34c759;
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
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: 12px;
}

.category-card {
  display: flex;
  align-items: flex-start;
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

.category-action {
  margin-left: auto;
  align-self: center;
  font-size: 12px;
  font-weight: 600;
  color: #66b3ff;
  white-space: nowrap;
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
  background: #007aff;
  border-color: #007aff;
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
  min-width: 0;
}

.category-name,
.app-group-name {
  font-size: 14px;
  font-weight: 600;
  color: #ffffff;
}

.category-subtitle,
.app-group-identifier {
  font-size: 11px;
  color: #8892b0;
  word-break: break-all;
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

.detail-title-group {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.detail-subtitle {
  font-size: 12px;
  color: #8892b0;
  word-break: break-all;
}

.section-actions {
  display: flex;
  gap: 8px;
}

.app-groups {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.app-group {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 16px;
  overflow: hidden;
}

.app-group-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 18px;
  cursor: pointer;
  background: rgba(255, 255, 255, 0.03);
}

.app-group-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.app-group-size {
  font-size: 13px;
  font-weight: 600;
  color: #34c759;
}

.junk-list {
  max-height: 360px;
  overflow-y: auto;
}

.junk-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  cursor: pointer;
  transition: background 0.15s ease;
  border-top: 1px solid rgba(255, 255, 255, 0.04);
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
  color: #34c759;
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
  color: #34c759;
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
  max-width: 360px;
}
</style>
