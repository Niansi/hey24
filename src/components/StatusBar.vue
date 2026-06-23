<script setup lang="ts">
import { useProjectStore } from '../stores/project';

const store = useProjectStore();
</script>

<template>
  <footer class="status-bar">
    <span class="status-message">{{ store.statusMessage }}</span>
    <span class="status-right">
      <span v-if="store.isProcessing" class="spinner"></span>
    </span>
  </footer>
</template>

<style scoped>
.status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 16px;
  background: var(--bg-secondary);
  border-top: 1px solid var(--border);
  font-size: 12px;
  color: var(--text-secondary);
}

.status-right {
  display: flex;
  align-items: center;
}

.spinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--border);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.6s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
