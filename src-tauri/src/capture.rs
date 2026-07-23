use crate::{
    error::{AppError, ErrorCode},
    providers::normalize_png,
};
use image::{ImageFormat, RgbaImage, imageops};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use xcap::Monitor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScreenPermission {
    Granted,
    Denied,
    Unknown,
}

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
        let mut frozen = Vec::new();
        for monitor in Monitor::all().map_err(capture_error)? {
            let id = monitor.id().map_err(capture_error)?.to_string();
            let bounds = PhysicalRect {
                x: monitor.x().map_err(capture_error)?,
                y: monitor.y().map_err(capture_error)?,
                width: monitor.width().map_err(capture_error)?,
                height: monitor.height().map_err(capture_error)?,
            };
            let image = monitor.capture_image().map_err(capture_error)?;
            let mut frame = FrozenMonitor::new(
                id,
                bounds,
                monitor.scale_factor().map_err(capture_error)?,
                image,
            )?;
            frame.summary.name = monitor
                .friendly_name()
                .or_else(|_| monitor.name())
                .unwrap_or_else(|_| "Display".into());
            frame.summary.primary = monitor.is_primary().unwrap_or(false);
            frozen.push(frame);
        }
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

fn capture_error(_: impl std::fmt::Display) -> AppError {
    AppError::new(
        ErrorCode::CaptureFailed,
        "无法读取显示器画面",
        false,
        Some("retry"),
    )
}
