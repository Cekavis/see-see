import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import packageJson from "../../package.json";
import { NotificationProvider } from "../components/Notifications";
import { SettingsShell } from "./SettingsShell";

const mocks = vi.hoisted(() => ({
  getAppSnapshot: vi.fn(),
  getSettings: vi.fn(),
  listModelConfigs: vi.fn(),
  listPromptPresets: vi.fn(),
  queryHistory: vi.fn(),
}));
const getVersion = vi.hoisted(() => vi.fn());
const check = vi.hoisted(() => vi.fn());
const relaunch = vi.hoisted(() => vi.fn());
const listen = vi.hoisted(() => vi.fn().mockResolvedValue(vi.fn()));

vi.mock("../ipc", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../ipc")>()),
  ipc: mocks,
}));
vi.mock("@tauri-apps/api/event", () => ({ listen }));
vi.mock("@tauri-apps/api/app", () => ({ getVersion }));
vi.mock("@tauri-apps/plugin-updater", () => ({ check }));
vi.mock("@tauri-apps/plugin-process", () => ({ relaunch }));

const settings = {
  activeModelConfigId: "m1",
  activePromptId: "p1",
  captureShortcut: "Alt+Shift+A",
  saveHistory: true,
  autostart: false,
  resultAlwaysOnTop: true,
  onboardingCompleted: true,
};

describe("SettingsShell", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.getAppSnapshot.mockResolvedValue({
      settings,
      promptCount: 1,
      modelConfigCount: 1,
      activePromptId: "p1",
      activeModelConfigId: "m1",
      screenPermission: "granted",
    });
    mocks.getSettings.mockResolvedValue(settings);
    mocks.listModelConfigs.mockResolvedValue([]);
    mocks.listPromptPresets.mockResolvedValue([]);
    mocks.queryHistory.mockResolvedValue({ items: [], nextCursor: null });
    getVersion.mockResolvedValue(packageJson.version);
    check.mockResolvedValue(null);
    relaunch.mockResolvedValue(undefined);
  });

  it("switches all settings sections locally without a capture control", async () => {
    render(
      <NotificationProvider>
        <SettingsShell />
      </NotificationProvider>,
    );
    expect(
      await screen.findByRole("heading", { name: "常规" }),
    ).toBeInTheDocument();
    expect(
      screen.queryByText("管理首次设置、全局快捷键和本地应用偏好。"),
    ).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "常规" })).toHaveAttribute(
      "aria-current",
      "page",
    );
    expect(
      screen.queryByRole("button", { name: "开始截图" }),
    ).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "模型" }));
    expect(
      await screen.findByRole("heading", { name: "模型" }),
    ).toBeInTheDocument();
    expect(mocks.listModelConfigs).toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: "提示词" }));
    expect(
      await screen.findByRole("heading", { name: "提示词" }),
    ).toBeInTheDocument();
    expect(mocks.listPromptPresets).toHaveBeenCalled();

    fireEvent.click(screen.getByRole("button", { name: "历史" }));
    expect(
      await screen.findByRole("heading", { name: "历史记录" }),
    ).toBeInTheDocument();
    expect(mocks.queryHistory).toHaveBeenCalledWith({});

    fireEvent.click(screen.getByRole("button", { name: "关于" }));
    expect(
      await screen.findByRole("heading", { name: "关于 See See" }),
    ).toBeInTheDocument();
    await waitFor(() => expect(getVersion).toHaveBeenCalled());
    expect(screen.getByText(packageJson.version)).toBeInTheDocument();
    expect(
      screen.getByText(/API Key 与模型端点以明文保存在本机/),
    ).toBeInTheDocument();
    expect(
      screen.queryByText(/用全局快捷键截取屏幕区域/),
    ).not.toBeInTheDocument();
  });

  it("keeps the packaged version when the native version call fails", async () => {
    getVersion.mockRejectedValue(new Error("unavailable"));
    render(
      <NotificationProvider>
        <SettingsShell />
      </NotificationProvider>,
    );

    fireEvent.click(screen.getByRole("button", { name: "关于" }));
    expect(await screen.findByText(packageJson.version)).toBeInTheDocument();
  });

  it("checks for updates and shows current or available release details", async () => {
    const downloadAndInstall = vi.fn().mockResolvedValue(undefined);
    check.mockResolvedValueOnce(null).mockResolvedValueOnce({
      version: "0.8.0",
      body: "新增一键更新\n修复发布流程",
      downloadAndInstall,
    });
    render(
      <NotificationProvider>
        <SettingsShell />
      </NotificationProvider>,
    );

    fireEvent.click(screen.getByRole("button", { name: "关于" }));
    const button = screen.getByRole("button", { name: "检查更新" });
    fireEvent.click(button);
    expect(await screen.findByText("已是最新版本")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "再次检查" }));
    expect(
      await screen.findByRole("button", { name: "安装 0.8.0" }),
    ).toBeInTheDocument();
    expect(screen.getByText(/新增一键更新/)).toHaveTextContent(
      "新增一键更新 修复发布流程",
    );
    expect(downloadAndInstall).not.toHaveBeenCalled();
  });

  it("reports install progress, blocks duplicate actions, and relaunches", async () => {
    let finishInstall: (() => void) | undefined;
    const downloadAndInstall = vi.fn(
      (onEvent: (event: unknown) => void) =>
        new Promise<void>((resolve) => {
          finishInstall = resolve;
          onEvent({ event: "Started", data: { contentLength: 100 } });
          onEvent({ event: "Progress", data: { chunkLength: 40 } });
        }),
    );
    check.mockResolvedValue({
      version: "0.8.0",
      body: "更新说明",
      downloadAndInstall,
    });
    render(
      <NotificationProvider>
        <SettingsShell />
      </NotificationProvider>,
    );

    fireEvent.click(screen.getByRole("button", { name: "关于" }));
    fireEvent.click(screen.getByRole("button", { name: "检查更新" }));
    const install = await screen.findByRole("button", { name: "安装 0.8.0" });
    fireEvent.click(install);

    expect(await screen.findByText("正在下载 40%…")).toBeInTheDocument();
    expect(install).toBeDisabled();
    fireEvent.click(install);
    expect(downloadAndInstall).toHaveBeenCalledOnce();

    finishInstall?.();
    await waitFor(() => expect(relaunch).toHaveBeenCalledOnce());
  });

  it("keeps the current app usable when update checks or installs fail", async () => {
    const downloadAndInstall = vi.fn().mockRejectedValue(new Error("安装失败"));
    check.mockRejectedValueOnce(new Error("网络不可用")).mockResolvedValueOnce({
      version: "0.8.0",
      body: null,
      downloadAndInstall,
    });
    render(
      <NotificationProvider>
        <SettingsShell />
      </NotificationProvider>,
    );

    fireEvent.click(screen.getByRole("button", { name: "关于" }));
    fireEvent.click(screen.getByRole("button", { name: "检查更新" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("网络不可用");

    fireEvent.click(screen.getByRole("button", { name: "重试" }));
    fireEvent.click(await screen.findByRole("button", { name: "安装 0.8.0" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("安装失败");
    expect(screen.getByRole("button", { name: "安装 0.8.0" })).toBeEnabled();
    expect(relaunch).not.toHaveBeenCalled();
  });
});
