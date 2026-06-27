git add .
git commit -m "feat: setup clean architecture, advanced logging, and local fs structure

- Refactored Tauri backend into core, infrastructure, application, and event_system layers
- Replaced log and simplelog with tracing and tracing-appender for structured context logging
- Implemented FsManager to securely create and validate cache, temp, logs, and game directories
- Implemented PathResolver to compute paths from AppData and user-defined game path
- Default game path now resolves to C:\Users\<User>\Games\WufusCraft
- Replaced synchronous startup blocks with async initialize_fs call from SplashPage
- Added proper UI error handling when folder creation or write access fails
- Cleaned up temp folder automatically on application start
"
