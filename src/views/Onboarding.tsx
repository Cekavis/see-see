import { useCallback, useEffect, useState } from "react";
import { Button } from "../components/Button";
import { ErrorNotice } from "../components/ErrorNotice";
import { ipc, type AppError, type AppSnapshot } from "../ipc";

export type OnboardingApi = {
  getAppSnapshot: () => Promise<AppSnapshot>;
  completeOnboarding: () => Promise<void>;
  openScreenPermissionSettings: () => Promise<void>;
};

export function Onboarding({
  api = ipc,
  onSelectSection,
}: {
  api?: OnboardingApi;
  onSelectSection: (section: "models" | "prompts") => void;
}) {
  const [snapshot, setSnapshot] = useState<AppSnapshot>();
  const [error, setError] = useState<string>();
  const refresh = useCallback(
    () =>
      api
        .getAppSnapshot()
        .then(setSnapshot)
        .catch((failure: AppError) => setError(failure.message)),
    [api],
  );
  useEffect(() => {
    void refresh();
  }, [refresh]);
  if (!snapshot)
    return (
      <section className="settings-group onboarding" aria-label="首次设置">
        {error ? (
          <ErrorNotice message={error} onRetry={() => void refresh()} />
        ) : (
          <p>正在检查桌面环境…</p>
        )}
      </section>
    );
  if (snapshot.settings.onboardingCompleted) return null;
  const permissionReady = snapshot.screenPermission === "granted";
  const modelReady = Boolean(snapshot.activeModelConfigId);
  const promptReady = Boolean(snapshot.activePromptId);
  const ready = permissionReady && modelReady && promptReady;
  return (
    <section
      className="settings-group onboarding"
      aria-labelledby="onboarding-title"
    >
      <header>
        <h2 id="onboarding-title">欢迎使用 See See</h2>
        <p>完成三项本地设置后，即可用快捷键直接把截图交给多模态模型。</p>
      </header>
      {error && <ErrorNotice message={error} />}
      <ol className="onboarding-steps">
        <li>
          <h2>1. 屏幕截图权限</h2>
          <p>
            {permissionReady
              ? "屏幕权限已就绪"
              : snapshot.screenPermission === "denied"
                ? "屏幕权限被拒绝"
                : "尚未确认屏幕权限"}
          </p>
          {!permissionReady && (
            <Button
              onClick={() =>
                void api.openScreenPermissionSettings().then(refresh)
              }
            >
              打开系统权限设置
            </Button>
          )}
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
            .then(() =>
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
              ),
            )
            .catch((failure: AppError) => setError(failure.message))
        }
      >
        完成设置
      </Button>
    </section>
  );
}
