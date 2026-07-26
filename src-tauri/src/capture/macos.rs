use super::{
    FrozenMonitor, MonitorMetadata, PhysicalRect, capture_error, frozen_monitor_from_bgra,
};
use crate::error::AppError;
use block2::RcBlock;
use objc2::{class, msg_send, rc::Retained, runtime::AnyObject};
use objc2_core_graphics::{CGDataProvider, CGImage};
use objc2_foundation::{NSArray, NSError};
use std::{
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender},
    time::{Duration, Instant},
};
use xcap::Monitor;

#[link(name = "ScreenCaptureKit", kind = "framework")]
unsafe extern "C" {}

const CAPTURE_TIMEOUT: Duration = Duration::from_secs(30);
const BGRA_PIXEL_FORMAT: u32 = u32::from_be_bytes(*b"BGRA");

enum CaptureMessage {
    Frame(Result<FrozenMonitor, AppError>),
    Fatal(AppError),
}

pub(super) fn capture_all() -> Result<Vec<FrozenMonitor>, AppError> {
    let metadata = monitor_metadata()?;
    if metadata.is_empty() {
        return Ok(Vec::new());
    }

    let display_order: Vec<_> = metadata.iter().map(|monitor| monitor.id.clone()).collect();
    let (sender, receiver) = mpsc::channel();
    request_shareable_content(metadata, sender);
    receive_frames(receiver, &display_order)
}

fn monitor_metadata() -> Result<Vec<MonitorMetadata>, AppError> {
    Monitor::all()
        .map_err(|error| capture_error("list_displays", error))?
        .into_iter()
        .map(|monitor| {
            let id = monitor
                .id()
                .map_err(|error| capture_error("read_display_id", error))?;
            Ok(MonitorMetadata {
                id: id.to_string(),
                name: monitor
                    .friendly_name()
                    .or_else(|_| monitor.name())
                    .unwrap_or_else(|_| "Display".into()),
                bounds: PhysicalRect {
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
                },
                scale_factor: monitor
                    .scale_factor()
                    .map_err(|error| capture_error("read_display_scale", error))?,
                primary: monitor.is_primary().unwrap_or(false),
            })
        })
        .collect()
}

fn request_shareable_content(metadata: Vec<MonitorMetadata>, sender: Sender<CaptureMessage>) {
    let completion = RcBlock::new(move |content: *mut AnyObject, error: *mut NSError| {
        if content.is_null() {
            let _ = sender.send(CaptureMessage::Fatal(capture_error(
                "load_shareable_content",
                error_description(error),
            )));
            return;
        }

        let displays: *mut NSArray<AnyObject> = unsafe { msg_send![content, displays] };
        if displays.is_null() {
            let _ = sender.send(CaptureMessage::Fatal(capture_error(
                "load_shareable_content",
                "ScreenCaptureKit returned no display list",
            )));
            return;
        }

        for monitor in metadata.clone() {
            let display = unsafe { find_display(&*displays, &monitor.id) };
            match display {
                Some(display) => request_display_image(display, monitor, sender.clone()),
                None => {
                    let _ = sender.send(CaptureMessage::Frame(Err(capture_error(
                        "match_display",
                        format!("display {} was not shareable", monitor.id),
                    ))));
                }
            }
        }
    });

    unsafe {
        let _: () = msg_send![
            class!(SCShareableContent),
            getShareableContentExcludingDesktopWindows: false,
            onScreenWindowsOnly: true,
            completionHandler: &*completion
        ];
    }
}

unsafe fn find_display(
    displays: &NSArray<AnyObject>,
    target_id: &str,
) -> Option<Retained<AnyObject>> {
    for index in 0..displays.count() {
        let display = displays.objectAtIndex(index);
        let display_id: u32 = unsafe { msg_send![&*display, displayID] };
        if display_id.to_string() == target_id {
            return Some(display);
        }
    }
    None
}

