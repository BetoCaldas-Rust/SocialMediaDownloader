# Social Media Downloader - Task Breakdown

> CONCLUÍDO (F0–F8, issues #1–#10 fechadas): o conteúdo abaixo é o plano histórico da era Python+Rust, preservado para referência. O app atual é um binário Rust único — ver README.md.

> Direção atual (F0, issue #2): o app está migrando para um **binário Rust único** — o backend Python, a camada HTTP, o Docker e as janelas de terminal separadas foram removidos. Os itens de backend/Docker abaixo estão cancelados e mantidos apenas como histórico; o trabalho ativo está nos GitHub issues #1–#10.

## Planning Phase
- [x] Create project structure and architecture plan
- [x] Define hybrid Rust + Python REST API architecture (superseded: single Rust binary per issues #1–#10)
- [ ] ~~Design Docker containerization strategy~~ (cancelled in F0)

## Backend Development (cancelled in F0 — Python backend deleted; kept as history)
- [ ] ~~Set up Python backend project structure~~
- [ ] ~~Install backend dependencies~~
- [ ] ~~Implement REST API endpoints~~
  - [ ] ~~POST /download - initiate download~~
  - [ ] ~~GET /status/{id} - check download progress~~
  - [ ] ~~GET /config - get settings~~
  - [ ] ~~POST /config - update settings~~
  - [ ] ~~GET /platforms - list supported platforms~~
- [ ] Integrate yt-dlp for video downloads (moved into the single Rust binary; see issues #1–#10)
  - [ ] URL validation and platform detection
  - [ ] Progress tracking with callbacks
  - [ ] Error handling
- [ ] Implement path management
  - [ ] Dynamic folder structure per platform
  - [ ] Settings persistence
- [ ] ~~Create Dockerfile and container orchestration~~

## Frontend Development (Rust + GUI)
- [ ] Set up Rust project structure
- [ ] Install Rust dependencies (egui, tokio, tracing, arboard, global-hotkey)
- [ ] Implement GUI with black/yellow theme
  - [ ] Main window layout
  - [ ] URL input field
  - [ ] Download button with loading state
  - [ ] Progress bar
  - [ ] Settings panel
- [ ] Implement HTTP client (removed in F0 — single binary, no HTTP layer; see issues #1–#10)
  - [ ] ~~API communication with backend~~
  - [ ] Error handling and retries
- [ ] Clipboard & Hotkey integration
  - [ ] Global hotkey Win/Cmd+Shift+X
  - [ ] Auto-populate URL from clipboard
- [ ] File system operations
  - [ ] Folder selection dialogs
  - [ ] Display download paths

## Docker & Deployment (cancelled in F0 — kept as history)
- [ ] ~~Create Dockerfile for backend~~
- [ ] ~~Create container orchestration manifest~~
- [ ] ~~Test Docker container locally~~
- [ ] Document deployment instructions

## Testing & Verification
- [ ] ~~Test backend API endpoints~~ (cancelled in F0 — no HTTP layer)
- [ ] Test frontend GUI on Windows
- [ ] Test downloads from multiple platforms
- [ ] ~~Verify Docker container functionality~~ (cancelled in F0)
- [ ] Test cross-platform (Linux/Mac if available)
- [ ] Validate global hotkeys
- [ ] Test settings persistence
