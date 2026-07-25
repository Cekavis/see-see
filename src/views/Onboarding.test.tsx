import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { NotificationProvider } from "../components/Notifications";
import { Onboarding, type OnboardingApi } from "./Onboarding";

function api(overrides: Partial<OnboardingApi> = {}): OnboardingApi {
  return {
    getAppSnapshot: vi.fn().mockResolvedValue({
      settings: {
        activeModelConfigId: null,
        activePromptId: "p1",
        captureShortcut: "Alt+Shift+A",
        saveHistory: true,
        autostart: false,
        resultAlwaysOnTop: true,
        onboardingCompleted: false,
      },
      promptCount: 2,
      modelConfigCount: 0,
      activePromptId: "p1",
      activeModelConfigId: null,
      screenPermission: "granted",
    }),
    completeOnboarding: vi.fn().mockResolvedValue(undefined),
    openScreenPermissionSettings: vi.fn().mockResolvedValue(undefined),
    ...overrides,
  };
}

function renderOnboarding(service: OnboardingApi, onSelectSection = vi.fn()) {
  return render(
    <NotificationProvider>
      <Onboarding api={service} onSelectSection={onSelectSection} />
    </NotificationProvider>,
  );
}

describe("Onboarding", () => {
  it("shows permission/model/prompt steps and blocks completion until configured", async () => {
    const service = api();
    const onSelectSection = vi.fn();
    renderOnboarding(service, onSelectSection);
    expect(await screen.findByText("屏幕权限已就绪")).toBeInTheDocument();
    expect(screen.getByText("尚未配置可用模型")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "完成设置" })).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "配置模型" }));
    expect(onSelectSection).toHaveBeenCalledWith("models");
    fireEvent.click(screen.getByRole("button", { name: "管理提示词" }));
    expect(onSelectSection).toHaveBeenCalledWith("prompts");
  });

  it("offers recovery when screen permission is denied", async () => {
    const service = api({
      getAppSnapshot: vi.fn().mockResolvedValue({
        settings: {
          activeModelConfigId: "m1",
          activePromptId: "p1",
          captureShortcut: "Alt+Shift+A",
          saveHistory: true,
          autostart: false,
          resultAlwaysOnTop: true,
          onboardingCompleted: false,
        },
        promptCount: 2,
        modelConfigCount: 1,
        activePromptId: "p1",
        activeModelConfigId: "m1",
        screenPermission: "denied",
      }),
    });
    renderOnboarding(service);
    fireEvent.click(
      await screen.findByRole("button", { name: "打开系统权限设置" }),
    );
    await waitFor(() =>
      expect(service.openScreenPermissionSettings).toHaveBeenCalled(),
    );
  });

  it("completes ready onboarding without reloading the window", async () => {
    const service = api({
      getAppSnapshot: vi.fn().mockResolvedValue({
        settings: {
          activeModelConfigId: "m1",
          activePromptId: "p1",
          captureShortcut: "Alt+Shift+A",
          saveHistory: true,
          autostart: false,
          resultAlwaysOnTop: true,
          onboardingCompleted: false,
        },
        promptCount: 1,
        modelConfigCount: 1,
        activePromptId: "p1",
        activeModelConfigId: "m1",
        screenPermission: "granted",
      }),
    });
    renderOnboarding(service);
    fireEvent.click(await screen.findByRole("button", { name: "完成设置" }));
    await waitFor(() => expect(service.completeOnboarding).toHaveBeenCalled());
    expect(
      screen.queryByRole("heading", { name: "欢迎使用 See See" }),
    ).not.toBeInTheDocument();
  });

  it("publishes a recoverable loading error in the shared notification layer", async () => {
    const service = api();
    vi.mocked(service.getAppSnapshot)
      .mockRejectedValueOnce({ message: "桌面环境读取失败" })
      .mockResolvedValueOnce(await api().getAppSnapshot());
    renderOnboarding(service);

    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("桌面环境读取失败");
    fireEvent.click(screen.getByRole("button", { name: "重试" }));
    expect(await screen.findByText("屏幕权限已就绪")).toBeInTheDocument();
  });
});
