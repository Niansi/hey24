import { defineStore } from 'pinia';
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import type { PhotoItem, TemplateInfo, OutputSettings, FaceBox } from '../types';

function generateId(): string {
  return Math.random().toString(36).substring(2, 11);
}

export const useProjectStore = defineStore('project', {
  state: () => ({
    photos: [] as PhotoItem[],
    currentTemplate: null as TemplateInfo | null,
    outputSettings: {
      width: 1080,
      height: 1920,
      fps: 5,
      durationPerPhoto: 0.2,
      bgmPath: null,
    } as OutputSettings,
    statusMessage: '就绪' as string,
    isProcessing: false as boolean,
    outputVideoPath: null as string | null,
  }),

  getters: {
    photoCount: (state) => state.photos.length,
  },

  actions: {
    async addPhotos(filePaths: string[]) {
      for (const filePath of filePaths) {
        const fileName = filePath.split('/').pop() || filePath.split('\\').pop() || filePath;
        const photo: PhotoItem = {
          id: generateId(),
          filePath,
          fileName,
          thumbnailUrl: convertFileSrc(filePath),
          faces: [],
          selectedFaceIndex: null,
          status: 'pending',
        };
        this.photos.push(photo);
      }
    },

    removePhoto(id: string) {
      this.photos = this.photos.filter((p) => p.id !== id);
    },

    reorderPhotos(fromIndex: number, toIndex: number) {
      const item = this.photos.splice(fromIndex, 1)[0];
      if (item) {
        this.photos.splice(toIndex, 0, item);
      }
    },

    async loadTemplate(templateId: string) {
      try {
        const template = await invoke<TemplateInfo>('load_template', {
          templateId,
        });
        this.currentTemplate = template;
        this.outputSettings = {
          width: template.output.default.width,
          height: template.output.default.height,
          fps: template.output.default.fps,
          durationPerPhoto: template.output.default.duration_per_photo,
          bgmPath: null,
        };
      } catch (err) {
        this.statusMessage = '⚠️ 加载模板失败: ' + String(err);
      }
    },

    async detectFaces() {
      const pendingPhotos = this.photos.filter(
        (p) => p.status === 'pending'
      );
      if (pendingPhotos.length === 0) return;

      this.isProcessing = true;
      this.statusMessage = '正在分析人脸...';

      // Set status to analyzing BEFORE invoke
      for (const photo of pendingPhotos) {
        const found = this.photos.find((p) => p.id === photo.id);
        if (found) found.status = 'analyzing';
      }

      try {
        const photoPaths = pendingPhotos.map((p) => p.filePath);
        // detect_faces returns Vec<DetectFacesResult> = { photo_index, faces }[]
        const results = await invoke<{ photo_index: number; faces: FaceBox[] }[]>('detect_faces', {
          photoPaths,
        });

        for (const result of results) {
          const photo = pendingPhotos[result.photo_index];
          if (!photo) continue;

          // Find the actual photo in this.photos to update
          const storePhoto = this.photos.find((p) => p.id === photo.id);
          if (!storePhoto) continue;

          storePhoto.faces = result.faces;
          storePhoto.status = 'done';

          if (result.faces.length === 0) {
            storePhoto.status = 'no-face';
            storePhoto.selectedFaceIndex = null;
          } else if (result.faces.length === 1) {
            storePhoto.selectedFaceIndex = 0;
          }
          // Multiple faces: leave selectedFaceIndex unset for user to choose
        }

        this.statusMessage = `人脸分析完成（${results.length} 张）`;
      } catch (err) {
        // Reset status on failure so user can retry
        for (const photo of pendingPhotos) {
          const storePhoto = this.photos.find((p) => p.id === photo.id);
          if (storePhoto) storePhoto.status = 'pending';
        }
        this.statusMessage = '分析失败: ' + String(err);
      } finally {
        this.isProcessing = false;
      }
    },

    selectFace(photoId: string, faceIndex: number) {
      const photo = this.photos.find((p) => p.id === photoId);
      if (photo) {
        photo.selectedFaceIndex = faceIndex;
      }
    },

    async checkSystem() {
      try {
        const version = await invoke<string>('check_system');
        this.statusMessage = `FFmpeg: ${version.split('\n')[0]}`;
      } catch (err) {
        this.statusMessage = `⚠️ ${err}`;
      }
    },

    async generateVideo() {
      this.isProcessing = true;
      this.statusMessage = '正在对齐照片...';

      try {
        // align_photos expects AlignPhotosInput { photo_paths, selected_faces, template_id }
        const alignedPaths = await invoke<string[]>('align_photos', {
          photoPaths: this.photos.map((p) => p.filePath),
          selectedFaces: this.photos.map((p) => p.selectedFaceIndex),
          templateId: this.currentTemplate?.id || 'silent-grow',
        });

        this.statusMessage = '正在渲染视频...';

        // render_video expects RenderVideoInput { aligned_photo_paths, output_path, template_id, bgm_path }
        const outputPath = await invoke<string>('render_video', {
          alignedPhotoPaths: alignedPaths,
          outputPath: '/tmp/hey24-output.mp4',
          templateId: this.currentTemplate?.id || 'silent-grow',
          bgmPath: this.outputSettings.bgmPath,
        });

        this.outputVideoPath = convertFileSrc(outputPath);
        this.statusMessage = '视频生成完成！';
      } catch (err) {
        this.statusMessage = '生成失败: ' + String(err);
      } finally {
        this.isProcessing = false;
      }
    },
  },
});
