use see_see_lib::{
    database::Database,
    settings::{
        PromptPresetInput, delete_prompt_preset, duplicate_prompt_preset, list_prompt_presets,
        load_prompt, save_prompt_preset, set_prompt_shortcut_value,
    },
};

#[test]
fn builtins_and_prompt_limits_are_preserved() {
    let db = Database::open_in_memory().unwrap();
    let prompts = list_prompt_presets(&db).unwrap();
    assert_eq!(prompts.len(), 2);
    assert!(prompts.iter().all(|prompt| prompt.is_builtin));
    assert!(
        save_prompt_preset(
            &db,
            PromptPresetInput {
                id: None,
                name: "".into(),
                body: "x".into()
            }
        )
        .is_err()
    );
    assert!(
        save_prompt_preset(
            &db,
            PromptPresetInput {
                id: None,
                name: "有效".into(),
                body: "".into()
            }
        )
        .is_err()
    );
    assert!(
        save_prompt_preset(
            &db,
            PromptPresetInput {
                id: None,
                name: "有效".into(),
                body: "x".repeat(20_001)
            }
        )
        .is_err()
    );
}

#[test]
fn duplicate_names_are_unique_and_clones_start_without_shortcuts() {
    let db = Database::open_in_memory().unwrap();
    let original = list_prompt_presets(&db).unwrap().remove(0);
    let first = duplicate_prompt_preset(&db, &original.id).unwrap();
    let second = duplicate_prompt_preset(&db, &original.id).unwrap();
    assert_ne!(first.name, second.name);
    assert!(first.capture_shortcut.is_none());
    assert!(second.capture_shortcut.is_none());
    delete_prompt_preset(&db, &first.id).unwrap();
    assert!(
        list_prompt_presets(&db)
            .unwrap()
            .iter()
            .all(|prompt| prompt.id != first.id)
    );
}

#[test]
fn loaded_snapshot_does_not_change_after_edit() {
    let db = Database::open_in_memory().unwrap();
    let prompt = save_prompt_preset(
        &db,
        PromptPresetInput {
            id: None,
            name: "快照".into(),
            body: "原正文".into(),
        },
    )
    .unwrap();
    let snapshot = load_prompt(&db, &prompt.id).unwrap().unwrap();
    save_prompt_preset(
        &db,
        PromptPresetInput {
            id: Some(prompt.id),
            name: "快照".into(),
            body: "新正文".into(),
        },
    )
    .unwrap();
    assert_eq!(snapshot.body, "原正文");
}

#[test]
fn prompt_shortcuts_are_unique_and_previous_value_survives_conflict() {
    let db = Database::open_in_memory().unwrap();
    let mut prompts = list_prompt_presets(&db).unwrap();
    let first = prompts.remove(0);
    let second = prompts.remove(0);
    let first = set_prompt_shortcut_value(&db, &first.id, Some("Ctrl+Shift+X")).unwrap();
    assert_eq!(first.capture_shortcut.as_deref(), Some("Ctrl+Shift+X"));
    assert!(set_prompt_shortcut_value(&db, &second.id, Some("Ctrl+Shift+Y")).is_ok());
    let conflict = set_prompt_shortcut_value(&db, &second.id, Some("Ctrl+Shift+X"));
    assert_eq!(
        conflict.unwrap_err().code,
        see_see_lib::error::ErrorCode::ShortcutConflict
    );
    assert_eq!(
        list_prompt_presets(&db)
            .unwrap()
            .into_iter()
            .find(|prompt| prompt.id == second.id)
            .unwrap()
            .capture_shortcut
            .as_deref(),
        Some("Ctrl+Shift+Y")
    );
}
