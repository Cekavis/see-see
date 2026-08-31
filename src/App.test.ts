import { describe, expect, it } from "vitest";
import type { AnalysisSnapshot } from "./ipc";
import {
  mergeAttachedAnalysisSnapshot,
  shouldCloseWindowOnKeydown,
  updateAnalysisSnapshot,
} from "./App";

const key = (overrides: Partial<KeyboardEvent> = {}) => ({
  key: "",
  code: "",
  ctrlKey: false,
  metaKey: false,
  altKey: false,
  shiftKey: false,
  ...overrides,
});

describe("window close shortcuts", () => {
  it("closes result windows with Escape or Ctrl+W", () => {
    expect(
      shouldCloseWindowOnKeydown("result-run-1", key({ key: "Escape" })),
    ).toBe(true);
    expect(
      shouldCloseWindowOnKeydown(
        "result-run-1",
        key({ key: "w", ctrlKey: true }),
      ),
    ).toBe(true);
  });

  it("closes the main window with Ctrl+W only", () => {
    expect(
      shouldCloseWindowOnKeydown("main", key({ key: "w", ctrlKey: true })),
    ).toBe(true);
    expect(shouldCloseWindowOnKeydown("main", key({ key: "Escape" }))).toBe(
      false,
    );
  });

  it("ignores shortcuts for other windows and modified Escape", () => {
    expect(
      shouldCloseWindowOnKeydown("settings", key({ key: "w", ctrlKey: true })),
    ).toBe(false);
    expect(
      shouldCloseWindowOnKeydown(
        "result-run-1",
        key({ key: "Escape", shiftKey: true }),
      ),
    ).toBe(false);
  });
});

describe("analysis event state", () => {
  it("does not replace a window with an attached snapshot from another run", () => {
    const current: AnalysisSnapshot = {
      runId: "run-1",
      modelConfigName: "模型一",
      promptConfigName: "提示词一",
      state: "streaming",
      thinking: "",
      text: "第一路",
      savedToHistory: false,
      error: null,
    };
    const attached = { ...current, runId: "run-2", text: "第二路" };

    expect(mergeAttachedAnalysisSnapshot(current, attached, "run-1")).toBe(
      current,
    );
  });

  it("does not let an attach snapshot overwrite live updates", () => {
    const current: AnalysisSnapshot = {
      runId: "run-1",
      modelConfigName: "模型配置",
      promptConfigName: "提示词配置",
      state: "streaming",
      thinking: "",
      text: "已收到增量",
      savedToHistory: false,
      error: null,
    };
    const attached = { ...current, state: "submitting" as const, text: "" };

    expect(mergeAttachedAnalysisSnapshot(current, attached, "run-1")).toBe(
      current,
    );
  });

  it("clears the previous failure when a retry starts", () => {
    const failed: AnalysisSnapshot = {
      runId: "run-1",
      modelConfigName: "原模型配置",
      promptConfigName: "原提示词配置",
      state: "failed",
      thinking: "old thinking",
      text: "partial",
      savedToHistory: true,
      error: {
        code: "timeout",
        message: "模型请求超时",
        details: "request timed out",
        retryable: true,
      },
    };

    expect(
      updateAnalysisSnapshot(failed, {
        type: "started",
        runId: "run-1",
        modelConfigName: "重试模型配置",
        promptConfigName: "重试提示词配置",
      }),
    ).toEqual({
      runId: "run-1",
      modelConfigName: "重试模型配置",
      promptConfigName: "重试提示词配置",
      state: "submitting",
      thinking: "",
      text: "",
      savedToHistory: false,
      error: null,
    });
  });

  it("accumulates thinking separately and uses the terminal snapshot", () => {
    const initial: AnalysisSnapshot = {
      runId: "run-1",
      modelConfigName: "模型配置",
      promptConfigName: "提示词配置",
      state: "submitting",
      thinking: "",
      text: "",
      savedToHistory: false,
      error: null,
    };
    const thinking = updateAnalysisSnapshot(initial, {
      type: "thinkingDelta",
      runId: "run-1",
      text: "分析",
    });
    expect(thinking).toMatchObject({
      state: "streaming",
      modelConfigName: "模型配置",
      promptConfigName: "提示词配置",
      thinking: "分析",
      text: "",
    });
    const answer = updateAnalysisSnapshot(thinking, {
      type: "delta",
      runId: "run-1",
      text: "答案",
    });
    expect(answer).toMatchObject({ thinking: "分析", text: "答案" });
    expect(
      updateAnalysisSnapshot(answer, {
        type: "completed",
        runId: "run-1",
        thinking: "完整分析",
        text: "完整答案",
        savedToHistory: true,
      }),
    ).toMatchObject({
      state: "completed",
      thinking: "完整分析",
      text: "完整答案",
      savedToHistory: true,
    });
  });
});
