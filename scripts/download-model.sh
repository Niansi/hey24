#!/bin/bash
set -euo pipefail

MODEL_DIR="src-tauri/models"
MODEL_FILE="$MODEL_DIR/yunet.onnx"
MODEL_URL="https://github.com/opencv/opencv_zoo/raw/main/models/face_detection_yunet/face_detection_yunet_2023mar.onnx"

mkdir -p "$MODEL_DIR"

if [ ! -f "$MODEL_FILE" ]; then
  echo "Downloading YuNet ONNX model..."
  curl -L -o "$MODEL_FILE" "$MODEL_URL"
  echo "Done: $MODEL_FILE"
else
  echo "Model already exists: $MODEL_FILE"
fi
