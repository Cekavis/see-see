use crate::error::AppError;
use rusqlite::{Connection, Transaction};
use std::{path::Path, sync::Mutex};

const INITIAL_SCHEMA: &str = include_str!("../migrations/0001_init.sql");
const PLAINTEXT_KEY_MIGRATION: &str = include_str!("../migrations/0002_plaintext_model_keys.sql");
const HISTORY_THINKING_MIGRATION: &str = include_str!("../migrations/0003_history_thinking.sql");
const LEGACY_DEFAULT_CAPTURE_SHORTCUT: &str = "Alt+Shift+A";
pub const WINDOWS_DEFAULT_CAPTURE_SHORTCUT: &str = "Ctrl+Shift+X";
pub const MACOS_DEFAULT_CAPTURE_SHORTCUT: &str = "Command+Shift+X";

#[cfg(target_os = "macos")]
pub const DEFAULT_CAPTURE_SHORTCUT: &str = MACOS_DEFAULT_CAPTURE_SHORTCUT;
#[cfg(not(target_os = "macos"))]
pub const DEFAULT_CAPTURE_SHORTCUT: &str = WINDOWS_DEFAULT_CAPTURE_SHORTCUT;

pub struct Database {
    connection: Mutex<Connection>,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, AppError> {
        let connection =
            Connection::open(path).map_err(|_| AppError::storage("无法打开本地数据库"))?;
        Self::initialize(connection)
    }

    pub fn open_in_memory() -> Result<Self, AppError> {
        let connection =
            Connection::open_in_memory().map_err(|_| AppError::storage("无法创建测试数据库"))?;
        Self::initialize(connection)
    }

