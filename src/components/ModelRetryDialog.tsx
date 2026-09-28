import { useEffect, useId, useRef, useState } from "react";
import type { KeyboardEvent } from "react";
import { getErrorMessage, type ModelConfigSummary } from "../ipc";
import { Button } from "./Button";
import { ErrorNotice } from "./ErrorNotice";

type Props = {
  open: boolean;
  loadModels: () => Promise<ModelConfigSummary[]>;
  onSelect: (modelConfigId: string) => Promise<unknown>;
  onClose: () => void;
};

function ModelChoices({ loadModels, onSelect, onClose }: Omit<Props, "open">) {
  const listRef = useRef<HTMLUListElement>(null);
  const activeRef = useRef(true);
  const pendingRef = useRef(false);
  const [models, setModels] = useState<ModelConfigSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [loadRevision, setLoadRevision] = useState(0);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [selectionError, setSelectionError] = useState<string | null>(null);
  const [pending, setPending] = useState(false);

  useEffect(() => {
    activeRef.current = true;
    return () => {
      activeRef.current = false;
    };
  }, []);

  useEffect(() => {
    let active = true;
    async function load() {
      try {
        const value = await loadModels();
        if (active) setModels(value);
      } catch (failure) {
        if (active) setLoadError(getErrorMessage(failure, "加载模型失败"));
      } finally {
        if (active) setLoading(false);
      }
    }
    void load();
    return () => {
      active = false;
    };
  }, [loadModels, loadRevision]);

  useEffect(() => {
    listRef.current?.querySelector("button")?.focus();
  }, [models]);

  async function select(modelConfigId: string) {
    if (pendingRef.current) return;
    pendingRef.current = true;
    setPending(true);
    setSelectionError(null);
    try {
      await onSelect(modelConfigId);
      if (activeRef.current) onClose();
    } catch (failure) {
      if (activeRef.current) {
        setSelectionError(getErrorMessage(failure, "创建结果窗口失败，请重试"));
      }
    } finally {
      if (activeRef.current) {
        pendingRef.current = false;
        setPending(false);
      }
    }
  }

  function navigate(event: KeyboardEvent<HTMLElement>) {
    if (
      pending ||
      !["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)
    ) {
      return;
    }
    const buttons = Array.from(
      listRef.current?.querySelectorAll("button") ?? [],
    );
    if (buttons.length === 0) return;
    const index = buttons.findIndex((button) => button === event.target);
    if (index < 0) return;
    event.preventDefault();
    const target =
      event.key === "Home"
        ? buttons[0]
        : event.key === "End"
          ? buttons.at(-1)
          : buttons[index + (event.key === "ArrowDown" ? 1 : -1)];
    target?.focus();
    target?.scrollIntoView?.({ block: "nearest" });
  }

  return (
    <div className="model-retry-dialog__body" aria-busy={loading || pending}>
      {loading ? null : loadError ? (
        <ErrorNotice
          message={loadError}
          onRetry={() => {
            setLoadError(null);
            setLoading(true);
            setLoadRevision((value) => value + 1);
          }}
        />
      ) : models.length === 0 ? (
        <div className="field__hint" role="status">
          尚未配置模型
        </div>
      ) : (
        <>
          {selectionError && <ErrorNotice message={selectionError} />}
          <ul
            ref={listRef}
            className="model-retry-dialog__list"
            aria-label="可用模型"
            aria-busy={pending}
            onKeyDown={navigate}
          >
            {models.map((model) => (
              <li key={model.id}>
                <Button
                  type="button"
                  className="model-retry-dialog__model"
                  title={model.name}
                  disabled={pending}
                  onClick={() => void select(model.id)}
                >
                  <span>{model.name}</span>
                </Button>
              </li>
            ))}
          </ul>
        </>
      )}
    </div>
  );
}

export function ModelRetryDialog({ open, ...props }: Props) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const titleId = useId();

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog || !open) return;
    const active = document.activeElement;
    const trigger =
      active instanceof HTMLElement && !dialog.contains(active) ? active : null;
    if (!dialog.hasAttribute("open")) {
      if (typeof dialog.showModal === "function") dialog.showModal();
      else dialog.setAttribute("open", "");
    }
    dialog.querySelector("button")?.focus();

    return () => {
      if (dialog.hasAttribute("open")) {
        if (typeof dialog.close === "function") dialog.close();
        else dialog.removeAttribute("open");
      }
      if (trigger?.isConnected) trigger.focus();
    };
  }, [open]);

  return (
    <dialog
      ref={dialogRef}
      className="confirm-dialog model-retry-dialog"
      aria-labelledby={titleId}
      onCancel={(event) => {
        event.preventDefault();
        event.stopPropagation();
        props.onClose();
      }}
      onKeyDown={(event) => {
        if (event.key === "Escape") {
          event.preventDefault();
          event.stopPropagation();
          props.onClose();
        } else if (event.key === "Tab") {
          const controls = event.currentTarget.querySelectorAll<HTMLElement>(
            "button:not(:disabled)",
          );
          const first = controls[0];
          const last = controls[controls.length - 1];
          if (event.shiftKey && document.activeElement === first) {
            event.preventDefault();
            last?.focus();
          } else if (!event.shiftKey && document.activeElement === last) {
            event.preventDefault();
            first?.focus();
          }
        }
      }}
    >
      <div className="confirm-dialog__surface model-retry-dialog__surface">
        <h2 id={titleId}>选择重试模型</h2>
        {open && <ModelChoices {...props} />}
        <div className="confirm-dialog__actions">
          <Button type="button" onClick={props.onClose}>
            取消
          </Button>
        </div>
      </div>
    </dialog>
  );
}
