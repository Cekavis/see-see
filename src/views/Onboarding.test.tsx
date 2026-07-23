import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
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
    openView: vi.fn().mockResolvedValue(undefined),
    openScreenPermissionSettings: vi.fn().mockResolvedValue(undefined),
    ...overrides,
  };
}

describe("Onboarding", () => {
  it("shows permission/model/prompt steps and blocks completion until configured", async () => {
    const service = api();
    render(<Onboarding api={service} />);
    expect(await screen.findByText("屏幕权限已就绪")).toBeInTheDocument();
    expect(screen.getByText("尚未配置可用模型")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "完成设置" })).toBeDisabled();
    fireEvent.click(screen.getByRole("button", { name: "配置模型" }));
    expect(service.openView).toHaveBeenCalledWith("settings");
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
    render(<Onboarding api={service} />);
    fireEvent.click(
      await screen.findByRole("button", { name: "打开系统权限设置" }),
    );
    await waitFor(() =>
      expect(service.openScreenPermissionSettings).toHaveBeenCalled(),
    );
  });
});
