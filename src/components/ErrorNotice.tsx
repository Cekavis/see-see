import { Button } from "./Button";

export function ErrorNotice({
  message,
  onRetry,
}: {
  message: string;
  onRetry?: () => void;
}) {
  return (
    <div className="inline-error-notice" role="alert">
      <span>{message}</span>
      {onRetry && <Button onClick={onRetry}>重试</Button>}
    </div>
  );
}
