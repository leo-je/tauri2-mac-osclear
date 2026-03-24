<template>
  <div class="junk-view">
    <div class="page-header">
      <div class="header-left">
        <h1 class="page-title">垃圾清理</h1>
        <p class="page-subtitle">清除系统垃圾文件和已卸载程序残留，释放磁盘空间</p>
      </div>
    </div>

    <div class="mode-switch">
      <n-button
        class="mode-button"
        quaternary
        :class="{ active: activePanel === 'system' }"
        @click="activePanel = 'system'"
      >
        系统垃圾
      </n-button>
      <n-button
        class="mode-button"
        quaternary
        :class="{ active: activePanel === 'uninstalledApps' }"
        @click="activePanel = 'uninstalledApps'"
      >
        卸载残留
      </n-button>
    </div>

    <SystemJunkCleanerPanel v-if="activePanel === 'system'" />
    <UninstalledAppsCleanerPanel v-else />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { NButton } from 'naive-ui'
import SystemJunkCleanerPanel from './SystemJunkCleanerPanel.vue'
import UninstalledAppsCleanerPanel from './UninstalledAppsCleanerPanel.vue'

const activePanel = ref<'system' | 'uninstalledApps'>('system')
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
  margin-bottom: 20px;
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

.mode-switch {
  display: inline-flex;
  gap: 8px;
  padding: 6px;
  margin-bottom: 28px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 16px;
}

.mode-button {
  min-width: 108px;
  height: 42px;
  color: #8892b0;
  font-size: 14px;
  font-weight: 600;
  transition: all 0.15s ease;
}

.mode-button:not(.active):hover {
  color: #ffffff;
}

.mode-button.active {
  color: #ffffff;
  background: rgba(0, 122, 255, 0.18);
  box-shadow: inset 0 0 0 1px rgba(0, 122, 255, 0.35);
}

.mode-button.active :deep(.n-button__border),
.mode-button.active :deep(.n-button__state-border) {
  opacity: 0;
}
</style>