    fn initialize(connection: Connection) -> Result<Self, AppError> {
        let previous_version = connection
            .query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
            .map_err(|_| AppError::storage("无法检查数据库版本"))?;
        connection
            .execute_batch(
                "PRAGMA foreign_keys=ON;
                 PRAGMA secure_delete=ON;
                 PRAGMA journal_mode=DELETE;
                 PRAGMA synchronous=FULL;
                 PRAGMA trusted_schema=OFF;",
            )
            .map_err(|_| AppError::storage("无法配置本地数据库"))?;
        connection
            .execute_batch(INITIAL_SCHEMA)
            .map_err(|_| AppError::storage("无法初始化本地数据库"))?;
        let has_onboarding = {
            let mut statement = connection
                .prepare("PRAGMA table_info(app_settings)")
                .map_err(|_| AppError::storage("无法检查数据库版本"))?;
            let columns = statement
                .query_map([], |row| row.get::<_, String>(1))
                .map_err(|_| AppError::storage("无法检查数据库版本"))?;
            columns
                .filter_map(Result::ok)
                .any(|column| column == "onboarding_completed")
        };
        if !has_onboarding {
            connection
                .execute("ALTER TABLE app_settings ADD COLUMN onboarding_completed INTEGER NOT NULL DEFAULT 0 CHECK (onboarding_completed IN (0, 1))", [])
                .map_err(|_| AppError::storage("无法升级本地数据库"))?;
        }
        let has_plaintext_api_key = {
            let mut statement = connection
                .prepare("PRAGMA table_info(model_configs)")
                .map_err(|_| AppError::storage("无法检查模型配置数据库版本"))?;
            let columns = statement
                .query_map([], |row| row.get::<_, String>(1))
                .map_err(|_| AppError::storage("无法检查模型配置数据库版本"))?;
            columns
                .filter_map(Result::ok)
                .any(|column| column == "api_key")
        };
        if !has_plaintext_api_key {
            connection
                .execute_batch(PLAINTEXT_KEY_MIGRATION)
                .map_err(|_| AppError::storage("无法升级模型配置存储"))?;
        } else {
            connection
                .execute(
                    "UPDATE model_configs
                     SET test_status = 'untested', tested_at = NULL, test_error_code = NULL
                     WHERE test_status != 'untested' OR tested_at IS NOT NULL OR test_error_code IS NOT NULL",
                    [],
                )
                .map_err(|_| AppError::storage("无法清理旧模型测试结果"))?;
        }
        let has_history_thinking = {
            let mut statement = connection
                .prepare("PRAGMA table_info(history_entries)")
                .map_err(|_| AppError::storage("无法检查历史数据库版本"))?;
            let columns = statement
                .query_map([], |row| row.get::<_, String>(1))
                .map_err(|_| AppError::storage("无法检查历史数据库版本"))?;
            columns
                .filter_map(Result::ok)
                .any(|column| column == "thinking_text")
        };
        if !has_history_thinking {
            connection
                .execute_batch(HISTORY_THINKING_MIGRATION)
                .map_err(|_| AppError::storage("无法升级历史记录存储"))?;
        }
        if previous_version < 4 {
            connection
                .execute(
                    "UPDATE app_settings SET capture_shortcut = ?1 WHERE capture_shortcut = ?2",
                    rusqlite::params![DEFAULT_CAPTURE_SHORTCUT, LEGACY_DEFAULT_CAPTURE_SHORTCUT],
                )
                .map_err(|_| AppError::storage("无法升级默认截图快捷键"))?;
        }
        connection
            .pragma_update(None, "user_version", 5)
            .map_err(|_| AppError::storage("无法记录数据库版本"))?;
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    pub fn read<T>(
        &self,
        operation: impl FnOnce(&Connection) -> Result<T, rusqlite::Error>,
    ) -> Result<T, AppError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| AppError::storage("数据库状态不可用"))?;
        operation(&connection).map_err(|_| AppError::storage("读取本地数据失败"))
    }

    pub fn transaction<T>(
        &self,
        operation: impl FnOnce(&Transaction<'_>) -> Result<T, rusqlite::Error>,
    ) -> Result<T, AppError> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| AppError::storage("数据库状态不可用"))?;
        let transaction = connection
            .transaction()
            .map_err(|_| AppError::storage("无法开始本地事务"))?;
        let value = operation(&transaction).map_err(|_| AppError::storage("写入本地数据失败"))?;
        transaction
            .commit()
            .map_err(|_| AppError::storage("提交本地数据失败"))?;
        Ok(value)
    }

    pub fn pragma_i64(&self, name: &str) -> Result<i64, AppError> {
        if !matches!(name, "foreign_keys" | "secure_delete" | "user_version") {
            return Err(AppError::invalid("不允许读取该数据库设置"));
        }
        self.read(|connection| {
            connection.query_row(&format!("PRAGMA {name}"), [], |row| row.get(0))
        })
    }

    pub fn count(&self, table: &str) -> Result<i64, AppError> {
        if !matches!(
            table,
            "prompt_presets" | "model_configs" | "history_entries" | "history_images"
        ) {
            return Err(AppError::invalid("不允许查询该数据表"));
        }
        self.read(|connection| {
            connection.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn legacy_database(shortcut: &str) -> Database {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch(INITIAL_SCHEMA).unwrap();
        connection
            .execute(
                "UPDATE app_settings SET capture_shortcut = ?1 WHERE id = 1",
                [shortcut],
            )
            .unwrap();
        connection.pragma_update(None, "user_version", 3).unwrap();
        Database::initialize(connection).unwrap()
    }

    #[test]
    fn platform_defaults_are_explicit() {
        assert_eq!(WINDOWS_DEFAULT_CAPTURE_SHORTCUT, "Ctrl+Shift+X");
        assert_eq!(MACOS_DEFAULT_CAPTURE_SHORTCUT, "Command+Shift+X");
    }

    #[test]
    fn legacy_default_is_migrated_for_the_current_platform() {
        let database = legacy_database(LEGACY_DEFAULT_CAPTURE_SHORTCUT);
        let shortcut = database
            .read(|connection| {
                connection.query_row(
                    "SELECT capture_shortcut FROM app_settings WHERE id = 1",
                    [],
                    |row| row.get::<_, String>(0),
                )
            })
            .unwrap();

        assert_eq!(shortcut, DEFAULT_CAPTURE_SHORTCUT);
    }

    #[test]
    fn custom_shortcut_is_preserved_during_platform_migration() {
        let database = legacy_database("Ctrl+Alt+P");
        let shortcut = database
            .read(|connection| {
                connection.query_row(
                    "SELECT capture_shortcut FROM app_settings WHERE id = 1",
                    [],
                    |row| row.get::<_, String>(0),
                )
            })
            .unwrap();

        assert_eq!(shortcut, "Ctrl+Alt+P");
    }

    #[test]
    fn legacy_history_schema_adds_the_thinking_column() {
        let database = legacy_database(DEFAULT_CAPTURE_SHORTCUT);
        let has_thinking = database
            .read(|connection| {
                let mut statement = connection.prepare("PRAGMA table_info(history_entries)")?;
                let columns = statement.query_map([], |row| row.get::<_, String>(1))?;
                Ok(columns
                    .filter_map(Result::ok)
                    .any(|column| column == "thinking_text"))
            })
            .unwrap();

        assert!(has_thinking);
        assert_eq!(database.pragma_i64("user_version").unwrap(), 5);
    }
}
