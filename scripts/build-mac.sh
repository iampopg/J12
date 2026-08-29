#!/bin/bash
set -e

echo "🚀 Building J12 Forensic Suite for macOS (.app / .dmg)..."
cd "$(dirname "$0")/.."

# 1. Check Node & Rust
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed. Please install Node.js 18+."
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "❌ Rust / Cargo is not installed. Please install rustup (https://rustup.rs)."
    exit 1
fi

# 2. Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "📦 Installing npm dependencies..."
    npm install
fi

# 3. Build production bundle
echo "🔨 Running Tauri production bundle build..."
npx tauri build

echo ""
echo "✅ Build Complete!"
echo "📦 DMG & App Bundle output location:"
echo "   src-tauri/target/release/bundle/dmg/"
echo "   src-tauri/target/release/bundle/macos/"
