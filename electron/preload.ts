import { contextBridge, ipcRenderer } from "electron";

contextBridge.exposeInMainWorld("electronAPI", {
  getAppVersion: () => ipcRenderer.invoke("get-app-version"),
  getAppName: () => ipcRenderer.invoke("get-app-name"),
  checkUpdate: () => ipcRenderer.invoke("check-update"),
  platform: process.platform,
  minimize: () => ipcRenderer.invoke("win-minimize"),
  maximize: () => ipcRenderer.invoke("win-maximize"),
  close: () => ipcRenderer.invoke("win-close"),
  isMaximized: () => ipcRenderer.invoke("win-is-maximized"),
  onMaximizeChange: (cb: (maximized: boolean) => void) => {
    ipcRenderer.on("win-maximize-change", (_e, maximized) => cb(maximized));
  },
});
