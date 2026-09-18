import { useEffect, useState } from "react";
import { Button } from "../components/Button";
import { ConfirmDialog } from "../components/ConfirmDialog";
import { Field } from "../components/Field";
import { useNotifications } from "../components/Notifications";
import {
  getErrorMessage,
  ipc,
  type WebdavSettings as SavedWebdavSettings,
  type WebdavSettingsInput,
} from "../ipc";

export type WebdavSettingsApi = Pick<
  typeof ipc,
  | "getWebdavSettings"
  | "saveWebdavSettings"
  | "uploadConfiguration"
  | "downloadConfiguration"
>;

const DEFAULT_ROOT = "see-see";
type Operation = "loading" | "idle" | "saving" | "uploading" | "downloading";

function settingsForm(settings: SavedWebdavSettings): WebdavSettingsInput {
  return {
    url: settings.url,
    username: settings.username,
    remoteRoot: settings.remoteRoot || DEFAULT_ROOT,
  };
}

export function WebdavSettings({
  api = ipc,
  onDownloaded,
}: {
  api?: WebdavSettingsApi;
  onDownloaded?: () => void;
}) {
  const notifications = useNotifications();
  const [saved, setSaved] = useState<SavedWebdavSettings | null>(null);
  const [form, setForm] = useState<WebdavSettingsInput>({
    url: "",
    username: "",
    remoteRoot: DEFAULT_ROOT,
  });
  const [operation, setOperation] = useState<Operation>("loading");
  const [loadRevision, setLoadRevision] = useState(0);
  const [confirmDownload, setConfirmDownload] = useState(false);

  useEffect(() => {
    let active = true;
    void api
      .getWebdavSettings()
      .then((value) => {
        if (!active) return;
        setSaved(value);
        setForm(settingsForm(value));
      })
      .catch((failure: unknown) => {
        if (active) {
          notifications.error(getErrorMessage(failure, "加载 WebDAV 设置失败"));
        }
      })
      .finally(() => {
        if (active) setOperation("idle");
      });
    return () => {
      active = false;
    };
  }, [api, loadRevision, notifications]);

  const busy = operation !== "idle";
  const changed =
    saved !== null &&
    (form.url !== saved.url ||
      form.username !== saved.username ||
      form.remoteRoot !== (saved.remoteRoot || DEFAULT_ROOT) ||
      Boolean(form.password) ||
      Boolean(form.clearPassword));
  const canSync = !busy && Boolean(saved?.url) && !changed;

  async function save() {
    if (busy) return;
    notifications.clear();
    setOperation("saving");
    const input: WebdavSettingsInput = {
      url: form.url,
      username: form.username,
      remoteRoot: form.remoteRoot.trim() || DEFAULT_ROOT,
    };
    if (form.clearPassword) input.clearPassword = true;
    else if (form.password) input.password = form.password;
    try {
      const value = await api.saveWebdavSettings(input);
      setSaved(value);
      setForm(settingsForm(value));
      notifications.success("WebDAV 设置已保存");
    } catch (failure) {
      notifications.error(getErrorMessage(failure, "保存 WebDAV 设置失败"));
    } finally {
      setOperation("idle");
    }
  }

  async function synchronize(kind: "upload" | "download") {
    if (!canSync) return;
    notifications.clear();
    setOperation(kind === "upload" ? "uploading" : "downloading");
    const action = kind === "upload" ? "上传" : "下载";
    try {
      const result = await (kind === "upload"
        ? api.uploadConfiguration()
        : api.downloadConfiguration());
      notifications.success(
        `配置${action}完成：模型 ${result.models} 个，提示词 ${result.prompts} 个`,
      );
      if (kind === "download") onDownloaded?.();
    } catch (failure) {
      notifications.error(getErrorMessage(failure, `${action}配置失败`));
    } finally {
      setOperation("idle");
    }
  }

  return (
    <section className="settings-grid" aria-labelledby="webdav-settings-title">
      <h2 id="webdav-settings-title">WebDAV 配置</h2>
      {operation === "loading" ? (
        <div className="field__hint">正在加载 WebDAV 设置…</div>
      ) : !saved ? (
        <div className="button-row">
          <span className="field__hint">无法加载 WebDAV 设置</span>
          <Button
            onClick={() => {
              notifications.clear();
              setOperation("loading");
              setLoadRevision((value) => value + 1);
            }}
          >
            重试
          </Button>
        </div>
      ) : (
        <>
          <div className="field__hint">
            同步文件包含模型 API Key，请仅使用可信的 WebDAV。快捷键和 WebDAV
            密码仅保留在本机。
          </div>
          <Field
            label="WebDAV 地址"
            htmlFor="webdav-url"
            hint="使用 HTTPS 地址；仅本机回环地址支持 HTTP。"
          >
            <input
              id="webdav-url"
              type="url"
              value={form.url}
              disabled={busy}
              placeholder="https://dav.example.com/"
              autoComplete="url"
              onChange={(event) =>
                setForm({ ...form, url: event.target.value })
              }
            />
          </Field>
          <Field label="用户名" htmlFor="webdav-username">
            <input
              id="webdav-username"
              value={form.username}
              disabled={busy}
              autoComplete="username"
              onChange={(event) =>
                setForm({ ...form, username: event.target.value })
              }
            />
          </Field>
          <Field
            label="密码"
            htmlFor="webdav-password"
            hint={
              saved.hasPassword
                ? "密码已保存，留空保留现有密码。"
                : "密码会保存在本机凭据存储中。"
            }
          >
            <input
              id="webdav-password"
              type="password"
              value={form.password ?? ""}
              disabled={busy}
              autoComplete="new-password"
              onChange={(event) =>
                setForm({
                  ...form,
                  password: event.target.value,
                  clearPassword: false,
                })
              }
            />
          </Field>
          {saved.hasPassword && (
            <label className="toggle">
              <input
                type="checkbox"
                checked={form.clearPassword ?? false}
                disabled={busy}
                onChange={(event) =>
                  setForm({
                    ...form,
                    clearPassword: event.target.checked,
                    password: "",
                  })
                }
              />
              清除已保存密码
            </label>
          )}
          <Field
            label="远程根目录"
            htmlFor="webdav-remote-root"
            hint="使用相对目录，可用 / 分隔子目录；留空使用 see-see。"
          >
            <input
              id="webdav-remote-root"
              value={form.remoteRoot}
              disabled={busy}
              placeholder={DEFAULT_ROOT}
              onChange={(event) =>
                setForm({ ...form, remoteRoot: event.target.value })
              }
            />
          </Field>
          {changed && (
            <div className="field__hint">
              连接设置尚未保存，请先保存再同步。
            </div>
          )}
          <div className="button-row">
            <Button
              variant="primary"
              disabled={busy}
              aria-busy={operation === "saving"}
              onClick={() => void save()}
            >
              {operation === "saving" ? "正在保存…" : "保存 WebDAV 设置"}
            </Button>
            <Button
              disabled={!canSync}
              aria-busy={operation === "uploading"}
              onClick={() => void synchronize("upload")}
            >
              {operation === "uploading" ? "正在上传…" : "上传配置"}
            </Button>
            <Button
              disabled={!canSync}
              aria-busy={operation === "downloading"}
              onClick={() => setConfirmDownload(true)}
            >
              {operation === "downloading" ? "正在下载…" : "下载配置"}
            </Button>
          </div>
          <ConfirmDialog
            open={confirmDownload}
            title="下载并合并配置"
            description="远程模型（含 API Key）和提示词将新增或覆盖本机同 ID 或同名配置；本机独有配置和快捷键保留。"
            confirmLabel="下载并合并"
            onConfirm={() => {
              setConfirmDownload(false);
              void synchronize("download");
            }}
            onCancel={() => setConfirmDownload(false)}
          />
        </>
      )}
    </section>
  );
}
