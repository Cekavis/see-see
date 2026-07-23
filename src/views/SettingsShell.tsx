import { getVersion } from "@tauri-apps/api/app";
import { useEffect, useState } from "react";
import { Icon, type IconName } from "../components/Icon";
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
        <p>管理首次设置、全局快捷键和本地应用偏好。</p>
      </header>
      <div className="settings-groups">
        <Onboarding onSelectSection={onSelect} />
        <DesktopSettings />
      </div>
    </section>
  );
}

function About() {
  const [version, setVersion] = useState("0.2.0");

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
          <p>用全局快捷键截取屏幕区域，并交给你配置的多模态模型。</p>
        </div>
      </header>
      <div className="settings-group">
        <dl>
          <dt>版本</dt>
          <dd>{version}</dd>
          <dt>数据与隐私</dt>
          <dd>模型凭据保存在系统凭据存储中；历史记录仅在启用后保存在本机。</dd>
        </dl>
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
