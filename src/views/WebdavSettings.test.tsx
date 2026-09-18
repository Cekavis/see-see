import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { NotificationProvider } from "../components/Notifications";
import type {
  ConfigSyncResult,
  WebdavSettings as SavedWebdavSettings,
} from "../ipc";
import { WebdavSettings, type WebdavSettingsApi } from "./WebdavSettings";

const emptySettings: SavedWebdavSettings = {
  url: "",
  username: "",
  remoteRoot: "see-see",
  hasPassword: false,
};
const savedSettings: SavedWebdavSettings = {
  url: "https://dav.example.test/files/",
  username: "alice",
  remoteRoot: "shared/see-see",
  hasPassword: true,
};

function api(overrides: Partial<WebdavSettingsApi> = {}): WebdavSettingsApi {
  return {
    getWebdavSettings: vi.fn().mockResolvedValue(emptySettings),
    saveWebdavSettings: vi.fn().mockResolvedValue(savedSettings),
    uploadConfiguration: vi.fn().mockResolvedValue({ models: 2, prompts: 3 }),
    downloadConfiguration: vi.fn().mockResolvedValue({ models: 4, prompts: 5 }),
    ...overrides,
  };
}

function renderSettings(service: WebdavSettingsApi, onDownloaded = vi.fn()) {
  return render(
    <NotificationProvider>
      <WebdavSettings api={service} onDownloaded={onDownloaded} />
    </NotificationProvider>,
  );
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((finish) => {
    resolve = finish;
  });
  return { promise, resolve };
}

