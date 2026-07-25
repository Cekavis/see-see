import { useCallback, useEffect, useState } from "react";
import { Button } from "../components/Button";
import { ConfirmDialog } from "../components/ConfirmDialog";
import { EmptyState } from "../components/EmptyState";
import { Field } from "../components/Field";
import { useNotifications } from "../components/Notifications";
import {
  getErrorMessage,
  ipc,
  type ConnectionTestResult,
  type ModelConfigInput,
  type ModelConfigSummary,
  type ModelConnectionInput,
  type ModelProtocol,
  type RemoteModel,
} from "../ipc";

export type SettingsApi = {
  listModelConfigs: () => Promise<ModelConfigSummary[]>;
  saveModelConfig: (input: ModelConfigInput) => Promise<ModelConfigSummary>;
  deleteModelConfig: (id: string) => Promise<void>;
  setActiveModelConfig: (id: string) => Promise<void>;
  listRemoteModels: (draft: ModelConnectionInput) => Promise<RemoteModel[]>;
  testModelConfig: (
    draft: ModelConnectionInput,
  ) => Promise<ConnectionTestResult>;
};

const endpoints: Record<ModelProtocol, string> = {
  openai: "https://api.openai.com/v1",
  anthropic: "https://api.anthropic.com/v1",
  gemini: "https://generativelanguage.googleapis.com/v1beta",
};

const emptyForm = (): ModelConfigInput => ({
  name: "",
  protocol: "openai",
  baseUrl: endpoints.openai,
  modelId: "",
  apiKey: "",
  clearApiKey: false,
});

