use secrecy::{ExposeSecret, SecretString};
use see_see_lib::{
    credentials::{CredentialStore, MemoryCredentialStore},
    database::Database,
    error::AppError,
    providers::ProviderProtocol,
    settings::{
        ModelConfigInput, delete_model_config, duplicate_model_config, list_model_configs,
        load_model, load_model_api_key, migrate_model_credentials, save_model_config,
        set_active_model_config,
    },
};

fn input(name: &str, key: Option<&str>) -> ModelConfigInput {
    ModelConfigInput {
        id: None,
        name: name.into(),
        protocol: ProviderProtocol::OpenAi,
        base_url: "https://api.example.com/v1".into(),
        model_id: "vision-model".into(),
        api_key: key.map(str::to_owned),
        clear_api_key: false,
    }
}

fn key_value(db: &Database, id: &str) -> Option<String> {
    load_model_api_key(db, id)
        .unwrap()
        .map(|value| value.expose_secret().to_owned())
}

#[test]
fn model_crud_stores_plaintext_key_with_endpoint_but_redacts_summaries() {
    let db = Database::open_in_memory().unwrap();
    let created = save_model_config(&db, input("主模型", Some("  secret key  "))).unwrap();
    assert!(created.has_api_key);
    assert!(
        !serde_json::to_string(&created)
            .unwrap()
            .contains("secret key")
    );
    assert_eq!(
        key_value(&db, &created.id).as_deref(),
        Some("  secret key  ")
    );
    let stored = load_model(&db, &created.id).unwrap().unwrap();
    assert_eq!(stored.base_url, "https://api.example.com/v1");

    let mut edited = input("主模型", None);
    edited.id = Some(created.id.clone());
    edited.base_url = "https://gateway.example.com/v1".into();
    let edited = save_model_config(&db, edited).unwrap();
    assert!(edited.has_api_key);
    assert_eq!(
        key_value(&db, &created.id).as_deref(),
        Some("  secret key  ")
    );

    set_active_model_config(&db, &created.id).unwrap();
    delete_model_config(&db, &created.id).unwrap();
    assert!(list_model_configs(&db).unwrap().is_empty());
    assert!(key_value(&db, &created.id).is_none());
}

#[test]
fn validation_clear_and_untested_activation_are_supported() {
    let db = Database::open_in_memory().unwrap();
    let created = save_model_config(&db, input("唯一名称", Some("secret"))).unwrap();
    assert!(save_model_config(&db, input("唯一名称", Some("temporary"))).is_err());
    assert!(set_active_model_config(&db, &created.id).is_ok());

    let mut edited = input("唯一名称", None);
    edited.id = Some(created.id.clone());
    edited.clear_api_key = true;
    let updated = save_model_config(&db, edited).unwrap();
    assert!(!updated.has_api_key);
    assert!(key_value(&db, &created.id).is_none());
}

#[test]
fn duplicate_preserves_connection_values_and_current_selection() {
    let db = Database::open_in_memory().unwrap();
    let mut original_input = input(
        "这是一个用于验证名称边界的超长模型配置名称这是一个用于验证名称边界的超长模型配置名称",
        Some("copy-secret"),
    );
    original_input.model_id = "vision-copy".into();
    let original = save_model_config(&db, original_input).unwrap();
    set_active_model_config(&db, &original.id).unwrap();

    let first = duplicate_model_config(&db, &original.id).unwrap();
    let second = duplicate_model_config(&db, &original.id).unwrap();
    assert_ne!(first.id, original.id);
    assert_ne!(first.name, second.name);
    assert!(first.name.chars().count() <= 80);
    assert!(!first.is_active);
    assert_eq!(first.model_id, "vision-copy");
    assert_eq!(key_value(&db, &first.id).as_deref(), Some("copy-secret"));
    assert!(
        list_model_configs(&db)
            .unwrap()
            .into_iter()
            .find(|model| model.id == original.id)
            .unwrap()
            .is_active,
    );
}

#[test]
fn legacy_credentials_migrate_idempotently_after_sqlite_commit() {
    let db = Database::open_in_memory().unwrap();
    let created = save_model_config(&db, input("旧配置", None)).unwrap();
    db.transaction(|transaction| {
        transaction.execute(
            "UPDATE model_configs SET credential_ref = 'legacy:model' WHERE id = ?1",
            [&created.id],
        )?;
        Ok(())
    })
    .unwrap();
    let credentials = MemoryCredentialStore::default();
    credentials
        .set("legacy:model", &SecretString::from("legacy-secret"))
        .unwrap();

    migrate_model_credentials(&db, &credentials).unwrap();
    migrate_model_credentials(&db, &credentials).unwrap();

    assert_eq!(
        key_value(&db, &created.id).as_deref(),
        Some("legacy-secret"),
    );
    assert!(credentials.get("legacy:model").unwrap().is_none());
    let reference: Option<String> = db
        .read(|connection| {
            connection.query_row(
                "SELECT credential_ref FROM model_configs WHERE id = ?1",
                [&created.id],
                |row| row.get(0),
            )
        })
        .unwrap();
    assert!(reference.is_none());
}

struct FailingCredentialStore;

impl CredentialStore for FailingCredentialStore {
    fn get(&self, _: &str) -> Result<Option<SecretString>, AppError> {
        Err(AppError::storage("系统凭据暂不可用"))
    }

    fn set(&self, _: &str, _: &SecretString) -> Result<(), AppError> {
        unreachable!()
    }

    fn delete(&self, _: &str) -> Result<(), AppError> {
        unreachable!()
    }
}

#[test]
fn unavailable_legacy_store_does_not_block_models_or_discard_reference() {
    let db = Database::open_in_memory().unwrap();
    let created = save_model_config(&db, input("可恢复配置", None)).unwrap();
    db.transaction(|transaction| {
        transaction.execute(
            "UPDATE model_configs SET credential_ref = 'legacy:retry' WHERE id = ?1",
            [&created.id],
        )?;
        Ok(())
    })
    .unwrap();

    migrate_model_credentials(&db, &FailingCredentialStore).unwrap();

    assert_eq!(list_model_configs(&db).unwrap().len(), 1);
    let reference: Option<String> = db
        .read(|connection| {
            connection.query_row(
                "SELECT credential_ref FROM model_configs WHERE id = ?1",
                [&created.id],
                |row| row.get(0),
            )
        })
        .unwrap();
    assert_eq!(reference.as_deref(), Some("legacy:retry"));
}
