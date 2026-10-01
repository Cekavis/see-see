import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { Button } from "../components/Button";
import { EmptyState } from "../components/EmptyState";
import { ErrorNotice } from "../components/ErrorNotice";
import { formatDurationMs, formatTps } from "../components/PerformanceMetrics";
import {
  getErrorMessage,
  ipc,
  type ModelPerformanceSummary,
  type PerformanceQuery,
  type PerformanceReport,
} from "../ipc";

export type PerformanceApi = {
  queryPerformance: (query: PerformanceQuery) => Promise<PerformanceReport>;
};

type PeriodDays = PerformanceQuery["periodDays"];

function percent(value: number | null, maximum: number) {
  if (value === null || !Number.isFinite(value) || maximum <= 0) return 0;
  return Math.max(0, Math.min(100, (value / maximum) * 100));
}

function MetricChart({
  id,
  title,
  models,
  averageKey,
  medianKey,
  format,
}: {
  id: string;
  title: string;
  models: ModelPerformanceSummary[];
  averageKey: "averageTtftMs" | "averageTps";
  medianKey: "medianTtftMs" | "medianTps";
  format: (value: number | null) => string;
}) {
  const maximum = Math.max(
    1,
    ...models.flatMap((model) => [
      model[averageKey] ?? 0,
      model[medianKey] ?? 0,
    ]),
  );

  return (
    <section className="performance-chart" aria-labelledby={`${id}-title`}>
      <div className="performance-chart__header">
        <h2 id={`${id}-title`}>{title}</h2>
        <span className="performance-chart__legend" aria-hidden="true">
          <i className="performance-chart__legend-bar" />
          平均
          <i className="performance-chart__legend-marker" />
          中位数
        </span>
      </div>
      <div
        className="performance-chart__plot"
        role="img"
        aria-label={`${title}模型对比图`}
      >
        {models.map((model) => {
          const average = model[averageKey];
          const median = model[medianKey];
          return (
            <div className="performance-chart__row" key={model.key}>
              <span className="performance-chart__label" title={model.modelId}>
                {model.modelConfigName}
              </span>
              <span className="performance-chart__track">
                <span
                  className="performance-chart__bar"
                  style={{ width: `${percent(average, maximum)}%` }}
                />
                <span
                  className="performance-chart__marker"
                  style={{ left: `${percent(median, maximum)}%` }}
                />
              </span>
              <span className="performance-chart__value">
                {format(average)} / {format(median)}
              </span>
            </div>
          );
        })}
      </div>
    </section>
  );
}