export function Settings({ api = ipc }: { api?: SettingsApi }) {
  const notifications = useNotifications();
  const [configs, setConfigs] = useState<ModelConfigSummary[]>([]);
  const [form, setForm] = useState<ModelConfigInput>(emptyForm);
  const [models, setModels] = useState<RemoteModel[]>([]);
  const [busy, setBusy] = useState<"save" | "list" | "test">();
  const [deleteTarget, setDeleteTarget] = useState<ModelConfigSummary | null>(
    null,
  );

  const refresh = useCallback(() => {
    function run() {
      void api
        .listModelConfigs()
        .then(setConfigs)
        .catch((value: unknown) =>
          notifications.error(getErrorMessage(value), {
            action: { label: "重试", onClick: run },
          }),
        );
    }
    run();
  }, [api, notifications]);
  useEffect(() => {
    refresh();
  }, [refresh]);

  const draft = (): ModelConnectionInput => ({
    id: form.id,
    protocol: form.protocol,
    baseUrl: form.baseUrl,
    modelId: form.modelId,
    apiKey: form.apiKey || undefined,
  });

  const update = <K extends keyof ModelConfigInput>(
    key: K,
    value: ModelConfigInput[K],
  ) => setForm((current) => ({ ...current, [key]: value }));

  return (
    <section
      className="section-view settings-view"
      aria-labelledby="models-title"
    >
      <header className="settings-section__header">
        <h1 id="models-title">模型</h1>
        <p>
          配置支持图片输入的 OpenAI、Anthropic、Gemini 或兼容端点。API Key
          只保存在系统凭据存储中。
        </p>
      </header>
      <div className="section-split">
        <section className="settings-grid" aria-label="模型配置编辑器">
          <Field label="配置名称" htmlFor="model-name">
            <input
              id="model-name"
              value={form.name}
              maxLength={80}
              onChange={(event) => update("name", event.target.value)}
            />
          </Field>
          <Field label="协议" htmlFor="model-protocol">
            <select
              id="model-protocol"
              value={form.protocol}
              onChange={(event) => {
                const protocol = event.target.value as ModelProtocol;
                setForm((current) => ({
                  ...current,
                  protocol,
                  baseUrl: endpoints[protocol],
                }));
                setModels([]);
              }}
            >
              <option value="openai">OpenAI Chat Completions</option>
              <option value="anthropic">Anthropic Messages</option>
              <option value="gemini">Gemini GenerateContent</option>
            </select>
          </Field>
          <Field
            label="API 端点"
            htmlFor="model-endpoint"
            hint="远程地址必须使用 HTTPS；localhost 与回环地址可使用 HTTP。"
          >
            <input
              id="model-endpoint"
              type="url"
              value={form.baseUrl}
              onChange={(event) => update("baseUrl", event.target.value)}
            />
          </Field>
          <Field
            label="模型 ID"
            htmlFor="model-id"
            hint="模型列表不可用时仍可手动输入。"
          >
            <input
              id="model-id"
              list="remote-models"
              value={form.modelId}
              maxLength={200}
              onChange={(event) => update("modelId", event.target.value)}
            />
            <datalist id="remote-models">
              {models.map((model) => (
                <option key={model.id} value={model.id}>
                  {model.name}
                </option>
              ))}
            </datalist>
          </Field>
          <Field
            label="API Key"
            htmlFor="model-key"
            hint={
              form.id ? "留空表示保留现有 Key。" : "可用于无需认证的本机端点。"
            }
          >
            <input
              id="model-key"
              type="password"
              autoComplete="off"
              value={form.apiKey ?? ""}
              onChange={(event) => update("apiKey", event.target.value)}
            />
          </Field>
          {form.id && (
            <label className="toggle">
              <input
                type="checkbox"
                checked={Boolean(form.clearApiKey)}
                onChange={(event) =>
                  update("clearApiKey", event.target.checked)
                }
              />
              清除已保存的 API Key
            </label>
          )}
          <p className="field__hint">
            连接测试会先保存当前配置，再发送一张极小图片，可能产生少量调用费用。
          </p>
          <div className="button-row">
            <Button
              disabled={Boolean(busy)}
              onClick={() => {
                setBusy("list");
                notifications.clear();
                void api
                  .listRemoteModels(draft())
                  .then((items) => {
                    setModels(items);
                    notifications.success(
                      items.length
                        ? `已获取 ${items.length} 个模型`
                        : "端点未返回可用模型，可继续手动输入",
                    );
                  })
                  .catch((value: unknown) =>
                    notifications.error(getErrorMessage(value)),
                  )
                  .finally(() => setBusy(undefined));
              }}
            >
              获取模型列表
            </Button>
            <Button
              disabled={Boolean(busy)}
              onClick={() => {
                setBusy("test");
                notifications.clear();
                const input = { ...form, apiKey: form.apiKey || undefined };
                void api
                  .saveModelConfig(input)
                  .then(async (saved) => {
                    setForm({
                      id: saved.id,
                      name: saved.name,
                      protocol: saved.protocol,
                      baseUrl: saved.baseUrl,
                      modelId: saved.modelId,
                      apiKey: "",
                      clearApiKey: false,
                    });
                    const result = await api.testModelConfig({
                      id: saved.id,
                      protocol: saved.protocol,
                      baseUrl: saved.baseUrl,
                      modelId: saved.modelId,
                    });
                    if (!result.passed) {
                      notifications.error(
                        result.error?.message ?? "连接测试失败",
                      );
                      return;
                    }
                    await api.setActiveModelConfig(saved.id);
                    notifications.success(
                      `配置已保存、测试通过并设为当前模型（${result.latencyMs} ms）`,
                    );
                  })
                  .catch((value: unknown) =>
                    notifications.error(getErrorMessage(value)),
                  )
                  .finally(() => {
                    setBusy(undefined);
                    refresh();
                  });
              }}
            >
              测试连接
            </Button>
            <Button
              variant="primary"
              disabled={Boolean(busy)}
              onClick={() => {
                setBusy("save");
                notifications.clear();
                const input = { ...form, apiKey: form.apiKey || undefined };
                void api
                  .saveModelConfig(input)
                  .then((saved) => {
                    setForm({
                      id: saved.id,
                      name: saved.name,
                      protocol: saved.protocol,
                      baseUrl: saved.baseUrl,
                      modelId: saved.modelId,
                      apiKey: "",
                      clearApiKey: false,
                    });
                    notifications.success(
                      "配置已保存；修改后的配置需要重新测试连接。",
                    );
                    void refresh();
                  })
                  .catch((value: unknown) =>
                    notifications.error(getErrorMessage(value)),
                  )
                  .finally(() => setBusy(undefined));
              }}
            >
              保存配置
            </Button>
            {form.id && (
              <Button onClick={() => setForm(emptyForm())}>新建配置</Button>
            )}
          </div>
        </section>

        <section className="config-list" aria-label="已保存模型配置">
          <h2>已保存配置</h2>
          {configs.length === 0 ? (
            <EmptyState
              title="还没有模型配置"
              description="保存并测试一个支持图片输入的模型后即可截图。"
            />
          ) : (
            configs.map((config) => (
              <article className="config-card" key={config.id}>
                <div>
                  <h3>
                    {config.name}
                    {config.isActive ? " · 当前" : ""}
                  </h3>
                  <p>
                    {config.protocol} · {config.modelId}
                  </p>
                  <p>
                    {config.hasApiKey ? "已保存 Key" : "无 Key"} ·{" "}
                    {config.testStatus === "passed"
                      ? "测试通过"
                      : config.testStatus === "failed"
                        ? "测试失败"
                        : "未测试"}
                  </p>
                </div>
                <div className="button-row">
                  <Button
                    onClick={() =>
                      setForm({
                        id: config.id,
                        name: config.name,
                        protocol: config.protocol,
                        baseUrl: config.baseUrl,
                        modelId: config.modelId,
                        apiKey: "",
                        clearApiKey: false,
                      })
                    }
                  >
                    编辑
                  </Button>
                  <Button
                    disabled={config.testStatus !== "passed" || config.isActive}
                    onClick={() => {
                      notifications.clear();
                      void api
                        .setActiveModelConfig(config.id)
                        .then(() => {
                          notifications.success("已设为当前模型");
                          void refresh();
                        })
                        .catch((value: unknown) =>
                          notifications.error(getErrorMessage(value)),
                        );
                    }}
                  >
                    设为当前
                  </Button>
                  <Button
                    variant="danger"
                    onClick={() => setDeleteTarget(config)}
                  >
                    删除
                  </Button>
                </div>
              </article>
            ))
          )}
        </section>
      </div>
      <ConfirmDialog
        open={Boolean(deleteTarget)}
        title="删除模型配置？"
        description={`删除模型配置“${deleteTarget?.name ?? ""}”？此操作不可撤销。`}
        confirmLabel="删除配置"
        danger
        onCancel={() => setDeleteTarget(null)}
        onConfirm={() => {
          const target = deleteTarget;
          if (!target) return;
          setDeleteTarget(null);
          void api
            .deleteModelConfig(target.id)
            .then(() => {
              if (form.id === target.id) setForm(emptyForm());
              notifications.success("模型配置已删除");
              void refresh();
            })
            .catch((value: unknown) =>
              notifications.error(getErrorMessage(value)),
            );
        }}
      />
    </section>
  );
}
