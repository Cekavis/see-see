import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";
import type { ModelConfigSummary } from "../ipc";
import { ModelRetryDialog } from "./ModelRetryDialog";

function model(
  id: string,
  overrides: Partial<ModelConfigSummary> = {},
): ModelConfigSummary {
  return {
    id,
    name: `模型配置 ${id}`,
    protocol: "openai",
    baseUrl: "https://example.com/v1",
    modelId: `vision-${id}`,
    reasoningEffort: null,
    hasApiKey: true,
    isActive: false,
    ...overrides,
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

describe("ModelRetryDialog", () => {
  it("shows 500 name-only rows without search or counts and selects by ID", async () => {
    const models = Array.from({ length: 500 }, (_, index) => model(`${index}`));
    models[0] = model("0", { isActive: true, hasApiKey: false });
    models[498] = model("498", { name: "TEAM 同名配置", protocol: "gemini" });
    models[499] = model("499", {
      name: "TEAM 同名配置",
      protocol: "anthropic",
    });
    const loadModels = vi.fn().mockResolvedValue(models);
    const props = {
      loadModels,
      onSelect: vi.fn().mockResolvedValue({ runId: "new-run" }),
      onClose: vi.fn(),
    };
    const { rerender } = render(<ModelRetryDialog open={false} {...props} />);

    expect(loadModels).not.toHaveBeenCalled();
    rerender(<ModelRetryDialog open {...props} />);

    expect(screen.getByRole("dialog", { name: "选择重试模型" })).toBeVisible();
    expect(screen.queryByRole("searchbox")).not.toBeInTheDocument();
    const list = await screen.findByRole("list", { name: "可用模型" });
    expect(within(list).getAllByRole("button")).toHaveLength(500);
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
    const first = within(list).getByRole("button", {
      name: "模型配置 0",
    });
    expect(first).toBeEnabled();
    await waitFor(() => expect(first).toHaveFocus());
    expect(first).toHaveTextContent(/^模型配置 0$/);
    expect(first).toHaveAttribute("title", "模型配置 0");

    const duplicates = within(list).getAllByRole("button", {
      name: "TEAM 同名配置",
    });
    expect(duplicates).toHaveLength(2);
    expect(duplicates[1]).toHaveTextContent(/^TEAM 同名配置$/);
    fireEvent.keyDown(first, { key: "End" });
    expect(duplicates[1]).toHaveFocus();
    fireEvent.click(duplicates[1]);
    expect(props.onSelect).toHaveBeenCalledExactlyOnceWith("499");
    await waitFor(() => expect(props.onClose).toHaveBeenCalledOnce());
  });

  it("keeps loading quiet and shows the empty state without offering a model selection", async () => {
    const request = deferred<ModelConfigSummary[]>();
    render(
      <ModelRetryDialog
        open
        loadModels={() => request.promise}
        onSelect={vi.fn()}
        onClose={vi.fn()}
      />,
    );

    expect(screen.queryByRole("status")).not.toBeInTheDocument();
    expect(screen.queryByText(/正在/)).not.toBeInTheDocument();
    expect(screen.getAllByRole("button")).toHaveLength(1);
    expect(screen.getByRole("button", { name: "取消" })).toBeEnabled();
    await act(async () => request.resolve([]));
    expect(screen.getByRole("status")).toHaveTextContent("尚未配置模型");
    expect(screen.queryByRole("list")).not.toBeInTheDocument();
  });

  it("recovers from a model-list failure with an explicit retry", async () => {
    const loadModels = vi
      .fn()
      .mockRejectedValueOnce(new Error("无法读取模型配置"))
      .mockResolvedValueOnce([model("new")]);
    render(
      <ModelRetryDialog
        open
        loadModels={loadModels}
        onSelect={vi.fn()}
        onClose={vi.fn()}
      />,
    );

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "无法读取模型配置",
    );
    fireEvent.click(screen.getByRole("button", { name: "重试" }));
    expect(
      await screen.findByRole("button", { name: /模型配置 new/ }),
    ).toBeEnabled();
    expect(loadModels).toHaveBeenCalledTimes(2);
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("submits immediately only once while pending and closes after success", async () => {
    const request = deferred<unknown>();
    const onSelect = vi.fn(() => request.promise);
    const onClose = vi.fn();
    render(
      <ModelRetryDialog
        open
        loadModels={async () => [model("first"), model("second")]}
        onSelect={onSelect}
        onClose={onClose}
      />,
    );
    const first = await screen.findByRole("button", { name: /模型配置 first/ });

    act(() => {
      first.click();
      first.click();
    });

    expect(onSelect).toHaveBeenCalledExactlyOnceWith("first");
    expect(onClose).not.toHaveBeenCalled();
    expect(first).toBeDisabled();
    expect(
      screen.getByRole("button", { name: /模型配置 second/ }),
    ).toBeDisabled();
    expect(screen.getByRole("list")).toHaveAttribute("aria-busy", "true");
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
    expect(screen.queryByText(/正在/)).not.toBeInTheDocument();
    expect(screen.getByRole("button", { name: "取消" })).toBeEnabled();
    await act(async () => request.resolve({ runId: "new-run" }));
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("keeps a selection error recoverable and allows choosing a different model", async () => {
    const onSelect = vi
      .fn()
      .mockRejectedValueOnce({ message: "模型配置已删除" })
      .mockResolvedValueOnce({ runId: "new-run" });
    const onClose = vi.fn();
    render(
      <ModelRetryDialog
        open
        loadModels={async () => [model("first"), model("second")]}
        onSelect={onSelect}
        onClose={onClose}
      />,
    );

    fireEvent.click(
      await screen.findByRole("button", { name: /模型配置 first/ }),
    );
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "模型配置已删除",
    );
    expect(onClose).not.toHaveBeenCalled();
    expect(
      screen.getByRole("button", { name: /模型配置 first/ }),
    ).toBeEnabled();
    fireEvent.click(screen.getByRole("button", { name: /模型配置 second/ }));
    await waitFor(() => expect(onClose).toHaveBeenCalledOnce());
    expect(onSelect.mock.calls).toEqual([["first"], ["second"]]);
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("navigates model buttons and restores the trigger on Escape", async () => {
    const loadModels = vi
      .fn()
      .mockResolvedValue([model("first"), model("second")]);
    const onClose = vi.fn();
    function Harness() {
      const [open, setOpen] = useState(false);
      return (
        <>
          <button onClick={() => setOpen(true)}>使用其他模型重试</button>
          <ModelRetryDialog
            open={open}
            loadModels={loadModels}
            onSelect={vi.fn()}
            onClose={() => {
              onClose();
              setOpen(false);
            }}
          />
        </>
      );
    }
    render(<Harness />);
    const trigger = screen.getByRole("button", { name: "使用其他模型重试" });
    trigger.focus();
    fireEvent.click(trigger);
    const first = await screen.findByRole("button", { name: /模型配置 first/ });
    const second = screen.getByRole("button", { name: /模型配置 second/ });
    expect(first).toHaveFocus();
    expect(first).toHaveAttribute("type", "button");
    fireEvent.keyDown(first, { key: "ArrowDown" });
    expect(second).toHaveFocus();
    fireEvent.keyDown(second, { key: "ArrowUp" });
    expect(first).toHaveFocus();
    fireEvent.keyDown(first, { key: "ArrowUp" });
    expect(first).toHaveFocus();
    fireEvent.keyDown(first, { key: "End" });
    expect(second).toHaveFocus();
    fireEvent.keyDown(second, { key: "Home" });
    expect(first).toHaveFocus();
    fireEvent.keyDown(first, { key: "End" });
    expect(second).toHaveFocus();
    fireEvent.keyDown(second, { key: "Escape" });

    expect(onClose).toHaveBeenCalledOnce();
    await waitFor(() => expect(trigger).toHaveFocus());
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("handles the native modal cancel event", async () => {
    const onClose = vi.fn();
    render(
      <ModelRetryDialog
        open
        loadModels={async () => []}
        onSelect={vi.fn()}
        onClose={onClose}
      />,
    );
    await screen.findByText("尚未配置模型");
    const event = new Event("cancel", { cancelable: true });
    fireEvent(screen.getByRole("dialog"), event);
    expect(event.defaultPrevented).toBe(true);
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("keeps Tab and Shift+Tab inside the model chooser", async () => {
    render(
      <ModelRetryDialog
        open
        loadModels={async () => [model("first")]}
        onSelect={vi.fn()}
        onClose={vi.fn()}
      />,
    );
    const first = await screen.findByRole("button", { name: /模型配置 first/ });
    const cancel = screen.getByRole("button", { name: "取消" });
    first.focus();

    fireEvent.keyDown(first, { key: "Tab", shiftKey: true });
    expect(cancel).toHaveFocus();
    fireEvent.keyDown(cancel, { key: "Tab" });
    expect(first).toHaveFocus();
  });

  it("loads fresh models on reopening while ignoring the old load", async () => {
    const oldLoad = deferred<ModelConfigSummary[]>();
    const loadModels = vi
      .fn()
      .mockReturnValueOnce(oldLoad.promise)
      .mockResolvedValueOnce([model("fresh")]);
    const props = { loadModels, onSelect: vi.fn(), onClose: vi.fn() };
    const { rerender } = render(<ModelRetryDialog open {...props} />);
    rerender(<ModelRetryDialog open={false} {...props} />);
    rerender(<ModelRetryDialog open {...props} />);

    expect(
      await screen.findByRole("button", { name: /模型配置 fresh/ }),
    ).toBeVisible();
    await act(async () => oldLoad.resolve([model("stale")]));
    expect(
      screen.queryByRole("button", { name: /模型配置 stale/ }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /模型配置 fresh/ }),
    ).toBeVisible();
    expect(loadModels).toHaveBeenCalledTimes(2);
  });

  it.each(["resolve", "reject"] as const)(
    "ignores a stale submission %s after closing and reopening",
    async (completion) => {
      const oldSubmission = deferred<unknown>();
      const onClose = vi.fn();
      const props = {
        loadModels: vi.fn().mockResolvedValue([model("first")]),
        onSelect: vi.fn(() => oldSubmission.promise),
        onClose,
      };
      const { rerender } = render(<ModelRetryDialog open {...props} />);
      fireEvent.click(
        await screen.findByRole("button", { name: /模型配置 first/ }),
      );
      rerender(<ModelRetryDialog open={false} {...props} />);
      rerender(<ModelRetryDialog open {...props} />);
      await screen.findByRole("button", { name: /模型配置 first/ });

      await act(async () => oldSubmission[completion](new Error("stale")));

      expect(onClose).not.toHaveBeenCalled();
      expect(screen.queryByRole("alert")).not.toBeInTheDocument();
      expect(screen.getByRole("dialog")).toBeVisible();
      expect(
        screen.getByRole("button", { name: /模型配置 first/ }),
      ).toBeEnabled();
    },
  );
});
