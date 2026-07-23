use see_see_lib::{
    database::Database,
    settings::{
        PromptPresetInput, delete_prompt_preset, duplicate_prompt_preset, list_prompt_presets,
        load_active_prompt, save_prompt_preset, set_active_prompt,
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
fn duplicate_names_are_unique_and_deleting_active_clears_selection() {
    let db = Database::open_in_memory().unwrap();
    let original = list_prompt_presets(&db).unwrap().remove(0);
    let first = duplicate_prompt_preset(&db, &original.id).unwrap();
    let second = duplicate_prompt_preset(&db, &original.id).unwrap();
    assert_ne!(first.name, second.name);
    set_active_prompt(&db, &first.id).unwrap();
    delete_prompt_preset(&db, &first.id).unwrap();
    assert!(load_active_prompt(&db).unwrap().is_none());
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
    set_active_prompt(&db, &prompt.id).unwrap();
    let snapshot = load_active_prompt(&db).unwrap().unwrap();
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
