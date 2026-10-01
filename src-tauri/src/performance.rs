use crate::{database::Database, error::AppError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceQuery {
    pub period_days: Option<u32>,
    pub model_name: Option<String>,
    pub prompt_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceFilterOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelPerformanceSummary {
    pub key: String,
    pub model_config_name: String,
    pub model_id: String,
    pub protocol: String,
    pub sample_count: usize,
    pub ttft_sample_count: usize,
    pub tps_sample_count: usize,
    pub average_ttft_ms: Option<f64>,
    pub median_ttft_ms: Option<f64>,
    pub average_tps: Option<f64>,
    pub median_tps: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceReport {
    pub sample_count: usize,
    pub ttft_sample_count: usize,
    pub tps_sample_count: usize,
    pub average_ttft_ms: Option<f64>,
    pub median_ttft_ms: Option<f64>,
    pub average_tps: Option<f64>,
    pub median_tps: Option<f64>,
    pub models: Vec<ModelPerformanceSummary>,
    pub model_options: Vec<PerformanceFilterOption>,
    pub prompt_options: Vec<PerformanceFilterOption>,
}

#[derive(Debug, Clone)]
struct Sample {
    model_key: String,
    model_config_name: String,
    model_id: String,
    protocol: String,
    prompt_name: String,
    ttft_ms: Option<i64>,
    generation_ms: Option<i64>,
    output_tokens: Option<i64>,
}

#[derive(Default)]
struct Accumulator {
    model_config_name: String,
    model_id: String,
    protocol: String,
    sample_count: usize,
    ttft_ms: Vec<f64>,
    tps: Vec<f64>,
}

pub fn query_performance(
    database: &Database,
    query: PerformanceQuery,
) -> Result<PerformanceReport, AppError> {
    let period_days = query.period_days;
    if let Some(days) = period_days
        && !matches!(days, 7 | 30 | 90)
    {
        return Err(AppError::invalid("性能统计时间范围无效"));
    }
    let cutoff = period_days
        .map(|days| {
            (OffsetDateTime::now_utc() - Duration::days(days as i64))
                .format(&Rfc3339)
                .map_err(|_| AppError::storage("无法计算性能统计时间范围"))
        })
        .transpose()?;

    let samples = database.read(|connection| {
        let mut statement = connection.prepare(
            "SELECT model_config_id, model_config_name, model_id, protocol, prompt_name,
                    ttft_ms, generation_ms, output_tokens
             FROM history_entries
             WHERE status = 'success'
               AND (?1 IS NULL OR started_at >= ?1)
             ORDER BY started_at DESC, id DESC",
        )?;
        let rows = statement.query_map([cutoff.as_deref()], |row| {
            let model_config_id = row.get::<_, Option<String>>(0)?;
            let model_config_name = row.get::<_, String>(1)?;
            let model_id = row.get::<_, String>(2)?;
            let protocol = row.get::<_, String>(3)?;
            Ok(Sample {
                model_key: model_key(
                    model_config_id.as_deref(),
                    &model_config_name,
                    &protocol,
                    &model_id,
                ),
                model_config_name,
                model_id,
                protocol,
                prompt_name: row.get(4)?,
                ttft_ms: row.get(5)?,
                generation_ms: row.get(6)?,
                output_tokens: row.get(7)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
    })?;

    let model_options = model_options(&samples);
    let prompt_options = prompt_options(&samples);
    let model_filter = query
        .model_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let prompt_filter = query
        .prompt_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let mut accumulators: BTreeMap<String, Accumulator> = BTreeMap::new();
    let mut all_ttft_ms = Vec::new();
    let mut all_tps = Vec::new();
    for sample in samples.iter().filter(|sample| {
        model_filter.is_none_or(|value| sample.model_key == value)
            && prompt_filter.is_none_or(|value| sample.prompt_name == value)
    }) {
        let accumulator = accumulators
            .entry(sample.model_key.clone())
            .or_insert_with(|| Accumulator {
                model_config_name: sample.model_config_name.clone(),
                model_id: sample.model_id.clone(),
                protocol: sample.protocol.clone(),
                ..Accumulator::default()
            });
        accumulator.sample_count += 1;
        if let Some(ttft_ms) = sample.ttft_ms.filter(|value| *value >= 0) {
            let ttft_ms = ttft_ms as f64;
            accumulator.ttft_ms.push(ttft_ms);
            all_ttft_ms.push(ttft_ms);
        }
        if let (Some(output_tokens), Some(generation_ms)) =
            (sample.output_tokens, sample.generation_ms)
            && output_tokens >= 0
            && generation_ms > 0
        {
            let tps = output_tokens as f64 * 1000.0 / generation_ms as f64;
            accumulator.tps.push(tps);
            all_tps.push(tps);
        }
    }

    let mut models = accumulators
        .into_iter()
        .map(|(key, accumulator)| ModelPerformanceSummary {
            key,
            model_config_name: accumulator.model_config_name,
            model_id: accumulator.model_id,
            protocol: accumulator.protocol,
            sample_count: accumulator.sample_count,
            ttft_sample_count: accumulator.ttft_ms.len(),
            tps_sample_count: accumulator.tps.len(),
            average_ttft_ms: average(&accumulator.ttft_ms),
            median_ttft_ms: median(&accumulator.ttft_ms),
            average_tps: average(&accumulator.tps),
            median_tps: median(&accumulator.tps),
        })
        .collect::<Vec<_>>();
    models.sort_by(|left, right| {
        left.model_config_name
            .to_lowercase()
            .cmp(&right.model_config_name.to_lowercase())
            .then_with(|| left.model_id.cmp(&right.model_id))
    });

    Ok(PerformanceReport {
        sample_count: models.iter().map(|model| model.sample_count).sum(),
        ttft_sample_count: models.iter().map(|model| model.ttft_sample_count).sum(),
        tps_sample_count: models.iter().map(|model| model.tps_sample_count).sum(),
        average_ttft_ms: average(&all_ttft_ms),
        median_ttft_ms: median(&all_ttft_ms),
        average_tps: average(&all_tps),
        median_tps: median(&all_tps),
        models,
        model_options,
        prompt_options,
    })
}

fn model_key(
    model_config_id: Option<&str>,
    model_config_name: &str,
    protocol: &str,
    model_id: &str,
) -> String {
    model_config_id
        .map(str::to_owned)
        .unwrap_or_else(|| format!("{model_config_name}\u{1f}{protocol}\u{1f}{model_id}"))
}

fn model_options(samples: &[Sample]) -> Vec<PerformanceFilterOption> {
    let mut options = BTreeMap::new();
    for sample in samples {
        options
            .entry(sample.model_key.clone())
            .or_insert_with(|| PerformanceFilterOption {
                value: sample.model_key.clone(),
                label: format!("{} · {}", sample.model_config_name, sample.model_id),
            });
    }
    options.into_values().collect()
}

fn prompt_options(samples: &[Sample]) -> Vec<PerformanceFilterOption> {
    let mut values = BTreeSet::new();
    for sample in samples {
        values.insert(sample.prompt_name.clone());
    }
    values
        .into_iter()
        .map(|value| PerformanceFilterOption {
            label: value.clone(),
            value,
        })
        .collect()
}

fn average(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let middle = sorted.len() / 2;
    Some(if sorted.len() % 2 == 0 {
        (sorted[middle - 1] + sorted[middle]) / 2.0
    } else {
        sorted[middle]
    })
}
