use crate::{
    credentials::CredentialStore,
    database::Database,
    error::{AppError, ErrorCode},
    providers::{ProviderProtocol, validate_endpoint},
};
use rusqlite::OptionalExtension;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub active_model_config_id: Option<String>,
    pub active_prompt_id: Option<String>,
    pub capture_shortcut: String,
    pub save_history: bool,
    pub autostart: bool,
    pub result_always_on_top: bool,
    pub onboarding_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub settings: AppSettings,
    pub prompt_count: i64,
    pub model_config_count: i64,
    pub active_prompt_id: Option<String>,
    pub active_model_config_id: Option<String>,
    pub screen_permission: crate::capture::ScreenPermission,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptSnapshot {
    pub id: String,
    pub name: String,
    pub body: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptPresetInput {
    pub id: Option<String>,
    pub name: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptPreset {
    pub id: String,
    pub name: String,
    pub body: String,
    pub is_builtin: bool,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModelSnapshot {
    pub id: String,
    pub name: String,
    pub protocol: ProviderProtocol,
    pub base_url: String,
    pub model_id: String,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelConfigInput {
    pub id: Option<String>,
    pub name: String,
    pub protocol: ProviderProtocol,
    pub base_url: String,
    pub model_id: String,
    pub api_key: Option<String>,
    #[serde(default)]
    pub clear_api_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModelConfigSummary {
    pub id: String,
    pub name: String,
    pub protocol: ProviderProtocol,
    pub base_url: String,
    pub model_id: String,
    pub has_api_key: bool,
    pub is_active: bool,
}

struct StoredModel {
    name: String,
    protocol: String,
    base_url: String,
    model_id: String,
    api_key: Option<String>,
}

pub fn load_app_snapshot(database: &Database) -> Result<AppSnapshot, AppError> {
    database.read(|connection| {
        let settings = connection.query_row(
            "SELECT active_model_config_id, active_prompt_id, capture_shortcut,
                    save_history, autostart, result_always_on_top, onboarding_completed
             FROM app_settings WHERE id = 1",
            [],
            |row| {
                Ok(AppSettings {
                    active_model_config_id: row.get(0)?,
                    active_prompt_id: row.get(1)?,
                    capture_shortcut: row.get(2)?,
                    save_history: row.get(3)?,
                    autostart: row.get(4)?,
                    result_always_on_top: row.get(5)?,
                    onboarding_completed: row.get(6)?,
                })
            },
        )?;
        let prompt_count =
            connection.query_row("SELECT COUNT(*) FROM prompt_presets", [], |row| row.get(0))?;
        let model_config_count =
            connection.query_row("SELECT COUNT(*) FROM model_configs", [], |row| row.get(0))?;
        Ok(AppSnapshot {
            active_prompt_id: settings.active_prompt_id.clone(),
            active_model_config_id: settings.active_model_config_id.clone(),
            settings,
            prompt_count,
            model_config_count,
            screen_permission: crate::capture::screen_permission_status(),
        })
    })
}

pub fn replace_shortcut<E>(
    old: &str,
    new: &str,
    mut register: impl FnMut(&str) -> Result<(), E>,
    mut unregister: impl FnMut(&str) -> Result<(), E>,
) -> Result<(), AppError> {
    register(new)
        .map_err(|_| AppError::new(ErrorCode::ShortcutConflict, "快捷键已被占用", false, None))?;
    if unregister(old).is_err() {
        let _ = unregister(new);
        return Err(AppError::new(
            ErrorCode::ShortcutConflict,
            "无法替换旧快捷键",
            false,
            None,
        ));
    }
    Ok(())
}

pub fn set_capture_shortcut_value(
    database: &Database,
    shortcut: &str,
) -> Result<AppSettings, AppError> {
    let shortcut = shortcut.trim();
    if shortcut.is_empty() || shortcut.chars().count() > 100 {
        return Err(AppError::invalid("快捷键格式无效"));
    }
    database.transaction(|transaction| {
        transaction.execute(
            "UPDATE app_settings SET capture_shortcut = ?1, updated_at = ?2 WHERE id = 1",
            rusqlite::params![shortcut, crate::analysis::now()],
        )?;
        Ok(())
    })?;
    Ok(load_app_snapshot(database)?.settings)
}

pub fn set_autostart_with<E>(
    database: &Database,
    value: bool,
    apply: impl FnOnce(bool) -> Result<(), E>,
) -> Result<AppSettings, AppError> {
    apply(value).map_err(|_| AppError::storage("无法更新开机启动状态"))?;
    database.transaction(|transaction| {
        transaction.execute(
            "UPDATE app_settings SET autostart = ?1, updated_at = ?2 WHERE id = 1",
            rusqlite::params![value, crate::analysis::now()],
        )?;
        Ok(())
    })?;
    Ok(load_app_snapshot(database)?.settings)
}

pub fn complete_onboarding(database: &Database) -> Result<(), AppError> {
    let snapshot = load_app_snapshot(database)?;
    if snapshot.active_model_config_id.is_none()
        || snapshot.active_prompt_id.is_none()
        || snapshot.screen_permission != crate::capture::ScreenPermission::Granted
    {
        return Err(AppError::invalid("截图权限、模型和提示词尚未全部就绪"));
    }
    database.transaction(|transaction| {
        transaction.execute(
            "UPDATE app_settings SET onboarding_completed = 1, updated_at = ?1 WHERE id = 1",
            [crate::analysis::now()],
        )?;
        Ok(())
    })
}

pub fn sanitize_log_line(line: &str) -> String {
    let mut value = line.to_owned();
    for marker in ["Bearer ", "api_key=", "x-api-key="] {
        let mut search_from = 0;
        while let Some(offset) = value[search_from..].find(marker) {
            let start = search_from + offset;
            let secret_start = start + marker.len();
            let secret_end = value[secret_start..]
                .find(char::is_whitespace)
                .map(|offset| secret_start + offset)
                .unwrap_or(value.len());
            value.replace_range(secret_start..secret_end, "[REDACTED]");
            search_from = secret_start + "[REDACTED]".len();
        }
    }
    if let Some(start) = value.find("raw_response=") {
        value.truncate(start);
        value.push_str("raw_response=[REDACTED]");
    }
    value
}

pub fn set_save_history(database: &Database, value: bool) -> Result<AppSettings, AppError> {
    database.transaction(|transaction| {
        transaction.execute(
            "UPDATE app_settings SET save_history = ?1, updated_at = ?2 WHERE id = 1",
            rusqlite::params![value, crate::analysis::now()],
        )?;
        Ok(())
    })?;
    Ok(load_app_snapshot(database)?.settings)
}

pub fn load_active_prompt(database: &Database) -> Result<Option<PromptSnapshot>, AppError> {
    database.read(|connection| {
        connection
            .query_row(
                "SELECT p.id, p.name, p.body
                 FROM app_settings s JOIN prompt_presets p ON p.id = s.active_prompt_id
                 WHERE s.id = 1",
                [],
                |row| {
                    Ok(PromptSnapshot {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        body: row.get(2)?,
                    })
                },
            )
            .optional()
    })
}

pub fn list_prompt_presets(database: &Database) -> Result<Vec<PromptPreset>, AppError> {
    database.read(|connection| {
        let active: Option<String> = connection.query_row(
            "SELECT active_prompt_id FROM app_settings WHERE id = 1",
            [],
            |row| row.get(0),
        )?;
        let mut statement = connection.prepare(
            "SELECT id, name, body, is_builtin FROM prompt_presets ORDER BY name COLLATE NOCASE, id",
        )?;
        let rows = statement.query_map([], |row| {
            let id: String = row.get(0)?;
            Ok(PromptPreset {
                is_active: active.as_deref() == Some(id.as_str()),
                id,
                name: row.get(1)?,
                body: row.get(2)?,
                is_builtin: row.get(3)?,
            })
        })?;
        rows.collect()
    })
}

pub fn save_prompt_preset(
    database: &Database,
    mut input: PromptPresetInput,
) -> Result<PromptPreset, AppError> {
    validate_prompt(&mut input)?;
    let id = input
        .id
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    if input.id.is_some() && !prompt_exists(database, &id)? {
        return Err(AppError::new(
            ErrorCode::NotFound,
            "提示词不存在",
            false,
            None,
        ));
    }
    let now = crate::analysis::now();
    database.transaction(|transaction| {
        transaction.execute(
            "INSERT INTO prompt_presets (id, name, body, is_builtin, created_at, updated_at)
             VALUES (?1, ?2, ?3, 0, ?4, ?4)
             ON CONFLICT(id) DO UPDATE SET name = excluded.name, body = excluded.body, updated_at = excluded.updated_at",
            rusqlite::params![id, input.name, input.body, now],
        )?;
        Ok(())
    })?;
    list_prompt_presets(database)?
        .into_iter()
        .find(|prompt| prompt.id == id)
        .ok_or_else(|| AppError::storage("保存后的提示词不可用"))
}

pub fn duplicate_prompt_preset(database: &Database, id: &str) -> Result<PromptPreset, AppError> {
    let original = list_prompt_presets(database)?
        .into_iter()
        .find(|prompt| prompt.id == id)
        .ok_or_else(|| AppError::new(ErrorCode::NotFound, "提示词不存在", false, None))?;
    for number in 1..=10_000 {
        let name = if number == 1 {
            format!("{} 副本", original.name)
        } else {
            format!("{} 副本 ({number})", original.name)
        };
        let result = save_prompt_preset(
            database,
            PromptPresetInput {
                id: None,
                name,
                body: original.body.clone(),
            },
        );
        if result.is_ok() {
            return result;
        }
    }
    Err(AppError::storage("无法生成唯一的提示词副本名称"))
}

pub fn delete_prompt_preset(database: &Database, id: &str) -> Result<(), AppError> {
    database.transaction(|transaction| {
        let deleted = transaction.execute("DELETE FROM prompt_presets WHERE id = ?1", [id])?;
        if deleted == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        Ok(())
    })
}

pub fn set_active_prompt(database: &Database, id: &str) -> Result<(), AppError> {
    if !prompt_exists(database, id)? {
        return Err(AppError::new(
            ErrorCode::NotFound,
            "提示词不存在",
            false,
            None,
        ));
    }
    database.transaction(|transaction| {
        transaction.execute(
            "UPDATE app_settings SET active_prompt_id = ?1, updated_at = ?2 WHERE id = 1",
            rusqlite::params![id, crate::analysis::now()],
        )?;
        Ok(())
    })
}

fn prompt_exists(database: &Database, id: &str) -> Result<bool, AppError> {
    database.read(|connection| {
        connection.query_row(
            "SELECT EXISTS(SELECT 1 FROM prompt_presets WHERE id = ?1)",
            [id],
            |row| row.get(0),
        )
    })
}

fn validate_prompt(input: &mut PromptPresetInput) -> Result<(), AppError> {
    input.name = input.name.trim().to_owned();
    input.body = input.body.trim().to_owned();
    if !(1..=80).contains(&input.name.chars().count()) {
        return Err(AppError::invalid("提示词名称需为 1 到 80 个字符"));
    }
    if !(1..=20_000).contains(&input.body.chars().count()) {
        return Err(AppError::invalid("提示词正文需为 1 到 20000 个字符"));
    }
    Ok(())
}

pub fn load_active_model(database: &Database) -> Result<Option<ModelSnapshot>, AppError> {
    database.read(|connection| {
        connection
            .query_row(
                "SELECT m.id, m.name, m.protocol, m.base_url, m.model_id
                 FROM app_settings s JOIN model_configs m ON m.id = s.active_model_config_id
                 WHERE s.id = 1",
                [],
                |row| {
                    let protocol: String = row.get(2)?;
                    Ok(ModelSnapshot {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        protocol: ProviderProtocol::try_from(protocol.as_str()).map_err(|_| {
                            rusqlite::Error::InvalidColumnType(
                                2,
                                "protocol".into(),
                                rusqlite::types::Type::Text,
                            )
                        })?,
                        base_url: row.get(3)?,
                        model_id: row.get(4)?,
                    })
                },
            )
            .optional()
    })
}

pub fn list_model_configs(database: &Database) -> Result<Vec<ModelConfigSummary>, AppError> {
    let (active, rows) = database.read(|connection| {
        let active: Option<String> = connection.query_row(
            "SELECT active_model_config_id FROM app_settings WHERE id = 1",
            [],
            |row| row.get(0),
        )?;
        let mut statement = connection.prepare(
            "SELECT id, name, protocol, base_url, model_id, api_key IS NOT NULL
             FROM model_configs ORDER BY name COLLATE NOCASE, id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, bool>(5)?,
            ))
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map(|rows| (active, rows))
    })?;
    rows.into_iter()
        .map(|(id, name, protocol, base_url, model_id, has_api_key)| {
            Ok(ModelConfigSummary {
                is_active: active.as_deref() == Some(id.as_str()),
                id,
                name,
                protocol: ProviderProtocol::try_from(protocol.as_str())?,
                base_url,
                model_id,
                has_api_key,
            })
        })
        .collect()
}

pub fn save_model_config(
    database: &Database,
    mut input: ModelConfigInput,
) -> Result<ModelConfigSummary, AppError> {
    if input.api_key.as_deref() == Some("") {
        input.api_key = None;
    }
    validate_model_input(&mut input)?;
    let id = input
        .id
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().to_string());
    let existing = load_stored_model(database, &id)?;
    if input.id.is_some() && existing.is_none() {
        return Err(AppError::new(
            ErrorCode::NotFound,
            "模型配置不存在",
            false,
            None,
        ));
    }
    let api_key = if input.clear_api_key {
        None
    } else {
        input
            .api_key
            .clone()
            .or_else(|| existing.as_ref().and_then(|model| model.api_key.clone()))
    };
    let now = crate::analysis::now();
    database.transaction(|transaction| {
        transaction.execute(
            "INSERT INTO model_configs (
                id, name, protocol, base_url, model_id, api_key, credential_ref,
                test_status, tested_at, test_error_code, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, 'untested', NULL, NULL, ?7, ?7)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                protocol = excluded.protocol,
                base_url = excluded.base_url,
                model_id = excluded.model_id,
                api_key = excluded.api_key,
                credential_ref = NULL,
                test_status = 'untested',
                tested_at = NULL,
                test_error_code = NULL,
                updated_at = excluded.updated_at",
            rusqlite::params![
                id,
                input.name,
                input.protocol.as_str(),
                input.base_url,
                input.model_id,
                api_key,
                now,
            ],
        )?;
        Ok(())
    })?;
    list_model_configs(database)?
        .into_iter()
        .find(|model| model.id == id)
        .ok_or_else(|| AppError::storage("保存后的模型配置不可用"))
}

pub fn delete_model_config(database: &Database, id: &str) -> Result<(), AppError> {
    database.transaction(|transaction| {
        let deleted = transaction.execute("DELETE FROM model_configs WHERE id = ?1", [id])?;
        if deleted == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        Ok(())
    })
}

pub fn set_active_model_config(database: &Database, id: &str) -> Result<(), AppError> {
    database.transaction(|transaction| {
        let exists: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM model_configs WHERE id = ?1)",
            [id],
            |row| row.get(0),
        )?;
        if !exists {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        transaction.execute(
            "UPDATE app_settings SET active_model_config_id = ?1, updated_at = ?2 WHERE id = 1",
            rusqlite::params![id, crate::analysis::now()],
        )?;
        Ok(())
    })
}

pub fn duplicate_model_config(
    database: &Database,
    id: &str,
) -> Result<ModelConfigSummary, AppError> {
    let original = load_stored_model(database, id)?
        .ok_or_else(|| AppError::new(ErrorCode::NotFound, "模型配置不存在", false, None))?;
    for number in 1..=10_000 {
        let suffix = if number == 1 {
            " 副本".to_owned()
        } else {
            format!(" 副本 ({number})")
        };
        let available = 80usize.saturating_sub(suffix.chars().count());
        let base = original.name.chars().take(available).collect::<String>();
        let name = format!("{base}{suffix}");
        let exists = database.read(|connection| {
            connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM model_configs WHERE name = ?1 COLLATE NOCASE)",
                [&name],
                |row| row.get::<_, bool>(0),
            )
        })?;
        if exists {
            continue;
        }
        return save_model_config(
            database,
            ModelConfigInput {
                id: None,
                name,
                protocol: ProviderProtocol::try_from(original.protocol.as_str())?,
                base_url: original.base_url,
                model_id: original.model_id,
                api_key: original.api_key,
                clear_api_key: false,
            },
        );
    }
    Err(AppError::storage("无法生成唯一的模型配置副本名称"))
}

