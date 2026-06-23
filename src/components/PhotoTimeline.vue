<script setup lang="ts">
import { useProjectStore } from '../stores/project';
import PhotoCard from './PhotoCard.vue';

const store = useProjectStore();

function handleRemove(id: string) {
  store.removePhoto(id);
}
</script>

<template>
  <section class="timeline">
    <header class="timeline-header">
      <h2 class="timeline-title">照片 ({{ store.photoCount }})</h2>
      <div class="timeline-actions">
        <button class="btn-secondary" disabled title="添加照片（功能待接入）">+ 添加照片</button>
        <button
          class="btn-primary"
          :disabled="store.photoCount === 0 || store.isProcessing"
          @click="store.detectFaces()"
        >
          分析人脸
        </button>
      </div>
    </header>

    <div class="photo-strip" v-if="store.photoCount > 0">
      <PhotoCard
        v-for="photo in store.photos"
        :key="photo.id"
        :photo="photo"
        @remove="handleRemove"
      />
    </div>

    <div class="empty-state" v-else>
      <span class="empty-hint">拖入照片或点击「+ 添加照片」开始</span>
    </div>
  </section>
</template>

<style scoped>
.timeline {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 16px;
  overflow: hidden;
}

.timeline-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
}

.timeline-title {
  font-size: 16px;
  font-weight: 600;
}

.timeline-actions {
  display: flex;
  gap: 8px;
}

.photo-strip {
  display: flex;
  gap: 8px;
  overflow-x: auto;
  padding-bottom: 8px;
  flex: 1;
  align-content: flex-start;
}

.photo-strip::-webkit-scrollbar {
  height: 6px;
}

.photo-strip::-webkit-scrollbar-thumb {
  background: var(--border);
  border-radius: 3px;
}

.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 2px dashed var(--border);
  border-radius: var(--radius);
  margin: 0;
}

.empty-hint {
  color: var(--text-secondary);
  font-size: 14px;
}
</style>
