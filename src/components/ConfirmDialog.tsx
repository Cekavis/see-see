import { useEffect, useId, useRef } from "react";

type Props = {
  open: boolean;
  title: string;
  description?: string;
  confirmLabel: string;
  onConfirm: () => void;
  onCancel: () => void;
  danger?: boolean;
};

export function ConfirmDialog({
  open,
  title,
  description,
  confirmLabel,
  onConfirm,
  onCancel,
  danger = false,
}: Props) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const cancelRef = useRef<HTMLButtonElement>(null);
  const triggerRef = useRef<HTMLElement | null>(null);
  const titleId = useId();
  const descriptionId = useId();

  useEffect(() => {
    const dialog = dialogRef.current;
    if (!dialog) return;

    if (open) {
      const active = document.activeElement;
      if (active instanceof HTMLElement && !dialog.contains(active)) {
        triggerRef.current = active;
      }
      if (!dialog.hasAttribute("open")) {
        if (typeof dialog.showModal === "function") dialog.showModal();
        else dialog.setAttribute("open", "");
      }
      cancelRef.current?.focus();
      return;
    }

    if (dialog.hasAttribute("open")) {
      if (typeof dialog.close === "function") dialog.close();
      else dialog.removeAttribute("open");
    }
    if (triggerRef.current?.isConnected) triggerRef.current.focus();
    triggerRef.current = null;
  }, [open]);

  return (
    <dialog
      ref={dialogRef}
      className={`confirm-dialog${danger ? " confirm-dialog--danger" : ""}`}
      aria-labelledby={titleId}
      aria-describedby={description ? descriptionId : undefined}
      onCancel={(event) => {
        event.preventDefault();
        onCancel();
      }}
      onKeyDown={(event) => {
        if (event.key === "Escape") {
          event.preventDefault();
          onCancel();
        }
      }}
    >
      <div className="confirm-dialog__surface">
        <h2 id={titleId}>{title}</h2>
        {description && <p id={descriptionId}>{description}</p>}
        <div className="confirm-dialog__actions">
          <button
            ref={cancelRef}
            className="button button--secondary"
            type="button"
            autoFocus
            onClick={onCancel}
          >
            取消
          </button>
          <button
            className={`button button--${danger ? "danger" : "primary"}`}
            type="button"
            onClick={onConfirm}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </dialog>
  );
}
