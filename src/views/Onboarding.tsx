import { useCallback, useEffect, useState } from "react";
import { Button } from "../components/Button";
import { useNotifications } from "../components/Notifications";
import {
  ipc,
  type AppError,
  type AppSnapshot,
  type ScreenPermission,
} from "../ipc";

export type OnboardingApi = {
  getAppSnapshot: () => Promise<AppSnapshot>;
  completeOnboarding: () => Promise<void>;
  requestScreenPermission: () => Promise<ScreenPermission>;
  openScreenPermissionSettings: () => Promise<void>;
};

export function Onboarding({
  api = ipc,
  onSelectSection,
}: {
  api?: OnboardingApi;
  onSelectSection: (section: "models" | "prompts") => void;
}) {
  const notifications = useNotifications();
  const [snapshot, setSnapshot] = useState<AppSnapshot>();
  const [requestingPermission, setRequestingPermission] = useState(false);
  const refresh = useCallback(() => {
    function run() {
      void api
        .getAppSnapshot()
        .then((value) =>
          setSnapshot((current) =>
            current?.screenPermission === "denied" &&
            value.screenPermission === "unknown"
              ? { ...value, screenPermission: "denied" }
              : value,
          ),
        )
        .catch((failure: AppError) =>
          notifications.error(failure.message, {
            action: { label: "重试", onClick: run },
          }),
        );
    }
    run();
  }, [api, notifications]);
  useEffect(() => {
    refresh();
  }, [refresh]);
  useEffect(() => {
    window.addEventListener("focus", refresh);
    return () => window.removeEventListener("focus", refresh);
  }, [refresh]);
  if (!snapshot)
    return (
      <section className="settings-group onboarding" aria-label="首次设置">
        <p>正在检查桌面环境…</p>
      </section>
    );
  const permissionReady = snapshot.screenPermission === "granted";
  const modelReady = Boolean(snapshot.activeModelConfigId);
  const promptReady = Boolean(snapshot.activePromptId);
  const ready = permissionReady && modelReady && promptReady;
  const permissionMessage = permissionReady
    ? "屏幕权限已就绪"
    : snapshot.screenPermission === "denied"
      ? "屏幕录制权限尚未授予，请在系统设置中允许 See See"
      : "尚未请求屏幕录制权限";
  const permissionActions =
    permissionReady ? null : snapshot.screenPermission === "denied" ? (
      <div className="onboarding-permission-actions">
        <Button
          onClick={() =>
            void api
              .openScreenPermissionSettings()
              .catch((failure: AppError) =>
                notifications.error(failure.message),
              )
          }
        >
          打开系统权限设置
        </Button>
        <Button onClick={refresh}>重新检查权限</Button>
      </div>
    ) : (
      <Button
        disabled={requestingPermission}
        onClick={() => {
          setRequestingPermission(true);
          void api
            .requestScreenPermission()
            .then((screenPermission) =>
              setSnapshot((current) =>
                current ? { ...current, screenPermission } : current,
              ),
            )
            .catch((failure: AppError) => notifications.error(failure.message))
            .finally(() => setRequestingPermission(false));
        }}
      >
        {requestingPermission ? "正在请求…" : "请求屏幕录制权限"}
      </Button>
    );
  if (snapshot.settings.onboardingCompleted) {
    if (permissionReady) return null;
    return (
      <section
        className="settings-group onboarding"
        aria-labelledby="screen-permission-title"
      >
        <header>
          <h2 id="screen-permission-title">屏幕录制权限需要恢复</h2>
          <p>{permissionMessage}</p>
        </header>
        {permissionActions}
      </section>
    );
  }
  return (
    <section
      className="settings-group onboarding"
      aria-labelledby="onboarding-title"
    >
      <header>
        <h2 id="onboarding-title">欢迎使用 See See</h2>
        <p>完成三项本地设置后，即可用快捷键直接把截图交给多模态模型。</p>
      </header>
      <ol className="onboarding-steps">
        <li>
          <h2>1. 屏幕截图权限</h2>
          <p>{permissionMessage}</p>
          {permissionActions}
        </li>
        <li>
          <h2>2. 多模态模型</h2>
          <p>{modelReady ? "已选择测试通过的模型" : "尚未配置可用模型"}</p>
          <Button onClick={() => onSelectSection("models")}>配置模型</Button>
        </li>
        <li>
          <h2>3. 提示词</h2>
          <p>{promptReady ? "已选择提示词" : "尚未选择提示词"}</p>
          <Button onClick={() => onSelectSection("prompts")}>管理提示词</Button>
        </li>
      </ol>
      <Button
        variant="primary"
        disabled={!ready}
        onClick={() =>
          void api
            .completeOnboarding()
            .then(() => {
              notifications.success("首次设置已完成");
              setSnapshot((current) =>
                current
                  ? {
                      ...current,
                      settings: {
                        ...current.settings,
                        onboardingCompleted: true,
                      },
                    }
                  : current,
              );
            })
            .catch((failure: AppError) => notifications.error(failure.message))
        }
      >
        完成设置
      </Button>
    </section>
  );
}
