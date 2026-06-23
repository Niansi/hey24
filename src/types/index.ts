export interface FaceBox {
  bbox: [number, number, number, number];
  confidence: number;
  landmarks: [[number, number], [number, number], [number, number], [number, number], [number, number]];
}

export interface PhotoItem {
  id: string;
  filePath: string;
  fileName: string;
  thumbnailUrl: string;
  faces: FaceBox[];
  selectedFaceIndex: number | null;
  status: 'pending' | 'analyzing' | 'done' | 'no-face';
}

export interface TemplateInfo {
  name: string;
  id: string;
  version: number;
  output: {
    default: {
      width: number;
      height: number;
      fps: number;
      duration_per_photo: number;
    };
  };
}

export interface OutputSettings {
  width: number;
  height: number;
  fps: number;
  durationPerPhoto: number;
  bgmPath: string | null;
}
