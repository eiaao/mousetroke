import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Tauri 期望固定端口；生产构建产物输出到 ui/dist（tauri.conf.json frontendDist）
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  // 词条文件在前端工程目录之外（仓库根 i18n/），开发服务器要放行读取
  server: { port: 5174, strictPort: true, fs: { allow: ["../../.."] } },
  build: { target: "es2022", outDir: "dist", emptyOutDir: true, sourcemap: false },
});
