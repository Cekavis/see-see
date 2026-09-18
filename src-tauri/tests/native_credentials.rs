//! Explicit desktop check; requires access to the user's unlocked OS credential store.
#![cfg(any(target_os = "windows", target_os = "macos"))]

use secrecy::{ExposeSecret, SecretString};
use see_see_lib::credentials::{CredentialStore, SystemCredentialStore};

#[test]
#[ignore = "requires an unlocked desktop credential store"]
fn password_survives_independent_processes() {
    let key = format!("webdav-test-{}", uuid::Uuid::new_v4());
    let executable = std::env::current_exe().unwrap();
    let run = |operation: &str| {
        std::process::Command::new(&executable)
            .args(["--exact", "credential_process_probe", "--ignored"])
            .env("SEE_SEE_CREDENTIAL_TEST_KEY", &key)
            .env("SEE_SEE_CREDENTIAL_TEST_OPERATION", operation)
            .output()
            .unwrap()
            .status
            .success()
    };
    let wrote = run("write");
    let read = wrote && run("read");
    let removed = SystemCredentialStore.delete(&key).is_ok();
    assert!(wrote, "native credential write failed");
    assert!(
        read,
        "native credential did not survive the writing process"
    );
    assert!(removed, "test credential cleanup failed");
}

#[test]
#[ignore = "subprocess helper for password_survives_independent_processes"]
fn credential_process_probe() {
    let Ok(key) = std::env::var("SEE_SEE_CREDENTIAL_TEST_KEY") else {
        return;
    };
    if std::env::var("SEE_SEE_CREDENTIAL_TEST_OPERATION").unwrap() == "write" {
        SystemCredentialStore
            .set(&key, &SecretString::from("temporary-test-password"))
            .unwrap();
    } else {
        let value = SystemCredentialStore.get(&key).unwrap();
        assert!(value.is_some_and(|secret| secret.expose_secret() == "temporary-test-password"));
    }
}
