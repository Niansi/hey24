<script setup lang="ts">
import { useProjectStore } from '../stores/project';

const store = useProjectStore();
</script>

<template>
  <aside class="sidebar sidebar-right">
    <h3 class="preview-title">预览</h3>

    <div class="preview-area">
      <div class="preview-frame">
        <video
          v-if="store.outputVideoPath"
          :src="'asset://localhost/' + store.outputVideoPath"
          controls
          class="preview-video"
        ></video>
        <div v-else class="preview-placeholder">
          <span>上传照片后点击生成</span>
        </div>
      </div>
    </div>

    <button
      class="btn-primary generate-btn"
      :disabled="store.photoCount === 0 || store.isProcessing"
      @click="store.generateVideo()"
    >
      一键生成
    </button>
  </aside>
</template>

<style scoped>
.sidebar-right {
  width: 280px;
  min-width: 280px;
  background: var(--bg-secondary);
  border-left: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  padding: 16px;
}

.preview-title {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 12px;
}

.preview-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  overflow: hidden;
}

.preview-frame {
  aspect-ratio: 9 / 16;
  max-height: 60vh;
  width: 100%;
  background: #000;
  border-radius: var(--radius);
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
}

.preview-video {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.preview-placeholder {
  color: var(--text-secondary);
  font-size: 14px;
  text-align: center;
  padding: 16px;
}

.generate-btn {
  width: 100%;
  padding: 12px;
  font-size: 16px;
  margin-top: 12px;
}
</style>