fn request_display_image(
    display: Retained<AnyObject>,
    metadata: MonitorMetadata,
    sender: Sender<CaptureMessage>,
) {
    let empty_windows = NSArray::<AnyObject>::new();
    let filter = unsafe {
        let allocated: *mut AnyObject = msg_send![class!(SCContentFilter), alloc];
        let initialized: *mut AnyObject =
            msg_send![allocated, initWithDisplay: &*display, excludingWindows: &*empty_windows];
        Retained::from_raw(initialized)
    };
    let Some(filter) = filter else {
        let _ = sender.send(CaptureMessage::Frame(Err(capture_error(
            "create_content_filter",
            "SCContentFilter initializer returned null",
        ))));
        return;
    };
    let configuration: Retained<AnyObject> =
        unsafe { msg_send![class!(SCStreamConfiguration), new] };
    unsafe {
        let _: () = msg_send![&*configuration, setWidth: metadata.bounds.width as usize];
        let _: () = msg_send![&*configuration, setHeight: metadata.bounds.height as usize];
        let _: () = msg_send![&*configuration, setPixelFormat: BGRA_PIXEL_FORMAT];
        let _: () = msg_send![&*configuration, setShowsCursor: false];
        let _: () = msg_send![&*configuration, setScalesToFit: true];
        let _: () = msg_send![&*configuration, setPreservesAspectRatio: true];
    }

    let completion = RcBlock::new(move |image: *mut CGImage, error: *mut NSError| {
        let frame = if image.is_null() {
            Err(capture_error("capture_display", error_description(error)))
        } else {
            cg_image_to_monitor(unsafe { &*image }, metadata.clone())
        };
        let _ = sender.send(CaptureMessage::Frame(frame));
    });

    unsafe {
        let _: () = msg_send![
            class!(SCScreenshotManager),
            captureImageWithFilter: &*filter,
            configuration: &*configuration,
            completionHandler: &*completion
        ];
    }
}

fn cg_image_to_monitor(
    image: &CGImage,
    metadata: MonitorMetadata,
) -> Result<FrozenMonitor, AppError> {
    let provider = CGImage::data_provider(Some(image))
        .ok_or_else(|| capture_error("copy_image_data", "CGImage has no data provider"))?;
    let data = CGDataProvider::data(Some(&provider))
        .ok_or_else(|| capture_error("copy_image_data", "unable to copy CGImage data"))?
        .to_vec();
    frozen_monitor_from_bgra(
        metadata,
        CGImage::width(Some(image)),
        CGImage::height(Some(image)),
        CGImage::bytes_per_row(Some(image)),
        &data,
    )
}

fn receive_frames(
    receiver: Receiver<CaptureMessage>,
    display_order: &[String],
) -> Result<Vec<FrozenMonitor>, AppError> {
    let deadline = Instant::now() + CAPTURE_TIMEOUT;
    let mut monitors = Vec::with_capacity(display_order.len());
    while monitors.len() < display_order.len() {
        let remaining = deadline.saturating_duration_since(Instant::now());
        match receiver.recv_timeout(remaining) {
            Ok(CaptureMessage::Frame(Ok(monitor))) => monitors.push(monitor),
            Ok(CaptureMessage::Frame(Err(error)) | CaptureMessage::Fatal(error)) => {
                return Err(error);
            }
            Err(RecvTimeoutError::Timeout) => {
                return Err(capture_error(
                    "capture_timeout",
                    "ScreenCaptureKit did not finish within 30 seconds",
                ));
            }
            Err(RecvTimeoutError::Disconnected) => {
                return Err(capture_error(
                    "capture_channel",
                    "ScreenCaptureKit completion channel disconnected",
                ));
            }
        }
    }
    monitors.sort_by_key(|monitor| {
        display_order
            .iter()
            .position(|id| id == &monitor.summary.id)
            .unwrap_or(usize::MAX)
    });
    Ok(monitors)
}

fn error_description(error: *mut NSError) -> String {
    if error.is_null() {
        "ScreenCaptureKit returned no error details".into()
    } else {
        unsafe { (*error).localizedDescription().to_string() }
    }
}
