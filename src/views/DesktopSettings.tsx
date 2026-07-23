import { useCallback, useEffect, useState } from "react";
import { Button } from "../components/Button";
import { ErrorNotice } from "../components/ErrorNotice";
import { Field } from "../components/Field";
import { ipc, type AppError, type AppSettings } from "../ipc";

export type DesktopSettingsApi = {
  getSettings: () => Promise<AppSettings>;
  setCaptureShortcut: (shortcut: string) => Promise<AppSettings>;
  setAutostart: (value: boolean) => Promise<AppSettings>;
  exportSanitizedLogs: () => Promise<{ exported: boolean }>;
};

export function DesktopSettings({ api = ipc }: { api?: DesktopSettingsApi }) {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [shortcut, setShortcut] = useState("");
  const [error, setError] = useState<string>();
  const load = useCallback(
    () =>
      api
        .getSettings()
        .then((value) => {
          setSettings(value);
          setShortcut(value.captureShortcut);
        })
        .catch((failure: AppError) => setError(failure.message)),
    [api],
  );
  useEffect(() => {
    void load();
  }, [load]);

  if (!settings) return <p>正在加载桌面设置…</p>;
  return (
    <section className="settings-grid" aria-label="桌面设置">
      <h2>桌面行为</h2>
      {error && <ErrorNotice message={error} />}
      <Field
        label="截图快捷键"
        htmlFor="capture-shortcut"
        hint="先成功注册新组合后才会释放旧组合。"
      >
        <input
          id="capture-shortcut"
          value={shortcut}
          onChange={(event) => setShortcut(event.target.value)}
        />
      </Field>
      <Button
        onClick={() => {
          setError(undefined);
          void api
            .setCaptureShortcut(shortcut)
            .then((value) => {
              setSettings(value);
              setShortcut(value.captureShortcut);
            })
            .catch((failure: AppError) => {
              setShortcut(settings.captureShortcut);
              setError(failure.message);
            });
        }}
      >
        保存快捷键
      </Button>
      <label className="toggle">
        <input
          aria-label="开机启动"
          type="checkbox"
          checked={settings.autostart}
          onChange={(event) => {
            const value = event.target.checked;
            void api
              .setAutostart(value)
              .then(setSettings)
              .catch((failure: AppError) => setError(failure.message));
          }}
        />
        开机启动
      </label>
      <div>
        <Button
          onClick={() =>
            void api
              .exportSanitizedLogs()
              .catch((failure: AppError) => setError(failure.message))
          }
        >
          导出诊断日志
        </Button>
        <p className="field__hint">
          导出内容会移除凭据、供应商原始响应和模型输出；应用不采集遥测。
        </p>
      </div>
    </section>
  );
}