pub fn migrate_model_credentials(
    database: &Database,
    credentials: &dyn CredentialStore,
) -> Result<(), AppError> {
    let rows = database.read(|connection| {
        let mut statement = connection.prepare(
            "SELECT id, credential_ref, api_key
             FROM model_configs WHERE credential_ref IS NOT NULL",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })?;
        rows.collect::<Result<Vec<_>, _>>()
    })?;

    for (id, reference, existing_key) in rows {
        let has_database_key = if existing_key.is_some() {
            true
        } else {
            match credentials.get(&reference) {
                Ok(Some(secret)) => {
                    database.transaction(|transaction| {
                        transaction.execute(
                            "UPDATE model_configs SET api_key = ?2 WHERE id = ?1",
                            rusqlite::params![id, secret.expose_secret()],
                        )?;
                        Ok(())
                    })?;
                    true
                }
                Ok(None) => {
                    database.transaction(|transaction| {
                        transaction.execute(
                            "UPDATE model_configs SET credential_ref = NULL WHERE id = ?1",
                            [&id],
                        )?;
                        Ok(())
                    })?;
                    false
                }
                Err(_) => false,
            }
        };
        if has_database_key && credentials.delete(&reference).is_ok() {
            database.transaction(|transaction| {
                transaction.execute(
                    "UPDATE model_configs SET credential_ref = NULL WHERE id = ?1",
                    [&id],
                )?;
                Ok(())
            })?;
        }
    }
    Ok(())
}

fn load_stored_model(database: &Database, id: &str) -> Result<Option<StoredModel>, AppError> {
    database.read(|connection| {
        connection
            .query_row(
                "SELECT id, name, protocol, base_url, model_id, api_key
                 FROM model_configs WHERE id = ?1",
                [id],
                |row| {
                    Ok(StoredModel {
                        name: row.get(1)?,
                        protocol: row.get(2)?,
                        base_url: row.get(3)?,
                        model_id: row.get(4)?,
                        api_key: row.get(5)?,
                    })
                },
            )
            .optional()
    })
}

pub fn load_model(database: &Database, id: &str) -> Result<Option<ModelSnapshot>, AppError> {
    database.read(|connection| {
        connection
            .query_row(
                "SELECT id, name, protocol, base_url, model_id
                 FROM model_configs WHERE id = ?1",
                [id],
                |row| {
                    let protocol: String = row.get(2)?;
                    Ok(ModelSnapshot {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        protocol: ProviderProtocol::try_from(protocol.as_str()).map_err(|_| {
                            rusqlite::Error::InvalidColumnType(
                                2,
                                "protocol".into(),
                                rusqlite::types::Type::Text,
                            )
                        })?,
                        base_url: row.get(3)?,
                        model_id: row.get(4)?,
                    })
                },
            )
            .optional()
    })
}

pub fn load_model_api_key(database: &Database, id: &str) -> Result<Option<SecretString>, AppError> {
    database
        .read(|connection| {
            connection
                .query_row(
                    "SELECT api_key FROM model_configs WHERE id = ?1",
                    [id],
                    |row| row.get::<_, Option<String>>(0),
                )
                .optional()
                .map(Option::flatten)
        })
        .map(|value| value.map(SecretString::from))
}

fn validate_model_input(input: &mut ModelConfigInput) -> Result<(), AppError> {
    input.name = input.name.trim().to_owned();
    input.base_url = input.base_url.trim().trim_end_matches('/').to_owned();
    input.model_id = input.model_id.trim().to_owned();
    if input.api_key.is_some() && input.clear_api_key {
        return Err(AppError::invalid("API Key 与清除选项不能同时提交"));
    }
    if !(1..=80).contains(&input.name.chars().count()) {
        return Err(AppError::invalid("配置名称需为 1 到 80 个字符"));
    }
    if !(1..=200).contains(&input.model_id.chars().count()) {
        return Err(AppError::invalid("模型 ID 需为 1 到 200 个字符"));
    }
    validate_endpoint(&input.base_url)?;
    Ok(())
}
