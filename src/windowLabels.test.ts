import { describe, expect, it } from "vitest";
import { isResultWindowLabel } from "./windowLabels";

describe("window labels", () => {
  it("routes run-specific result windows without matching empty or unrelated labels", () => {
    expect(isResultWindowLabel("result-run-id")).toBe(true);
    expect(isResultWindowLabel("result-")).toBe(false);
    expect(isResultWindowLabel("result")).toBe(false);
    expect(isResultWindowLabel("main")).toBe(false);
  });
});
