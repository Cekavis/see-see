import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { NotificationProvider } from "../components/Notifications";
import { DesktopSettings, type DesktopSettingsApi } from "./DesktopSettings";

const settings = {
  activeModelConfigId: null,
  saveHistory: true,
  autostart: false,
  resultAlwaysOnTop: true,
  onboardingCompleted: true,
};

function api(overrides: Partial<DesktopSettingsApi> = {}): DesktopSettingsApi {
  return {
    getSettings: vi.fn().mockResolvedValue(settings),
    setAutostart: vi.fn().mockResolvedValue({ ...settings, autostart: true }),
    openLoginItemsSettings: vi.fn().mockResolvedValue(undefined),
    setSaveHistory: vi
      .fn()
      .mockResolvedValue({ ...settings, saveHistory: false }),
    exportSanitizedLogs: vi.fn().mockResolvedValue({ exported: true }),
    ...overrides,
  };
}

function renderSettings(service: DesktopSettingsApi) {
  return render(
    <NotificationProvider>
      <DesktopSettings api={service} />
    </NotificationProvider>,
  );
}

describe("DesktopSettings", () => {
  it("has no application-wide capture shortcut control", async () => {
    renderSettings(api());
    await screen.findByLabelText("开机启动");
    expect(
      screen.queryByRole("button", { name: "截图快捷键" }),
    ).not.toBeInTheDocument();
  });
  it("syncs autostart, history preference, and exports sanitized logs", async () => {
    const service = api();
    renderSettings(service);
    fireEvent.click(await screen.findByLabelText("开机启动"));
    fireEvent.click(screen.getByLabelText("保存历史记录"));
    fireEvent.click(screen.getByRole("button", { name: "导出诊断日志" }));
    await waitFor(() =>
      expect(service.setAutostart).toHaveBeenCalledWith(true),
    );
    expect(service.setSaveHistory).toHaveBeenCalledWith(false);
    expect(service.exportSanitizedLogs).toHaveBeenCalled();
    expect(await screen.findByText("诊断日志已导出")).toBeInTheDocument();
    expect(
      screen.queryByText("登录系统后自动启动 See See。"),
    ).not.toBeInTheDocument();
    expect(
      screen.getByText("原始截图、结果和提示词快照仅保存在本机。"),
    ).toBeInTheDocument();
  });

  it("opens macOS Login Items when autostart requires approval", async () => {
    const service = api({
      setAutostart: vi.fn().mockRejectedValue({
        code: "autostart_approval_required",
        message: "请在系统设置的“登录项与扩展”中允许 See See",
        action: "open_login_items",
      }),
    });
    renderSettings(service);

    fireEvent.click(await screen.findByLabelText("开机启动"));
    fireEvent.click(
      await screen.findByRole("button", { name: "打开系统设置" }),
    );

    expect(service.openLoginItemsSettings).toHaveBeenCalledOnce();
  });
});
