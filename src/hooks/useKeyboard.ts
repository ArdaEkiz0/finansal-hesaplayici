import { useEffect } from "react";
import { useStore } from "../store/useStore";

export function useKeyboard() {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const t = e.target as HTMLElement | null;
      const inField = t && (t.tagName === "INPUT" || t.tagName === "TEXTAREA");
      const k = e.key;

      if (k === "F1") { e.preventDefault(); useStore.getState().toggleHelp(); return; }
      if (k === "F2") { e.preventDefault(); useStore.getState().toggleSettings(); return; }

      if (inField) {
        if (k === "Enter") { e.preventDefault(); useStore.getState().confirmInput(); }
        else if (k === "Escape") { (t as HTMLElement).blur(); useStore.getState().clearAll(); }
        return;
      }

      if (k >= "0" && k <= "9") {
        e.preventDefault();
        useStore.getState().appendDigit(k);
      } else if (k === "+" || k === "-" || k === "*" || k === "/") {
        e.preventDefault();
        useStore.getState().appendDigit(k);
      } else if (k === "." || k === ",") {
        e.preventDefault();
        useStore.getState().appendDigit(".");
      } else if (k === "Backspace") {
        e.preventDefault();
        useStore.getState().deleteLast();
      } else if (k === "Escape" || k === "Delete") {
        e.preventDefault();
        useStore.getState().clearAll();
      } else if (k === "Enter" || k === "=") {
        e.preventDefault();
        useStore.getState().confirmInput();
      } else if (k === "Tab") {
        e.preventDefault();
        useStore.getState().switchField();
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);
}
