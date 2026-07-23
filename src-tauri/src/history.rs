use crate::{database::Database, error::AppError};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use image::{ImageFormat, ImageReader};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HistoryStatus {
    Success,
    Failed,
}

impl TryFrom<&str> for HistoryStatus {
    type Error = AppError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "success" => Ok(Self::Success),
            "failed" => Ok(Self::Failed),
            _ => Err(AppError::storage("历史状态无效")),
        }
    }
}

impl HistoryStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct HistoryInput {
    pub id: String,
    pub status: HistoryStatus,
    pub result_text: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub prompt_name: String,
    pub prompt_body: String,
    pub model_config_name: String,
    pub protocol: String,
    pub model_id: String,
    pub started_at: String,
    pub completed_at: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryQuery {
    pub text: Option<String>,
    pub prompt_name: Option<String>,
    pub status: Option<HistoryStatus>,
    pub cursor: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryListItem {
    pub id: String,
    pub status: HistoryStatus,
    pub result_preview: Option<String>,
    pub error_message: Option<String>,
    pub prompt_name: String,
    pub model_config_name: String,
    pub model_id: String,
    pub started_at: String,
    pub completed_at: String,
    pub has_image: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryPage {
    pub items: Vec<HistoryListItem>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntryDetail {
    pub id: String,
    pub status: HistoryStatus,
    pub result_text: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub prompt_name: String,
    pub prompt_body: String,
    pub model_config_name: String,
    pub protocol: String,
    pub model_id: String,
    pub started_at: String,
    pub completed_at: String,
    pub has_image: bool,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HistoryImageVariant {
    Thumbnail,
    Original,
}

pub fn save_history(
    database: &Database,
    enabled: bool,
    input: &HistoryInput,
    image_png: Option<&[u8]>,
) -> Result<bool, AppError> {
    if !enabled {
        return Ok(false);
    }
    match input.status {
        HistoryStatus::Success
            if input
                .result_text
                .as_deref()
                .is_none_or(|text| text.trim().is_empty()) =>
        {
            return Err(AppError::invalid("成功记录必须包含结果"));
        }
        HistoryStatus::Failed if input.error_code.is_none() => {
            return Err(AppError::invalid("失败记录必须包含错误码"));
        }
        _ => {}
    }

    let prepared_image = image_png.map(prepare_image).transpose()?;
    database.transaction(|transaction| {
        transaction.execute(
            "INSERT INTO history_entries (
                id, status, result_text, error_code, error_message, prompt_name, prompt_body,
                model_config_name, protocol, model_id, started_at, completed_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            rusqlite::params![
                input.id,
                input.status.as_str(),
                input.result_text,
                input.error_code,
                input.error_message,
                input.prompt_name,
                input.prompt_body,
                input.model_config_name,
                input.protocol,
                input.model_id,
                input.started_at,
                input.completed_at,
            ],
        )?;
        if let Some((width, height, thumbnail)) = &prepared_image {
            transaction.execute(
                "INSERT INTO history_images (
                    history_id, mime_type, width, height, original_bytes, thumbnail_bytes
                 ) VALUES (?1, 'image/png', ?2, ?3, ?4, ?5)",
                rusqlite::params![input.id, *width, *height, image_png, thumbnail],
            )?;
        }
        Ok(())
    })?;
    Ok(true)
}

fn prepare_image(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), AppError> {
    let image = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|_| AppError::invalid("无法读取截图"))?
        .decode()
        .map_err(|_| AppError::invalid("截图格式无效"))?;
    let thumbnail = image.thumbnail(320, 320);
    let mut encoded = Cursor::new(Vec::new());
    thumbnail
        .write_to(&mut encoded, ImageFormat::Png)
        .map_err(|_| AppError::storage("无法生成历史缩略图"))?;
    Ok((image.width(), image.height(), encoded.into_inner()))
}

pub fn query_history(database: &Database, query: HistoryQuery) -> Result<HistoryPage, AppError> {
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let text = query
        .text
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("%{}%", escape_like(value)));
    let prompt = query.prompt_name.filter(|value| !value.trim().is_empty());
    let status = query.status.map(|value| value.as_str().to_owned());
    let (cursor_started, cursor_id) = query
        .cursor
        .as_deref()
        .map(decode_cursor)
        .transpose()?
        .unwrap_or_default();
    let mut items = database.read(|connection| {
        let mut statement = connection.prepare(
            "SELECT h.id, h.status, substr(h.result_text, 1, 240), h.error_message,
                    h.prompt_name, h.model_config_name, h.model_id, h.started_at, h.completed_at,
                    EXISTS(SELECT 1 FROM history_images i WHERE i.history_id = h.id)
             FROM history_entries h
             WHERE (?1 IS NULL OR h.result_text LIKE ?1 ESCAPE '\\')
               AND (?2 IS NULL OR h.prompt_name = ?2)
               AND (?3 IS NULL OR h.status = ?3)
               AND (?4 IS NULL OR h.started_at < ?4 OR (h.started_at = ?4 AND h.id < ?5))
             ORDER BY h.started_at DESC, h.id DESC
             LIMIT ?6",
        )?;
        let rows = statement.query_map(
            rusqlite::params![
                text,
                prompt,
                status,
                cursor_started,
                cursor_id,
                (limit + 1) as i64
            ],
            |row| {
                let status: String = row.get(1)?;
                Ok((
                    row.get::<_, String>(0)?,
                    status,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, String>(8)?,
                    row.get::<_, bool>(9)?,
                ))
            },
        )?;
        rows.collect::<Result<Vec<_>, _>>()
    })?;
    let has_more = items.len() > limit;
    items.truncate(limit);
    let items = items
        .into_iter()
        .map(
            |(
                id,
                status,
                result_preview,
                error_message,
                prompt_name,
                model_config_name,
                model_id,
                started_at,
                completed_at,
                has_image,
            )| {
                Ok(HistoryListItem {
                    id,
                    status: HistoryStatus::try_from(status.as_str())?,
                    result_preview,
                    error_message,
                    prompt_name,
                    model_config_name,
                    model_id,
                    started_at,
                    completed_at,
                    has_image,
                })
            },
        )
        .collect::<Result<Vec<_>, AppError>>()?;
    let next_cursor = has_more
        .then(|| {
            items
                .last()
                .map(|item| encode_cursor(&item.started_at, &item.id))
        })
        .flatten();
    Ok(HistoryPage { items, next_cursor })
}

pub fn get_history_detail(database: &Database, id: &str) -> Result<HistoryEntryDetail, AppError> {
    database
        .read(|connection| {
            connection
                .query_row(
                    "SELECT h.id, h.status, h.result_text, h.error_code, h.error_message,
                        h.prompt_name, h.prompt_body, h.model_config_name, h.protocol, h.model_id,
                        h.started_at, h.completed_at,
                        EXISTS(SELECT 1 FROM history_images i WHERE i.history_id = h.id)
                 FROM history_entries h WHERE h.id = ?1",
                    [id],
                    |row| {
                        let status: String = row.get(1)?;
                        Ok((
                            row.get::<_, String>(0)?,
                            status,
                            row.get::<_, Option<String>>(2)?,
                            row.get::<_, Option<String>>(3)?,
                            row.get::<_, Option<String>>(4)?,
                            row.get::<_, String>(5)?,
                            row.get::<_, String>(6)?,
                            row.get::<_, String>(7)?,
                            row.get::<_, String>(8)?,
                            row.get::<_, String>(9)?,
                            row.get::<_, String>(10)?,
                            row.get::<_, String>(11)?,
                            row.get::<_, bool>(12)?,
                        ))
                    },
                )
                .optional()
        })?
        .map(
            |(
                id,
                status,
                result_text,
                error_code,
                error_message,
                prompt_name,
                prompt_body,
                model_config_name,
                protocol,
                model_id,
                started_at,
                completed_at,
                has_image,
            )| {
                Ok(HistoryEntryDetail {
                    id,
                    status: HistoryStatus::try_from(status.as_str())?,
                    result_text,
                    error_code,
                    error_message,
                    prompt_name,
                    prompt_body,
                    model_config_name,
                    protocol,
                    model_id,
                    started_at,
                    completed_at,
                    has_image,
                })
            },
        )
        .transpose()?
        .ok_or_else(|| {
            crate::error::AppError::new(
                crate::error::ErrorCode::NotFound,
                "历史记录不存在",
                false,
                None,
            )
        })
}

pub fn get_history_image(
    database: &Database,
    id: &str,
    variant: HistoryImageVariant,
) -> Result<Vec<u8>, AppError> {
    let column = match variant {
        HistoryImageVariant::Thumbnail => "thumbnail_bytes",
        HistoryImageVariant::Original => "original_bytes",
    };
    let bytes = database
        .read(|connection| {
            connection
                .query_row(
                    &format!("SELECT {column} FROM history_images WHERE history_id = ?1"),
                    [id],
                    |row| row.get(0),
                )
                .optional()
        })?
        .ok_or_else(|| {
            crate::error::AppError::new(
                crate::error::ErrorCode::NotFound,
                "历史图片不存在",
                false,
                None,
            )
        })?;
    ImageReader::new(Cursor::new(&bytes))
        .with_guessed_format()
        .map_err(|_| AppError::storage("历史图片损坏"))?
        .decode()
        .map_err(|_| AppError::storage("历史图片损坏"))?;
    Ok(bytes)
}

pub fn delete_history_entry(database: &Database, id: &str) -> Result<(), AppError> {
    database.transaction(|transaction| {
        let deleted = transaction.execute("DELETE FROM history_entries WHERE id = ?1", [id])?;
        if deleted == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
        Ok(())
    })
}

pub fn clear_history(database: &Database) -> Result<usize, AppError> {
    let count = database.transaction(|transaction| {
        let count = transaction.query_row("SELECT COUNT(*) FROM history_entries", [], |row| {
            row.get::<_, usize>(0)
        })?;
        transaction.execute("DELETE FROM history_entries", [])?;
        Ok(count)
    })?;
    database.read(|connection| connection.execute_batch("VACUUM"))?;
    Ok(count)
}

fn escape_like(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn encode_cursor(started_at: &str, id: &str) -> String {
    URL_SAFE_NO_PAD.encode(format!("{started_at}\n{id}"))
}

fn decode_cursor(value: &str) -> Result<(Option<String>, Option<String>), AppError> {
    let decoded = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| AppError::invalid("历史游标无效"))?;
    let decoded = String::from_utf8(decoded).map_err(|_| AppError::invalid("历史游标无效"))?;
    let (started_at, id) = decoded
        .split_once('\n')
        .ok_or_else(|| AppError::invalid("历史游标无效"))?;
    Ok((Some(started_at.to_owned()), Some(id.to_owned())))
}
