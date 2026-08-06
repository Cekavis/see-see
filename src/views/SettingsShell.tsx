import { getVersion } from "@tauri-apps/api/app";
import { relaunch } from "@tauri-apps/plugin-process";
import {
  check,
  type DownloadEvent,
  type Update,
} from "@tauri-apps/plugin-updater";
import { useEffect, useRef, useState } from "react";
import packageJson from "../../package.json";
import { Button } from "../components/Button";
import { Icon, type IconName } from "../components/Icon";
import { useNotifications } from "../components/Notifications";
import { getErrorMessage } from "../ipc";
import { DesktopSettings } from "./DesktopSettings";
import { History } from "./History";
import { Onboarding } from "./Onboarding";
import { Prompts } from "./Prompts";
import { Settings } from "./Settings";

export type SettingsSection =
  "general" | "models" | "prompts" | "history" | "about";

const sections: Array<{
  id: SettingsSection;
  label: string;
  icon: IconName;
}> = [
  { id: "general", label: "常规", icon: "general" },
  { id: "models", label: "模型", icon: "models" },
  { id: "prompts", label: "提示词", icon: "prompts" },
  { id: "history", label: "历史", icon: "history" },
  { id: "about", label: "关于", icon: "about" },
];

function General({ onSelect }: { onSelect: (value: SettingsSection) => void }) {
  return (
    <section
      className="settings-section"
      aria-labelledby="general-section-title"
    >
      <header className="settings-section__header">
        <h1 id="general-section-title">常规</h1>
      </header>
      <div className="settings-groups">
        <Onboarding onSelectSection={onSelect} />
        <DesktopSettings />
      </div>
    </section>
  );
}

function About() {
  const notifications = useNotifications();
  const [version, setVersion] = useState(packageJson.version);
  const [update, setUpdate] = useState<Update | null>(null);
  const [status, setStatus] = useState<
    | "idle"
    | "checking"
    | "current"
    | "available"
    | "installing"
    | "restarting"
    | "failed"
  >("idle");
  const [progress, setProgress] = useState<{
    downloaded: number;
    total?: number;
  } | null>(null);
  const mounted = useRef(true);

  useEffect(() => {
    let active = true;
    void getVersion()
      .then((value) => {
        if (active) setVersion(value);
      })
      .catch(() => undefined);
    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);

  const busy = ["checking", "installing", "restarting"].includes(status);

  async function checkForUpdates() {
    if (busy) return;
    notifications.clear();
    setStatus("checking");
    setProgress(null);
    try {
      const available = await check({ timeout: 10_000 });
      if (!mounted.current) return;
      setUpdate(available);
      setStatus(available ? "available" : "current");
    } catch (failure) {
      if (!mounted.current) return;
      setUpdate(null);
      setStatus("failed");
      notifications.error(getErrorMessage(failure, "检查更新失败"), {
        action: { label: "重试", onClick: () => void checkForUpdates() },
      });
    }
  }

  async function installUpdate() {
    if (!update || busy) return;
    notifications.clear();
    setStatus("installing");
    setProgress({ downloaded: 0 });
    let downloaded = 0;
    let total: number | undefined;
    try {
      await update.downloadAndInstall((event: DownloadEvent) => {
        if (event.event === "Started") total = event.data.contentLength;
        if (event.event === "Progress") downloaded += event.data.chunkLength;
        if (mounted.current) setProgress({ downloaded, total });
      });
      if (mounted.current) setStatus("restarting");
      await relaunch();
    } catch (failure) {
      if (!mounted.current) return;
      setStatus("available");
      notifications.error(getErrorMessage(failure, "安装更新失败"));
    }
  }

  const statusText = (() => {
    if (status === "checking") return "正在检查更新…";
    if (status === "current") return "已是最新版本";
    if (status === "available") return `发现新版本 ${update?.version}`;
    if (status === "restarting") return "更新已安装，正在重启…";
    if (status === "failed") return "检查失败，可重试";
    if (status === "installing") {
      if (progress?.total) {
        return `正在下载 ${Math.min(
          100,
          Math.round((progress.downloaded / progress.total) * 100),
        )}%…`;
      }
      return "正在下载安装…";
    }
    return null;
  })();

  const buttonLabel = update
    ? `安装 ${update.version}`
    : status === "current"
      ? "再次检查"
      : status === "failed"
        ? "重新检查"
        : "检查更新";

  return (
    <section
      className="settings-section about-view"
      aria-labelledby="about-title"
    >
      <header className="settings-section__header">
        <span className="about-view__mark" aria-hidden="true">
          <Icon name="brand" />
        </span>
        <div>
          <h1 id="about-title">关于 See See</h1>
        </div>
      </header>
      <div className="settings-group">
        <dl>
          <dt>版本</dt>
          <dd>{version}</dd>
          <dt>数据与隐私</dt>
          <dd>
            API Key 与模型端点以明文保存在本机；历史记录仅在启用后保存在本机。
          </dd>
        </dl>
        <div className="setting-row about-update">
          <div className="setting-row__body" aria-live="polite">
            <strong>软件更新</strong>
            {statusText ? (
              <span className="field__hint">{statusText}</span>
            ) : null}
            {status === "available" && update?.body ? (
              <pre className="about-update__notes">{update.body}</pre>
            ) : null}
          </div>
          <Button
            disabled={busy}
            aria-busy={busy}
            onClick={() => void (update ? installUpdate() : checkForUpdates())}
          >
            {buttonLabel}
          </Button>
        </div>
      </div>
    </section>
  );
}

export function SettingsShell() {
  const [section, setSection] = useState<SettingsSection>("general");

  return (
    <main className="settings-shell">
      <aside className="settings-sidebar">
        <div className="settings-brand">
          <span className="settings-brand__mark" aria-hidden="true">
            <Icon name="brand" />
          </span>
          <span>See See</span>
        </div>
        <nav className="settings-nav" aria-label="设置栏目">
          {sections.map((item) => {
            const active = section === item.id;
            return (
              <button
                key={item.id}
                type="button"
                className={`settings-nav__item${active ? " settings-nav__item--active" : ""}`}
                aria-current={active ? "page" : undefined}
                onClick={() => setSection(item.id)}
              >
                <span
                  className={`settings-nav__icon settings-nav__icon--${item.id}`}
                  aria-hidden="true"
                >
                  <Icon name={item.icon} />
                </span>
                <span>{item.label}</span>
              </button>
            );
          })}
        </nav>
      </aside>
      <div className="settings-content">
        {section === "general" && <General onSelect={setSection} />}
        {section === "models" && <Settings />}
        {section === "prompts" && <Prompts />}
        {section === "history" && <History />}
        {section === "about" && <About />}
      </div>
    </main>
  );
}
