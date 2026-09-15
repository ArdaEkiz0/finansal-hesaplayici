export {};

declare global {
  interface Window {
    electronAPI?: {
      getAppVersion: () => Promise<string>;
      getAppName: () => Promise<string>;
      checkUpdate: () => Promise<any>;
      platform: string;
      minimize: () => void;
      maximize: () => void;
      close: () => void;
      isMaximized: () => Promise<boolean>;
      onMaximizeChange: (cb: (maximized: boolean) => void) => void;
    };
  }
}
