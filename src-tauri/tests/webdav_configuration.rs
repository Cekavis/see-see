use secrecy::ExposeSecret;
use see_see_lib::{
    credentials::{CredentialStore, MemoryCredentialStore},
    database::Database,
    error::ErrorCode,
    providers::{ProviderProtocol, ReasoningEffort},
    settings::{
        self, ConfigSyncSnapshot, ModelConfigInput, PromptPresetInput, PromptSnapshot,
        SyncModelConfig, WebdavSettingsInput,
    },
};

fn model_input(name: &str, api_key: Option<&str>) -> ModelConfigInput {
    ModelConfigInput {
        id: None,
        name: name.into(),
        protocol: ProviderProtocol::OpenAiResponses,
        base_url: "https://models.example.test/v1".into(),
        model_id: "vision-test".into(),
        reasoning_effort: Some(ReasoningEffort::High),
        api_key: api_key.map(str::to_owned),
        clear_api_key: false,
    }
}

fn sync_model(id: &str, name: &str, api_key: Option<&str>) -> SyncModelConfig {
    SyncModelConfig {
        id: id.into(),
        name: name.into(),
        protocol: ProviderProtocol::OpenAiResponses,
        base_url: "https://remote-models.example.test/v1".into(),
        model_id: "remote-vision-test".into(),
        reasoning_effort: Some(ReasoningEffort::XHigh),
        api_key: api_key.map(str::to_owned),
    }
}

fn sync_prompt(id: &str, name: &str) -> PromptSnapshot {
    PromptSnapshot {
        id: id.into(),
        name: name.into(),
        body: "来自另一台设备的提示词正文。".into(),
    }
}

fn snapshot(models: Vec<SyncModelConfig>, prompts: Vec<PromptSnapshot>) -> ConfigSyncSnapshot {
    ConfigSyncSnapshot {
        version: 1,
        active_model_config_id: models.first().map(|model| model.id.clone()),
        models,
        prompts,
    }
}

fn create_prompt(database: &Database, name: &str) -> settings::PromptPreset {
    settings::save_prompt_preset(
        database,
        PromptPresetInput {
            id: None,
            name: name.into(),
            body: "原有的本机提示词正文。".into(),
        },
    )
    .unwrap()
}

fn stored_key(database: &Database, id: &str) -> Option<String> {
    settings::load_model_api_key(database, id)
        .unwrap()
        .map(|key| key.expose_secret().to_owned())
}

fn stored_password(credentials: &dyn CredentialStore) -> Option<String> {
    credentials
        .get(settings::WEBDAV_PASSWORD_CREDENTIAL_KEY)
        .unwrap()
        .map(|password| password.expose_secret().to_owned())
}

fn webdav_input(password: Option<&str>) -> WebdavSettingsInput {
    WebdavSettingsInput {
        url: " https://dav.example.test/root/ ".into(),
        username: " sync-user ".into(),
        remote_root: String::new(),
        password: password.map(str::to_owned),
        clear_password: false,
    }
}

