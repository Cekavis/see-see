use crate::{
    error::{AppError, ErrorCode},
    providers::normalize_png,
};
use image::{ImageFormat, RgbaImage, imageops};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[cfg(not(target_os = "macos"))]
use xcap::Monitor;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
use objc2_core_graphics::{CGPreflightScreenCaptureAccess, CGRequestScreenCaptureAccess};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScreenPermission {
    Granted,
    Denied,
    Unknown,
}

fn permission_status_from_grant(granted: bool) -> ScreenPermission {
    if granted {
        ScreenPermission::Granted
    } else {
        ScreenPermission::Unknown
    }
}

#[cfg(target_os = "macos")]
pub fn screen_permission_status() -> ScreenPermission {
    permission_status_from_grant(CGPreflightScreenCaptureAccess())
}

#[cfg(not(target_os = "macos"))]
pub fn screen_permission_status() -> ScreenPermission {
    let Ok(monitors) = Monitor::all() else {
        return ScreenPermission::Unknown;
    };
    let Some(monitor) = monitors.first() else {
        return ScreenPermission::Unknown;
    };
    match monitor.capture_region(0, 0, 1, 1) {
        Ok(_) => ScreenPermission::Granted,
        Err(error)
            if error
                .to_string()
                .to_ascii_lowercase()
                .contains("permission") =>
        {
            ScreenPermission::Denied
        }
        Err(_) => ScreenPermission::Unknown,
    }
}

#[cfg(target_os = "macos")]
pub fn request_screen_permission() -> ScreenPermission {
    if CGRequestScreenCaptureAccess() {
        ScreenPermission::Granted
    } else {
        ScreenPermission::Denied
    }
}

#[cfg(not(target_os = "macos"))]
pub fn request_screen_permission() -> ScreenPermission {
    screen_permission_status()
}

