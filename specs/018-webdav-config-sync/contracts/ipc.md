# IPC Contract: WebDAV 配置同步

All command arguments use Tauri camelCase serialization.

## `get_webdav_settings`

Returns:

```ts
type WebdavSettings = {
  url: string;
  username: string;
  remoteRoot: string;
  hasPassword: boolean;
};
```

## `save_webdav_settings`

Input:

```ts
type WebdavSettingsInput = {
  url: string;
  username: string;
  remoteRoot: string;
  password?: string;
  clearPassword?: boolean;
};
```

Returns the saved `WebdavSettings`. An omitted or empty password retains the saved credential unless `clearPassword` is true.

## `upload_configuration`

No input. Uses the saved WebDAV settings and returns:

```ts
type ConfigSyncResult = {
  models: number;
  prompts: number;
};
```

## `download_configuration`

No input. Uses the saved WebDAV settings, validates the complete remote snapshot, merges it into local storage, and returns `ConfigSyncResult` with the number of affected model and prompt rows.

## Error behavior

Commands return the existing `AppError` shape. Missing credentials, invalid endpoint/path, authentication failures, missing remote files, invalid snapshot content, and transport failures are surfaced without partial local writes.
