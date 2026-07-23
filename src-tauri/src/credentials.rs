use crate::error::{AppError, ErrorCode};
use secrecy::{ExposeSecret, SecretString};
use std::{collections::HashMap, sync::Mutex};

pub trait CredentialStore: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<SecretString>, AppError>;
    fn set(&self, key: &str, value: &SecretString) -> Result<(), AppError>;
    fn delete(&self, key: &str) -> Result<(), AppError>;
}

pub struct SystemCredentialStore;

impl CredentialStore for SystemCredentialStore {
    fn get(&self, key: &str) -> Result<Option<SecretString>, AppError> {
        let entry = keyring::Entry::new("See See", key)
            .map_err(|_| AppError::storage("无法访问系统凭据存储"))?;
        match entry.get_password() {
            Ok(value) => Ok(Some(SecretString::from(value))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(AppError::storage("读取系统凭据失败")),
        }
    }

    fn set(&self, key: &str, value: &SecretString) -> Result<(), AppError> {
        keyring::Entry::new("See See", key)
            .and_then(|entry| entry.set_password(value.expose_secret()))
            .map_err(|_| AppError::storage("保存系统凭据失败"))
    }

    fn delete(&self, key: &str) -> Result<(), AppError> {
        let entry = keyring::Entry::new("See See", key)
            .map_err(|_| AppError::storage("无法访问系统凭据存储"))?;
        match entry.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err(AppError::new(
                ErrorCode::StorageFailed,
                "删除系统凭据失败",
                false,
                Some("retry"),
            )),
        }
    }
}

#[derive(Default)]
pub struct MemoryCredentialStore {
    values: Mutex<HashMap<String, String>>,
}

impl CredentialStore for MemoryCredentialStore {
    fn get(&self, key: &str) -> Result<Option<SecretString>, AppError> {
        let values = self
            .values
            .lock()
            .map_err(|_| AppError::storage("测试凭据状态不可用"))?;
        Ok(values.get(key).cloned().map(SecretString::from))
    }

    fn set(&self, key: &str, value: &SecretString) -> Result<(), AppError> {
        self.values
            .lock()
            .map_err(|_| AppError::storage("测试凭据状态不可用"))?
            .insert(key.to_owned(), value.expose_secret().to_owned());
        Ok(())
    }

    fn delete(&self, key: &str) -> Result<(), AppError> {
        self.values
            .lock()
            .map_err(|_| AppError::storage("测试凭据状态不可用"))?
            .remove(key);
        Ok(())
    }
}