pub fn require_screen_permission(status: ScreenPermission) -> Result<(), AppError> {
    if status == ScreenPermission::Granted {
        Ok(())
    } else {
        Err(screen_permission_error())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalPoint {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhysicalRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl PhysicalRect {
    fn right(self) -> i64 {
        i64::from(self.x) + i64::from(self.width)
    }

    fn bottom(self) -> i64 {
        i64::from(self.y) + i64::from(self.height)
    }

    fn intersection(self, other: Self) -> Option<Self> {
        let left = i64::from(self.x).max(i64::from(other.x));
        let top = i64::from(self.y).max(i64::from(other.y));
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        (right > left && bottom > top).then_some(Self {
            x: left as i32,
            y: top as i32,
            width: (right - left) as u32,
            height: (bottom - top) as u32,
        })
    }
}

pub fn normalize_selection(start: PhysicalPoint, end: PhysicalPoint) -> Option<PhysicalRect> {
    let left = start.x.min(end.x);
    let top = start.y.min(end.y);
    let width = start.x.abs_diff(end.x);
    let height = start.y.abs_diff(end.y);
    (width > 0 && height > 0).then_some(PhysicalRect {
        x: left,
        y: top,
        width,
        height,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorSummary {
    pub id: String,
    pub name: String,
    pub bounds: PhysicalRect,
    pub scale_factor: f32,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureSessionSummary {
    pub session_id: String,
    pub monitors: Vec<MonitorSummary>,
}

#[derive(Clone)]
pub struct FrozenMonitor {
    pub summary: MonitorSummary,
    pub image: RgbaImage,
}

impl FrozenMonitor {
    pub fn new(
        id: impl Into<String>,
        bounds: PhysicalRect,
        scale_factor: f32,
        image: RgbaImage,
    ) -> Result<Self, AppError> {
        if image.dimensions() != (bounds.width, bounds.height) {
            return Err(AppError::new(
                ErrorCode::CaptureFailed,
                "显示器截图尺寸与坐标信息不一致",
                false,
                Some("retry"),
            ));
        }
        Ok(Self {
            summary: MonitorSummary {
                id: id.into(),
                name: "Display".into(),
                bounds,
                scale_factor,
                primary: false,
            },
            image,
        })
    }

    pub fn png(&self) -> Result<Vec<u8>, AppError> {
        let mut bytes = Cursor::new(Vec::new());
        self.image
            .write_to(&mut bytes, ImageFormat::Png)
            .map_err(|_| {
                AppError::new(
                    ErrorCode::CaptureFailed,
                    "无法编码显示器截图",
                    false,
                    Some("retry"),
                )
            })?;
        Ok(bytes.into_inner())
    }
}

pub struct CaptureSession {
    pub id: String,
    pub monitors: Vec<FrozenMonitor>,
    pub selection: Option<PhysicalRect>,
}

impl CaptureSession {
    pub fn capture_all(id: impl Into<String>) -> Result<Self, AppError> {
        #[cfg(target_os = "macos")]
        let frozen = macos::capture_all()?;
        #[cfg(not(target_os = "macos"))]
        let frozen = capture_all_with_xcap()?;

        if frozen.is_empty() {
            return Err(AppError::new(
                ErrorCode::CaptureFailed,
                "没有可用显示器",
                false,
                Some("retry"),
            ));
        }
        Ok(Self {
            id: id.into(),
            monitors: frozen,
            selection: None,
        })
    }

    pub fn summary(&self) -> CaptureSessionSummary {
        CaptureSessionSummary {
            session_id: self.id.clone(),
            monitors: self
                .monitors
                .iter()
                .map(|monitor| monitor.summary.clone())
                .collect(),
        }
    }

    pub fn frame(&self, monitor_id: &str) -> Result<Vec<u8>, AppError> {
        self.monitors
            .iter()
            .find(|monitor| monitor.summary.id == monitor_id)
            .ok_or_else(|| AppError::new(ErrorCode::NotFound, "截图帧不存在", false, None))?
            .png()
    }

    pub fn update_selection(&mut self, selection: PhysicalRect) -> Result<(), AppError> {
        if selection.width == 0 || selection.height == 0 {
            return Err(AppError::invalid("截图选区不能为空"));
        }
        if !self
            .monitors
            .iter()
            .any(|monitor| monitor.summary.bounds.intersection(selection).is_some())
        {
            return Err(AppError::invalid("截图选区不在任何显示器内"));
        }
        self.selection = Some(selection);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct MonitorMetadata {
    pub id: String,
    pub name: String,
    pub bounds: PhysicalRect,
    pub scale_factor: f32,
    pub primary: bool,
}

pub(crate) fn frozen_monitor_from_bgra(
    metadata: MonitorMetadata,
    width: usize,
    height: usize,
    bytes_per_row: usize,
    data: &[u8],
) -> Result<FrozenMonitor, AppError> {
    let row_bytes = width
        .checked_mul(4)
        .ok_or_else(|| capture_error("convert_image", "captured image row length overflow"))?;
    let data_len = bytes_per_row
        .checked_mul(height)
        .ok_or_else(|| capture_error("convert_image", "captured image buffer length overflow"))?;
    if width != metadata.bounds.width as usize
        || height != metadata.bounds.height as usize
        || bytes_per_row < row_bytes
        || data.len() < data_len
    {
        return Err(capture_error(
            "convert_image",
            format!(
                "invalid BGRA frame: image={width}x{height}, row={bytes_per_row}, data={}, expected={}x{}",
                data.len(),
                metadata.bounds.width,
                metadata.bounds.height
            ),
        ));
    }

    let mut rgba = vec![0; row_bytes * height];
    for row in 0..height {
        let source = &data[row * bytes_per_row..row * bytes_per_row + row_bytes];
        let target = &mut rgba[row * row_bytes..(row + 1) * row_bytes];
        for (bgra, rgba) in source.chunks_exact(4).zip(target.chunks_exact_mut(4)) {
            rgba.copy_from_slice(&[bgra[2], bgra[1], bgra[0], bgra[3]]);
        }
    }
    let image = RgbaImage::from_raw(width as u32, height as u32, rgba)
        .ok_or_else(|| capture_error("convert_image", "unable to create RGBA image"))?;
    let mut monitor =
        FrozenMonitor::new(metadata.id, metadata.bounds, metadata.scale_factor, image)?;
    monitor.summary.name = metadata.name;
    monitor.summary.primary = metadata.primary;
    Ok(monitor)
}

#[cfg(not(target_os = "macos"))]
fn capture_all_with_xcap() -> Result<Vec<FrozenMonitor>, AppError> {
    let mut frozen = Vec::new();
    for monitor in Monitor::all().map_err(|error| capture_error("list_displays", error))? {
        let id = monitor
            .id()
            .map_err(|error| capture_error("read_display_id", error))?
            .to_string();
        let bounds = PhysicalRect {
            x: monitor
                .x()
                .map_err(|error| capture_error("read_display_bounds", error))?,
            y: monitor
                .y()
                .map_err(|error| capture_error("read_display_bounds", error))?,
            width: monitor
                .width()
                .map_err(|error| capture_error("read_display_bounds", error))?,
            height: monitor
                .height()
                .map_err(|error| capture_error("read_display_bounds", error))?,
        };
        let image = monitor
            .capture_image()
            .map_err(|error| capture_error("capture_display", error))?;
        let mut frame = FrozenMonitor::new(
            id,
            bounds,
            monitor
                .scale_factor()
                .map_err(|error| capture_error("read_display_scale", error))?,
            image,
        )?;
        frame.summary.name = monitor
            .friendly_name()
            .or_else(|_| monitor.name())
            .unwrap_or_else(|_| "Display".into());
        frame.summary.primary = monitor.is_primary().unwrap_or(false);
        frozen.push(frame);
    }
    Ok(frozen)
}

pub fn compose_selection(
    monitors: &[FrozenMonitor],
    selection: PhysicalRect,
) -> Result<Vec<u8>, AppError> {
    if selection.width == 0 || selection.height == 0 {
        return Err(AppError::invalid("截图选区不能为空"));
    }
    let mut output = RgbaImage::new(selection.width, selection.height);
    let mut copied = false;
    for monitor in monitors {
        let Some(intersection) = monitor.summary.bounds.intersection(selection) else {
            continue;
        };
        let source_x = (intersection.x - monitor.summary.bounds.x) as u32;
        let source_y = (intersection.y - monitor.summary.bounds.y) as u32;
        let target_x = i64::from(intersection.x - selection.x);
        let target_y = i64::from(intersection.y - selection.y);
        let crop = imageops::crop_imm(
            &monitor.image,
            source_x,
            source_y,
            intersection.width,
            intersection.height,
        )
        .to_image();
        imageops::replace(&mut output, &crop, target_x, target_y);
        copied = true;
    }
    if !copied {
        return Err(AppError::invalid("截图选区不在任何显示器内"));
    }
    let mut encoded = Cursor::new(Vec::new());
    output
        .write_to(&mut encoded, ImageFormat::Png)
        .map_err(|_| {
            AppError::new(
                ErrorCode::CaptureFailed,
                "无法生成截图",
                false,
                Some("retry"),
            )
        })?;
    normalize_png(&encoded.into_inner())
}

pub(crate) fn capture_error(stage: &'static str, error: impl std::fmt::Display) -> AppError {
    let detail = sanitized_diagnostic(error);
    log::warn!("screen capture backend failed at {stage}: {detail}");
    let lower = detail.to_ascii_lowercase();
    if ["permission", "access", "denied", "declined"]
        .iter()
        .any(|needle| lower.contains(needle))
    {
        return screen_permission_error();
    }
    AppError::new(
        ErrorCode::CaptureFailed,
        "无法读取显示器画面",
        false,
        Some("retry"),
    )
}

fn sanitized_diagnostic(error: impl std::fmt::Display) -> String {
    error
        .to_string()
        .chars()
        .map(|character| {
            if character.is_control() {
                ' '
            } else {
                character
            }
        })
        .take(400)
        .collect()
}

fn screen_permission_error() -> AppError {
    AppError::new(
        ErrorCode::ScreenPermissionDenied,
        "需要屏幕录制权限才能截图",
        false,
        Some("open_screen_permission_settings"),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        MonitorMetadata, PhysicalRect, ScreenPermission, capture_error, frozen_monitor_from_bgra,
        permission_status_from_grant, require_screen_permission, sanitized_diagnostic,
    };
    use crate::error::ErrorCode;

    #[test]
    fn passive_preflight_maps_missing_access_without_claiming_denial() {
        assert_eq!(
            permission_status_from_grant(true),
            ScreenPermission::Granted
        );
        assert_eq!(
            permission_status_from_grant(false),
            ScreenPermission::Unknown
        );
    }

    #[test]
    fn capture_permission_guard_and_backend_error_use_recovery_code() {
        let guarded = require_screen_permission(ScreenPermission::Unknown).unwrap_err();
        assert_eq!(guarded.code, ErrorCode::ScreenPermissionDenied);
        assert_eq!(
            guarded.action.as_deref(),
            Some("open_screen_permission_settings")
        );

        let backend = capture_error("capture_display", "screen capture permission denied");
        assert_eq!(backend.code, ErrorCode::ScreenPermissionDenied);
        assert!(require_screen_permission(ScreenPermission::Granted).is_ok());
    }

    #[test]
    fn screen_capture_adapter_converts_padded_bgra_and_preserves_metadata() {
        let metadata = MonitorMetadata {
            id: "42".into(),
            name: "Studio Display".into(),
            bounds: PhysicalRect {
                x: -2,
                y: 3,
                width: 2,
                height: 1,
            },
            scale_factor: 2.0,
            primary: true,
        };
        let monitor = frozen_monitor_from_bgra(
            metadata,
            2,
            1,
            12,
            &[30, 20, 10, 255, 60, 50, 40, 128, 0, 0, 0, 0],
        )
        .unwrap();

        assert_eq!(monitor.summary.id, "42");
        assert_eq!(monitor.summary.name, "Studio Display");
        assert_eq!(monitor.summary.bounds.x, -2);
        assert_eq!(monitor.summary.scale_factor, 2.0);
        assert!(monitor.summary.primary);
        assert_eq!(monitor.image.get_pixel(0, 0).0, [10, 20, 30, 255]);
        assert_eq!(monitor.image.get_pixel(1, 0).0, [40, 50, 60, 128]);
    }

    #[test]
    fn screen_capture_adapter_rejects_bad_frames_without_leaking_details() {
        let metadata = MonitorMetadata {
            id: "1".into(),
            name: "Display".into(),
            bounds: PhysicalRect {
                x: 0,
                y: 0,
                width: 2,
                height: 1,
            },
            scale_factor: 1.0,
            primary: false,
        };
        let error = match frozen_monitor_from_bgra(metadata, 2, 1, 8, &[0; 4]) {
            Ok(_) => panic!("short image data should be rejected"),
            Err(error) => error,
        };

        assert_eq!(error.code, ErrorCode::CaptureFailed);
        assert_eq!(error.message, "无法读取显示器画面");
        assert!(!error.message.contains("BGRA"));
        assert_eq!(sanitized_diagnostic("first\nsecond"), "first second");
        assert_eq!(sanitized_diagnostic("x".repeat(500)).len(), 400);
    }
}
