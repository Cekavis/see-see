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
  duplicateModelConfig: (id: string) => Promise<ModelConfigSummary>;
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
  const [form, setForm] = useState<ModelConfigInput | null>(null);
  const [models, setModels] = useState<RemoteModel[]>([]);
  const [busy, setBusy] = useState<"save" | "list" | "test" | "copy">();
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

  const update = <K extends keyof ModelConfigInput>(
    key: K,
    value: ModelConfigInput[K],
  ) => setForm((current) => (current ? { ...current, [key]: value } : current));

  const connectionDraft = (): ModelConnectionInput | null =>
    form
      ? {
          id: form.id,
          protocol: form.protocol,
          baseUrl: form.baseUrl,
          modelId: form.modelId,
          apiKey: form.apiKey || undefined,
        }
      : null;

  const closeEditor = () => {
    setForm(null);
    setModels([]);
  };

  const isBusy = Boolean(busy);

  return (
    <section
      className="section-view settings-view"
      aria-labelledby="models-title"
    >
      <header className="settings-section__header action-header">
        <h1 id="models-title">模型</h1>
        <Button
          type="button"
          variant="primary"
          disabled={isBusy}
          aria-expanded={Boolean(form)}
          aria-controls="model-config-editor"
          onClick={() => {
            setForm(emptyForm());
            setModels([]);
          }}
        >
          新增配置
        </Button>
      </header>

      <div className={form ? "section-split" : "section-stack"}>
        <section className="config-list" aria-label="已保存模型配置">
          <h2>已保存配置</h2>
          {configs.length === 0 ? (
            <EmptyState
              title="还没有模型配置"
              description="新增模型配置并设为当前后即可截图。"
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
                  <p className="config-card__endpoint">{config.baseUrl}</p>
                  <p>{config.hasApiKey ? "已保存 Key" : "无 Key"}</p>
                </div>
                <div className="button-row">
                  <Button
                    type="button"
                    disabled={isBusy}
                    aria-label={`编辑 ${config.name}`}
                    onClick={() => {
                      setForm({
                        id: config.id,
                        name: config.name,
                        protocol: config.protocol,
                        baseUrl: config.baseUrl,
                        modelId: config.modelId,
                        apiKey: "",
                        clearApiKey: false,
                      });
                      setModels([]);
                    }}
                  >
                    编辑
                  </Button>
                  <Button
                    type="button"
                    disabled={isBusy}
                    aria-label={`复制 ${config.name}`}
                    onClick={() => {
                      setBusy("copy");
                      notifications.clear();
                      void api
                        .duplicateModelConfig(config.id)
                        .then(() => {
                          notifications.success("模型配置已复制");
                          refresh();
                        })
                        .catch((value: unknown) =>
                          notifications.error(getErrorMessage(value)),
                        )
                        .finally(() => setBusy(undefined));
                    }}
                  >
                    复制
                  </Button>
                  <Button
                    type="button"
                    disabled={isBusy || config.isActive}
                    aria-label={`设为当前 ${config.name}`}
                    onClick={() => {
                      notifications.clear();
                      void api
                        .setActiveModelConfig(config.id)
                        .then(() => {
                          notifications.success("已设为当前模型");
                          refresh();
                        })
                        .catch((value: unknown) =>
                          notifications.error(getErrorMessage(value)),
                        );
                    }}
                  >
                    设为当前
                  </Button>
                  <Button
                    type="button"
                    variant="danger"
                    disabled={isBusy}
                    aria-label={`删除 ${config.name}`}
                    onClick={() => setDeleteTarget(config)}
                  >
                    删除
                  </Button>
                </div>
              </article>
            ))
          )}
        </section>

        {form && (
          <section
            id="model-config-editor"
            className="settings-grid model-config-editor"
            aria-labelledby="model-editor-title"
            aria-busy={isBusy}
          >
            <h2 id="model-editor-title">{form.id ? "编辑配置" : "新增配置"}</h2>
            <Field label="配置名称" htmlFor="model-name">
              <input
                autoFocus
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
                  setForm((current) =>
                    current
                      ? { ...current, protocol, baseUrl: endpoints[protocol] }
                      : current,
                  );
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
                form.id
                  ? "留空保留已保存的 Key。API Key 会与端点一起以明文保存在本机。"
                  : "API Key 会与端点一起以明文保存在本机；无需认证时可留空。"
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
              连接测试会发送一张极小图片，可能产生少量调用费用；测试结果不会保存。
            </p>
            <div className="button-row">
              <Button
                type="button"
                disabled={isBusy}
                onClick={() => {
                  const draft = connectionDraft();
                  if (!draft) return;
                  setBusy("list");
                  notifications.clear();
                  void api
                    .listRemoteModels(draft)
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
                type="button"
                disabled={isBusy}
                onClick={() => {
                  const draft = connectionDraft();
                  if (!draft) return;
                  setBusy("test");
                  notifications.clear();
                  void api
                    .testModelConfig(draft)
                    .then((result) => {
                      if (!result.passed) {
                        notifications.error(
                          result.error?.message ?? "连接测试失败",
                        );
                        return;
                      }
                      notifications.success(
                        `连接测试通过（${result.latencyMs} ms）`,
                      );
                    })
                    .catch((value: unknown) =>
                      notifications.error(getErrorMessage(value)),
                    )
                    .finally(() => setBusy(undefined));
                }}
              >
                测试连接
              </Button>
              <Button
                type="button"
                variant="primary"
                disabled={isBusy}
                onClick={() => {
                  setBusy("save");
                  notifications.clear();
                  const input = {
                    ...form,
                    apiKey: form.apiKey || undefined,
                  };
                  void api
                    .saveModelConfig(input)
                    .then(() => {
                      closeEditor();
                      notifications.success("配置已保存");
                      refresh();
                    })
                    .catch((value: unknown) =>
                      notifications.error(getErrorMessage(value)),
                    )
                    .finally(() => setBusy(undefined));
                }}
              >
                保存配置
              </Button>
              <Button type="button" disabled={isBusy} onClick={closeEditor}>
                取消
              </Button>
            </div>
          </section>
        )}
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
              if (form?.id === target.id) closeEditor();
              notifications.success("模型配置已删除");
              refresh();
            })
            .catch((value: unknown) =>
              notifications.error(getErrorMessage(value)),
            );
        }}
      />
    </section>
  );
}
