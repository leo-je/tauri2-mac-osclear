<template>
  <div class="settings-view">
    <div class="page-header">
      <div class="header-copy">
        <span class="eyebrow">Preferences</span>
        <h1 class="page-title">设置</h1>
        <p class="page-subtitle">自定义启动方式、清理习惯和界面体验，设置会自动保存在本机。</p>
      </div>

      <div class="header-actions">
        <n-button quaternary @click="refreshSystemInfo" :loading="isLoadingSystemInfo">
          刷新设备信息
        </n-button>
        <n-button type="default" @click="handleReset">
          恢复默认
        </n-button>
      </div>
    </div>

    <div class="settings-hero">
      <div class="hero-card primary">
        <span class="hero-label">默认启动页</span>
        <strong class="hero-value">{{ startupViewLabel }}</strong>
        <p class="hero-description">下次打开应用时，直接进入你最常使用的工作区。</p>
      </div>
      <div class="hero-card">
        <span class="hero-label">清理确认</span>
        <strong class="hero-value">{{ settings.confirmBeforeCleaning ? '已开启' : '已关闭' }}</strong>
        <p class="hero-description">避免误删项目，尤其适合批量清理前再核对一次。</p>
      </div>
      <div class="hero-card accent">
        <span class="hero-label">内存阈值</span>
        <strong class="hero-value">{{ settings.warningUsageThreshold }}% / {{ settings.criticalUsageThreshold }}%</strong>
        <p class="hero-description">决定总览和内存页里“正常 / 注意 / 紧张”的判定区间。</p>
      </div>
    </div>

    <div class="settings-grid">
      <section class="settings-card wide">
        <div class="card-head">
          <div>
            <h2>启动与界面</h2>
            <p>让应用更贴近你的工作习惯。</p>
          </div>
          <span class="card-tag">体验</span>
        </div>

        <div class="setting-list">
          <div class="setting-row">
            <div class="setting-copy">
              <span class="setting-title">默认启动页</span>
              <span class="setting-description">应用启动后首先显示的页面。</span>
            </div>
            <n-select
              class="setting-control select-control"
              :value="settings.startupView"
              :options="viewOptions"
              @update:value="handleStartupViewChange"
            />
          </div>

          <div class="setting-row">
            <div class="setting-copy">
              <span class="setting-title">显示总览页快速操作</span>
              <span class="setting-description">在首页展示常用入口，减少层级切换。</span>
            </div>
            <n-switch
              :value="settings.showDashboardQuickActions"
              @update:value="value => updateSettings({ showDashboardQuickActions: value })"
            />
          </div>

          <div class="setting-row">
            <div class="setting-copy">
              <span class="setting-title">减少动画效果</span>
              <span class="setting-description">关闭多数过渡与动效，适合偏好稳定界面或录屏时使用。</span>
            </div>
            <n-switch
              :value="settings.reduceMotion"
              @update:value="value => updateSettings({ reduceMotion: value })"
            />
          </div>
        </div>
      </section>

      <section class="settings-card wide">
        <div class="card-head">
          <div>
            <h2>扫描目标</h2>
            <p>选择需要扫描的系统目录。</p>
          </div>
          <span class="card-tag">目标</span>
        </div>

        <div class="target-grid">
          <div
            v-for="target in junkTargets"
            :key="target.id"
            class="target-chip"
            :class="{ active: settings.enabledJunkTargets.includes(target.id) }"
            @click="toggleTarget(target.id)"
          >
            <span class="target-label">{{ target.label }}</span>
            <span class="target-desc">{{ target.description }}</span>
          </div>
        </div>
      </section>

      <section class="settings-card">
        <div class="card-head">
          <div>
            <h2>清理偏好</h2>
            <p>控制扫描后的默认行为。</p>
          </div>
          <span class="card-tag">安全</span>
        </div>

        <div class="setting-list">
          <div class="setting-row compact">
            <div class="setting-copy">
              <span class="setting-title">扫描后自动全选</span>
              <span class="setting-description">开启后会默认选中扫描出的项目。</span>
            </div>
            <n-switch
              :value="settings.autoSelectScanResults"
              @update:value="value => updateSettings({ autoSelectScanResults: value })"
            />
          </div>

          <div class="setting-row compact">
            <div class="setting-copy">
              <span class="setting-title">清理前二次确认</span>
              <span class="setting-description">点击清理按钮后先弹窗确认，再执行移入废纸篓。</span>
            </div>
            <n-switch
              :value="settings.confirmBeforeCleaning"
              @update:value="value => updateSettings({ confirmBeforeCleaning: value })"
            />
          </div>

          <div class="setting-row compact">
            <div class="setting-copy">
              <span class="setting-title">清理后重新扫描</span>
              <span class="setting-description">清理完成后自动重新扫描，检查是否还有残留文件。</span>
            </div>
            <n-switch
              :value="settings.rescanAfterCleaning"
              @update:value="value => updateSettings({ rescanAfterCleaning: value })"
            />
          </div>
        </div>
      </section>

      <section class="settings-card">
        <div class="card-head">
          <div>
            <h2>下载文件过滤</h2>
            <p>避免误删有用的下载文件。</p>
          </div>
          <span class="card-tag">策略</span>
        </div>

        <div class="setting-list">
          <div class="setting-row vertical">
            <div class="setting-copy">
              <span class="setting-title">最小文件年龄（天）</span>
              <span class="setting-description">只有超过这个天数的下载文件才会被扫描为垃圾。设为 0 可禁用下载文件扫描。</span>
            </div>
            <div class="threshold-control">
              <n-slider
                class="slider-control"
                :value="settings.downloadsMinAgeDays"
                :min="0"
                :max="180"
                :step="1"
                @update:value="handleDownloadsMinAgeChange"
              />
              <span class="threshold-value">{{ settings.downloadsMinAgeDays }}天</span>
            </div>
          </div>

          <div class="setting-row vertical">
            <div class="setting-copy">
              <span class="setting-title">最小文件大小（MB）</span>
              <span class="setting-description">只有大于这个大小的下载文件才会被考虑清理。</span>
            </div>
            <div class="threshold-control">
              <n-slider
                class="slider-control"
                :value="settings.downloadsMinSizeMB"
                :min="10"
                :max="1000"
                :step="10"
                @update:value="handleDownloadsMinSizeChange"
              />
              <span class="threshold-value">{{ settings.downloadsMinSizeMB }}MB</span>
            </div>
          </div>
        </div>
      </section>

      <section class="settings-card">
        <div class="card-head">
          <div>
            <h2>内存提醒阈值</h2>
            <p>设置“注意”和“紧张”状态的分界线。</p>
          </div>
          <span class="card-tag">监控</span>
        </div>

        <div class="setting-list">
          <div class="setting-row vertical">
            <div class="setting-copy">
              <span class="setting-title">注意阈值</span>
              <span class="setting-description">达到这个使用率后，界面会切换为提醒态。</span>
            </div>
            <div class="threshold-control">
              <n-slider
                class="slider-control"
                :value="settings.warningUsageThreshold"
                :min="50"
                :max="90"
                :step="5"
                @update:value="handleWarningThresholdChange"
              />
              <span class="threshold-value">{{ settings.warningUsageThreshold }}%</span>
            </div>
          </div>

          <div class="setting-row vertical">
            <div class="setting-copy">
              <span class="setting-title">紧张阈值</span>
              <span class="setting-description">超过这个值后，显示为需要重点关注的状态。</span>
            </div>
            <div class="threshold-control">
              <n-slider
                class="slider-control"
                :value="settings.criticalUsageThreshold"
                :min="55"
                :max="95"
                :step="5"
                @update:value="handleCriticalThresholdChange"
              />
              <span class="threshold-value">{{ settings.criticalUsageThreshold }}%</span>
            </div>
          </div>
        </div>
      </section>

      <section class="settings-card wide info-card">
        <div class="card-head">
          <div>
            <h2>设备与应用信息</h2>
            <p>方便确认当前运行环境和系统概况。</p>
          </div>
          <span class="card-tag">About</span>
        </div>

        <div v-if="systemInfo" class="info-grid">
          <div class="info-item">
            <span class="info-label">系统</span>
            <span class="info-value">{{ systemInfo.os_name }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">内核版本</span>
            <span class="info-value">{{ systemInfo.os_version }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">主机名</span>
            <span class="info-value">{{ systemInfo.hostname }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">CPU 核心</span>
            <span class="info-value">{{ systemInfo.cpu_count }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">总内存</span>
            <span class="info-value">{{ formatSize(systemInfo.total_memory) }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">应用版本</span>
            <span class="info-value">v1.0.0</span>
          </div>
        </div>

        <div v-else class="info-empty">
          <span>暂时无法读取设备信息。</span>
          <span class="info-hint">你可以点击上方“刷新设备信息”再试一次。</span>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { NButton, NSelect, NSlider, NSwitch, useMessage } from 'naive-ui'
import { useSettings } from '../composables/useSettings'
import { JUNK_SCAN_TARGETS } from '../constants/junkScanTargets'
import type { AppView, SystemInfo, JunkScanTargetId } from '../types'

const message = useMessage()
const { settings, updateSettings, resetSettings } = useSettings()

const systemInfo = ref<SystemInfo | null>(null)
const isLoadingSystemInfo = ref(false)

const viewOptions = [
  { label: '系统总览', value: 'dashboard' },
  { label: '内存清理', value: 'memory' },
  { label: '垃圾清理', value: 'junk' },
  { label: '设置页', value: 'settings' }
]

const startupViewLabel = computed(() => {
  return viewOptions.find(option => option.value === settings.startupView)?.label ?? '系统总览'
})

const junkTargets = JUNK_SCAN_TARGETS

const toggleTarget = (id: JunkScanTargetId) => {
  const current = [...settings.enabledJunkTargets]
  const index = current.indexOf(id)
  if (index > -1) {
    current.splice(index, 1)
  } else {
    current.push(id)
  }
  updateSettings({ enabledJunkTargets: current })
}

const formatSize = (bytes: number): string => {
  if (bytes === 0) return '0 B'

  const base = 1024
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const unitIndex = Math.floor(Math.log(bytes) / Math.log(base))

  return `${parseFloat((bytes / Math.pow(base, unitIndex)).toFixed(2))} ${units[unitIndex]}`
}

const refreshSystemInfo = async () => {
  isLoadingSystemInfo.value = true

  try {
    systemInfo.value = await invoke<SystemInfo>('get_system_info')
  } catch (error) {
    message.error('读取设备信息失败: ' + String(error))
  } finally {
    isLoadingSystemInfo.value = false
  }
}

const handleStartupViewChange = (value: string | null) => {
  if (!value) return
  updateSettings({ startupView: value as AppView })
}

const handleWarningThresholdChange = (value: number) => {
  updateSettings({ warningUsageThreshold: value })
}

const handleCriticalThresholdChange = (value: number) => {
  updateSettings({ criticalUsageThreshold: value })
}

const handleDownloadsMinAgeChange = (value: number) => {
  updateSettings({ downloadsMinAgeDays: value })
}

const handleDownloadsMinSizeChange = (value: number) => {
  updateSettings({ downloadsMinSizeMB: value })
}

const handleReset = () => {
  resetSettings()
  message.success('已恢复默认设置')
}

onMounted(() => {
  refreshSystemInfo()
})
</script>

<style scoped>
.settings-view {
  animation: slideIn 0.32s ease;
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
  gap: 20px;
  align-items: flex-start;
  margin-bottom: 28px;
}

.header-copy {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-width: 620px;
}

.eyebrow {
  font-size: 12px;
  letter-spacing: 0.18em;
  text-transform: uppercase;
  color: rgba(120, 198, 255, 0.72);
}

.page-title {
  font-size: 30px;
  font-weight: 700;
  color: #ffffff;
  letter-spacing: -0.04em;
}

.page-subtitle {
  font-size: 14px;
  line-height: 1.7;
  color: #9aacd0;
}

.header-actions {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
  justify-content: flex-end;
}

.settings-hero {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 16px;
  margin-bottom: 20px;
}

.hero-card {
  position: relative;
  overflow: hidden;
  padding: 20px;
  border-radius: 20px;
  background:
    radial-gradient(circle at top right, rgba(255, 255, 255, 0.14), transparent 42%),
    rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: 0 20px 40px rgba(5, 12, 32, 0.22);
}

.hero-card.primary {
  background:
    radial-gradient(circle at top right, rgba(64, 190, 255, 0.38), transparent 40%),
    linear-gradient(140deg, rgba(14, 84, 180, 0.36), rgba(17, 33, 72, 0.74));
}

.hero-card.accent {
  background:
    radial-gradient(circle at top right, rgba(255, 179, 71, 0.34), transparent 42%),
    linear-gradient(140deg, rgba(85, 43, 18, 0.64), rgba(23, 25, 54, 0.8));
}

.hero-label {
  display: block;
  margin-bottom: 12px;
  font-size: 12px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: rgba(255, 255, 255, 0.66);
}

.hero-value {
  display: block;
  margin-bottom: 10px;
  font-size: 24px;
  font-weight: 700;
  color: #ffffff;
}

.hero-description {
  font-size: 13px;
  line-height: 1.6;
  color: rgba(227, 235, 255, 0.78);
}

.settings-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
}

.settings-card {
  padding: 22px;
  border-radius: 22px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.08);
  box-shadow: 0 16px 40px rgba(6, 14, 30, 0.22);
}

.settings-card.wide {
  grid-column: span 2;
}

.card-head {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  align-items: flex-start;
  margin-bottom: 18px;
}

.card-head h2 {
  font-size: 18px;
  font-weight: 700;
  color: #ffffff;
  margin-bottom: 6px;
}

.card-head p {
  font-size: 13px;
  color: #8fa0c0;
  line-height: 1.6;
}

.card-tag {
  flex-shrink: 0;
  padding: 6px 10px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.08);
  color: #b4c3df;
  font-size: 11px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.setting-list {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  padding: 16px 18px;
  border-radius: 16px;
  background: rgba(7, 13, 28, 0.34);
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.setting-row.vertical {
  align-items: stretch;
  flex-direction: column;
  gap: 14px;
}

.setting-copy {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.setting-title {
  font-size: 14px;
  font-weight: 600;
  color: #ffffff;
}

.setting-description {
  font-size: 12px;
  line-height: 1.6;
  color: #90a1c0;
}

.setting-control {
  flex-shrink: 0;
}

.select-control {
  width: 180px;
}

.threshold-control {
  display: flex;
  align-items: center;
  gap: 14px;
}

.slider-control {
  flex: 1;
}

.threshold-value {
  min-width: 52px;
  text-align: right;
  font-size: 15px;
  font-weight: 700;
  color: #7bc4ff;
}

.target-grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 10px;
}

.target-chip {
  padding: 12px 14px;
  border-radius: 14px;
  background: rgba(7, 13, 28, 0.34);
  border: 1px solid rgba(255, 255, 255, 0.05);
  cursor: pointer;
  transition: all 0.2s ease;
}

.target-chip:hover {
  background: rgba(255, 255, 255, 0.08);
}

.target-chip.active {
  background: rgba(64, 190, 255, 0.15);
  border-color: rgba(64, 190, 255, 0.4);
}

.target-label {
  display: block;
  font-size: 13px;
  font-weight: 600;
  color: #ffffff;
  margin-bottom: 4px;
}

.target-desc {
  display: block;
  font-size: 11px;
  line-height: 1.5;
  color: #90a1c0;
}

.target-chip.active .target-label {
  color: #7bc4ff;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 12px;
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 16px;
  border-radius: 16px;
  background: rgba(7, 13, 28, 0.34);
  border: 1px solid rgba(255, 255, 255, 0.05);
}

.info-label {
  font-size: 11px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: #7f90b1;
}

.info-value {
  font-size: 14px;
  font-weight: 600;
  color: #f4f7ff;
  word-break: break-word;
}

.info-empty {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 18px;
  border-radius: 16px;
  background: rgba(7, 13, 28, 0.34);
  color: #dfe6f6;
}

.info-hint {
  font-size: 12px;
  color: #90a1c0;
}

@media (max-width: 860px) {
  .page-header {
    flex-direction: column;
  }

  .header-actions {
    justify-content: flex-start;
  }

  .settings-hero,
  .settings-grid,
  .info-grid,
  .target-grid {
    grid-template-columns: 1fr;
  }

  .settings-card.wide {
    grid-column: span 1;
  }

  .setting-row {
    flex-direction: column;
    align-items: flex-start;
  }

  .select-control,
  .threshold-control {
    width: 100%;
  }
}
</style>
