import { useCallback, useEffect, useState } from "react";
import { Button } from "../components/Button";
import { ErrorNotice } from "../components/ErrorNotice";
import { Field } from "../components/Field";
import { ipc, type AppError, type AppSettings } from "../ipc";

export type DesktopSettingsApi = {
  getSettings: () => Promise<AppSettings>;
  setCaptureShortcut: (shortcut: string) => Promise<AppSettings>;
  setAutostart: (value: boolean) => Promise<AppSettings>;
  setSaveHistory: (value: boolean) => Promise<AppSettings>;
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

  if (!settings) {
    return (
      <section className="settings-group" aria-label="桌面设置">
        {error ? (
          <ErrorNotice message={error} onRetry={() => void load()} />
        ) : (
          <p className="settings-loading">正在加载桌面设置…</p>
        )}
      </section>
    );
  }
  return (
    <section className="settings-group" aria-label="桌面设置">
      <h2>应用偏好</h2>
      {error && <ErrorNotice message={error} />}
      <div className="setting-row">
        <div className="setting-row__body">
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
        </div>
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
      </div>
      <label className="setting-row switch">
        <span className="setting-row__body">
          <strong>开机启动</strong>
          <span className="field__hint">登录系统后自动启动 See See。</span>
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
              .catch((failure: AppError) => setError(failure.message));
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
              .catch((failure: AppError) => setError(failure.message));
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
          onClick={() =>
            void api
              .exportSanitizedLogs()
              .catch((failure: AppError) => setError(failure.message))
          }
        >
          导出诊断日志
        </Button>
      </div>
    </section>
  );
}
