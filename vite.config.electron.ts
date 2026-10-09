import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import electron from "vite-plugin-electron";
import fs from "fs";
import path from "path";

function copySplash(): import("vite").Plugin {
  return {
    name: "copy-splash",
    closeBundle() {
      const src = path.resolve(__dirname, "electron/splash.html");
      const dest = path.resolve(__dirname, "dist-electron/splash.html");
      if (fs.existsSync(src)) {
        const pkg = JSON.parse(fs.readFileSync(path.resolve(__dirname, "package.json"), "utf-8"));
        const html = fs.readFileSync(src, "utf-8").replaceAll("{{APP_VERSION}}", pkg.version ?? "");
        fs.writeFileSync(dest, html);
        console.log(`✓ splash.html kopyalandi (v${pkg.version})`);
      }
    },
  };
}

export default defineConfig({
  plugins: [
    react(),
    tailwindcss(),
    electron([
      {
        entry: "electron/main.ts",
        vite: {
          build: {
            outDir: "dist-electron",
            rollupOptions: {
              external: ["electron"],
            },
          },
        },
      },
      {
        entry: "electron/preload.ts",
        onstart(args) {
          args.reload();
        },
        vite: {
          build: {
            outDir: "dist-electron",
            rollupOptions: {
              external: ["electron"],
            },
          },
        },
      },
    ]),
    copySplash(),
  ],
});
