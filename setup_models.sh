#!/bin/bash

set -e

MODEL_NAME="sherpa-onnx-streaming-paraformer-bilingual-zh-en"
MODEL_VERSION="1.10.0"
MODEL_URL="https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/${MODEL_NAME}.tar.bz2"
MODEL_DIR="./models"

echo "🔥 Cinnabar Model Setup"
echo "Downloading: ${MODEL_NAME}"
echo ""

if [ -d "${MODEL_DIR}" ]; then
    echo "⚠️  Model directory already exists. Removing..."
    rm -rf "${MODEL_DIR}"
fi

mkdir -p "${MODEL_DIR}"

echo "📥 Downloading model..."
wget -q --show-progress "${MODEL_URL}" -O /tmp/model.tar.bz2

echo "📦 Extracting model..."
tar -xjf /tmp/model.tar.bz2 -C /tmp/

echo "📂 Moving model files..."
mv /tmp/${MODEL_NAME}/*.onnx "${MODEL_DIR}/"
mv /tmp/${MODEL_NAME}/tokens.txt "${MODEL_DIR}/"

echo "🧹 Cleaning up..."
rm -rf /tmp/model.tar.bz2 /tmp/${MODEL_NAME}

echo ""
echo "✅ Model setup complete!"
echo ""
echo "Required files:"
ls -lh "${MODEL_DIR}"

echo ""
echo "🚀 Run: cargo run --release"
