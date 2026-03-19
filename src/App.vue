<template>
  <n-config-provider :theme="darkTheme">
    <n-message-provider>
      <n-dialog-provider>
        <div class="app-container">
          <header class="app-header">
            <div class="header-content">
              <h1>🧹 macOS Cleaner</h1>
              <p class="subtitle">系统清理与优化工具</p>
            </div>
          </header>

          <main class="main-content">
            <n-tabs type="segment" animated placement="top" v-model:value="activeTab">
              <n-tab-pane name="memory" tab="💾 内存清理">
                <MemoryCard />
              </n-tab-pane>

              <n-tab-pane name="junk" tab="🗑️ 垃圾清理">
                <JunkCleanerCard />
              </n-tab-pane>
            </n-tabs>
          </main>

          <footer class="app-footer">
            <p>macOS Cleaner v1.0.0</p>
          </footer>
        </div>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { darkTheme, NConfigProvider, NTabs, NTabPane, NMessageProvider, NDialogProvider } from 'naive-ui'
import MemoryCard from './components/MemoryCard.vue'
import JunkCleanerCard from './components/JunkCleanerCard.vue'

const activeTab = ref<'memory' | 'junk'>('memory')
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  height: 100%;
  width: 100%;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f3460 100%);
  min-height: 100vh;
  color: #e0e0e0;
  overflow-x: hidden;
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
</style>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
  max-width: 1000px;
  margin: 0 auto;
  padding: 20px;
}

.app-header {
  text-align: center;
  padding: 30px 0;
  margin-bottom: 20px;
}

.header-content h1 {
  font-size: 2.8rem;
  font-weight: 800;
  background: linear-gradient(135deg, #00d9ff, #00ff88);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  margin-bottom: 8px;
  letter-spacing: -1px;
}

.subtitle {
  font-size: 1.1rem;
  color: #8892b0;
  letter-spacing: 2px;
}

.main-content {
  flex: 1;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 20px;
  padding: 24px;
  border: 1px solid rgba(255, 255, 255, 0.08);
  backdrop-filter: blur(10px);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
}

.app-footer {
  text-align: center;
  padding: 20px 0;
  color: #5a6a8a;
  font-size: 0.85rem;
}

:deep(.n-tabs) {
  height: 100%;
}

:deep(.n-tab-pane) {
  padding: 20px 0;
}

:deep(.n-tabs-tab) {
  font-size: 1rem;
  padding: 12px 24px;
}

:deep(.n-card) {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
}
</style>