#[test]
fn exported_model_keys_round_trip_with_metadata_and_active_selection() {
    let source = Database::open_in_memory().unwrap();
    let keyed = settings::save_model_config(
        &source,
        model_input("带凭据的模型", Some("  example-model-key  ")),
    )
    .unwrap();
    let keyless = settings::save_model_config(&source, model_input("无凭据的模型", None)).unwrap();
    settings::set_active_model_config(&source, &keyed.id).unwrap();
    let credentials = MemoryCredentialStore::default();
    settings::save_webdav_settings(
        &source,
        &credentials,
        webdav_input(Some("example-webdav-password")),
    )
    .unwrap();
    let exported = settings::export_configuration_snapshot(&source).unwrap();
    let payload = serde_json::to_vec(&exported).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&payload).unwrap();
    assert!(
        json["models"]
            .as_array()
            .unwrap()
            .iter()
            .all(|model| model.get("apiKey").is_some())
    );
    assert!(
        !String::from_utf8(payload.clone())
            .unwrap()
            .contains("example-webdav-password")
    );
    assert!(json.get("webdav").is_none());
    let decoded: ConfigSyncSnapshot = serde_json::from_slice(&payload).unwrap();

    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("configuration.sqlite3");
    {
        let destination = Database::open(&path).unwrap();
        let counts = settings::apply_configuration_snapshot(&destination, &decoded).unwrap();
        assert_eq!(counts.models, 2);
        assert_eq!(counts.prompts, 2);
    }
    let destination = Database::open(&path).unwrap();
    assert!(stored_key(&destination, &keyed.id).as_deref() == Some("  example-model-key  "));
    assert!(stored_key(&destination, &keyless.id).is_none());
    let imported = settings::load_model(&destination, &keyed.id)
        .unwrap()
        .unwrap();
    assert_eq!(imported.protocol, ProviderProtocol::OpenAiResponses);
    assert_eq!(imported.reasoning_effort, Some(ReasoningEffort::High));
    assert_eq!(imported.model_id, "vision-test");
    assert_eq!(
        settings::load_app_snapshot(&destination)
            .unwrap()
            .active_model_config_id,
        Some(keyed.id)
    );
}

#[test]
fn absent_api_key_is_rejected_and_explicit_null_clears_changed_endpoint_key() {
    let database = Database::open_in_memory().unwrap();
    let local =
        settings::save_model_config(&database, model_input("Local", Some("old-test-key"))).unwrap();
    let incoming = snapshot(vec![sync_model(&local.id, "Local", None)], vec![]);
    let mut incomplete = serde_json::to_value(&incoming).unwrap();
    incomplete["models"][0]
        .as_object_mut()
        .unwrap()
        .remove("apiKey");
    assert!(serde_json::from_value::<ConfigSyncSnapshot>(incomplete).is_err());
    assert!(stored_key(&database, &local.id).as_deref() == Some("old-test-key"));

    let encoded = serde_json::to_value(&incoming).unwrap();
    assert!(encoded["models"][0]["apiKey"].is_null());
    let decoded = serde_json::from_value::<ConfigSyncSnapshot>(encoded).unwrap();
    settings::apply_configuration_snapshot(&database, &decoded).unwrap();
    assert!(stored_key(&database, &local.id).is_none());
    assert_eq!(
        settings::load_model(&database, &local.id)
            .unwrap()
            .unwrap()
            .base_url,
        "https://remote-models.example.test/v1"
    );
}

#[test]
fn shortcuts_are_neither_exported_nor_imported_for_windows_and_macos_values() {
    let source = Database::open_in_memory().unwrap();
    let original = create_prompt(&source, "Shared prompt");
    settings::set_prompt_shortcut_value(&source, &original.id, Some("Ctrl+Shift+X")).unwrap();
    let exported = settings::export_configuration_snapshot(&source).unwrap();
    let json = serde_json::to_value(&exported).unwrap();
    for prompt in json["prompts"].as_array().unwrap() {
        assert!(prompt.get("captureShortcut").is_none());
    }

    for shortcut in ["Ctrl+Shift+Q", "Command+Shift+X"] {
        let destination = Database::open_in_memory().unwrap();
        let local = create_prompt(&destination, "Shared prompt");
        settings::set_prompt_shortcut_value(&destination, &local.id, Some(shortcut)).unwrap();
        let mut incoming_json = serde_json::to_value(snapshot(
            vec![],
            vec![
                sync_prompt(&original.id, "Shared prompt"),
                sync_prompt("new-prompt", "New prompt"),
            ],
        ))
        .unwrap();
        incoming_json["prompts"][0]["captureShortcut"] = "Alt+Shift+F1".into();
        incoming_json["prompts"][1]["captureShortcut"] = "Alt+Shift+F2".into();
        let incoming: ConfigSyncSnapshot = serde_json::from_value(incoming_json).unwrap();
        settings::apply_configuration_snapshot(&destination, &incoming).unwrap();
        let prompts = settings::list_prompt_presets(&destination).unwrap();
        let updated = prompts.iter().find(|prompt| prompt.id == local.id).unwrap();
        assert_eq!(updated.capture_shortcut.as_deref(), Some(shortcut));
        assert_eq!(updated.body, "来自另一台设备的提示词正文。");
        let added = prompts
            .iter()
            .find(|prompt| prompt.id == "new-prompt")
            .unwrap();
        assert_eq!(added.capture_shortcut, None);
        assert!(!prompts.iter().any(|prompt| prompt.id == original.id));
    }
}

