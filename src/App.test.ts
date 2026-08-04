import { describe, expect, it } from "vitest";
import type { AnalysisSnapshot } from "./ipc";
import { updateAnalysisSnapshot } from "./App";

describe("analysis event state", () => {
  it("clears the previous failure when a retry starts", () => {
    const failed: AnalysisSnapshot = {
      runId: "run-1",
      state: "failed",
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
      text: "",
      savedToHistory: false,
      error: null,
    });
  });
});