function PerformanceTable({ models }: { models: ModelPerformanceSummary[] }) {
  return (
    <div className="performance-table-wrap">
      <table className="performance-table">
        <caption>模型性能明细</caption>
        <thead>
          <tr>
            <th scope="col">模型</th>
            <th scope="col">有效样本</th>
            <th scope="col">TTFT 平均 / 中位数</th>
            <th scope="col">速度平均 / 中位数</th>
          </tr>
        </thead>
        <tbody>
          {models.map((model) => (
            <tr key={model.key}>
              <th scope="row">
                <span>{model.modelConfigName}</span>
                <small>{model.modelId}</small>
              </th>
              <td>{model.sampleCount}</td>
              <td>
                {formatDurationMs(model.averageTtftMs)} /{" "}
                {formatDurationMs(model.medianTtftMs)}
              </td>
              <td>
                {formatTps(model.averageTps)} / {formatTps(model.medianTps)}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

export function Performance({ api = ipc }: { api?: PerformanceApi }) {
  const [periodDays, setPeriodDays] = useState<PeriodDays>(null);
  const [modelName, setModelName] = useState("");
  const [promptName, setPromptName] = useState("");
  const [report, setReport] = useState<PerformanceReport | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const requestId = useRef(0);

  const query = useMemo<PerformanceQuery>(
    () => ({
      periodDays,
      modelName: modelName || undefined,
      promptName: promptName || undefined,
    }),
    [modelName, periodDays, promptName],
  );

  const load = useCallback(() => {
    const currentRequest = ++requestId.current;
    setLoading(true);
    setError(null);
    void api
      .queryPerformance(query)
      .then((next) => {
        if (requestId.current === currentRequest) setReport(next);
      })
      .catch((value: unknown) => {
        if (requestId.current !== currentRequest) return;
        const message = getErrorMessage(value, "加载性能统计失败");
        setError(message);
      })
      .finally(() => {
        if (requestId.current === currentRequest) setLoading(false);
      });
    return () => {
      if (requestId.current === currentRequest) requestId.current += 1;
    };
  }, [api, query]);

  useEffect(() => {
    let active = true;
    let cancel: (() => void) | undefined;
    const timer = window.setTimeout(() => {
      if (active) cancel = load();
    }, 0);
    return () => {
      active = false;
      window.clearTimeout(timer);
      cancel?.();
    };
  }, [load]);

  useEffect(() => {
    let active = true;
    let unlisten: (() => void) | undefined;
    void listen("history-updated", () => {
      if (active) load();
    }).then((remove) => {
      if (active) unlisten = remove;
      else remove();
    });
    return () => {
      active = false;
      unlisten?.();
    };
  }, [load]);

  const hasFilters = Boolean(periodDays || modelName || promptName);
  const clearFilters = () => {
    setPeriodDays(null);
    setModelName("");
    setPromptName("");
  };

  return (
    <section
      className="section-view performance-view"
      aria-labelledby="performance-title"
      aria-busy={loading}
    >
      <header className="settings-section__header performance-header">
        <h1 id="performance-title">性能</h1>
        <Button disabled={loading} onClick={() => load()}>
          {loading ? "刷新中…" : "刷新"}
        </Button>
      </header>
      <section className="performance-filters" aria-label="性能筛选">
        <label>
          时间范围
          <select
            aria-label="时间范围"
            value={periodDays ?? "all"}
            onChange={(event) => {
              const value = event.target.value;
              setPeriodDays(
                value === "all" ? null : (Number(value) as 7 | 30 | 90),
              );
            }}
          >
            <option value="all">全部时间</option>
            <option value="7">最近 7 天</option>
            <option value="30">最近 30 天</option>
            <option value="90">最近 90 天</option>
          </select>
        </label>
        <label>
          模型
          <select
            aria-label="模型"
            value={modelName}
            onChange={(event) => setModelName(event.target.value)}
          >
            <option value="">全部模型</option>
            {report?.modelOptions.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </label>
        <label>
          提示词
          <select
            aria-label="提示词"
            value={promptName}
            onChange={(event) => setPromptName(event.target.value)}
          >
            <option value="">全部提示词</option>
            {report?.promptOptions.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
        </label>
        <Button disabled={!hasFilters} onClick={clearFilters}>
          清除筛选
        </Button>
      </section>

      {error ? (
        <div className="performance-error">
          <ErrorNotice message={error} onRetry={() => void load()} />
        </div>
      ) : null}

      {loading && !report ? (
        <p className="performance-status" role="status">
          正在加载性能统计…
        </p>
      ) : loading ? (
        <p className="performance-status" role="status">
          正在刷新性能统计…
        </p>
      ) : report && report.sampleCount === 0 ? (
        <EmptyState
          title={hasFilters ? "没有匹配的性能样本" : "还没有性能样本"}
          description="已保存且具备 TTFT 和速度指标的请求会出现在这里。"
        />
      ) : report ? (
        <div className="performance-content">
          <section className="performance-summary" aria-label="性能摘要">
            <article>
              <span>有效样本</span>
              <strong>{report.sampleCount}</strong>
            </article>
            <article>
              <span>TTFT 平均 / 中位数</span>
              <strong>
                {formatDurationMs(report.averageTtftMs)} /{" "}
                {formatDurationMs(report.medianTtftMs)}
              </strong>
            </article>
            <article>
              <span>速度平均 / 中位数</span>
              <strong>
                {formatTps(report.averageTps)} / {formatTps(report.medianTps)}
              </strong>
            </article>
          </section>
          <div className="performance-charts">
            <MetricChart
              id="performance-ttft"
              title="TTFT 对比"
              models={report.models}
              averageKey="averageTtftMs"
              medianKey="medianTtftMs"
              format={formatDurationMs}
            />
            <MetricChart
              id="performance-tps"
              title="速度对比"
              models={report.models}
              averageKey="averageTps"
              medianKey="medianTps"
              format={formatTps}
            />
          </div>
          <PerformanceTable models={report.models} />
        </div>
      ) : null}
    </section>
  );
}