#[test]
fn merge_prefers_ids_then_case_insensitive_names_and_preserves_local_only_rows() {
    let database = Database::open_in_memory().unwrap();
    let by_id = settings::save_model_config(
        &database,
        model_input("Original name", Some("old-test-key")),
    )
    .unwrap();
    let by_name =
        settings::save_model_config(&database, model_input("Shared model", None)).unwrap();
    let local_only = settings::save_model_config(
        &database,
        model_input("Device only", Some("local-test-key")),
    )
    .unwrap();
    let prompt_by_id = create_prompt(&database, "Original prompt");
    let prompt_by_name = create_prompt(&database, "Shared prompt");
    let local_prompt = create_prompt(&database, "Device prompt");
    let mut incoming = snapshot(
        vec![
            sync_model(&by_id.id, "Renamed model", Some("updated-test-key")),
            sync_model("remote-model-id", "sHARED MODEL", Some("shared-test-key")),
        ],
        vec![
            sync_prompt(&prompt_by_id.id, "Renamed prompt"),
            sync_prompt("remote-prompt-id", "sHARED PROMPT"),
        ],
    );
    incoming.active_model_config_id = Some("remote-model-id".into());
    let result = settings::apply_configuration_snapshot(&database, &incoming).unwrap();
    assert_eq!(result.models, 2);
    assert_eq!(result.prompts, 2);
    assert_eq!(database.count("model_configs").unwrap(), 3);
    assert_eq!(
        settings::load_model(&database, &by_id.id)
            .unwrap()
            .unwrap()
            .name,
        "Renamed model"
    );
    assert!(
        settings::load_model(&database, "remote-model-id")
            .unwrap()
            .is_none()
    );
    assert!(stored_key(&database, &by_name.id).as_deref() == Some("shared-test-key"));
    assert!(stored_key(&database, &local_only.id).as_deref() == Some("local-test-key"));
    assert_eq!(
        settings::load_app_snapshot(&database)
            .unwrap()
            .active_model_config_id,
        Some(by_name.id)
    );
    let prompts = settings::list_prompt_presets(&database).unwrap();
    assert_eq!(prompts.len(), 5);
    assert_eq!(
        prompts
            .iter()
            .find(|prompt| prompt.id == prompt_by_name.id)
            .unwrap()
            .name,
        "sHARED PROMPT"
    );
    assert_eq!(
        prompts
            .iter()
            .find(|prompt| prompt.id == prompt_by_id.id)
            .unwrap()
            .name,
        "Renamed prompt"
    );
    assert_eq!(
        prompts
            .iter()
            .find(|prompt| prompt.id == local_prompt.id)
            .unwrap()
            .body,
        local_prompt.body
    );
}

