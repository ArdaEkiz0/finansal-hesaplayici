import { app, BrowserWindow, shell, ipcMain, dialog } from "electron";
import path from "path";
import https from "https";
import fs from "fs";
import { execSync } from "child_process";

const isDev = !app.isPackaged;
const REPO = "ArdaEkiz0/finansal-hesaplayici";
const CURRENT_VERSION = app.getVersion();

let mainWindow: BrowserWindow | null = null;

function createWindow(): void {
  mainWindow = new BrowserWindow({
    width: 1200,
    height: 800,
    minWidth: 900,
    minHeight: 600,
    title: "Finansal Hesaplaci",
    titleBarStyle: "hidden",
    backgroundColor: "#030712",
    show: false,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      preload: path.join(__dirname, "preload.js"),
    },
  });

  mainWindow.setMenu(null);

  if (isDev && process.env.VITE_DEV_SERVER_URL) {
    mainWindow.loadURL(process.env.VITE_DEV_SERVER_URL);
    mainWindow.webContents.openDevTools({ mode: "detach" });
  } else {
    mainWindow.loadFile(path.join(__dirname, "../dist/index.html"));
  }

  mainWindow.once("ready-to-show", () => {
    mainWindow?.show();
  });

  mainWindow.on("closed", () => {
    mainWindow = null;
  });

  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    shell.openExternal(url);
    return { action: "deny" };
  });
}

function checkForUpdates(): Promise<{ hasUpdate: boolean; version: string; notes: string; url: string } | null> {
  return new Promise((resolve) => {
    const options = {
      hostname: "api.github.com",
      path: `/repos/${REPO}/releases/latest`,
      headers: { "User-Agent": "FinansalHesaplaci-UpdateChecker" },
    };

    https
      .get(options, (res) => {
        let data = "";
        res.on("data", (chunk) => (data += chunk));
        res.on("end", () => {
          try {
            const release = JSON.parse(data);
            const latestVersion = release.tag_name?.replace("v", "") || "";
            const notes = release.body || "";
            const asset = release.assets?.find((a: any) => a.name.endsWith(".zip"));
            const downloadUrl = asset?.browser_download_url || "";

            if (latestVersion && latestVersion !== CURRENT_VERSION) {
              resolve({ hasUpdate: true, version: latestVersion, notes, url: downloadUrl });
            } else {
              resolve(null);
            }
          } catch {
            resolve(null);
          }
        });
      })
      .on("error", () => resolve(null));
  });
}

function downloadAndInstall(updateUrl: string): Promise<boolean> {
  return new Promise((resolve) => {
    const zipPath = path.join(app.getPath("temp"), "finansal-hesaplaci-update.zip");
    const extractPath = path.join(app.getPath("temp"), "finansal-hesaplaci-extract");
    const appDir = app.isPackaged ? path.dirname(app.getPath("exe")) : path.join(__dirname, "..");

    https
      .get(updateUrl, (res) => {
        if (res.statusCode === 302 || res.statusCode === 301) {
          https.get(res.headers.location!, (res2) => {
            const file = fs.createWriteStream(zipPath);
            res2.pipe(file);
            file.on("finish", () => {
              file.close();
              try {
                if (fs.existsSync(extractPath)) fs.rmSync(extractPath, { recursive: true });
                fs.mkdirSync(extractPath, { recursive: true });
                execSync(`tar -xf "${zipPath}" -C "${extractPath}"`, { stdio: "ignore" });

                const srcDir = path.join(extractPath, "package");
                if (fs.existsSync(srcDir)) {
                  copyDirSync(srcDir, appDir);
                }

                fs.rmSync(zipPath, { force: true });
                fs.rmSync(extractPath, { recursive: true, force: true });
                resolve(true);
              } catch {
                resolve(false);
              }
            });
          });
        } else {
          const file = fs.createWriteStream(zipPath);
          res.pipe(file);
          file.on("finish", () => {
            file.close();
            try {
              if (fs.existsSync(extractPath)) fs.rmSync(extractPath, { recursive: true });
              fs.mkdirSync(extractPath, { recursive: true });
              execSync(`tar -xf "${zipPath}" -C "${extractPath}"`, { stdio: "ignore" });

              const srcDir = path.join(extractPath, "package");
              if (fs.existsSync(srcDir)) {
                copyDirSync(srcDir, appDir);
              }

              fs.rmSync(zipPath, { force: true });
              fs.rmSync(extractPath, { recursive: true, force: true });
              resolve(true);
            } catch {
              resolve(false);
            }
          });
        }
      })
      .on("error", () => resolve(false));
  });
}

function copyDirSync(src: string, dest: string) {
  if (!fs.existsSync(dest)) fs.mkdirSync(dest, { recursive: true });
  for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
    const srcPath = path.join(src, entry.name);
    const destPath = path.join(dest, entry.name);
    if (entry.isDirectory()) {
      copyDirSync(srcPath, destPath);
    } else {
      fs.copyFileSync(srcPath, destPath);
    }
  }
}

const gotTheLock = app.requestSingleInstanceLock();
if (!gotTheLock) {
  app.quit();
} else {
  app.on("second-instance", () => {
    if (mainWindow) {
      if (mainWindow.isMinimized()) mainWindow.restore();
      mainWindow.focus();
    }
  });

  app.whenReady().then(() => {
    createWindow();

    setTimeout(async () => {
      const update = await checkForUpdates();
      if (update && mainWindow) {
        const result = await dialog.showMessageBox(mainWindow, {
          type: "info",
          title: "Guncelleme Mevcut",
          message: `Yeni surum: v${update.version}`,
          detail: update.notes || "Guncellemeyi yuklemek ister misiniz?",
          buttons: ["Guncelle", "Atla"],
          defaultId: 0,
        });

        if (result.response === 0) {
          const success = await downloadAndInstall(update.url);
          if (success) {
            app.relaunch();
            app.exit(0);
          }
        }
      }
    }, 3000);

    app.on("activate", () => {
      if (BrowserWindow.getAllWindows().length === 0) createWindow();
    });
  });
}

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") app.quit();
});

ipcMain.handle("get-app-version", () => app.getVersion());
ipcMain.handle("get-app-name", () => app.getName());
ipcMain.handle("check-update", async () => {
  const update = await checkForUpdates();
  return update;
});
