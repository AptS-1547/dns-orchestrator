import { readFileSync } from "node:fs"
import path from "node:path"
import tailwindcss from "@tailwindcss/vite"
import react from "@vitejs/plugin-react"
import { defineConfig } from "vite"

const host = process.env.TAURI_DEV_HOST

const pkg = JSON.parse(readFileSync("./package.json", "utf-8"))

// 平台类型：tauri（默认）或 web
const platform = process.env.VITE_PLATFORM ?? "tauri"

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [react(), tailwindcss()],
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
    __PLATFORM__: JSON.stringify(platform),
  },
  resolve: {
    alias: {
      // 编译时切换 Transport 实现（需要放在 @ 之前，否则会被 @ 优先匹配）
      "#transport-impl": path.resolve(
        __dirname,
        platform === "web"
          ? "./src/services/transport/http.transport.ts"
          : "./src/services/transport/tauri.transport.ts"
      ),
      // 编译时切换平台检测实现
      "#platform-impl": path.resolve(
        __dirname,
        platform === "web" ? "./src/lib/platform.web.ts" : "./src/lib/platform.tauri.ts"
      ),
      "@": path.resolve(__dirname, "./src"),
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,

  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (!id.includes("node_modules")) return
          if (/[\\/](react-dom|react[\\/]|react-router)/.test(id)) return "vendor-react"
          if (id.includes("@radix-ui")) return "vendor-radix"
          if (id.includes("@tauri-apps")) return "vendor-tauri"
          if (id.includes("i18next")) return "vendor-i18n"
        },
      },
    },
  },

  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**", "**/src-actix-web/**", "**/dns-orchestrator-provider/**"],
    },
    // Web 模式下的代理配置
    ...(platform === "web" && {
      proxy: {
        "/api": {
          target: "http://localhost:8080",
          changeOrigin: true,
        },
      },
    }),
  },
}))
