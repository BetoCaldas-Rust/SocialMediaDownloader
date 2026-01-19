# Social Media Downloader - Task Breakdown

## Planning Phase
- [x] Create project structure and architecture plan
- [x] Define hybrid Rust + Python REST API architecture
- [ ] Design Docker containerization strategy

## Backend Development (Python + FastAPI)
- [ ] Set up Python backend project structure
- [ ] Install backend dependencies (FastAPI, yt-dlp, uvicorn)
- [ ] Implement REST API endpoints
  - [ ] POST /download - initiate download
  - [ ] GET /status/{id} - check download progress
  - [ ] GET /config - get settings
  - [ ] POST /config - update settings
  - [ ] GET /platforms - list supported platforms
- [ ] Integrate yt-dlp for video downloads
  - [ ] URL validation and platform detection
  - [ ] Progress tracking with callbacks
  - [ ] Error handling
- [ ] Implement path management
  - [ ] Dynamic folder structure per platform
  - [ ] Settings persistence
- [ ] Create Dockerfile and docker-compose

## Frontend Development (Rust + GUI)
- [ ] Set up Rust project structure
- [ ] Install Rust dependencies (egui/iced, reqwest, arboard, global-hotkey)
- [ ] Implement GUI with black/yellow theme
  - [ ] Main window layout
  - [ ] URL input field
  - [ ] Download button with loading state
  - [ ] Progress bar
  - [ ] Settings panel
- [ ] Implement HTTP client
  - [ ] API communication with backend
  - [ ] Error handling and retries
- [ ] Clipboard & Hotkey integration
  - [ ] Global hotkey Win/Cmd+Shift+X
  - [ ] Auto-populate URL from clipboard
- [ ] File system operations
  - [ ] Folder selection dialogs
  - [ ] Display download paths

## Docker & Deployment
- [ ] Create Dockerfile for backend
- [ ] Create docker-compose.yml
- [ ] Test Docker container locally
- [ ] Document deployment instructions

## Testing & Verification
- [ ] Test backend API endpoints
- [ ] Test frontend GUI on Windows
- [ ] Test downloads from multiple platforms
- [ ] Verify Docker container functionality
- [ ] Test cross-platform (Linux/Mac if available)
- [ ] Validate global hotkeys
- [ ] Test settings persistence
