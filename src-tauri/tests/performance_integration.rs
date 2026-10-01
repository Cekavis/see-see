use see_see_lib::{
    database::Database,
    history::{clear_history, delete_history_entry},
    performance::{PerformanceQuery, query_performance},
};

fn insert_sample(
    db: &Database,
    id: &str,
    status: &str,
    model_id: &str,
    model_name: &str,
    prompt_name: &str,
    ttft_ms: Option<i64>,
    generation_ms: Option<i64>,
    output_tokens: Option<i64>,
    started_at: &str,
) {
    db.transaction(|transaction| {
        transaction.execute(
            "INSERT INTO history_entries (
                id, status, result_text, error_code, prompt_name, prompt_body,
                model_config_name, protocol, model_id, output_tokens, ttft_ms,
                generation_ms, started_at, completed_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, '正文', ?6, 'openai', ?7, ?8, ?9, ?10, ?11, ?11)",
            rusqlite::params![
                id,
                status,
                (status == "success").then_some("结果"),
                (status == "failed").then_some("timeout"),
                prompt_name,
                model_name,
                model_id,
                output_tokens,
                ttft_ms,
                generation_ms,
                started_at,
            ],
        )?;
        Ok(())
    })
    .unwrap();
}

#[test]
fn performance_query_aggregates_successful_samples_and_excludes_missing_tps() {
    let db = Database::open_in_memory().unwrap();
    insert_sample(
        &db,
        "m1-a",
        "success",
        "vision-a",
        "模型 A",
        "翻译",
        Some(100),
        Some(1_000),
        Some(20),
        "2026-09-01T00:00:00Z",
    );
    insert_sample(
        &db,
        "m1-b",
        "success",
        "vision-a",
        "模型 A",
        "翻译",
        Some(300),
        Some(2_000),
        Some(40),
        "2026-09-02T00:00:00Z",
    );
    insert_sample(
        &db,
        "m2-a",
        "success",
        "vision-b",
        "模型 B",
        "解析",
        Some(500),
        Some(1_000),
        None,
        "2026-09-03T00:00:00Z",
    );
    insert_sample(
        &db,
        "failed",
        "failed",
        "vision-a",
        "模型 A",
        "翻译",
        Some(999),
        Some(1_000),
        Some(99),
        "2026-09-04T00:00:00Z",
    );
    insert_sample(
        &db,
        "old",
        "success",
        "vision-a",
        "模型 A",
        "翻译",
        Some(700),
        Some(1_000),
        Some(10),
        "2026-01-01T00:00:00Z",
    );

    let report = query_performance(
        &db,
        PerformanceQuery {
            period_days: Some(90),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(report.sample_count, 3);
    assert_eq!(report.ttft_sample_count, 3);
    assert_eq!(report.tps_sample_count, 2);
    assert_eq!(report.average_ttft_ms, Some(300.0));
    assert_eq!(report.median_ttft_ms, Some(300.0));
    assert_eq!(report.average_tps, Some(20.0));
    assert_eq!(report.median_tps, Some(20.0));

    let model_a = report
        .models
        .iter()
        .find(|model| model.model_id == "vision-a")
        .unwrap();
    assert_eq!(model_a.sample_count, 2);
    assert_eq!(model_a.average_ttft_ms, Some(200.0));
    assert_eq!(model_a.median_ttft_ms, Some(200.0));
    assert_eq!(model_a.tps_sample_count, 2);
    assert_eq!(model_a.average_tps, Some(20.0));

    let model_b = report
        .models
        .iter()
        .find(|model| model.model_id == "vision-b")
        .unwrap();
    assert_eq!(model_b.sample_count, 1);
    assert_eq!(model_b.tps_sample_count, 0);
    assert_eq!(model_b.average_tps, None);
}

#[test]
fn performance_query_filters_by_model_prompt_and_rejects_unknown_period() {
    let db = Database::open_in_memory().unwrap();
    insert_sample(
        &db,
        "a",
        "success",
        "vision-a",
        "模型 A",
        "翻译",
        Some(100),
        Some(1_000),
        Some(10),
        "2026-09-01T00:00:00Z",
    );
    insert_sample(
        &db,
        "b",
        "success",
        "vision-b",
        "模型 B",
        "解析",
        Some(200),
        Some(1_000),
        Some(20),
        "2026-09-02T00:00:00Z",
    );

    let model_key = "模型 A\u{1f}openai\u{1f}vision-a".to_owned();
    let filtered = query_performance(
        &db,
        PerformanceQuery {
            model_name: Some(model_key),
            prompt_name: Some("翻译".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(filtered.sample_count, 1);
    assert_eq!(filtered.models.len(), 1);
    assert_eq!(filtered.models[0].model_id, "vision-a");

    assert!(
        query_performance(
            &db,
            PerformanceQuery {
                period_days: Some(1),
                ..Default::default()
            },
        )
        .is_err()
    );
}

#[test]
fn performance_query_drops_samples_after_delete_and_clear() {
    let db = Database::open_in_memory().unwrap();
    insert_sample(
        &db,
        "delete-me",
        "success",
        "vision-a",
        "模型 A",
        "翻译",
        Some(100),
        Some(1_000),
        Some(10),
        "2026-09-01T00:00:00Z",
    );
    insert_sample(
        &db,
        "clear-me",
        "success",
        "vision-b",
        "模型 B",
        "解析",
        Some(200),
        Some(1_000),
        Some(20),
        "2026-09-02T00:00:00Z",
    );

    assert_eq!(
        query_performance(&db, PerformanceQuery::default())
            .unwrap()
            .sample_count,
        2
    );
    delete_history_entry(&db, "delete-me").unwrap();
    assert_eq!(
        query_performance(&db, PerformanceQuery::default())
            .unwrap()
            .sample_count,
        1
    );
    clear_history(&db).unwrap();
    assert_eq!(
        query_performance(&db, PerformanceQuery::default())
            .unwrap()
            .sample_count,
        0
    );
}
