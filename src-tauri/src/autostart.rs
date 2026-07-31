use crate::error::{AppError, ErrorCode};
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt as AutostartExt;

pub const AUTOSTART_ARG: &str = "--autostart";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemAutostartStatus {
    NotRegistered,
    Enabled,
    RequiresApproval,
    NotFound,
    Unknown(isize),
}

impl SystemAutostartStatus {
    pub fn from_raw(value: isize) -> Self {
        match value {
            0 => Self::NotRegistered,
            1 => Self::Enabled,
            2 => Self::RequiresApproval,
            3 => Self::NotFound,
            value => Self::Unknown(value),
        }
    }

    pub fn is_enabled(self) -> bool {
        self == Self::Enabled
    }
}

pub fn confirm_enabled(status: SystemAutostartStatus) -> Result<(), AppError> {
    match status {
        SystemAutostartStatus::Enabled => Ok(()),
        SystemAutostartStatus::RequiresApproval => Err(AppError::new(
            ErrorCode::AutostartApprovalRequired,
            "请在系统设置的“登录项与扩展”中允许 See See",
            false,
            Some("open_login_items"),
        )),
        _ => Err(AppError::storage("macOS 未能启用登录项")),
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::{AppError, SystemAutostartStatus, confirm_enabled};
    use objc2::{extern_class, extern_methods, rc::Retained};
    use objc2_foundation::{NSAppleEventManager, NSError, NSObject};

    const fn four_char_code(value: [u8; 4]) -> u32 {
        u32::from_be_bytes(value)
    }

    #[link(name = "ServiceManagement", kind = "framework")]
    unsafe extern "C" {}

    extern_class!(
        #[unsafe(super(NSObject))]
        struct SMAppService;
    );

    impl SMAppService {
        extern_methods!(
            #[unsafe(method(mainAppService))]
            #[unsafe(method_family = none)]
            fn main_app_service() -> Retained<Self>;

            #[unsafe(method(registerAndReturnError:_))]
            #[unsafe(method_family = none)]
            fn register(&self) -> Result<(), Retained<NSError>>;

            #[unsafe(method(unregisterAndReturnError:_))]
            #[unsafe(method_family = none)]
            fn unregister(&self) -> Result<(), Retained<NSError>>;

            #[unsafe(method(status))]
            #[unsafe(method_family = none)]
            fn status(&self) -> isize;

            #[unsafe(method(openSystemSettingsLoginItems))]
            #[unsafe(method_family = none)]
            fn open_system_settings_login_items();
        );
    }

    pub(super) fn status() -> SystemAutostartStatus {
        SystemAutostartStatus::from_raw(SMAppService::main_app_service().status())
    }

    pub(super) fn set_enabled(value: bool) -> Result<(), AppError> {
        let service = SMAppService::main_app_service();
        if value {
            service
                .register()
                .map_err(|error| AppError::storage(error.localizedDescription().to_string()))?;
            confirm_enabled(SystemAutostartStatus::from_raw(service.status()))
        } else {
            if matches!(
                SystemAutostartStatus::from_raw(service.status()),
                SystemAutostartStatus::NotRegistered | SystemAutostartStatus::NotFound
            ) {
                return Ok(());
            }
            service
                .unregister()
                .map_err(|error| AppError::storage(error.localizedDescription().to_string()))
        }
    }

    pub(super) fn open_settings() {
        SMAppService::open_system_settings_login_items();
    }

    pub(super) fn launched_as_login_item() -> bool {
        let Some(event) = NSAppleEventManager::sharedAppleEventManager().currentAppleEvent() else {
            return false;
        };

        event.eventID() == four_char_code(*b"oapp")
            && event
                .paramDescriptorForKeyword(four_char_code(*b"prdt"))
                .is_some_and(|value| value.enumCodeValue() == four_char_code(*b"lgit"))
    }
}

pub fn launched_as_login_item() -> bool {
    #[cfg(target_os = "macos")]
    {
        macos::launched_as_login_item()
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

pub fn system_status(app: &AppHandle) -> Result<SystemAutostartStatus, AppError> {
    #[cfg(target_os = "macos")]
    {
        let _ = app;
        Ok(macos::status())
    }
    #[cfg(not(target_os = "macos"))]
    {
        app.autolaunch()
            .is_enabled()
            .map(|enabled| {
                if enabled {
                    SystemAutostartStatus::Enabled
                } else {
                    SystemAutostartStatus::NotRegistered
                }
            })
            .map_err(|error| AppError::storage(error.to_string()))
    }
}

pub fn set_system_enabled(app: &AppHandle, value: bool) -> Result<(), AppError> {
    #[cfg(target_os = "macos")]
    {
        macos::set_enabled(value)?;
        if app.autolaunch().is_enabled().unwrap_or(false)
            && let Err(error) = app.autolaunch().disable()
        {
            if value {
                let _ = macos::set_enabled(false);
            }
            return Err(AppError::storage(error.to_string()));
        }
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        if value {
            app.autolaunch().enable()
        } else {
            app.autolaunch().disable()
        }
        .map_err(|error| AppError::storage(error.to_string()))
    }
}

pub fn reconcile_on_startup(app: &AppHandle, stored: bool) -> Result<bool, AppError> {
    #[cfg(target_os = "macos")]
    {
        let legacy_enabled = app.autolaunch().is_enabled().unwrap_or(false);
        if stored && legacy_enabled && !system_status(app)?.is_enabled() {
            macos::set_enabled(true)?;
        }
        let actual = system_status(app)?.is_enabled();
        if legacy_enabled && (actual || !stored) {
            app.autolaunch()
                .disable()
                .map_err(|error| AppError::storage(error.to_string()))?;
        }
        Ok(actual)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let actual = system_status(app)?.is_enabled();
        if stored && actual {
            app.autolaunch()
                .enable()
                .map_err(|error| AppError::storage(error.to_string()))?;
        }
        Ok(actual)
    }
}

pub fn open_login_items_settings() -> Result<(), AppError> {
    #[cfg(target_os = "macos")]
    {
        macos::open_settings();
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err(AppError::invalid("当前平台不提供 macOS 登录项设置"))
    }
}
