/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_PLATFORM?: "tauri" | "web"
  readonly VITE_API_BASE_URL?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}

declare const __APP_VERSION__: string
declare const __PLATFORM__: "tauri" | "web"
