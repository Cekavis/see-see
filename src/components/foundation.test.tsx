import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { t } from "../i18n";
import { Button } from "./Button";

describe("frontend foundation", () => {
  it("uses the Chinese resource table", () => {
    expect(t("openHistory")).toBe("历史记录");
  });

  it("keeps shared buttons keyboard and click accessible", () => {
    const onClick = vi.fn();
    render(<Button onClick={onClick}>继续</Button>);
    const button = screen.getByRole("button", { name: "继续" });
    fireEvent.click(button);
    expect(onClick).toHaveBeenCalledOnce();
  });
});
