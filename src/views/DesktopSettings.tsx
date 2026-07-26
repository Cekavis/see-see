import { useCallback, useEffect, useState } from "react";
import { Button } from "../components/Button";
import { Field } from "../components/Field";
import { useNotifications } from "../components/Notifications";
import { ipc, type AppError, type AppSettings } from "../ipc";

const MODIFIER_CODES = new Set([
  "AltLeft",
  "AltRight",
  "ControlLeft",
  "ControlRight",
  "MetaLeft",
  "MetaRight",
  "ShiftLeft",
  "ShiftRight",
]);

const MODIFIER_KEYS = new Set(["Alt", "Control", "Meta", "OS", "Shift"]);

const SUPPORTED_SHORTCUT_CODES = new Set([
  "ArrowDown",
  "ArrowLeft",
  "ArrowRight",
  "ArrowUp",
  "AudioVolumeDown",
  "AudioVolumeMute",
  "AudioVolumeUp",
  "Backquote",
  "Backslash",
  "Backspace",
  "BracketLeft",
  "BracketRight",
  "CapsLock",
  "Comma",
  "Delete",
  "End",
  "Enter",
  "Equal",
  "Escape",
  "Home",
  "Insert",
  "MediaPause",
  "MediaPlay",
  "MediaPlayPause",
  "MediaStop",
  "MediaTrackNext",
  "MediaTrackPrevious",
  "Minus",
  "NumLock",
  "NumpadAdd",
  "NumpadDecimal",
  "NumpadDivide",
  "NumpadEnter",
  "NumpadEqual",
  "NumpadMultiply",
  "NumpadSubtract",
  "PageDown",
  "PageUp",
  "Pause",
  "Period",
  "PrintScreen",
  "Quote",
  "ScrollLock",
  "Semicolon",
  "Slash",
  "Space",
  "Tab",
]);

const SHORTCUT_KEY_ALIASES: Record<string, string> = {
  " ": "Space",
  "'": "Quote",
  ",": "Comma",
  "-": "Minus",
  ".": "Period",
  "/": "Slash",
  ";": "Semicolon",
  "=": "Equal",
  "[": "BracketLeft",
  "\\": "Backslash",
  "]": "BracketRight",
  "`": "Backquote",
  Down: "ArrowDown",
  Esc: "Escape",
  Left: "ArrowLeft",
  Right: "ArrowRight",
  Up: "ArrowUp",
};

function normalizedShortcutKey(
  event: Pick<KeyboardEvent, "code" | "key">,
): string | null {
  if (MODIFIER_CODES.has(event.code) || MODIFIER_KEYS.has(event.key)) {
    return null;
  }
  if (/^Key[A-Z]$/.test(event.code)) return event.code.slice(3);
  if (/^Digit[0-9]$/.test(event.code)) return event.code.slice(5);
  if (
    /^F(?:[1-9]|1[0-9]|2[0-4])$/.test(event.code) ||
    /^Numpad[0-9]$/.test(event.code) ||
    SUPPORTED_SHORTCUT_CODES.has(event.code)
  ) {
    return event.code;
  }

  if (/^[a-z0-9]$/i.test(event.key)) return event.key.toUpperCase();
  const fallback = SHORTCUT_KEY_ALIASES[event.key] ?? event.key;
  return SUPPORTED_SHORTCUT_CODES.has(fallback) ? fallback : null;
}

export function shortcutFromKeyboardEvent(
  event: Pick<
    KeyboardEvent,
    "altKey" | "code" | "ctrlKey" | "key" | "metaKey" | "shiftKey"
  >,
): string | null {
  const key = normalizedShortcutKey(event);
  if (!key) return null;

  const modifiers = [
    event.metaKey && "Command",
    event.ctrlKey && "Ctrl",
    event.altKey && "Alt",
    event.shiftKey && "Shift",
  ].filter(Boolean);
  if (modifiers.length === 0 && !/^F(?:[1-9]|1[0-9]|2[0-4])$/.test(key)) {
    return null;
  }

  return [...modifiers, key].join("+");
}

export type DesktopSettingsApi = {
  getSettings: () => Promise<AppSettings>;
  setCaptureShortcut: (shortcut: string) => Promise<AppSettings>;
  setAutostart: (value: boolean) => Promise<AppSettings>;
  openLoginItemsSettings: () => Promise<void>;
  setSaveHistory: (value: boolean) => Promise<AppSettings>;
  exportSanitizedLogs: () => Promise<{ exported: boolean }>;
};

