import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { NotificationProvider } from "../components/Notifications";
import { DesktopSettings, type DesktopSettingsApi } from "./DesktopSettings";

const settings = {
  activeModelConfigId: null,
  activePromptId: "p1",
  captureShortcut: "Alt+Shift+A",
  saveHistory: true,
  autostart: false,
  resultAlwaysOnTop: true,
  onboardingCompleted: true,
};

function api(overrides: Partial<DesktopSettingsApi> = {}): DesktopSettingsApi {
  return {
    getSettings: vi.fn().mockResolvedValue(settings),
    setCaptureShortcut: vi
      .fn()
      .mockResolvedValue({ ...settings, captureShortcut: "Ctrl+Shift+X" }),
    setAutostart: vi.fn().mockResolvedValue({ ...settings, autostart: true }),
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
  it("defaults autostart off and updates shortcut without losing the old value on conflict", async () => {
    const service = api({
      setCaptureShortcut: vi.fn().mockRejectedValue({
        code: "shortcut_conflict",
        message: "快捷键已占用",
      }),
    });
    renderSettings(service);
    expect(await screen.findByLabelText("开机启动")).not.toBeChecked();
    fireEvent.change(screen.getByLabelText("截图快捷键"), {
      target: { value: "Taken" },
    });
    fireEvent.click(screen.getByRole("button", { name: "保存快捷键" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("快捷键已占用");
    expect(screen.getByLabelText("截图快捷键")).toHaveValue("Alt+Shift+A");
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
  });
});
