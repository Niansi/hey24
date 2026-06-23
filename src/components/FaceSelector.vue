<script setup lang="ts">
import type { FaceBox } from '../types';

defineProps<{
  photoUrl: string;
  faces: FaceBox[];
}>();

const emit = defineEmits<{
  select: [faceIndex: number];
  close: [];
}>();

function handleBackdropClick(event: MouseEvent) {
  if ((event.target as HTMLElement).classList.contains('backdrop')) {
    emit('close');
  }
}
</script>

<template>
  <div class="backdrop" @click="handleBackdropClick">
    <div class="modal">
      <button class="close-btn" @click="emit('close')">&times;</button>
      <div class="image-container">
        <img :src="photoUrl" alt="请选择人脸" class="face-image" />
        <div
          v-for="(face, index) in faces"
          :key="index"
          class="face-box"
          :style="{
            left: (face.bbox[0] * 100) + '%',
            top: (face.bbox[1] * 100) + '%',
            width: (face.bbox[2] * 100) + '%',
            height: (face.bbox[3] * 100) + '%',
          }"
          @click="emit('select', index)"
        >
          <span class="face-label">{{ index + 1 }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.modal {
  position: relative;
  max-width: 80vw;
  max-height: 80vh;
  background: var(--bg-primary);
  border-radius: var(--radius);
  padding: 16px;
  overflow: hidden;
}

.close-btn {
  position: absolute;
  top: 8px;
  right: 8px;
  z-index: 10;
  width: 28px;
  height: 28px;
  border-radius: 50%;
  background: rgba(0, 0, 0, 0.6);
  color: white;
  border: none;
  font-size: 18px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.image-container {
  position: relative;
  display: inline-block;
  max-width: 100%;
  max-height: 75vh;
}

.face-image {
  display: block;
  max-width: 100%;
  max-height: 75vh;
  object-fit: contain;
}

.face-box {
  position: absolute;
  border: 2px solid var(--accent);
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.2s;
  display: flex;
  align-items: flex-end;
  justify-content: flex-start;
}

.face-box:hover {
  background: rgba(233, 69, 96, 0.25);
}

.face-label {
  font-size: 12px;
  font-weight: 700;
  color: white;
  background: var(--accent);
  padding: 1px 5px;
  border-radius: 2px;
  margin: 2px;
}
</style>