export function DesktopSettings({ api = ipc }: { api?: DesktopSettingsApi }) {
  const notifications = useNotifications();
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [shortcut, setShortcut] = useState("");
  const [isRecordingShortcut, setIsRecordingShortcut] = useState(false);
  const [isSavingShortcut, setIsSavingShortcut] = useState(false);
  const load = useCallback(() => {
    function run() {
      void api
        .getSettings()
        .then((value) => {
          setSettings(value);
          setShortcut(value.captureShortcut);
        })
        .catch((failure: AppError) =>
          notifications.error(failure.message, {
            action: { label: "重试", onClick: run },
          }),
        );
    }
    run();
  }, [api, notifications]);
  useEffect(() => {
    load();
  }, [load]);

  const saveCapturedShortcut = useCallback(
    (nextShortcut: string) => {
      if (!settings) return;
      setIsRecordingShortcut(false);
      setIsSavingShortcut(true);
      notifications.clear();
      void api
        .setCaptureShortcut(nextShortcut)
        .then((value) => {
          setSettings(value);
          setShortcut(value.captureShortcut);
          notifications.success("快捷键已保存并生效");
        })
        .catch((failure: AppError) => {
          setShortcut(settings.captureShortcut);
          notifications.error(failure.message);
        })
        .finally(() => setIsSavingShortcut(false));
    },
    [api, notifications, settings],
  );

  useEffect(() => {
    if (!isRecordingShortcut) return;
    const recordShortcut = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();
      if (
        (event.code === "Escape" || event.key === "Escape") &&
        !event.altKey &&
        !event.ctrlKey &&
        !event.metaKey &&
        !event.shiftKey
      ) {
        setIsRecordingShortcut(false);
        return;
      }
      const nextShortcut = shortcutFromKeyboardEvent(event);
      if (nextShortcut) saveCapturedShortcut(nextShortcut);
    };
    window.addEventListener("keydown", recordShortcut, true);
    return () => window.removeEventListener("keydown", recordShortcut, true);
  }, [isRecordingShortcut, saveCapturedShortcut]);

  if (!settings) {
    return (
      <section className="settings-group" aria-label="桌面设置">
        <p className="settings-loading">正在加载桌面设置…</p>
      </section>
    );
  }
  return (
    <section className="settings-group" aria-label="桌面设置">
      <h2>应用偏好</h2>
      <div className="setting-row">
        <div className="setting-row__body">
          <Field
            label="截图快捷键"
            htmlFor="capture-shortcut"
            hint="点击后直接按下新组合键；按 Esc 取消。"
          >
            <Button
              id="capture-shortcut"
              type="button"
              className="shortcut-recorder"
              aria-label="截图快捷键"
              aria-pressed={isRecordingShortcut}
              disabled={isSavingShortcut}
              onClick={() => setIsRecordingShortcut((recording) => !recording)}
            >
              {isSavingShortcut
                ? "正在应用…"
                : isRecordingShortcut
                  ? "请按新的快捷键…"
                  : shortcut}
            </Button>
          </Field>
        </div>
      </div>
      <label className="setting-row switch">
        <span className="setting-row__body">
          <strong>开机启动</strong>
        </span>
        <input
          aria-label="开机启动"
          type="checkbox"
          role="switch"
          checked={settings.autostart}
          onChange={(event) => {
            const value = event.target.checked;
            void api
              .setAutostart(value)
              .then(setSettings)
              .catch((failure: AppError) => {
                notifications.error(
                  failure.message,
                  failure.action === "open_login_items"
                    ? {
                        action: {
                          label: "打开系统设置",
                          onClick: () => void api.openLoginItemsSettings(),
                        },
                      }
                    : undefined,
                );
              });
          }}
        />
      </label>
      <label className="setting-row switch">
        <span className="setting-row__body">
          <strong>保存历史记录</strong>
          <span className="field__hint">
            原始截图、结果和提示词快照仅保存在本机。
          </span>
        </span>
        <input
          aria-label="保存历史记录"
          type="checkbox"
          role="switch"
          checked={settings.saveHistory}
          onChange={(event) => {
            void api
              .setSaveHistory(event.target.checked)
              .then(setSettings)
              .catch((failure: AppError) =>
                notifications.error(failure.message),
              );
          }}
        />
      </label>
      <div className="setting-row">
        <div className="setting-row__body">
          <strong>诊断日志</strong>
          <span className="field__hint">
            导出内容会移除凭据、供应商原始响应和模型输出；应用不采集遥测。
          </span>
        </div>
        <Button
          onClick={() => {
            notifications.clear();
            void api
              .exportSanitizedLogs()
              .then((result) => {
                if (result.exported) notifications.success("诊断日志已导出");
              })
              .catch((failure: AppError) =>
                notifications.error(failure.message),
              );
          }}
        >
          导出诊断日志
        </Button>
      </div>
    </section>
  );
}