describe("WebdavSettings", () => {
  it("loads the default root and explains that API Keys are synced", async () => {
    renderSettings(api());

    expect(screen.getByText("正在加载 WebDAV 设置…")).toBeInTheDocument();
    expect(await screen.findByLabelText("远程根目录")).toHaveValue("see-see");
    expect(screen.getByLabelText("WebDAV 地址")).toHaveValue("");
    expect(screen.getByRole("button", { name: "上传配置" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "下载配置" })).toBeDisabled();
    expect(screen.getByText(/同步文件包含模型 API Key/)).toHaveTextContent(
      "请仅使用可信的 WebDAV",
    );
  });

  it("loads saved fields without echoing a password and retains it when saving an empty password", async () => {
    const service = api({
      getWebdavSettings: vi.fn().mockResolvedValue(savedSettings),
    });
    renderSettings(service);

    expect(await screen.findByLabelText("WebDAV 地址")).toHaveValue(
      savedSettings.url,
    );
    expect(screen.getByLabelText("用户名")).toHaveValue(savedSettings.username);
    expect(screen.getByLabelText("远程根目录")).toHaveValue(
      savedSettings.remoteRoot,
    );
    expect(screen.getByLabelText("密码")).toHaveAttribute("type", "password");
    expect(screen.getByLabelText("密码")).toHaveValue("");
    expect(
      screen.getByText("密码已保存，留空保留现有密码。"),
    ).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "保存 WebDAV 设置" }));

    await waitFor(() =>
      expect(service.saveWebdavSettings).toHaveBeenCalledWith({
        url: savedSettings.url,
        username: savedSettings.username,
        remoteRoot: savedSettings.remoteRoot,
      }),
    );
    expect(await screen.findByRole("status")).toHaveTextContent(
      "WebDAV 设置已保存",
    );
  });

  it("reloads saved connection fields on reopening while keeping the saved password blank", async () => {
    let stored = emptySettings;
    const service = api({
      getWebdavSettings: vi.fn(async () => stored),
      saveWebdavSettings: vi.fn(async (input) => {
        stored = {
          url: input.url,
          username: input.username,
          remoteRoot: input.remoteRoot,
          hasPassword: Boolean(input.password),
        };
        return stored;
      }),
    });
    const view = renderSettings(service);
    fireEvent.change(await screen.findByLabelText("WebDAV 地址"), {
      target: { value: savedSettings.url },
    });
    fireEvent.change(screen.getByLabelText("用户名"), {
      target: { value: savedSettings.username },
    });
    fireEvent.change(screen.getByLabelText("远程根目录"), {
      target: { value: savedSettings.remoteRoot },
    });
    fireEvent.change(screen.getByLabelText("密码"), {
      target: { value: "test-password" },
    });
    fireEvent.click(screen.getByRole("button", { name: "保存 WebDAV 设置" }));
    expect(await screen.findByRole("status")).toHaveTextContent(
      "WebDAV 设置已保存",
    );
    expect(screen.getByLabelText("密码")).toHaveValue("");
    view.unmount();

    renderSettings(service);

    expect(await screen.findByLabelText("WebDAV 地址")).toHaveValue(
      savedSettings.url,
    );
    expect(screen.getByLabelText("用户名")).toHaveValue(savedSettings.username);
    expect(screen.getByLabelText("远程根目录")).toHaveValue(
      savedSettings.remoteRoot,
    );
    expect(screen.getByLabelText("密码")).toHaveValue("");
    expect(
      screen.getByText("密码已保存，留空保留现有密码。"),
    ).toBeInTheDocument();
    expect(service.getWebdavSettings).toHaveBeenCalledTimes(2);
  });

  it("requires explicit saving before syncing changed connection fields", async () => {
    const service = api({
      getWebdavSettings: vi.fn().mockResolvedValue(savedSettings),
      saveWebdavSettings: vi
        .fn()
        .mockResolvedValue({ ...savedSettings, remoteRoot: "see-see" }),
    });
    renderSettings(service);
    const root = await screen.findByLabelText("远程根目录");
    expect(screen.getByRole("button", { name: "上传配置" })).toBeEnabled();

    fireEvent.change(root, { target: { value: "" } });

    expect(service.saveWebdavSettings).not.toHaveBeenCalled();
    expect(screen.getByRole("button", { name: "上传配置" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "下载配置" })).toBeDisabled();
    expect(
      screen.getByText("连接设置尚未保存，请先保存再同步。"),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "保存 WebDAV 设置" }));

    await waitFor(() =>
      expect(service.saveWebdavSettings).toHaveBeenCalledWith({
        url: savedSettings.url,
        username: savedSettings.username,
        remoteRoot: "see-see",
      }),
    );
    expect(await screen.findByRole("status")).toHaveTextContent(
      "WebDAV 设置已保存",
    );
    expect(root).toHaveValue("see-see");
    expect(screen.getByRole("button", { name: "上传配置" })).toBeEnabled();
    expect(
      screen.queryByText("连接设置尚未保存，请先保存再同步。"),
    ).not.toBeInTheDocument();
  });

  it("disables fields and all actions during a save and supports clearing the password", async () => {
    const saving = deferred<SavedWebdavSettings>();
    const service = api({
      getWebdavSettings: vi.fn().mockResolvedValue(savedSettings),
      saveWebdavSettings: vi.fn(() => saving.promise),
    });
    renderSettings(service);
    fireEvent.click(await screen.findByLabelText("清除已保存密码"));
    fireEvent.click(screen.getByRole("button", { name: "保存 WebDAV 设置" }));

    for (const label of [
      "WebDAV 地址",
      "用户名",
      "密码",
      "远程根目录",
      "清除已保存密码",
    ]) {
      expect(screen.getByLabelText(label)).toBeDisabled();
    }
    expect(screen.getByRole("button", { name: "正在保存…" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "上传配置" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "下载配置" })).toBeDisabled();
    expect(service.saveWebdavSettings).toHaveBeenCalledWith({
      url: savedSettings.url,
      username: savedSettings.username,
      remoteRoot: savedSettings.remoteRoot,
      clearPassword: true,
    });

    saving.resolve({ ...savedSettings, hasPassword: false });
    expect(await screen.findByRole("status")).toHaveTextContent(
      "WebDAV 设置已保存",
    );
    expect(screen.getByLabelText("WebDAV 地址")).toBeEnabled();
    expect(screen.queryByLabelText("清除已保存密码")).not.toBeInTheDocument();
  });

  it("disables fields during upload and announces the counts once", async () => {
    const uploading = deferred<ConfigSyncResult>();
    const onDownloaded = vi.fn();
    const service = api({
      getWebdavSettings: vi.fn().mockResolvedValue(savedSettings),
      uploadConfiguration: vi.fn(() => uploading.promise),
    });
    renderSettings(service, onDownloaded);
    fireEvent.click(await screen.findByRole("button", { name: "上传配置" }));

    for (const label of [
      "WebDAV 地址",
      "用户名",
      "密码",
      "远程根目录",
      "清除已保存密码",
    ]) {
      expect(screen.getByLabelText(label)).toBeDisabled();
    }
    expect(screen.getByRole("button", { name: "正在上传…" })).toHaveAttribute(
      "aria-busy",
      "true",
    );
    expect(screen.getByRole("button", { name: "下载配置" })).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "保存 WebDAV 设置" }),
    ).toBeDisabled();

    uploading.resolve({ models: 2, prompts: 3 });
    expect(await screen.findByRole("status")).toHaveTextContent(
      "配置上传完成：模型 2 个，提示词 3 个",
    );
    expect(
      screen.getAllByText("配置上传完成：模型 2 个，提示词 3 个"),
    ).toHaveLength(1);
    expect(screen.getByLabelText("WebDAV 地址")).toBeEnabled();
    expect(service.uploadConfiguration).toHaveBeenCalledOnce();
    expect(onDownloaded).not.toHaveBeenCalled();
  });

  it("confirms the merge before downloading and refreshes readiness only on success", async () => {
    const downloading = deferred<ConfigSyncResult>();
    const onDownloaded = vi.fn();
    const service = api({
      getWebdavSettings: vi.fn().mockResolvedValue(savedSettings),
      downloadConfiguration: vi.fn(() => downloading.promise),
    });
    renderSettings(service, onDownloaded);
    fireEvent.click(await screen.findByRole("button", { name: "下载配置" }));

    const dialog = screen.getByRole("dialog", { name: "下载并合并配置" });
    expect(dialog).toHaveAccessibleDescription(
      "远程模型（含 API Key）和提示词将新增或覆盖本机同 ID 或同名配置；本机独有配置和快捷键保留。",
    );
    expect(service.downloadConfiguration).not.toHaveBeenCalled();
    fireEvent.click(within(dialog).getByRole("button", { name: "取消" }));
    expect(service.downloadConfiguration).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "下载配置" }));
    fireEvent.click(within(dialog).getByRole("button", { name: "下载并合并" }));

    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "正在下载…" })).toBeDisabled();
    expect(screen.getByLabelText("WebDAV 地址")).toBeDisabled();
    expect(onDownloaded).not.toHaveBeenCalled();

    downloading.resolve({ models: 4, prompts: 5 });
    expect(await screen.findByRole("status")).toHaveTextContent(
      "配置下载完成：模型 4 个，提示词 5 个",
    );
    expect(onDownloaded).toHaveBeenCalledOnce();
    expect(screen.getByRole("button", { name: "下载配置" })).toBeEnabled();
  });

  it("allows retrying a failed settings load", async () => {
    const service = api({
      getWebdavSettings: vi
        .fn()
        .mockRejectedValueOnce(new Error("凭据存储暂不可用"))
        .mockResolvedValueOnce(savedSettings),
    });
    renderSettings(service);

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "凭据存储暂不可用",
    );
    fireEvent.click(screen.getByRole("button", { name: "重试" }));

    expect(await screen.findByLabelText("WebDAV 地址")).toHaveValue(
      savedSettings.url,
    );
    expect(service.getWebdavSettings).toHaveBeenCalledTimes(2);
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("preserves typed settings after a save failure and restores the save action", async () => {
    const service = api({
      getWebdavSettings: vi.fn().mockResolvedValue(savedSettings),
      saveWebdavSettings: vi
        .fn()
        .mockRejectedValueOnce({ message: "根目录无效" })
        .mockResolvedValueOnce(savedSettings),
    });
    renderSettings(service);
    fireEvent.change(await screen.findByLabelText("远程根目录"), {
      target: { value: "../invalid" },
    });
    fireEvent.click(screen.getByRole("button", { name: "保存 WebDAV 设置" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("根目录无效");
    expect(screen.getByLabelText("远程根目录")).toHaveValue("../invalid");
    expect(
      screen.getByRole("button", { name: "保存 WebDAV 设置" }),
    ).toBeEnabled();
    fireEvent.change(screen.getByLabelText("远程根目录"), {
      target: { value: savedSettings.remoteRoot },
    });
    fireEvent.click(screen.getByRole("button", { name: "保存 WebDAV 设置" }));

    expect(await screen.findByRole("status")).toHaveTextContent(
      "WebDAV 设置已保存",
    );
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    expect(service.saveWebdavSettings).toHaveBeenCalledTimes(2);
  });

  it("recovers from a download failure without reporting a completed import", async () => {
    const onDownloaded = vi.fn();
    const service = api({
      getWebdavSettings: vi.fn().mockResolvedValue(savedSettings),
      downloadConfiguration: vi
        .fn()
        .mockRejectedValueOnce({ message: "远程配置不存在" })
        .mockResolvedValueOnce({ models: 4, prompts: 5 }),
    });
    renderSettings(service, onDownloaded);
    fireEvent.click(await screen.findByRole("button", { name: "下载配置" }));
    fireEvent.click(screen.getByRole("button", { name: "下载并合并" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "远程配置不存在",
    );
    expect(screen.getByLabelText("WebDAV 地址")).toBeEnabled();
    expect(onDownloaded).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "下载配置" }));
    expect(service.downloadConfiguration).toHaveBeenCalledOnce();
    fireEvent.click(screen.getByRole("button", { name: "下载并合并" }));

    expect(await screen.findByRole("status")).toHaveTextContent(
      "配置下载完成：模型 4 个，提示词 5 个",
    );
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    expect(onDownloaded).toHaveBeenCalledOnce();
    expect(service.downloadConfiguration).toHaveBeenCalledTimes(2);
  });
});
