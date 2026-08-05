import { describe, expect, it } from "vitest";
import type { AnalysisSnapshot } from "./ipc";
import { updateAnalysisSnapshot } from "./App";

describe("analysis event state", () => {
  it("clears the previous failure when a retry starts", () => {
    const failed: AnalysisSnapshot = {
      runId: "run-1",
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
      updateAnalysisSnapshot(failed, { type: "started", runId: "run-1" }),
    ).toEqual({
      runId: "run-1",
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
