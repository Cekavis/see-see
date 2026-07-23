import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { SettingsShell } from "./SettingsShell";

const mocks = vi.hoisted(() => ({
  getAppSnapshot: vi.fn(),
  getSettings: vi.fn(),
  listModelConfigs: vi.fn(),
  listPromptPresets: vi.fn(),
  queryHistory: vi.fn(),
}));
const getVersion = vi.hoisted(() => vi.fn());

vi.mock("../ipc", () => ({ ipc: mocks }));
vi.mock("@tauri-apps/api/app", () => ({ getVersion }));

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
    getVersion.mockResolvedValue("0.2.0");
  });

  it("switches all settings sections locally without a capture control", async () => {
    render(<SettingsShell />);
    expect(
      await screen.findByRole("heading", { name: "常规" }),
    ).toBeInTheDocument();
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
    expect(screen.getByText("0.2.0")).toBeInTheDocument();
  });
});
