import { useEffect } from "react";
import { useStore } from "../store/useStore";

export function useKeyboard() {
  useEffect(() => {
    let lastTime = 0;
    let lastKey = "";

    const handler = (e: KeyboardEvent) => {
      const now = Date.now();
      const k = e.key;

      if (k === "F1") { e.preventDefault(); useStore.getState().toggleHelp(); return; }
      if (k === "F2") { e.preventDefault(); useStore.getState().toggleSettings(); return; }

      if (k >= "0" && k <= "9") {
        if (lastKey === k && now - lastTime < 80) { lastTime = now; return; }
        e.preventDefault();
        useStore.getState().appendDigit(k);
        lastKey = k;
        lastTime = now;
      } else if (k === "." || k === ",") {
        e.preventDefault();
        useStore.getState().appendDigit(".");
        lastKey = ".";
        lastTime = now;
      } else if (k === "Backspace") {
        e.preventDefault();
        useStore.getState().deleteLast();
        lastKey = "";
      } else if (k === "Escape" || k === "Delete") {
        e.preventDefault();
        useStore.getState().clearAll();
        lastKey = "";
      } else if (k === "Enter" || k === "=") {
        e.preventDefault();
        useStore.getState().confirmInput();
        lastKey = "";
      } else if (k === "Tab") {
        e.preventDefault();
        useStore.getState().switchField();
        lastKey = "";
      } else {
        lastKey = "";
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);
}
