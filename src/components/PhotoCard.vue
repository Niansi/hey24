<script setup lang="ts">
import type { PhotoItem } from '../types';

const props = defineProps<{
  photo: PhotoItem;
}>();

const emit = defineEmits<{
  remove: [id: string];
}>();

function statusIcon(status: PhotoItem['status']): string {
  switch (status) {
    case 'analyzing': return '⏳';
    case 'no-face': return '⚠️';
    case 'done': {
      if (props.photo.faces.length > 1 && props.photo.selectedFaceIndex === null) {
        return '👥';
      }
      return '✅';
    }
    default: return '';
  }
}

function borderColor(status: PhotoItem['status']): string {
  switch (status) {
    case 'no-face': return '#e9a545';
    case 'done': return '#45e960';
    default: return 'transparent';
  }
}
</script>

<template>
  <div
    class="photo-card"
    :style="{ borderColor: borderColor(photo.status) }"
  >
    <div class="thumbnail-wrapper">
      <div
        class="thumbnail"
        :style="{ backgroundImage: `url(${photo.thumbnailUrl || photo.filePath})` }"
      ></div>
      <span class="status-overlay">{{ statusIcon(photo.status) }}</span>
      <button class="remove-btn" title="移除" @click="emit('remove', photo.id)">&times;</button>
    </div>
    <div class="photo-name">{{ photo.fileName }}</div>
  </div>
</template>

<style scoped>
.photo-card {
  width: 80px;
  min-width: 80px;
  border: 2px solid transparent;
  border-radius: var(--radius);
  overflow: hidden;
  background: var(--bg-card);
  transition: border-color 0.2s;
}

.thumbnail-wrapper {
  width: 80px;
  height: 105px;
  position: relative;
  overflow: hidden;
}

.thumbnail {
  width: 100%;
  height: 100%;
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;
}

.status-overlay {
  position: absolute;
  top: 4px;
  left: 4px;
  font-size: 16px;
  line-height: 1;
  filter: drop-shadow(0 0 2px rgba(0,0,0,0.6));
}

.remove-btn {
  position: absolute;
  top: 2px;
  right: 2px;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: rgba(0,0,0,0.6);
  color: white;
  border: none;
  font-size: 14px;
  line-height: 1;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.2s;
}

.photo-card:hover .remove-btn {
  opacity: 1;
}

.photo-name {
  padding: 4px 6px;
  font-size: 10px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  text-align: center;
}
</style>
