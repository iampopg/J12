# Installation Guide

## Prerequisites

- **Node.js** 18+ and npm
- **Rust** 1.70+ (install via [rustup](https://rustup.rs/))
- **SQLite** (bundled with the app)
- **Tauri CLI** (`npm install -g @tauri-apps/cli`)

## Platform Requirements

### macOS
- Xcode Command Line Tools (`xcode-select --install`)

### Windows
- Microsoft Visual Studio C++ Build Tools
- WebView2 Runtime (included with Windows 11)

### Linux
```bash
sudo apt install libwebkit2gtk-4.0-dev \
  build-essential \
  curl \
  wget \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

## Setup

```bash
# Clone the repository
git clone https://github.com/iampopg/J12.git
cd J12

# Install frontend dependencies
cd frontend && npm install && cd ..

# Run in development mode
cd frontend && npx tauri dev
```

## Build for Production

```bash
cd frontend && npx tauri build
```

The built application will be in `src-tauri/target/release/`.

## Default Login

- **Username:** `admin`
- **Password:** `admin123`

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Port 5173 in use | Kill other Vite instances or change port |
| Rust compilation errors | Run `rustup update` and try again |
| Missing dependencies | Install platform-specific prerequisites above |
