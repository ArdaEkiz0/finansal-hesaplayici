import { app, BrowserWindow, shell, ipcMain, dialog } from "electron";
import path from "path";
import https from "https";
import http from "http";
import fs from "fs";
import { execSync } from "child_process";

const isDev = !app.isPackaged;
const REPO = "ArdaEkiz0/finansal-hesaplayici";
const CURRENT_VERSION = app.getVersion();

let mainWindow: BrowserWindow | null = null;
let splashWindow: BrowserWindow | null = null;

function log(msg: string) {
  const line = `[${new Date().toLocaleTimeString("tr-TR")}] ${msg}\n`;
  try {
    const logPath = path.join(app.getPath("userData"), "updater.log");
    fs.appendFileSync(logPath, line);
  } catch {}
}

function createSplash(): void {
  splashWindow = new BrowserWindow({
    width: 380,
    height: 460,
    frame: false,
    transparent: true,
    resizable: false,
    alwaysOnTop: true,
    skipTaskbar: true,
    webPreferences: { nodeIntegration: false, contextIsolation: true },
  });

  splashWindow.setMenu(null);
  splashWindow.loadFile(path.join(__dirname, "splash.html"));

  splashWindow.once("closed", () => {
    splashWindow = null;
  });
}

function createMainWindow(): void {
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
    setTimeout(() => {
      if (splashWindow) splashWindow.close();
      mainWindow?.show();
    }, 2200);
  });

  mainWindow.on("closed", () => {
    mainWindow = null;
  });

  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    shell.openExternal(url);
    return { action: "deny" };
  });
}

function httpGet(url: string): Promise<Buffer> {
  return new Promise((resolve, reject) => {
    const client = url.startsWith("https") ? https : http;
    client
      .get(url, { headers: { "User-Agent": "FinansalHesaplaci" } }, (res) => {
        if (res.statusCode === 301 || res.statusCode === 302) {
          httpGet(res.headers.location!).then(resolve).catch(reject);
          return;
        }
        const chunks: Buffer[] = [];
        res.on("data", (chunk) => chunks.push(chunk));
        res.on("end", () => resolve(Buffer.concat(chunks)));
        res.on("error", reject);
      })
      .on("error", reject);
  });
}

function checkForUpdates(): Promise<{ hasUpdate: boolean; version: string; notes: string; url: string } | null> {
  return new Promise((resolve) => {
    log("Guncelleme kontrol ediliyor...");
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

            log(`Mevcut: v${CURRENT_VERSION}, Son: v${latestVersion}, URL: ${downloadUrl}`);

            if (latestVersion && latestVersion !== CURRENT_VERSION && downloadUrl) {
              resolve({ hasUpdate: true, version: latestVersion, notes, url: downloadUrl });
            } else {
              resolve(null);
            }
          } catch (e) {
            log(`Parse hatasi: ${e}`);
            resolve(null);
          }
        });
      })
      .on("error", (e) => {
        log(`API hatasi: ${e}`);
        resolve(null);
      });
  });
}

function downloadAndInstall(updateUrl: string): Promise<boolean> {
  return new Promise(async (resolve) => {
    const zipPath = path.join(app.getPath("temp"), "fh-update.zip");
    const extractPath = path.join(app.getPath("temp"), "fh-extract");
    const appDir = app.isPackaged ? path.dirname(app.getPath("exe")) : path.join(__dirname, "..");

    log(`Indiriliyor: ${updateUrl}`);
    log(`Hedef klasor: ${appDir}`);

    try {
      const data = await httpGet(updateUrl);
      fs.writeFileSync(zipPath, data);
      log(`Indirildi: ${(data.length / 1024).toFixed(0)} KB`);

      if (fs.existsSync(extractPath)) fs.rmSync(extractPath, { recursive: true, force: true });
      fs.mkdirSync(extractPath, { recursive: true });

      log("Zip aciliyor...");
      execSync(`powershell -Command "Expand-Archive -Path '${zipPath}' -DestinationPath '${extractPath}' -Force"`, { stdio: "pipe" });

      // Zip icindeki dosyalari kontrol et
      const extractedItems = fs.readdirSync(extractPath);
      log(`Zip icerigi: ${extractedItems.join(", ")}`);

      // package/ klasoru varsa onu kullan, yoksa root'u kullan
      let srcDir = path.join(extractPath, "package");
      if (!fs.existsSync(srcDir)) {
        // Zip root'ta dist/ varsa direkt root'u kullan
        if (fs.existsSync(path.join(extractPath, "dist"))) {
          srcDir = extractPath;
          log("Root'tan kopyalanacak (package/ yok)");
        } else {
          log(`HATA: Gecerli guncelleme icerigi bulunamadi. Icerik: ${extractedItems.join(", ")}`);
          resolve(false);
          return;
        }
      }

      log("Dosyalar kopyalanıyor...");
      copyDirSync(srcDir, appDir);

      log("Temizleniyor...");
      try { fs.rmSync(zipPath, { force: true }); } catch {}
      try { fs.rmSync(extractPath, { recursive: true, force: true }); } catch {}

      log("Guncelleme tamamlandi!");
      resolve(true);
    } catch (e) {
      log(`Guncelleme hatasi: ${e}`);
      try { fs.rmSync(zipPath, { force: true }); } catch {}
      try { fs.rmSync(extractPath, { recursive: true, force: true }); } catch {}
      resolve(false);
    }
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
      try {
        fs.copyFileSync(srcPath, destPath);
      } catch {
        try {
          const tmpPath = destPath + ".new";
          fs.copyFileSync(srcPath, tmpPath);
          fs.renameSync(tmpPath, destPath);
        } catch (e) {
          log(`Kopyalama hatasi: ${destPath} - ${e}`);
        }
      }
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
    createSplash();

    setTimeout(() => {
      createMainWindow();
    }, 500);

    setTimeout(async () => {
      const update = await checkForUpdates();
      if (update && mainWindow) {
        log(`Guncelleme mevcut: v${update.version}`);
        const result = await dialog.showMessageBox(mainWindow, {
          type: "info",
          title: "Guncelleme Mevcut",
          message: `Yeni surum: v${update.version}`,
          detail: `Mevcut: v${CURRENT_VERSION}\nYeni: v${update.version}\n\nGuncellemeyi yuklemek ister misiniz?`,
          buttons: ["Guncelle", "Atla"],
          defaultId: 0,
        });

        if (result.response === 0) {
          log("Kullanici guncellemeyi onayladi");
          const success = await downloadAndInstall(update.url);
          if (success) {
            log("Uygulama yeniden baslatiliyor...");
            app.relaunch();
            app.exit(0);
          } else {
            log("Guncelleme basarisiz");
            dialog.showErrorBox("Guncelleme HATASI", "Guncelleme yuklenemedi. Lutfen tekrar deneyin veya GitHub'dan manuel indirin.");
          }
        }
      } else {
        log("Guncelleme yok veya kontrol edilemedi");
      }
    }, 5000);

    app.on("activate", () => {
      if (BrowserWindow.getAllWindows().length === 0) createMainWindow();
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
