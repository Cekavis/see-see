use crate::{
    credentials::CredentialStore,
    database::Database,
    error::{AppError, ErrorCode},
    providers::{ProviderProtocol, ReasoningEffort, validate_endpoint},
    state::ResultWindowDimensions,
};
use rusqlite::OptionalExtension;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub const WEBDAV_PASSWORD_CREDENTIAL_KEY: &str = "webdav-password";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub active_model_config_id: Option<String>,
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
    pub active_model_config_id: Option<String>,
    pub screen_permission: crate::capture::ScreenPermission,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WebdavSettings {
    pub url: String,
    pub username: String,
    pub remote_root: String,
    pub has_password: bool,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebdavSettingsInput {
    pub url: String,
    pub username: String,
    pub remote_root: String,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub clear_password: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigSyncResult {
    pub models: usize,
    pub prompts: usize,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigSyncSnapshot {
    pub version: u32,
    pub active_model_config_id: Option<String>,
    pub models: Vec<SyncModelConfig>,
    pub prompts: Vec<PromptSnapshot>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SyncModelConfig {
    pub id: String,
    pub name: String,
    pub protocol: ProviderProtocol,
    pub base_url: String,
    pub model_id: String,
    pub reasoning_effort: Option<ReasoningEffort>,
    // Missing API keys must not silently retain or clear another device's credentials.
    // An explicit JSON null clears the local key; a string replaces it verbatim.
    #[serde(deserialize_with = "deserialize_required_api_key")]
    pub api_key: Option<String>,
}

fn deserialize_required_api_key<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer)
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
    pub capture_shortcut: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModelSnapshot {
    pub id: String,
    pub name: String,
    pub protocol: ProviderProtocol,
    pub base_url: String,
    pub model_id: String,
    pub reasoning_effort: Option<ReasoningEffort>,
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
    pub reasoning_effort: Option<ReasoningEffort>,
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
    pub reasoning_effort: Option<ReasoningEffort>,
    pub has_api_key: bool,
    pub is_active: bool,
}

struct StoredModel {
    name: String,
    protocol: String,
    base_url: String,
    model_id: String,
    reasoning_effort: Option<ReasoningEffort>,
    api_key: Option<String>,
}

fn parse_reasoning_effort(value: Option<&str>) -> Result<Option<ReasoningEffort>, AppError> {
    match value {
        None => Ok(None),
        Some("none") => Ok(Some(ReasoningEffort::None)),
        Some("minimal") => Ok(Some(ReasoningEffort::Minimal)),
        Some("low") => Ok(Some(ReasoningEffort::Low)),
        Some("medium") => Ok(Some(ReasoningEffort::Medium)),
        Some("high") => Ok(Some(ReasoningEffort::High)),
        Some("xhigh") => Ok(Some(ReasoningEffort::XHigh)),
        Some("max") => Ok(Some(ReasoningEffort::Max)),
        Some(_) => Err(AppError::storage("模型配置中的思考强度无效")),
    }
}

fn model_reasoning_effort(
    protocol: ProviderProtocol,
    value: Option<String>,
) -> Result<Option<ReasoningEffort>, AppError> {
    if !protocol.supports_reasoning() {
        return Ok(None);
    }
    Ok(parse_reasoning_effort(value.as_deref())?.or(Some(ReasoningEffort::Low)))
}

pub fn load_app_snapshot(database: &Database) -> Result<AppSnapshot, AppError> {
    database.read(|connection| {
        let settings = connection.query_row(
            "SELECT active_model_config_id,
                    save_history, autostart, result_always_on_top, onboarding_completed
             FROM app_settings WHERE id = 1",
            [],
            |row| {
                Ok(AppSettings {
                    active_model_config_id: row.get(0)?,
                    save_history: row.get(1)?,
                    autostart: row.get(2)?,
                    result_always_on_top: row.get(3)?,
                    onboarding_completed: row.get(4)?,
                })
            },
        )?;
        let prompt_count =
            connection.query_row("SELECT COUNT(*) FROM prompt_presets", [], |row| row.get(0))?;
        let model_config_count =
            connection.query_row("SELECT COUNT(*) FROM model_configs", [], |row| row.get(0))?;
        Ok(AppSnapshot {
            active_model_config_id: settings.active_model_config_id.clone(),
            settings,
            prompt_count,
            model_config_count,
            screen_permission: crate::capture::screen_permission_status(),
        })
    })
}

pub fn load_webdav_settings(
    database: &Database,
    credentials: &dyn CredentialStore,
) -> Result<WebdavSettings, AppError> {
    let (url, username, remote_root) = database.read(|connection| {
        connection.query_row(
            "SELECT webdav_url, webdav_username, webdav_remote_root
             FROM app_settings WHERE id = 1",
            [],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
    })?;
    let has_password = credentials.get(WEBDAV_PASSWORD_CREDENTIAL_KEY)?.is_some();
    Ok(WebdavSettings {
        url,
        username,
        remote_root: if remote_root.trim().is_empty() {
            "see-see".to_owned()
        } else {
            remote_root
        },
        has_password,
    })
}

pub fn save_webdav_settings(
    database: &Database,
    credentials: &dyn CredentialStore,
    mut input: WebdavSettingsInput,
) -> Result<WebdavSettings, AppError> {
    crate::webdav::normalize_settings_input(&mut input)?;
    let previous_password = credentials.get(WEBDAV_PASSWORD_CREDENTIAL_KEY)?;
    let password = input.password.as_deref().filter(|value| !value.is_empty());
    let changes_password = input.clear_password || password.is_some();
    let has_password = !input.clear_password && (password.is_some() || previous_password.is_some());

    if input.clear_password {
        credentials.delete(WEBDAV_PASSWORD_CREDENTIAL_KEY)?;
    } else if let Some(password) = password {
        credentials.set(
            WEBDAV_PASSWORD_CREDENTIAL_KEY,
            &SecretString::from(password.to_owned()),
        )?;
    }

    let saved = database.transaction(|transaction| {
        let updated = transaction.execute(
            "UPDATE app_settings
             SET webdav_url = ?1, webdav_username = ?2, webdav_remote_root = ?3, updated_at = ?4
             WHERE id = 1",
            rusqlite::params![
                input.url,
                input.username,
                input.remote_root,
                crate::analysis::now()
            ],
        )?;
        if updated != 1 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        Ok(())
    });
    if let Err(error) = saved {
        if changes_password {
            let restored = match previous_password.as_ref() {
                Some(password) => credentials.set(WEBDAV_PASSWORD_CREDENTIAL_KEY, password),
                None => credentials.delete(WEBDAV_PASSWORD_CREDENTIAL_KEY),
            };
            if restored.is_err() {
                return Err(AppError::storage(
                    "保存 WebDAV 设置失败，且无法恢复原密码，请重新保存连接信息",
                ));
            }
        }
        return Err(error);
    }
    Ok(WebdavSettings {
        url: input.url,
        username: input.username,
        remote_root: input.remote_root,
        has_password,
    })
}

pub fn export_configuration_snapshot(database: &Database) -> Result<ConfigSyncSnapshot, AppError> {
    let (active_model_config_id, model_rows, prompt_rows) = database.read(|connection| {
        let active_model_config_id = connection.query_row(
            "SELECT active_model_config_id FROM app_settings WHERE id = 1",
            [],
            |row| row.get::<_, Option<String>>(0),
        )?;
        let mut models = connection.prepare(
            "SELECT id, name, protocol, base_url, model_id, reasoning_effort, api_key
             FROM model_configs ORDER BY name COLLATE NOCASE, id",
        )?;
        let model_rows = models
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, Option<String>>(6)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        let mut prompts = connection.prepare(
            "SELECT id, name, body FROM prompt_presets ORDER BY name COLLATE NOCASE, id",
        )?;
        let prompt_rows = prompts
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok((active_model_config_id, model_rows, prompt_rows))
    })?;

    let models = model_rows
        .into_iter()
        .map(
            |(id, name, raw_protocol, base_url, model_id, raw_reasoning_effort, api_key)| {
                let protocol = ProviderProtocol::try_from(raw_protocol.as_str())?;
                Ok(SyncModelConfig {
                    id,
                    name,
                    protocol,
                    base_url,
                    model_id,
                    reasoning_effort: model_reasoning_effort(protocol, raw_reasoning_effort)?,
                    api_key,
                })
            },
        )
        .collect::<Result<Vec<_>, AppError>>()?;
    let prompts = prompt_rows
        .into_iter()
        .map(|(id, name, body)| PromptSnapshot { id, name, body })
        .collect();
    Ok(ConfigSyncSnapshot {
        version: 1,
        active_model_config_id,
        models,
        prompts,
    })
}

pub fn validate_configuration_snapshot(snapshot: &ConfigSyncSnapshot) -> Result<(), AppError> {
    if snapshot.version != 1 {
        return Err(AppError::invalid("WebDAV 配置文件版本不受支持"));
    }
    if snapshot.models.len() > 1_000 || snapshot.prompts.len() > 10_000 {
        return Err(AppError::invalid("WebDAV 配置文件内容过多"));
    }

    let mut model_ids = HashSet::new();
    let mut model_names = HashSet::new();
    for model in &snapshot.models {
        if !(1..=200).contains(&model.id.chars().count())
            || model.id.trim() != model.id
            || !model_ids.insert(model.id.as_str())
        {
            return Err(AppError::invalid("WebDAV 配置文件中的模型 ID 无效或重复"));
        }
        if !(1..=80).contains(&model.name.chars().count())
            || model.name.trim() != model.name
            || !model_names.insert(model.name.to_ascii_lowercase())
        {
            return Err(AppError::invalid("WebDAV 配置文件中的模型名称无效或重复"));
        }
        if !(1..=200).contains(&model.model_id.chars().count())
            || model.model_id.trim() != model.model_id
        {
            return Err(AppError::invalid("WebDAV 配置文件中的模型 ID 无效"));
        }
        if model.base_url.trim() != model.base_url || model.base_url.len() > 2_000 {
            return Err(AppError::invalid("WebDAV 配置文件中的模型端点无效"));
        }
        validate_endpoint(&model.base_url)?;
        if !model.protocol.supports_reasoning() && model.reasoning_effort.is_some() {
            return Err(AppError::invalid("WebDAV 配置文件中的思考强度无效"));
        }
        if model
            .api_key
            .as_ref()
            .is_some_and(|value| value.len() > 16_384)
        {
            return Err(AppError::invalid("WebDAV 配置文件中的模型 API Key 过长"));
        }
    }
    if let Some(active_model_config_id) = &snapshot.active_model_config_id
        && !model_ids.contains(active_model_config_id.as_str())
    {
        return Err(AppError::invalid("WebDAV 配置文件中的当前模型不存在"));
    }

    let mut prompt_ids = HashSet::new();
    let mut prompt_names = HashSet::new();
    for prompt in &snapshot.prompts {
        if !(1..=200).contains(&prompt.id.chars().count())
            || prompt.id.trim() != prompt.id
            || !prompt_ids.insert(prompt.id.as_str())
        {
            return Err(AppError::invalid("WebDAV 配置文件中的提示词 ID 无效或重复"));
        }
        if !(1..=80).contains(&prompt.name.chars().count())
            || prompt.name.trim() != prompt.name
            || !prompt_names.insert(prompt.name.to_ascii_lowercase())
        {
            return Err(AppError::invalid("WebDAV 配置文件中的提示词名称无效或重复"));
        }
        if !(1..=20_000).contains(&prompt.body.chars().count()) || prompt.body.trim() != prompt.body
        {
            return Err(AppError::invalid("WebDAV 配置文件中的提示词正文无效"));
        }
    }
    Ok(())
}

struct ConfigurationMergeTarget {
    id: String,
    temporary_name: Option<String>,
}

fn configuration_identities(
    connection: &rusqlite::Connection,
    query: &str,
) -> Result<Vec<(String, String)>, rusqlite::Error> {
    let mut statement = connection.prepare(query)?;
    statement
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect()
}

fn configuration_merge_targets(
    existing: &[(String, String)],
    incoming: &[(&str, &str)],
    label: &str,
) -> Result<Vec<ConfigurationMergeTarget>, AppError> {
    let existing_ids = existing
        .iter()
        .map(|(id, _)| id.as_str())
        .collect::<HashSet<_>>();
    // SQLite's built-in NOCASE collation folds ASCII characters only.
    let existing_names = existing
        .iter()
        .map(|(id, name)| (name.to_ascii_lowercase(), id.as_str()))
        .collect::<HashMap<_, _>>();
    let mut target_ids = HashSet::new();
    let mut targets = Vec::with_capacity(incoming.len());
    for &(id, name) in incoming {
        let target_id = if existing_ids.contains(id) {
            id
        } else {
            existing_names
                .get(&name.to_ascii_lowercase())
                .copied()
                .unwrap_or(id)
        };
        if !target_ids.insert(target_id.to_owned()) {
            return Err(AppError::invalid(format!(
                "多个远程{label}指向同一本机配置，请先解决 ID 或名称冲突"
            )));
        }
        targets.push(ConfigurationMergeTarget {
            id: target_id.to_owned(),
            temporary_name: None,
        });
    }
    for &(_, name) in incoming {
        if let Some(owner_id) = existing_names.get(&name.to_ascii_lowercase())
            && !target_ids.contains(*owner_id)
        {
            return Err(AppError::invalid(format!(
                "远程{label}名称与本机未同步配置冲突，请先调整名称"
            )));
        }
    }

    let mut reserved_names = existing_names.keys().cloned().collect::<HashSet<_>>();
    reserved_names.extend(incoming.iter().map(|(_, name)| name.to_ascii_lowercase()));
    for target in &mut targets {
        if existing_ids.contains(target.id.as_str()) {
            loop {
                let name = format!("__webdav_sync_{}", Uuid::new_v4());
                if reserved_names.insert(name.clone()) {
                    target.temporary_name = Some(name);
                    break;
                }
            }
        }
    }
    Ok(targets)
}

pub fn apply_configuration_snapshot(
    database: &Database,
    snapshot: &ConfigSyncSnapshot,
) -> Result<ConfigSyncResult, AppError> {
    validate_configuration_snapshot(snapshot)?;
    let now = crate::analysis::now();
    database.transaction(|transaction| {
        let model_identities =
            configuration_identities(transaction, "SELECT id, name FROM model_configs")?;
        let prompt_identities =
            configuration_identities(transaction, "SELECT id, name FROM prompt_presets")?;
        let incoming_models = snapshot
            .models
            .iter()
            .map(|model| (model.id.as_str(), model.name.as_str()))
            .collect::<Vec<_>>();
        let incoming_prompts = snapshot
            .prompts
            .iter()
            .map(|prompt| (prompt.id.as_str(), prompt.name.as_str()))
            .collect::<Vec<_>>();
        // Resolve both complete sets before any writes, so matching never depends on
        // import order and semantic conflicts leave the transaction unchanged.
        let model_targets =
            match configuration_merge_targets(&model_identities, &incoming_models, "模型配置") {
                Ok(targets) => targets,
                Err(error) => return Ok(Err(error)),
            };
        let prompt_targets =
            match configuration_merge_targets(&prompt_identities, &incoming_prompts, "提示词") {
                Ok(targets) => targets,
                Err(error) => return Ok(Err(error)),
            };

        // Vacate names within this transaction to support ID-preserving renames and
        // name swaps without violating the unique constraints midway through import.
        for target in &model_targets {
            if let Some(name) = &target.temporary_name {
                transaction.execute(
                    "UPDATE model_configs SET name = ?1 WHERE id = ?2",
                    rusqlite::params![name, target.id],
                )?;
            }
        }
        for target in &prompt_targets {
            if let Some(name) = &target.temporary_name {
                transaction.execute(
                    "UPDATE prompt_presets SET name = ?1 WHERE id = ?2",
                    rusqlite::params![name, target.id],
                )?;
            }
        }

        for (model, target) in snapshot.models.iter().zip(&model_targets) {
            transaction.execute(
                "INSERT INTO model_configs (
                    id, name, protocol, base_url, model_id, reasoning_effort, api_key,
                    credential_ref, test_status, tested_at, test_error_code, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, 'untested', NULL, NULL, ?8, ?8)
                 ON CONFLICT(id) DO UPDATE SET
                    name = excluded.name,
                    protocol = excluded.protocol,
                    base_url = excluded.base_url,
                    model_id = excluded.model_id,
                    reasoning_effort = excluded.reasoning_effort,
                    api_key = excluded.api_key,
                    credential_ref = NULL,
                    test_status = 'untested', tested_at = NULL, test_error_code = NULL,
                    updated_at = excluded.updated_at",
                rusqlite::params![
                    target.id,
                    model.name,
                    model.protocol.as_str(),
                    model.base_url,
                    model.model_id,
                    model.reasoning_effort.map(|value| value.as_str()),
                    model.api_key,
                    now,
                ],
            )?;
        }
        for (prompt, target) in snapshot.prompts.iter().zip(&prompt_targets) {
            transaction.execute(
                "INSERT INTO prompt_presets (
                    id, name, body, is_builtin, capture_shortcut, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, 0, NULL, ?4, ?4)
                 ON CONFLICT(id) DO UPDATE SET name = excluded.name, body = excluded.body,
                    updated_at = excluded.updated_at",
                rusqlite::params![target.id, prompt.name, prompt.body, now],
            )?;
        }

        if let Some(active_model_config_id) = snapshot.active_model_config_id.as_deref()
            && let Some(index) = snapshot
                .models
                .iter()
                .position(|model| model.id == active_model_config_id)
        {
            transaction.execute(
                "UPDATE app_settings SET active_model_config_id = ?1, updated_at = ?2 WHERE id = 1",
                rusqlite::params![model_targets[index].id, now],
            )?;
        }
        Ok(Ok(ConfigSyncResult {
            models: snapshot.models.len(),
            prompts: snapshot.prompts.len(),
        }))
    })?
}

pub fn load_result_window_size(
    database: &Database,
) -> Result<Option<ResultWindowDimensions>, AppError> {
    database.read(|connection| {
        let (width, height) = connection.query_row(
            "SELECT result_window_width, result_window_height FROM app_settings WHERE id = 1",
            [],
            |row| Ok((row.get::<_, Option<i64>>(0)?, row.get::<_, Option<i64>>(1)?)),
        )?;
        Ok(
            match (
                width.and_then(|value| u32::try_from(value).ok()),
                height.and_then(|value| u32::try_from(value).ok()),
            ) {
                (Some(width), Some(height)) => Some(ResultWindowDimensions::new(width, height)),
                _ => None,
            },
        )
    })
}

pub fn save_result_window_size(
    database: &Database,
    dimensions: ResultWindowDimensions,
) -> Result<(), AppError> {
    database.transaction(|transaction| {
        transaction.execute(
            "UPDATE app_settings
             SET result_window_width = ?1, result_window_height = ?2, updated_at = ?3
             WHERE id = 1",
            rusqlite::params![
                i64::from(dimensions.width),
                i64::from(dimensions.height),
                crate::analysis::now()
            ],
        )?;
        Ok(())
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

pub fn set_prompt_shortcut_value(
    database: &Database,
    id: &str,
    shortcut: Option<&str>,
) -> Result<PromptPreset, AppError> {
    if !prompt_exists(database, id)? {
        return Err(AppError::new(
            ErrorCode::NotFound,
            "提示词不存在",
            false,
            None,
        ));
    }
    if shortcut.is_some_and(|value| value.is_empty() || value.len() > 100) {
        return Err(AppError::invalid("快捷键格式无效"));
    }
    if let Some(value) = shortcut
        && list_prompt_presets(database)?
            .iter()
            .any(|prompt| prompt.id != id && prompt.capture_shortcut.as_deref() == Some(value))
    {
        return Err(AppError::new(
            ErrorCode::ShortcutConflict,
            "快捷键已被占用",
            false,
            None,
        ));
    }
    database.transaction(|transaction| {
        transaction.execute(
            "UPDATE prompt_presets SET capture_shortcut = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![shortcut, crate::analysis::now(), id],
        )?;
        Ok(())
    })?;
    list_prompt_presets(database)?
        .into_iter()
        .find(|prompt| prompt.id == id)
        .ok_or_else(|| AppError::storage("提示词不可用"))
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
        || snapshot.prompt_count == 0
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

pub fn load_prompt(database: &Database, id: &str) -> Result<Option<PromptSnapshot>, AppError> {
    database.read(|connection| {
        connection
            .query_row(
                "SELECT id, name, body FROM prompt_presets WHERE id = ?1",
                [id],
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
        let mut statement = connection.prepare(
            "SELECT id, name, body, is_builtin, capture_shortcut FROM prompt_presets ORDER BY name COLLATE NOCASE, id",
        )?;
        let rows = statement.query_map([], |row| {
            let id: String = row.get(0)?;
            Ok(PromptPreset {
                id,
                name: row.get(1)?,
                body: row.get(2)?,
                is_builtin: row.get(3)?,
                capture_shortcut: row.get(4)?,
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
    let row = database.read(|connection| {
        connection
            .query_row(
                "SELECT m.id, m.name, m.protocol, m.base_url, m.model_id, m.reasoning_effort
                 FROM app_settings s JOIN model_configs m ON m.id = s.active_model_config_id
                 WHERE s.id = 1",
                [],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, Option<String>>(5)?,
                    ))
                },
            )
            .optional()
    })?;
    row.map(
        |(id, name, protocol, base_url, model_id, reasoning_effort)| {
            let protocol = ProviderProtocol::try_from(protocol.as_str())?;
            Ok(ModelSnapshot {
                id,
                name,
                protocol,
                base_url,
                model_id,
                reasoning_effort: model_reasoning_effort(protocol, reasoning_effort)?,
            })
        },
    )
    .transpose()
}

pub fn list_model_configs(database: &Database) -> Result<Vec<ModelConfigSummary>, AppError> {
    let (active, rows) = database.read(|connection| {
        let active: Option<String> = connection.query_row(
            "SELECT active_model_config_id FROM app_settings WHERE id = 1",
            [],
            |row| row.get(0),
        )?;
        let mut statement = connection.prepare(
            "SELECT id, name, protocol, base_url, model_id, reasoning_effort, api_key IS NOT NULL
             FROM model_configs ORDER BY name COLLATE NOCASE, id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
                row.get::<_, bool>(6)?,
            ))
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map(|rows| (active, rows))
    })?;
    rows.into_iter()
        .map(
            |(id, name, protocol, base_url, model_id, raw_reasoning_effort, has_api_key)| {
                let protocol = ProviderProtocol::try_from(protocol.as_str())?;
                Ok(ModelConfigSummary {
                    is_active: active.as_deref() == Some(id.as_str()),
                    id,
                    name,
                    protocol,
                    base_url,
                    model_id,
                    reasoning_effort: model_reasoning_effort(protocol, raw_reasoning_effort)?,
                    has_api_key,
                })
            },
        )
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
                id, name, protocol, base_url, model_id, reasoning_effort, api_key, credential_ref,
                test_status, tested_at, test_error_code, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL, 'untested', NULL, NULL, ?8, ?8)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                protocol = excluded.protocol,
                base_url = excluded.base_url,
                model_id = excluded.model_id,
                reasoning_effort = excluded.reasoning_effort,
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
                input.reasoning_effort.map(|effort| effort.as_str()),
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
                reasoning_effort: original.reasoning_effort,
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
    let row = database.read(|connection| {
        connection
            .query_row(
                "SELECT name, protocol, base_url, model_id, reasoning_effort, api_key
                 FROM model_configs WHERE id = ?1",
                [id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, Option<String>>(4)?,
                        row.get::<_, Option<String>>(5)?,
                    ))
                },
            )
            .optional()
    })?;
    row.map(
        |(name, protocol, base_url, model_id, raw_reasoning_effort, api_key)| {
            let parsed_protocol = ProviderProtocol::try_from(protocol.as_str())?;
            Ok(StoredModel {
                name,
                protocol,
                base_url,
                model_id,
                reasoning_effort: model_reasoning_effort(parsed_protocol, raw_reasoning_effort)?,
                api_key,
            })
        },
    )
    .transpose()
}

pub fn load_model(database: &Database, id: &str) -> Result<Option<ModelSnapshot>, AppError> {
    let row = database.read(|connection| {
        connection
            .query_row(
                "SELECT id, name, protocol, base_url, model_id, reasoning_effort
                 FROM model_configs WHERE id = ?1",
                [id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                        row.get::<_, String>(3)?,
                        row.get::<_, String>(4)?,
                        row.get::<_, Option<String>>(5)?,
                    ))
                },
            )
            .optional()
    })?;
    row.map(
        |(id, name, protocol, base_url, model_id, raw_reasoning_effort)| {
            let protocol = ProviderProtocol::try_from(protocol.as_str())?;
            Ok(ModelSnapshot {
                id,
                name,
                protocol,
                base_url,
                model_id,
                reasoning_effort: model_reasoning_effort(protocol, raw_reasoning_effort)?,
            })
        },
    )
    .transpose()
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
    input.reasoning_effort = input
        .protocol
        .supports_reasoning()
        .then_some(input.reasoning_effort.unwrap_or_default());
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
