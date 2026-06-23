<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { useProjectStore } from '../stores/project';
import { open } from '@tauri-apps/plugin-dialog';
import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import PhotoCard from './PhotoCard.vue';
import FaceSelector from './FaceSelector.vue';
import type { PhotoItem } from '../types';

const store = useProjectStore();
const selectedPhotoForFaces = ref<PhotoItem | null>(null);

let unlisten: UnlistenFn | null = null;

async function handleFileSelect() {
  try {
    const selected = await open({
      multiple: true,
      filters: [{
        name: 'Images',
        extensions: ['png', 'jpg', 'jpeg', 'heic', 'webp'],
      }],
    });
    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      store.addPhotos(paths);
    }
  } catch (err) {
    console.error('File dialog failed:', err);
  }
}

function handlePhotoClick(photo: PhotoItem) {
  if (photo.faces.length > 1) {
    selectedPhotoForFaces.value = photo;
  }
}

function handleFaceSelect(faceIndex: number) {
  if (selectedPhotoForFaces.value) {
    store.selectFace(selectedPhotoForFaces.value.id, faceIndex);
    selectedPhotoForFaces.value = null;
  }
}

function handleRemove(id: string) {
  store.removePhoto(id);
}

onMounted(async () => {
  unlisten = await listen<string[]>('tauri://drag-drop', (event) => {
    const paths = event.payload;
    const imagePaths = paths.filter(p =>
      /\.(png|jpe?g|heic|webp)$/i.test(p)
    );
    if (imagePaths.length > 0) {
      store.addPhotos(imagePaths);
    }
  });
});

onUnmounted(() => {
  unlisten?.();
});
</script>

<template>
  <section class="timeline">
    <header class="timeline-header">
      <h2 class="timeline-title">照片 ({{ store.photoCount }})</h2>
      <div class="timeline-actions">
        <button class="btn-secondary" @click="handleFileSelect">+ 添加照片</button>
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
        @click="handlePhotoClick(photo)"
        @remove="handleRemove"
      />
    </div>

    <div class="empty-state" v-else>
      <span class="empty-hint">拖入照片或点击「+ 添加照片」开始</span>
    </div>

    <FaceSelector
      v-if="selectedPhotoForFaces"
      :photo-url="selectedPhotoForFaces.thumbnailUrl"
      :faces="selectedPhotoForFaces.faces"
      @select="handleFaceSelect"
      @close="selectedPhotoForFaces = null"
    />
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
