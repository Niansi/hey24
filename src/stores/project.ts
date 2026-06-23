import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
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
          thumbnailUrl: '', // Tauri will resolve via asset protocol
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
    },

    async detectFaces() {
      const pendingPhotos = this.photos.filter(
        (p) => p.status === 'pending' || p.status === 'analyzing'
      );
      if (pendingPhotos.length === 0) return;

      const photoPaths = pendingPhotos.map((p) => p.filePath);

      for (const photo of pendingPhotos) {
        const found = this.photos.find((p) => p.id === photo.id);
        if (found) found.status = 'analyzing';
      }

      const results = await invoke<[string, FaceBox[]][]>('detect_faces', {
        photoPaths,
      });

      for (const [path, faces] of results) {
        const photo = this.photos.find((p) => p.filePath === path);
        if (!photo) continue;

        photo.faces = faces;
        photo.status = 'done';

        if (faces.length === 0) {
          photo.status = 'no-face';
          photo.selectedFaceIndex = null;
        } else if (faces.length === 1) {
          photo.selectedFaceIndex = 0;
        } else {
          // Multiple faces: leave selectedFaceIndex unset for user to choose
        }
      }
    },

    selectFace(photoId: string, faceIndex: number) {
      const photo = this.photos.find((p) => p.id === photoId);
      if (photo) {
        photo.selectedFaceIndex = faceIndex;
      }
    },

    async generateVideo() {
      this.isProcessing = true;
      this.statusMessage = '正在对齐照片...';

      try {
        const alignResult = await invoke<string>('align_photos', {
          photos: this.photos,
        });

        this.statusMessage = '正在渲染视频...';

        const outputPath = await invoke<string>('render_video', {
          template: this.currentTemplate,
          outputSettings: this.outputSettings,
          alignedData: alignResult,
        });

        this.outputVideoPath = outputPath;
        this.statusMessage = '视频生成完成！';
      } catch (error) {
        this.statusMessage = '生成失败: ' + (error as Error).message;
        throw error;
      } finally {
        this.isProcessing = false;
      }
    },
  },
});
