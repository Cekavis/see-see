use crate::error::AppError;
use rusqlite::{Connection, Transaction};
use std::{path::Path, sync::Mutex};

const MIGRATION: &str = include_str!("../migrations/0001_init.sql");

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
            .execute_batch(MIGRATION)
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
        connection
            .pragma_update(None, "user_version", 2)
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