#[test]
fn model_and_prompt_name_swaps_preserve_id_and_local_shortcuts() {
    let database = Database::open_in_memory().unwrap();
    let first = settings::save_model_config(&database, model_input("First model", None)).unwrap();
    let second = settings::save_model_config(&database, model_input("Second model", None)).unwrap();
    let first_prompt = create_prompt(&database, "First prompt");
    let second_prompt = create_prompt(&database, "Second prompt");
    settings::set_prompt_shortcut_value(&database, &first_prompt.id, Some("Command+Shift+X"))
        .unwrap();
    settings::set_prompt_shortcut_value(&database, &second_prompt.id, Some("Ctrl+Shift+Q"))
        .unwrap();
    let incoming = snapshot(
        vec![
            sync_model(&first.id, "Second model", Some("first-test-key")),
            sync_model(&second.id, "First model", Some("second-test-key")),
        ],
        vec![
            sync_prompt(&first_prompt.id, "Second prompt"),
            sync_prompt(&second_prompt.id, "First prompt"),
        ],
    );
    settings::apply_configuration_snapshot(&database, &incoming).unwrap();
    assert_eq!(
        settings::load_model(&database, &first.id)
            .unwrap()
            .unwrap()
            .name,
        "Second model"
    );
    assert_eq!(
        settings::load_model(&database, &second.id)
            .unwrap()
            .unwrap()
            .name,
        "First model"
    );
    assert!(stored_key(&database, &first.id).as_deref() == Some("first-test-key"));
    let prompts = settings::list_prompt_presets(&database).unwrap();
    let first_updated = prompts
        .iter()
        .find(|prompt| prompt.id == first_prompt.id)
        .unwrap();
    assert_eq!(first_updated.name, "Second prompt");
    assert_eq!(
        first_updated.capture_shortcut.as_deref(),
        Some("Command+Shift+X")
    );
    let second_updated = prompts
        .iter()
        .find(|prompt| prompt.id == second_prompt.id)
        .unwrap();
    assert_eq!(second_updated.name, "First prompt");
    assert_eq!(
        second_updated.capture_shortcut.as_deref(),
        Some("Ctrl+Shift+Q")
    );
}

#[test]
fn multiple_remote_ids_cannot_target_the_same_local_model_or_prompt() {
    let database = Database::open_in_memory().unwrap();
    let model = settings::save_model_config(
        &database,
        model_input("Shared model", Some("local-test-key")),
    )
    .unwrap();
    let prompt = create_prompt(&database, "Shared prompt");
    let before = settings::export_configuration_snapshot(&database).unwrap();
    let model_collision = snapshot(
        vec![
            sync_model(&model.id, "Renamed model", None),
            sync_model("another-model-id", "Shared model", Some("remote-test-key")),
        ],
        vec![],
    );
    let error = settings::apply_configuration_snapshot(&database, &model_collision).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidInput);
    assert!(error.message.contains("同一本机配置"));
    assert!(settings::export_configuration_snapshot(&database).unwrap() == before);

    let prompt_collision = snapshot(
        vec![sync_model(&model.id, "Updated model", None)],
        vec![
            sync_prompt(&prompt.id, "Renamed prompt"),
            sync_prompt("another-prompt-id", "Shared prompt"),
        ],
    );
    let error = settings::apply_configuration_snapshot(&database, &prompt_collision).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidInput);
    assert!(error.message.contains("同一本机配置"));
    assert!(settings::export_configuration_snapshot(&database).unwrap() == before);
}

#[test]
fn rename_conflicts_with_local_only_rows_leave_all_configurations_unchanged() {
    let database = Database::open_in_memory().unwrap();
    let first =
        settings::save_model_config(&database, model_input("First", Some("first-test-key")))
            .unwrap();
    settings::save_model_config(&database, model_input("Reserved", None)).unwrap();
    let prompt = create_prompt(&database, "First prompt");
    create_prompt(&database, "Reserved prompt");
    let before = settings::export_configuration_snapshot(&database).unwrap();
    for incoming in [
        snapshot(vec![sync_model(&first.id, "Reserved", None)], vec![]),
        snapshot(
            vec![sync_model(&first.id, "Updated first", None)],
            vec![sync_prompt(&prompt.id, "Reserved prompt")],
        ),
    ] {
        let error = settings::apply_configuration_snapshot(&database, &incoming).unwrap_err();
        assert_eq!(error.code, ErrorCode::InvalidInput);
        assert!(error.message.contains("未同步配置冲突"));
        assert!(settings::export_configuration_snapshot(&database).unwrap() == before);
    }
}

