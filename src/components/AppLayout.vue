<template>
  <div class="app-layout">
    <Sidebar :activeItem="currentView" @navigate="handleNavigate" />
    <main class="main-content">
      <div class="content-wrapper">
        <slot />
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import Sidebar from './Sidebar.vue'

const currentView = ref('dashboard')

const emit = defineEmits<{
  navigate: [view: string]
}>()

const handleNavigate = (view: string) => {
  currentView.value = view
  emit('navigate', view)
}
</script>

<style scoped>
.app-layout {
  display: flex;
  height: 100vh;
  width: 100%;
  overflow: hidden;
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f3460 100%);
}

.main-content {
  flex: 1;
  margin-left: 220px;
  height: 100vh;
  width: calc(100% - 220px);
  overflow-y: auto;
  overflow-x: hidden;
}

.content-wrapper {
  padding: 32px;
  max-width: 900px;
  margin: 0 auto;
  animation: fadeIn 0.3s ease;
  box-sizing: border-box;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
</style>
