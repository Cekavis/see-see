use see_see_lib::{
    database::Database,
    history::{HistoryInput, HistoryStatus, save_history},
    settings::load_app_snapshot,
};

fn tiny_png() -> Vec<u8> {
    use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
    use std::io::Cursor;

    let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(2, 2, Rgba([0, 0, 0, 255])));
    let mut bytes = Cursor::new(Vec::new());
    image.write_to(&mut bytes, ImageFormat::Png).unwrap();
    bytes.into_inner()
}

#[test]
fn database_defaults_and_pragmas_match_the_plan() {
    let db = Database::open_in_memory().unwrap();
    let snapshot = load_app_snapshot(&db).unwrap();
    assert!(snapshot.settings.save_history);
    assert!(!snapshot.settings.autostart);
    assert_eq!(snapshot.prompt_count, 2);
    assert!(snapshot.active_prompt_id.is_some());
    assert_eq!(db.pragma_i64("foreign_keys").unwrap(), 1);
    assert_eq!(db.pragma_i64("secure_delete").unwrap(), 1);
}

#[test]
fn history_save_is_atomic_and_respects_the_setting() {
    let db = Database::open_in_memory().unwrap();
    let input = HistoryInput {
        id: "run-1".into(),
        status: HistoryStatus::Success,
        result_text: Some("结果".into()),
        error_code: None,
        error_message: None,
        prompt_name: "日语学习解析".into(),
        prompt_body: "解释图片".into(),
        model_config_name: "测试模型".into(),
        protocol: "openai".into(),
        model_id: "vision-model".into(),
        started_at: "2026-07-23T00:00:00Z".into(),
        completed_at: "2026-07-23T00:00:01Z".into(),
    };

    let png = tiny_png();
    assert!(!save_history(&db, false, &input, Some(&png)).unwrap());
    assert_eq!(db.count("history_entries").unwrap(), 0);

    assert!(save_history(&db, true, &input, Some(&png)).unwrap());
    assert_eq!(db.count("history_entries").unwrap(), 1);
    assert_eq!(db.count("history_images").unwrap(), 1);
}

#[test]
fn invalid_success_history_rolls_back() {
    let db = Database::open_in_memory().unwrap();
    let invalid = HistoryInput {
        id: "run-2".into(),
        status: HistoryStatus::Success,
        result_text: None,
        error_code: None,
        error_message: None,
        prompt_name: "提示词".into(),
        prompt_body: "正文".into(),
        model_config_name: "模型".into(),
        protocol: "openai".into(),
        model_id: "vision-model".into(),
        started_at: "2026-07-23T00:00:00Z".into(),
        completed_at: "2026-07-23T00:00:01Z".into(),
    };

    assert!(save_history(&db, true, &invalid, Some(&tiny_png())).is_err());
    assert_eq!(db.count("history_entries").unwrap(), 0);
    assert_eq!(db.count("history_images").unwrap(), 0);
}