#[test]
fn sql_failure_after_model_changes_rolls_back_the_entire_import() {
    let database = Database::open_in_memory().unwrap();
    let model = settings::save_model_config(
        &database,
        model_input("Original model", Some("original-test-key")),
    )
    .unwrap();
    let before = settings::export_configuration_snapshot(&database).unwrap();
    database
        .transaction(|transaction| {
            transaction.execute_batch(
                "CREATE TRIGGER reject_imported_prompt BEFORE INSERT ON prompt_presets
             WHEN NEW.name = 'Blocked prompt'
             BEGIN SELECT RAISE(ABORT, 'forced import failure'); END;",
            )
        })
        .unwrap();
    let incoming = snapshot(
        vec![sync_model(
            &model.id,
            "Updated model",
            Some("updated-test-key"),
        )],
        vec![sync_prompt("blocked-prompt", "Blocked prompt")],
    );
    assert!(settings::apply_configuration_snapshot(&database, &incoming).is_err());
    assert!(settings::export_configuration_snapshot(&database).unwrap() == before);
}

#[test]
fn invalid_documents_are_rejected_before_changes() {
    let database = Database::open_in_memory().unwrap();
    let before = settings::export_configuration_snapshot(&database).unwrap();
    let valid = snapshot(
        vec![sync_model("model-id", "Model", None)],
        vec![sync_prompt("prompt-id", "Prompt")],
    );
    let mut invalid_version = valid.clone();
    invalid_version.version = 2;
    let mut invalid_active_model = valid.clone();
    invalid_active_model.active_model_config_id = Some("missing-model".into());
    let mut duplicate_name = valid.clone();
    duplicate_name
        .models
        .push(sync_model("another-model", "mODEL", None));
    let mut invalid_endpoint = valid.clone();
    invalid_endpoint.models[0].base_url = "http://models.example.test/v1".into();
    let mut too_long_key = valid.clone();
    too_long_key.models[0].api_key = Some("x".repeat(16_385));
    let mut too_long_prompt = valid.clone();
    too_long_prompt.prompts[0].body = "x".repeat(20_001);
    for invalid in [
        invalid_version,
        invalid_active_model,
        duplicate_name,
        invalid_endpoint,
        too_long_key,
        too_long_prompt,
    ] {
        assert!(settings::apply_configuration_snapshot(&database, &invalid).is_err());
        assert!(settings::export_configuration_snapshot(&database).unwrap() == before);
    }
    let mut unknown_protocol = serde_json::to_value(valid).unwrap();
    unknown_protocol["models"][0]["protocol"] = "unknown-protocol".into();
    assert!(serde_json::from_value::<ConfigSyncSnapshot>(unknown_protocol).is_err());
}

#[test]
fn webdav_defaults_and_settings_survive_reopening_without_password_echo() {
    let credentials = MemoryCredentialStore::default();
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("settings.sqlite3");
    {
        let database = Database::open(&path).unwrap();
        let initial = settings::load_webdav_settings(&database, &credentials).unwrap();
        assert_eq!(initial.url, "");
        assert_eq!(initial.username, "");
        assert_eq!(initial.remote_root, "see-see");
        assert!(!initial.has_password);
        let saved = settings::save_webdav_settings(
            &database,
            &credentials,
            webdav_input(Some("  example-webdav-password  ")),
        )
        .unwrap();
        assert_eq!(saved.url, "https://dav.example.test/root");
        assert_eq!(saved.username, "sync-user");
        assert_eq!(saved.remote_root, "see-see");
        assert!(saved.has_password);
        assert!(
            serde_json::to_value(saved)
                .unwrap()
                .get("password")
                .is_none()
        );
    }
    let reopened = Database::open(&path).unwrap();
    let saved = settings::load_webdav_settings(&reopened, &credentials).unwrap();
    assert_eq!(saved.url, "https://dav.example.test/root");
    assert_eq!(saved.username, "sync-user");
    assert_eq!(saved.remote_root, "see-see");
    assert!(saved.has_password);
    assert!(stored_password(&credentials).as_deref() == Some("  example-webdav-password  "));
}

