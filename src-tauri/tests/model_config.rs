use secrecy::{ExposeSecret, SecretString};
use see_see_lib::{
    credentials::{CredentialStore, MemoryCredentialStore},
    database::Database,
    providers::ProviderProtocol,
    settings::{
        ModelConfigInput, ModelTestStatus, delete_model_config, list_model_configs,
        record_model_test, save_model_config, set_active_model_config,
    },
};
use std::sync::Arc;

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

#[test]
fn model_crud_keeps_secrets_out_of_sqlite_and_serialized_summaries() {
    let db = Database::open_in_memory().unwrap();
    let credentials = Arc::new(MemoryCredentialStore::default());
    let created = save_model_config(
        &db,
        credentials.as_ref(),
        input("主模型", Some("secret-key")),
    )
    .unwrap();
    assert!(created.has_api_key);
    assert!(
        !serde_json::to_string(&created)
            .unwrap()
            .contains("secret-key")
    );
    assert!(
        credentials
            .get(&format!("model:{}", created.id))
            .unwrap()
            .is_some()
    );

    let mut edited = input("主模型", None);
    edited.id = Some(created.id.clone());
    edited.base_url = "https://gateway.example.com/v1".into();
    let edited = save_model_config(&db, credentials.as_ref(), edited).unwrap();
    assert_eq!(edited.test_status, ModelTestStatus::Untested);
    assert!(edited.has_api_key);

    record_model_test(&db, &created.id, true, None).unwrap();
    set_active_model_config(&db, &created.id).unwrap();
    delete_model_config(&db, credentials.as_ref(), &created.id).unwrap();
    assert!(
        list_model_configs(&db, credentials.as_ref())
            .unwrap()
            .is_empty()
    );
    assert!(
        credentials
            .get(&format!("model:{}", created.id))
            .unwrap()
            .is_none()
    );
}

#[test]
fn model_validation_and_active_constraints_are_enforced() {
    let db = Database::open_in_memory().unwrap();
    let credentials = MemoryCredentialStore::default();
    let created = save_model_config(&db, &credentials, input("唯一名称", None)).unwrap();
    assert!(save_model_config(&db, &credentials, input("唯一名称", Some("temporary"))).is_err());
    assert!(credentials.get("model:temporary").unwrap().is_none());
    assert!(set_active_model_config(&db, &created.id).is_err());
    record_model_test(&db, &created.id, true, None).unwrap();
    assert!(set_active_model_config(&db, &created.id).is_ok());
}

#[test]
fn explicit_key_clear_updates_projection_and_store() {
    let db = Database::open_in_memory().unwrap();
    let credentials = MemoryCredentialStore::default();
    let created = save_model_config(&db, &credentials, input("可清除", Some("secret"))).unwrap();
    let mut edited = input("可清除", None);
    edited.id = Some(created.id.clone());
    edited.clear_api_key = true;
    let updated = save_model_config(&db, &credentials, edited).unwrap();
    assert!(!updated.has_api_key);
    assert!(
        credentials
            .get(&format!("model:{}", created.id))
            .unwrap()
            .is_none()
    );
}

#[allow(dead_code)]
fn secret_value(store: &dyn CredentialStore, key: &str) -> Option<String> {
    store
        .get(key)
        .unwrap()
        .map(|value: SecretString| value.expose_secret().to_owned())
}
