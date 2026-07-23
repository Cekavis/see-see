use see_see_lib::{
    database::Database,
    history::{HistoryQuery, query_history},
};
use std::time::Instant;

#[test]
fn ten_thousand_record_open_search_and_filter_stay_under_two_seconds() {
    let db = Database::open_in_memory().unwrap();
    db.transaction(|transaction| {
        let mut statement = transaction.prepare(
            "INSERT INTO history_entries (
                id, status, result_text, error_code, error_message, prompt_name, prompt_body,
                model_config_name, protocol, model_id, started_at, completed_at
             ) VALUES (?1, 'success', ?2, NULL, NULL, ?3, '正文', '模型', 'openai', 'vision', ?4, ?4)",
        )?;
        for index in 0..10_000 {
            let marker = if index == 8_765 { "目标旅行" } else { "普通旅行" };
            statement.execute(rusqlite::params![
                format!("id-{index:05}"),
                format!("{marker} 中文日文混合长结果 {}", "内容".repeat(80)),
                if index % 2 == 0 { "日语" } else { "通用" },
                format!("2026-07-23T00:{:02}:{:02}Z", index / 60 % 60, index % 60),
            ])?;
        }
        Ok(())
    }).unwrap();
    let started = Instant::now();
    assert_eq!(
        query_history(&db, HistoryQuery::default())
            .unwrap()
            .items
            .len(),
        50
    );
    assert_eq!(
        query_history(
            &db,
            HistoryQuery {
                text: Some("目标旅行".into()),
                ..Default::default()
            }
        )
        .unwrap()
        .items
        .len(),
        1
    );
    assert!(
        !query_history(
            &db,
            HistoryQuery {
                prompt_name: Some("日语".into()),
                ..Default::default()
            }
        )
        .unwrap()
        .items
        .is_empty()
    );
    assert!(
        started.elapsed().as_secs_f32() < 2.0,
        "history queries took {:?}",
        started.elapsed()
    );
}
