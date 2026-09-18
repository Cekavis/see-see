ALTER TABLE app_settings ADD COLUMN webdav_url TEXT NOT NULL DEFAULT '';
ALTER TABLE app_settings ADD COLUMN webdav_username TEXT NOT NULL DEFAULT '';
ALTER TABLE app_settings ADD COLUMN webdav_remote_root TEXT NOT NULL DEFAULT 'see-see';
