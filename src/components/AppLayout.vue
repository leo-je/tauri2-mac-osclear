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
  min-height: 100vh;
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 50%, #0f3460 100%);
}

.main-content {
  flex: 1;
  margin-left: 220px;
  min-height: 100vh;
}

.content-wrapper {
  padding: 32px;
  max-width: 900px;
  margin: 0 auto;
  animation: fadeIn 0.3s ease;
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
