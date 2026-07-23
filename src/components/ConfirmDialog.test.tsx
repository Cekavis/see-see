import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";
import { ConfirmDialog } from "./ConfirmDialog";

describe("ConfirmDialog", () => {
  it("opens with an accessible name, focuses cancel, and confirms once", () => {
    const onConfirm = vi.fn();
    render(
      <ConfirmDialog
        open
        title="删除提示词"
        description="此操作不可撤销。"
        confirmLabel="删除"
        onConfirm={onConfirm}
        onCancel={vi.fn()}
        danger
      />,
    );

    expect(
      screen.getByRole("dialog", { name: "删除提示词" }),
    ).toHaveAccessibleDescription("此操作不可撤销。");
    expect(screen.getByRole("button", { name: "取消" })).toHaveFocus();
    fireEvent.click(screen.getByRole("button", { name: "删除" }));
    expect(onConfirm).toHaveBeenCalledOnce();
  });

  it("cancels with Escape and restores focus to the trigger", async () => {
    const onCancel = vi.fn();

    function Harness() {
      const [open, setOpen] = useState(false);
      return (
        <>
          <button type="button" onClick={() => setOpen(true)}>
            删除记录
          </button>
          <ConfirmDialog
            open={open}
            title="删除记录"
            confirmLabel="删除"
            onConfirm={vi.fn()}
            onCancel={() => {
              onCancel();
              setOpen(false);
            }}
            danger
          />
        </>
      );
    }

    render(<Harness />);
    const trigger = screen.getByRole("button", { name: "删除记录" });
    trigger.focus();
    fireEvent.click(trigger);
    const dialog = screen.getByRole("dialog", { name: "删除记录" });
    fireEvent.keyDown(dialog, { key: "Escape" });

    expect(onCancel).toHaveBeenCalledOnce();
    await waitFor(() => expect(trigger).toHaveFocus());
    expect(dialog).not.toHaveAttribute("open");
  });
});
