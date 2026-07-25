import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { createPortal, flushSync } from "react-dom";
import { Button } from "./Button";
import { Icon } from "./Icon";

type NotificationVariant = "error" | "success";

type NotificationAction = {
  label: string;
  onClick: () => void;
};

type NotificationOptions = {
  action?: NotificationAction;
  durationMs?: number;
};

type Notification = NotificationOptions & {
  id: number;
  variant: NotificationVariant;
  message: string;
};

type NotificationApi = {
  error: (message: string, options?: NotificationOptions) => number;
  success: (message: string, options?: NotificationOptions) => number;
  dismiss: (id: number) => void;
  clear: () => void;
};

const NotificationContext = createContext<NotificationApi | null>(null);

export function useNotifications() {
  const value = useContext(NotificationContext);
  if (!value) {
    throw new Error(
      "useNotifications must be used within NotificationProvider",
    );
  }
  return value;
}

function NotificationCard({
  notification,
  onDismiss,
}: {
  notification: Notification;
  onDismiss: (id: number) => void;
}) {
  const dismiss = () => onDismiss(notification.id);
  const role = notification.variant === "error" ? "alert" : "status";
  const variantLabel = notification.variant === "error" ? "错误" : "成功";

  return (
    <section
      className={`notification notification--${notification.variant}`}
      role={role}
    >
      <span className="notification__icon" aria-hidden="true">
        <Icon name={notification.variant} />
      </span>
      <p className="notification__message">{notification.message}</p>
      {notification.action && (
        <Button
          className="notification__action"
          onClick={() => {
            flushSync(dismiss);
            notification.action?.onClick();
          }}
        >
          {notification.action.label}
        </Button>
      )}
      <button
        className="notification__dismiss"
        type="button"
        aria-label={`关闭${variantLabel}通知`}
        onClick={dismiss}
      >
        <Icon name="close" />
      </button>
    </section>
  );
}

export function NotificationProvider({ children }: { children: ReactNode }) {
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const nextId = useRef(0);
  const timers = useRef(new Map<number, number>());

  const dismiss = useCallback((id: number) => {
    const timer = timers.current.get(id);
    if (timer !== undefined) window.clearTimeout(timer);
    timers.current.delete(id);
    setNotifications((current) =>
      current.filter((notification) => notification.id !== id),
    );
  }, []);

  const clear = useCallback(() => {
    timers.current.forEach((timer) => window.clearTimeout(timer));
    timers.current.clear();
    setNotifications([]);
  }, []);

  useEffect(
    () => () => {
      timers.current.forEach((timer) => window.clearTimeout(timer));
      timers.current.clear();
    },
    [],
  );

  const publish = useCallback(
    (
      variant: NotificationVariant,
      message: string,
      options: NotificationOptions = {},
    ) => {
      const id = ++nextId.current;
      const durationMs =
        options.durationMs ?? (variant === "success" ? 4_000 : undefined);
      setNotifications((current) => [
        { id, variant, message, ...options, durationMs },
        ...current,
      ]);
      if (durationMs !== undefined) {
        timers.current.set(
          id,
          window.setTimeout(() => dismiss(id), durationMs),
        );
      }
      return id;
    },
    [dismiss],
  );

  const value = useMemo<NotificationApi>(
    () => ({
      error: (message, options) => publish("error", message, options),
      success: (message, options) => publish("success", message, options),
      dismiss,
      clear,
    }),
    [clear, dismiss, publish],
  );

  return (
    <NotificationContext.Provider value={value}>
      {children}
      {createPortal(
        <div
          className="notification-viewport"
          aria-label="应用通知"
          aria-relevant="additions"
        >
          {notifications.map((notification) => (
            <NotificationCard
              key={notification.id}
              notification={notification}
              onDismiss={dismiss}
            />
          ))}
        </div>,
        document.body,
      )}
    </NotificationContext.Provider>
  );
}
