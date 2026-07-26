import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { NotificationProvider } from "../components/Notifications";
import { DesktopSettings, type DesktopSettingsApi } from "./DesktopSettings";

const settings = {
  activeModelConfigId: null,
  activePromptId: "p1",
  captureShortcut: "Command+Shift+X",
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
  it("captures a shortcut and applies it immediately", async () => {
    const setCaptureShortcut = vi.fn().mockResolvedValue({
      ...settings,
      captureShortcut: "Command+Shift+K",
    });
    const service = api({ setCaptureShortcut });
    renderSettings(service);
    const recorder = await screen.findByRole("button", {
      name: "截图快捷键",
    });

    fireEvent.click(recorder);
    expect(recorder).toHaveAttribute("aria-pressed", "true");
    fireEvent.keyDown(window, {
      code: "MetaLeft",
      key: "Meta",
      metaKey: true,
    });
    expect(setCaptureShortcut).not.toHaveBeenCalled();
    fireEvent.keyDown(window, {
      code: "",
      key: "k",
      metaKey: true,
      shiftKey: true,
    });

    await waitFor(() =>
      expect(setCaptureShortcut).toHaveBeenCalledWith("Command+Shift+K"),
    );
    expect(await screen.findByText("快捷键已保存并生效")).toBeInTheDocument();
    expect(recorder).toHaveTextContent("Command+Shift+K");
  });

  it("keeps the old shortcut when the captured combination conflicts", async () => {
    const service = api({
      setCaptureShortcut: vi.fn().mockRejectedValue({
        code: "shortcut_conflict",
        message: "快捷键已占用",
      }),
    });
    renderSettings(service);
    expect(await screen.findByLabelText("开机启动")).not.toBeChecked();
    const recorder = screen.getByRole("button", { name: "截图快捷键" });
    fireEvent.click(recorder);
    fireEvent.keyDown(window, {
      code: "KeyX",
      key: "x",
      ctrlKey: true,
      shiftKey: true,
    });
    expect(await screen.findByRole("alert")).toHaveTextContent("快捷键已占用");
    expect(service.setCaptureShortcut).toHaveBeenCalledWith("Ctrl+Shift+X");
    expect(recorder).toHaveTextContent("Command+Shift+X");
  });

  it("cancels recording with Escape and removes the window listener", async () => {
    const service = api();
    renderSettings(service);
    const recorder = await screen.findByRole("button", {
      name: "截图快捷键",
    });

    fireEvent.click(recorder);
    fireEvent.keyDown(window, { code: "Escape", key: "Escape" });
    expect(recorder).toHaveAttribute("aria-pressed", "false");
    fireEvent.keyDown(window, {
      code: "KeyK",
      key: "k",
      metaKey: true,
      shiftKey: true,
    });
    expect(service.setCaptureShortcut).not.toHaveBeenCalled();
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
});
