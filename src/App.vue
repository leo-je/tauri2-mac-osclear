<template>
  <div class="app-shell" :class="{ 'reduce-motion': settings.reduceMotion }">
    <n-config-provider :theme="darkTheme">
      <n-message-provider>
        <n-dialog-provider>
          <AppLayout :activeItem="currentView" @navigate="handleNavigate">
          <!-- Dashboard View -->
          <div v-if="currentView === 'dashboard'" class="dashboard-view">
            <div class="page-header">
              <h1 class="page-title">系统总览</h1>
              <p class="page-subtitle">查看系统状态和快速清理</p>
            </div>

            <div class="dashboard-grid">
              <div class="overview-card memory-card">
                <div class="card-header">
                  <div class="card-icon memory">
                    <MemoryIcon />
                  </div>
                  <div class="card-info">
                    <span class="card-title">内存状态</span>
                    <span class="card-subtitle">实时监控</span>
                  </div>
                </div>
                <div class="card-stat">
                  <span class="stat-value">{{ memoryUsage.toFixed(1) }}%</span>
                  <span class="stat-label">使用率</span>
                </div>
                <n-progress
                  type="line"
                  :percentage="memoryUsage"
                  :show-indicator="false"
                  :height="6"
                  :border-radius="3"
                  :color="getPressureColor()"
                  rail-color="rgba(255,255,255,0.1)"
                />
                <n-button
                  type="primary"
                  size="small"
                  class="card-action"
                  @click="currentView = 'memory'"
                >
                  清理内存
                </n-button>
              </div>

              <div class="overview-card junk-card">
                <div class="card-header">
                  <div class="card-icon junk">
                    <TrashIcon />
                  </div>
                  <div class="card-info">
                    <span class="card-title">垃圾文件</span>
                    <span class="card-subtitle">系统清理</span>
                  </div>
                </div>
                <div class="card-stat">
                  <span class="stat-value">—</span>
                  <span class="stat-label">待扫描</span>
                </div>
                <div class="card-placeholder">
                  点击下方按钮开始扫描
                </div>
                <n-button
                  type="primary"
                  size="small"
                  class="card-action"
                  @click="currentView = 'junk'"
                >
                  开始扫描
                </n-button>
              </div>
            </div>

            <div v-if="settings.showDashboardQuickActions" class="quick-actions">
              <h3 class="section-title">快速操作</h3>
              <div class="action-grid">
                <div class="action-item" @click="currentView = 'memory'">
                  <ZapIcon />
                  <span>一键清理内存</span>
                </div>
                <div class="action-item" @click="currentView = 'junk'">
                  <SearchIcon />
                  <span>扫描垃圾文件</span>
                </div>
                <div class="action-item" @click="currentView = 'settings'">
                  <SettingsIcon />
                  <span>调整偏好设置</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Memory View -->
          <MemoryCard v-else-if="currentView === 'memory'" />

            <!-- Junk View -->
            <JunkCleanerCard v-else-if="currentView === 'junk'" />

            <!-- Settings View -->
            <SettingsPage v-else-if="currentView === 'settings'" />
          </AppLayout>
        </n-dialog-provider>
      </n-message-provider>
    </n-config-provider>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import type { AppView } from './types'
import { darkTheme, NConfigProvider, NMessageProvider, NDialogProvider, NProgress, NButton } from 'naive-ui'
import AppLayout from './components/AppLayout.vue'
import MemoryCard from './components/MemoryCard.vue'
import JunkCleanerCard from './components/JunkCleanerCard.vue'
import SettingsPage from './components/SettingsPage.vue'
import MemoryIcon from './components/icons/MemoryIcon.vue'
import TrashIcon from './components/icons/TrashIcon.vue'
import ZapIcon from './components/icons/ZapIcon.vue'
import SearchIcon from './components/icons/SearchIcon.vue'
import SettingsIcon from './components/icons/SettingsIcon.vue'
import { useMemory } from './composables/useMemory'
import { useSettings } from './composables/useSettings'

const { settings } = useSettings()
const currentView = ref<AppView>(settings.startupView)
const { memoryInfo, startListening } = useMemory()

const memoryUsage = computed(() => memoryInfo.value.usage)

const getPressureColor = () => {
  if (memoryUsage.value < settings.warningUsageThreshold) return '#00ff88'
  if (memoryUsage.value < settings.criticalUsageThreshold) return '#f0ad4e'
  return '#ff4757'
}

const handleNavigate = (view: string) => {
  currentView.value = view as AppView
}

onMounted(() => {
  startListening()
})
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html {
  height: 100%;
  width: 100%;
  overflow: hidden;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'SF Pro Display', 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  background: #1a1a2e;
  background-image: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f3460 100%);
  min-height: 100vh;
  height: 100%;
  width: 100%;
  color: #e0e0e0;
  overflow: hidden;
}

#app {
  height: 100%;
  width: 100%;
  overflow: hidden;
  background: #1a1a2e;
  background-image: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f3460 100%);
}

.app-shell {
  height: 100%;
}

.reduce-motion *,
.reduce-motion *::before,
.reduce-motion *::after {
  animation-duration: 0.01ms !important;
  animation-iteration-count: 1 !important;
  transition-duration: 0.01ms !important;
  scroll-behavior: auto !important;
}

::-webkit-scrollbar {
  width: 6px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 3px;
}

::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.2);
}
</style>

<style scoped>
.dashboard-view {
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
  margin-bottom: 32px;
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

.dashboard-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 20px;
  margin-bottom: 32px;
}

.overview-card {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 16px;
  padding: 24px;
  transition: all 0.2s ease;
  display: flex;
  flex-direction: column;
}

.overview-card:hover {
  background: rgba(255, 255, 255, 0.06);
  border-color: rgba(255, 255, 255, 0.12);
  transform: translateY(-2px);
}

.card-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
}

.card-icon {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
}

.card-icon.memory {
  background: linear-gradient(135deg, #007AFF 0%, #5856D6 100%);
}

.card-icon.junk {
  background: linear-gradient(135deg, #FF9500 0%, #FF3B30 100%);
}

.card-info {
  display: flex;
  flex-direction: column;
}

.card-title {
  font-size: 16px;
  font-weight: 600;
  color: #ffffff;
}

.card-subtitle {
  font-size: 12px;
  color: #8892b0;
}

.card-stat {
  display: flex;
  align-items: baseline;
  gap: 8px;
  margin-bottom: 16px;
}

.stat-value {
  font-size: 36px;
  font-weight: 700;
  color: #ffffff;
}

.stat-label {
  font-size: 14px;
  color: #8892b0;
}

.card-placeholder {
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  color: #666666;
  background: rgba(255, 255, 255, 0.02);
  border-radius: 8px;
  margin-bottom: 16px;
}

.card-action {
  width: 100%;
  margin-top: auto;
}

.quick-actions {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 16px;
  padding: 24px;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: #ffffff;
  margin-bottom: 16px;
}

.action-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 12px;
}

.action-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.15s ease;
  font-size: 14px;
  color: #8892b0;
}

.action-item:hover {
  background: rgba(0, 122, 255, 0.1);
  border-color: rgba(0, 122, 255, 0.3);
  color: #007AFF;
}

.action-item :deep(svg) {
  width: 20px;
  height: 20px;
}
</style>