#[test]
fn passwords_preserve_whitespace_and_empty_input_retains_until_explicit_clear() {
    let database = Database::open_in_memory().unwrap();
    let credentials = MemoryCredentialStore::default();
    settings::save_webdav_settings(
        &database,
        &credentials,
        webdav_input(Some("  password-with-spaces  ")),
    )
    .unwrap();
    for password in [None, Some("")] {
        let retained =
            settings::save_webdav_settings(&database, &credentials, webdav_input(password))
                .unwrap();
        assert!(retained.has_password);
        assert!(stored_password(&credentials).as_deref() == Some("  password-with-spaces  "));
    }
    settings::save_webdav_settings(&database, &credentials, webdav_input(Some("   "))).unwrap();
    assert!(stored_password(&credentials).as_deref() == Some("   "));
    let mut clear = webdav_input(None);
    clear.clear_password = true;
    let cleared = settings::save_webdav_settings(&database, &credentials, clear).unwrap();
    assert!(!cleared.has_password);
    assert!(stored_password(&credentials).is_none());
}

#[test]
fn invalid_settings_leave_saved_password_and_fields_unchanged() {
    let database = Database::open_in_memory().unwrap();
    let credentials = MemoryCredentialStore::default();
    let previous = settings::save_webdav_settings(
        &database,
        &credentials,
        webdav_input(Some("previous-test-password")),
    )
    .unwrap();
    let mut insecure = webdav_input(Some("replacement-test-password"));
    insecure.url = "http://dav.example.test/root".into();
    let mut query = webdav_input(Some("replacement-test-password"));
    query.url = "https://dav.example.test/root?token=test".into();
    let mut traversal = webdav_input(Some("replacement-test-password"));
    traversal.remote_root = "../another-app".into();
    let mut invalid_username = webdav_input(Some("replacement-test-password"));
    invalid_username.username = "user:name".into();
    let mut clear_with_spaces = webdav_input(Some("   "));
    clear_with_spaces.clear_password = true;
    for input in [
        insecure,
        query,
        traversal,
        invalid_username,
        clear_with_spaces,
    ] {
        assert!(settings::save_webdav_settings(&database, &credentials, input).is_err());
        assert_eq!(
            settings::load_webdav_settings(&database, &credentials).unwrap(),
            previous
        );
        assert!(stored_password(&credentials).as_deref() == Some("previous-test-password"));
    }
}

#[test]
fn database_save_failure_restores_previous_password_or_absence() {
    for previous_password in [None, Some("previous-test-password")] {
        for clear_password in [false, true] {
            let database = Database::open_in_memory().unwrap();
            let credentials = MemoryCredentialStore::default();
            let previous = settings::save_webdav_settings(
                &database,
                &credentials,
                webdav_input(previous_password),
            )
            .unwrap();
            database
                .transaction(|transaction| {
                    transaction.execute_batch(
                    "CREATE TRIGGER reject_webdav_save BEFORE UPDATE OF webdav_url ON app_settings
                     BEGIN SELECT RAISE(ABORT, 'forced settings save failure'); END;",
                )
                })
                .unwrap();
            let mut replacement =
                webdav_input((!clear_password).then_some("replacement-test-password"));
            replacement.url = "https://replacement.example.test/dav".into();
            replacement.remote_root = "custom-root".into();
            replacement.clear_password = clear_password;
            assert!(settings::save_webdav_settings(&database, &credentials, replacement).is_err());
            assert_eq!(
                settings::load_webdav_settings(&database, &credentials).unwrap(),
                previous
            );
            assert!(stored_password(&credentials).as_deref() == previous_password);
        }
    }
}
