use see_see_lib::{
    database::Database,
    history::{
        HistoryImageVariant, HistoryInput, HistoryQuery, HistoryStatus, clear_history,
        delete_history_entry, get_history_detail, get_history_image, query_history, save_history,
    },
    settings::{load_app_snapshot, set_save_history},
};

fn tiny_png() -> Vec<u8> {
    use image::{ImageFormat, Rgba, RgbaImage};
    use std::io::Cursor;
    let image = RgbaImage::from_pixel(3, 2, Rgba([20, 30, 40, 255]));
    let mut bytes = Cursor::new(Vec::new());
    image.write_to(&mut bytes, ImageFormat::Png).unwrap();
    bytes.into_inner()
}

fn insert(db: &Database, id: &str, result: &str, prompt: &str, status: HistoryStatus) {
    let failed = status == HistoryStatus::Failed;
    save_history(
        db,
        true,
        &HistoryInput {
            id: id.into(),
            status,
            result_text: (!failed).then(|| result.into()),
            error_code: failed.then(|| "timeout".into()),
            error_message: failed.then(|| "请求超时".into()),
            prompt_name: prompt.into(),
            prompt_body: "提示正文".into(),
            model_config_name: "模型".into(),
            protocol: "openai".into(),
            model_id: "vision".into(),
            started_at: format!("2026-07-23T00:00:0{id}Z"),
            completed_at: format!("2026-07-23T00:00:1{id}Z"),
        },
        Some(&tiny_png()),
    )
    .unwrap();
}

#[test]
fn history_query_supports_cursor_escaped_search_and_filters() {
    let db = Database::open_in_memory().unwrap();
    insert(&db, "1", "旅行 100%_完成", "日语", HistoryStatus::Success);
    insert(&db, "2", "普通结果", "通用", HistoryStatus::Success);
    insert(&db, "3", "", "日语", HistoryStatus::Failed);

    let first = query_history(
        &db,
        HistoryQuery {
            limit: Some(1),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(first.items.len(), 1);
    assert!(first.next_cursor.is_some());
    let second = query_history(
        &db,
        HistoryQuery {
            cursor: first.next_cursor,
            limit: Some(1),
            ..Default::default()
        },
    )
    .unwrap();
    assert_ne!(first.items[0].id, second.items[0].id);

    let searched = query_history(
        &db,
        HistoryQuery {
            text: Some("100%_".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(searched.items.len(), 1);
    let filtered = query_history(
        &db,
        HistoryQuery {
            prompt_name: Some("日语".into()),
            status: Some(HistoryStatus::Failed),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(filtered.items.len(), 1);
}

#[test]
fn detail_images_delete_and_clear_are_consistent() {
    let db = Database::open_in_memory().unwrap();
    insert(&db, "1", "结果", "日语", HistoryStatus::Success);
    assert_eq!(
        get_history_detail(&db, "1").unwrap().result_text.as_deref(),
        Some("结果")
    );
    assert!(
        !get_history_image(&db, "1", HistoryImageVariant::Original)
            .unwrap()
            .is_empty()
    );
    assert!(
        !get_history_image(&db, "1", HistoryImageVariant::Thumbnail)
            .unwrap()
            .is_empty()
    );
    delete_history_entry(&db, "1").unwrap();
    assert_eq!(db.count("history_entries").unwrap(), 0);
    assert_eq!(db.count("history_images").unwrap(), 0);

    insert(&db, "2", "结果", "日语", HistoryStatus::Success);
    insert(&db, "3", "结果", "通用", HistoryStatus::Success);
    assert_eq!(clear_history(&db).unwrap(), 2);
    assert_eq!(db.count("history_entries").unwrap(), 0);
}

#[test]
fn corrupt_images_return_a_stable_error() {
    let db = Database::open_in_memory().unwrap();
    insert(&db, "1", "结果", "日语", HistoryStatus::Success);
    db.transaction(|transaction| {
        transaction.execute(
            "UPDATE history_images SET original_bytes = X'00' WHERE history_id = '1'",
            [],
        )?;
        Ok(())
    })
    .unwrap();
    assert!(get_history_image(&db, "1", HistoryImageVariant::Original).is_err());
}

#[test]
fn history_setting_persists_and_disables_new_writes() {
    let db = Database::open_in_memory().unwrap();
    assert!(!set_save_history(&db, false).unwrap().save_history);
    assert!(!load_app_snapshot(&db).unwrap().settings.save_history);
    let input = HistoryInput {
        id: "disabled".into(),
        status: HistoryStatus::Success,
        result_text: Some("结果".into()),
        error_code: None,
        error_message: None,
        prompt_name: "日语".into(),
        prompt_body: "正文".into(),
        model_config_name: "模型".into(),
        protocol: "openai".into(),
        model_id: "vision".into(),
        started_at: "2026-07-23T00:00:00Z".into(),
        completed_at: "2026-07-23T00:00:01Z".into(),
    };
    assert!(!save_history(&db, false, &input, Some(&tiny_png())).unwrap());
    assert_eq!(db.count("history_entries").unwrap(), 0);
}
