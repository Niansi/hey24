<script setup lang="ts">
import { onMounted } from 'vue';
import TemplateList from './components/TemplateList.vue';
import PhotoTimeline from './components/PhotoTimeline.vue';
import PreviewPanel from './components/PreviewPanel.vue';
import StatusBar from './components/StatusBar.vue';
import { useProjectStore } from './stores/project';

const store = useProjectStore();

onMounted(async () => {
  await store.loadTemplate('silent-grow');
  await store.checkSystem();
});
</script>

<template>
  <div class="app-layout">
    <header class="app-header">
      <h1 class="app-title">Hey24</h1>
      <span class="template-badge" v-if="store.currentTemplate">
        模板: {{ store.currentTemplate.name }}
      </span>
    </header>
    <main class="app-main">
      <TemplateList />
      <PhotoTimeline />
      <PreviewPanel />
    </main>
    <StatusBar />
  </div>
</template>

<style scoped>
.app-layout { display: flex; flex-direction: column; height: 100vh; }
.app-header {
  display: flex; align-items: center; gap: 16px;
  padding: 12px 20px; background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
  -webkit-app-region: drag;
}
.app-title { font-size: 18px; font-weight: 700; }
.template-badge {
  font-size: 12px; color: var(--text-secondary);
  background: var(--bg-card); padding: 2px 10px; border-radius: 12px;
}
.app-main { display: flex; flex: 1; overflow: hidden; }
</style>
