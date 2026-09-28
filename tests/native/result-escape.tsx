import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { App } from "../../src/App";
import { NotificationProvider } from "../../src/components/Notifications";
import "../../src/styles.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <NotificationProvider>
      <App />
    </NotificationProvider>
  </StrictMode>,
);

async function until(check: () => boolean, message: string) {
  const deadline = Date.now() + 8000;
  while (!check()) {
    if (Date.now() > deadline) throw new Error(message);
    await new Promise<void>((resolve) =>
      requestAnimationFrame(() => resolve()),
    );
  }
}

async function press(key: string, expectClose = false) {
  await invoke("smoke_press_key", { key, expectClose });
}

async function report(message: string) {
  await invoke("smoke_report", { message, failed: false });
}

async function run() {
  const scenario = Number(new URLSearchParams(location.search).get("scenario"));
  await until(
    () =>
      document.body.textContent?.includes("Native Escape smoke result") ===
      true,
    "Result did not render",
  );
  if (scenario === 0) {
    let denial = "";
    try {
      await getCurrentWebviewWindow().close();
    } catch (error) {
      denial = String(error);
    }
    if (!denial.includes("not allowed"))
      throw new Error(
        `Expected production close permission denial, got: ${denial}`,
      );
    await report(
      "production capabilities reject the former frontend close route",
    );
    await press("Escape", true);
    return;
  }

  const trigger = Array.from(document.querySelectorAll("button")).find(
    (button) => button.textContent === "换模型重试",
  );
  if (!trigger) throw new Error("Model chooser trigger missing");
  trigger.focus();
  trigger.click();
  await until(
    () => !!document.querySelector("dialog[open] ul button"),
    "Chooser did not open",
  );
  if (scenario === 2) {
    await press("Ctrl+W", true);
    return;
  }

  await press("EscapeDown");
  await until(
    () => !document.querySelector("dialog[open]"),
    "Native Escape did not dismiss the chooser",
  );
  await report("native Escape dismissed only the chooser");
  await press("EscapeRepeatAndUp");
  await new Promise((resolve) => setTimeout(resolve, 200));
  if (document.activeElement !== trigger)
    throw new Error("Chooser trigger focus was not restored");
  await report("held Escape did not close the source window");
  await press("Escape", true);
}

void run().catch((error: unknown) => {
  void invoke("smoke_report", { message: String(error), failed: true });
});
