import { describe, expect, it } from "vitest";
import { getErrorMessage } from "./ipc";

describe("IPC error messages", () => {
  it("supports structured, Error, and string rejections without exposing objects", () => {
    expect(
      getErrorMessage({
        code: "invalid_input",
        message: "模型 ID 不能为空",
        retryable: false,
      }),
    ).toBe("模型 ID 不能为空");
    expect(getErrorMessage(new Error("网络不可用"))).toBe("网络不可用");
    expect(getErrorMessage("invalid args for command")).toBe(
      "invalid args for command",
    );
    expect(getErrorMessage({ rawResponse: "secret" })).toBe("操作失败，请重试");
  });
});
