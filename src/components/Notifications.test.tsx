import { act, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { NotificationProvider, useNotifications } from "./Notifications";

function Harness({ onRetry = vi.fn() }: { onRetry?: () => void }) {
  const notifications = useNotifications();
  return (
    <div data-testid="page-content">
      <button type="button" onClick={() => notifications.error("连接失败")}>
        错误
      </button>
      <button
        type="button"
        onClick={() =>
          notifications.error("可以重试", {
            action: { label: "重试", onClick: onRetry },
          })
        }
      >
        可恢复错误
      </button>
      <button type="button" onClick={() => notifications.success("保存成功")}>
        成功
      </button>
    </div>
  );
}

function renderHarness(onRetry?: () => void) {
  return render(
    <NotificationProvider>
      <Harness onRetry={onRetry} />
    </NotificationProvider>,
  );
}

afterEach(() => {
  vi.useRealTimers();
});

describe("NotificationProvider", () => {
  it("keeps distinct persistent errors and renders the newest first in a portal", () => {
    const { container } = renderHarness();

    fireEvent.click(screen.getByRole("button", { name: "错误" }));
    fireEvent.click(screen.getByRole("button", { name: "错误" }));

    const alerts = screen.getAllByRole("alert");
    expect(alerts).toHaveLength(2);
    expect(alerts.map((alert) => alert.textContent)).toEqual([
      expect.stringContaining("连接失败"),
      expect.stringContaining("连接失败"),
    ]);
    expect(container.querySelector(".notification-viewport")).toBeNull();
    expect(
      document.body.querySelector(".notification-viewport"),
    ).not.toBeNull();
  });

  it("dismisses one error without changing the remaining notification", () => {
    renderHarness();
    fireEvent.click(screen.getByRole("button", { name: "错误" }));
    fireEvent.click(screen.getByRole("button", { name: "错误" }));

    fireEvent.click(
      within(screen.getAllByRole("alert")[0]).getByRole("button", {
        name: "关闭错误通知",
      }),
    );

    expect(screen.getAllByRole("alert")).toHaveLength(1);
  });

  it("removes a recoverable error before invoking its keyboard-accessible action", () => {
    const onRetry = vi.fn(() => {
      expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    });
    renderHarness(onRetry);
    fireEvent.click(screen.getByRole("button", { name: "可恢复错误" }));

    const retry = screen.getByRole("button", { name: "重试" });
    retry.focus();
    fireEvent.keyDown(retry, { key: "Enter" });
    fireEvent.click(retry);

    expect(onRetry).toHaveBeenCalledOnce();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("announces success politely and dismisses it after four seconds", () => {
    vi.useFakeTimers();
    renderHarness();
    fireEvent.click(screen.getByRole("button", { name: "成功" }));

    const status = screen.getByRole("status");
    expect(status).toHaveTextContent("保存成功");
    expect(
      within(status).getByRole("button", { name: "关闭成功通知" }),
    ).toBeInTheDocument();

    act(() => vi.advanceTimersByTime(3_999));
    expect(screen.getByRole("status")).toBeInTheDocument();
    act(() => vi.advanceTimersByTime(1));
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
  });

  it("allows repeated identical successes to keep independent lifetimes", () => {
    vi.useFakeTimers();
    renderHarness();
    fireEvent.click(screen.getByRole("button", { name: "成功" }));
    act(() => vi.advanceTimersByTime(1_000));
    fireEvent.click(screen.getByRole("button", { name: "成功" }));

    expect(screen.getAllByRole("status")).toHaveLength(2);
    act(() => vi.advanceTimersByTime(3_000));
    expect(screen.getAllByRole("status")).toHaveLength(1);
    act(() => vi.advanceTimersByTime(1_000));
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
  });
});
