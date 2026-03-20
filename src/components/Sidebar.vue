<template>
  <nav class="sidebar">
    <div class="sidebar-header">
      <div class="app-logo">
        <img src="/icons/logo.png" alt="macOS Cleaner" class="logo-img" />
      </div>
      <div class="app-info">
        <span class="app-name">macOS Cleaner</span>
        <span class="app-version">v1.0.0</span>
      </div>
    </div>

    <div class="nav-section">
      <div class="nav-label">功能</div>
      <ul class="nav-list">
        <li
          v-for="item in navItems"
          :key="item.id"
          class="nav-item"
          :class="{ active: activeItem === item.id }"
          @click="$emit('navigate', item.id)"
        >
          <span class="nav-icon">
            <component :is="item.icon" />
          </span>
          <span class="nav-text">{{ item.label }}</span>
        </li>
      </ul>
    </div>

    <div class="nav-section">
      <div class="nav-label">其他</div>
      <ul class="nav-list">
        <li
          v-for="item in secondaryItems"
          :key="item.id"
          class="nav-item"
          :class="{ active: activeItem === item.id }"
          @click="$emit('navigate', item.id)"
        >
          <span class="nav-icon">
            <component :is="item.icon" />
          </span>
          <span class="nav-text">{{ item.label }}</span>
        </li>
      </ul>
    </div>

    <div class="sidebar-footer">
      <div class="system-info">
        <div class="info-row">
          <span class="info-label">操作系统</span>
          <span class="info-value">macOS</span>
        </div>
      </div>
    </div>
  </nav>
</template>

<script setup lang="ts">
import { markRaw } from 'vue'
import DashboardIcon from './icons/DashboardIcon.vue'
import MemoryIcon from './icons/MemoryIcon.vue'
import TrashIcon from './icons/TrashIcon.vue'
import SettingsIcon from './icons/SettingsIcon.vue'

interface Props {
  activeItem: string
}

defineProps<Props>()
defineEmits<{
  navigate: [item: string]
}>()

const navItems = [
  { id: 'dashboard', label: '总览', icon: markRaw(DashboardIcon) },
  { id: 'memory', label: '内存清理', icon: markRaw(MemoryIcon) },
  { id: 'junk', label: '垃圾清理', icon: markRaw(TrashIcon) }
]

const secondaryItems = [
  { id: 'settings', label: '设置', icon: markRaw(SettingsIcon) }
]
</script>

<style scoped>
.sidebar {
  width: 220px;
  height: 100vh;
  background: rgba(30, 30, 30, 0.85);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-right: 1px solid rgba(255, 255, 255, 0.08);
  display: flex;
  flex-direction: column;
  padding: 20px 12px;
  position: fixed;
  left: 0;
  top: 0;
  z-index: 100;
}

.sidebar-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0 8px 20px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  margin-bottom: 20px;
}

.app-logo {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  overflow: hidden;
}

.logo-img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.app-info {
  display: flex;
  flex-direction: column;
}

.app-name {
  font-size: 14px;
  font-weight: 600;
  color: #ffffff;
}

.app-version {
  font-size: 11px;
  color: #666666;
}

.nav-section {
  margin-bottom: 24px;
}

.nav-label {
  font-size: 11px;
  font-weight: 600;
  color: #666666;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  padding: 0 12px 8px;
}

.nav-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s ease;
  color: #8892b0;
  font-size: 13px;
  font-weight: 500;
  position: relative;
}

.nav-item:hover {
  background: rgba(255, 255, 255, 0.06);
  color: #ffffff;
}

.nav-item.active {
  background: rgba(0, 122, 255, 0.15);
  color: #007AFF;
}

.nav-item.active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 20px;
  background: #007AFF;
  border-radius: 0 3px 3px 0;
}

.nav-icon {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.nav-icon :deep(svg) {
  width: 18px;
  height: 18px;
}

.nav-text {
  flex: 1;
}

.sidebar-footer {
  margin-top: auto;
  padding-top: 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
}

.system-info {
  padding: 0 12px;
}

.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
}

.info-label {
  color: #666666;
}

.info-value {
  color: #8892b0;
  font-weight: 500;
}
</style>
