import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { NotificationProvider } from "../components/Notifications";
import type { PerformanceReport } from "../ipc";
import { Performance, type PerformanceApi } from "./Performance";

const listen = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/event", () => ({ listen }));

const report: PerformanceReport = {
  sampleCount: 2,
  ttftSampleCount: 2,
  tpsSampleCount: 2,
  averageTtftMs: 130,
  medianTtftMs: 120,
  averageTps: 24.5,
  medianTps: 24,
  models: [
    {
      key: "model-1",
      modelConfigName: "视觉模型",
      modelId: "vision-1",
      protocol: "openai",
      sampleCount: 1,
      ttftSampleCount: 1,
      tpsSampleCount: 1,
      averageTtftMs: 130,
      medianTtftMs: 130,
      averageTps: 24.5,
      medianTps: 24.5,
    },
    {
      key: "model-2",
      modelConfigName: "备用模型",
      modelId: "vision-2",
      protocol: "gemini",
      sampleCount: 1,
      ttftSampleCount: 1,
      tpsSampleCount: 1,
      averageTtftMs: 130,
      medianTtftMs: 110,
      averageTps: 24.5,
      medianTps: 23.5,
    },
  ],
  modelOptions: [
    { value: "model-1", label: "视觉模型 · vision-1" },
    { value: "model-2", label: "备用模型 · vision-2" },
  ],
  promptOptions: [
    { value: "日语解析", label: "日语解析" },
    { value: "英文解析", label: "英文解析" },
  ],
};

function createApi(
  queryPerformance: PerformanceApi["queryPerformance"] = vi
    .fn()
    .mockResolvedValue(report),
): PerformanceApi {
  return { queryPerformance };
}

function renderPerformance(api: PerformanceApi) {
  return render(
    <NotificationProvider>
      <div className="settings-content">
        <Performance api={api} />
      </div>
    </NotificationProvider>,
  );
}

describe("Performance", () => {
  beforeEach(() => {
    listen.mockReset();
    listen.mockResolvedValue(vi.fn());
  });

  it("renders summaries, charts, an accessible table, and filter options", async () => {
    const api = createApi();
    renderPerformance(api);

    expect(
      await screen.findByRole("heading", { name: "性能" }),
    ).toBeInTheDocument();
    expect((await screen.findAllByText("有效样本")).length).toBeGreaterThan(0);
    expect(screen.getAllByText("TTFT 平均 / 中位数").length).toBeGreaterThan(0);
    expect(screen.getAllByText("速度平均 / 中位数").length).toBeGreaterThan(0);
    expect(screen.getByText("24.5 tps / 24.0 tps")).toBeInTheDocument();
    expect(
      screen.getByRole("img", { name: "TTFT 对比模型对比图" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("img", { name: "速度对比模型对比图" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("table", { name: "模型性能明细" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("option", { name: "视觉模型 · vision-1" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("option", { name: "日语解析" }),
    ).toBeInTheDocument();
    expect(screen.getByText("130 ms / 120 ms")).toBeInTheDocument();
  });

  it("queries the selected time, model, and prompt filters", async () => {
    const queryPerformance = vi.fn().mockResolvedValue(report);
    const api = createApi(queryPerformance);
    renderPerformance(api);
    await screen.findByRole("heading", { name: "性能" });
    await screen.findByRole("option", { name: "视觉模型 · vision-1" });

    fireEvent.change(screen.getByLabelText("时间范围"), {
      target: { value: "7" },
    });
    fireEvent.change(screen.getByLabelText("模型"), {
      target: { value: "model-1" },
    });
    fireEvent.change(screen.getByLabelText("提示词"), {
      target: { value: "日语解析" },
    });

    await waitFor(() =>
      expect(queryPerformance).toHaveBeenLastCalledWith({
        periodDays: 7,
        modelName: "model-1",
        promptName: "日语解析",
      }),
    );
    expect(screen.getByRole("button", { name: "清除筛选" })).toBeEnabled();
    fireEvent.click(screen.getByRole("button", { name: "清除筛选" }));
    expect(screen.getByLabelText("时间范围")).toHaveValue("all");
    expect(screen.getByLabelText("模型")).toHaveValue("");
    expect(screen.getByLabelText("提示词")).toHaveValue("");
  });

  it("refreshes after a history-updated event", async () => {
    const queryPerformance = vi.fn().mockResolvedValue(report);
    renderPerformance(createApi(queryPerformance));
    await screen.findByRole("heading", { name: "性能" });
    await waitFor(() =>
      expect(listen).toHaveBeenCalledWith(
        "history-updated",
        expect.any(Function),
      ),
    );

    const onHistoryUpdated = listen.mock.calls[0][1] as () => void;
    onHistoryUpdated();
    await waitFor(() => expect(queryPerformance).toHaveBeenCalledTimes(2));
  });

  it("keeps the error visible and retries the failed request", async () => {
    const queryPerformance = vi
      .fn()
      .mockRejectedValueOnce(new Error("性能服务不可用"))
      .mockResolvedValueOnce(report);
    renderPerformance(createApi(queryPerformance));

    expect((await screen.findAllByText("性能服务不可用")).length).toBe(1);
    fireEvent.click(screen.getByRole("button", { name: "重试" }));
    expect((await screen.findAllByText("有效样本")).length).toBeGreaterThan(0);
    expect(queryPerformance).toHaveBeenCalledTimes(2);
  });

  it("shows a distinct empty state for an unfiltered report", async () => {
    const empty: PerformanceReport = {
      ...report,
      sampleCount: 0,
      ttftSampleCount: 0,
      tpsSampleCount: 0,
      averageTtftMs: null,
      medianTtftMs: null,
      averageTps: null,
      medianTps: null,
      models: [],
      modelOptions: [],
      promptOptions: [],
    };
    renderPerformance(createApi(vi.fn().mockResolvedValue(empty)));
    expect(await screen.findByText("还没有性能样本")).toBeInTheDocument();
    expect(
      screen.getByText("已保存且具备 TTFT 和速度指标的请求会出现在这里。"),
    ).toBeInTheDocument();
  });
});
