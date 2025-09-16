# Worksmart ⚡️

A powerful desktop time tracking and productivity monitoring application built with Tauri, React, and TypeScript. Worksmart helps you track your work sessions, capture screen activity, and monitor your productivity with advanced features like webcam integration and automatic session management.

## ✨ Features

### 🕐 Time Tracking

- **Session Management**: Start and stop work sessions with a single click
- **Real-time Timer**: Live tracking of active sessions with hour/minute display
- **Daily Time Summary**: Track total time worked today with visual indicators
- **Session History**: View past sessions with start/end times

### 📸 Screen & Camera Capture

- **Screen Recording**: Capture screen activity during work sessions
- **Webcam Integration**: Take periodic webcam shots for productivity monitoring
- **Camera Device Selection**: Choose from available camera devices
- **Image Compression**: Optimized storage with automatic image compression

### ⚙️ Advanced Configuration

- **Auto-startup**: Launch application on system startup
- **Auto-signin**: Automatic sign-in on application launch
- **Auto-tracking**: Start tracking immediately after sign-in
- **Customizable Delays**: Configurable webcam capture intervals (3s, 5s, 10s)
- **Storage Management**: Configurable storage directories for media and data

### 🎨 Modern UI/UX

- **Clean Interface**: Minimalist design with drag-and-drop window support
- **Real-time Updates**: Live session status and time tracking
- **Responsive Design**: Optimized for different screen sizes
- **Dark Theme**: Professional dark interface

### 🔒 Privacy & Security

- **Local Storage**: All data stored locally on your machine
- **Permission Management**: Granular control over camera and screen capture permissions
- **Secure Authentication**: JWT-based authentication system

## 🛠️ Tech Stack

### Frontend

- **React 18** - Modern UI framework
- **TypeScript** - Type-safe development
- **Vite** - Fast build tool and dev server
- **Tailwind CSS** - Utility-first CSS framework
- **Bootstrap 5** - Component library
- **Lucide React** - Beautiful icons
- **Sass** - CSS preprocessor

### Backend

- **Tauri 1.8** - Rust-based desktop app framework
- **Rust** - High-performance system programming
- **Tokio** - Async runtime for Rust
- **Serde** - Serialization framework

### Media & Capture

- **FFmpeg** - Video/audio processing and screen capture
- **Nokhwa** - Cross-platform camera access
- **Xcap** - Screen capture library
- **Image Processing** - Rust image manipulation

### System Integration

- **Auto-launch** - System startup integration
- **Core Graphics** - macOS graphics framework
- **Active Window Detection** - Focused window tracking
- **File System Access** - Local storage management

## 🚀 Getting Started

### Prerequisites

- **Node.js** (v16 or higher)
- **Yarn** package manager
- **Rust** (latest stable version)
- **macOS** (primary target platform)
- **Xcode Command Line Tools** (for macOS development)

### Installation

1. **Clone the repository**

   ```bash
   git clone <repository-url>
   cd worksmart
   ```

2. **Install dependencies**

   ```bash
   # Install frontend dependencies
   yarn install

   # Install Rust dependencies (automatic on first build)
   cargo build
   ```

3. **Set up FFmpeg**
   ```bash
   # FFmpeg binaries are included in the project
   # Located in: target/binaries/ffmpeg
   ```

### Development

#### Start Development Server

```bash
# Using Makefile (recommended)
make dev

# Or using Tauri CLI directly
yarn tauri dev
```

#### Build for Production

```bash
# Build development version
make build

# Build release version
make release
```

#### Clean Build Artifacts

```bash
make clean
```

### Development Workflow

1. **Frontend Development**

   - Edit React components in `src/`
   - Styles are in `src/styles/`
   - Hot reload is enabled during development

2. **Backend Development**

   - Rust code is in `src-tauri/src/`
   - Main modules: auth, camera, recorder, session, storage
   - Use `cargo check` for quick compilation checks

3. **Testing**
   - Test camera functionality in Settings
   - Verify screen capture permissions
   - Check session tracking accuracy

## 📁 Project Structure

```
worksmart/
├── src/                    # Frontend React application
│   ├── components/         # React components
│   ├── styles/            # CSS/SCSS styles
│   ├── ipc/               # Tauri IPC communication
│   └── types.ts           # TypeScript type definitions
├── src-tauri/             # Backend Rust application
│   ├── src/               # Rust source code
│   │   ├── auth.rs        # Authentication logic
│   │   ├── camera.rs      # Camera capture
│   │   ├── recorder.rs    # Screen recording
│   │   ├── session.rs     # Session management
│   │   └── storage.rs     # Data persistence
│   ├── scripts/           # Build scripts
│   └── Cargo.toml         # Rust dependencies
├── target/binaries/       # FFmpeg binaries
└── dist/                  # Built application
```

## 🔧 Configuration

### Application Settings

- **Launch on Startup**: Automatically start with system
- **Sign in on Launch**: Auto-authenticate on startup
- **Start Tracking on Sign-in**: Begin session immediately
- **Enable Camera**: Toggle webcam capture
- **Webcam Delay**: Set capture interval (3-10 seconds)

### Storage Locations

- **Capsule Storage**: Application data directory
- **Media Storage**: Screenshots and recordings
- **Configuration**: User preferences and settings

## 🤝 Contributing

### Development Setup

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes
4. Test thoroughly on macOS
5. Commit your changes: `git commit -m 'Add amazing feature'`
6. Push to the branch: `git push origin feature/amazing-feature`
7. Open a Pull Request

### Code Style

- **Frontend**: Follow React/TypeScript best practices
- **Backend**: Follow Rust conventions and use `cargo fmt`
- **Commits**: Use conventional commit messages

### Testing

- Test camera functionality across different devices
- Verify screen capture permissions
- Test session tracking accuracy
- Check auto-startup behavior

## 📋 Requirements

### System Requirements

- **macOS 10.15+** (primary target)
- **4GB RAM** minimum
- **500MB** disk space
- **Camera** (optional, for webcam features)
- **Screen Recording Permissions** (required for screen capture)

### Development Requirements

- **Node.js 16+**
- **Yarn 1.22+**
- **Rust 1.70+**
- **Xcode Command Line Tools**

## 🐛 Troubleshooting

### Common Issues

1. **Camera not working**

   - Check camera permissions in System Preferences
   - Verify camera device selection in Settings

2. **Screen capture fails**

   - Grant Screen Recording permissions
   - Restart application after permission changes

3. **Build errors**

   - Run `cargo clean` and rebuild
   - Ensure all dependencies are installed
   - Check Rust toolchain version

4. **FFmpeg issues**
   - Verify FFmpeg binaries in `target/binaries/`
   - Check file permissions

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- **Tauri** - For the amazing desktop app framework
- **React** - For the powerful UI library
- **FFmpeg** - For media processing capabilities
- **Rust Community** - For excellent crates and tools

---

**Built with ❤️ for productivity enthusiasts**
