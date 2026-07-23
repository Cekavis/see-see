use image::{ImageFormat, ImageReader, Rgba, RgbaImage};
use see_see_lib::{
    capture::{FrozenMonitor, PhysicalPoint, PhysicalRect, compose_selection, normalize_selection},
    providers::normalize_png,
};
use std::io::Cursor;

fn solid(width: u32, height: u32, color: [u8; 4]) -> RgbaImage {
    RgbaImage::from_pixel(width, height, Rgba(color))
}

#[test]
fn selection_supports_negative_coordinates_reverse_drag_and_zero_size_cancel() {
    assert_eq!(
        normalize_selection(
            PhysicalPoint { x: 20, y: 10 },
            PhysicalPoint { x: -5, y: -10 }
        ),
        Some(PhysicalRect {
            x: -5,
            y: -10,
            width: 25,
            height: 20
        })
    );
    assert_eq!(
        normalize_selection(PhysicalPoint { x: 1, y: 1 }, PhysicalPoint { x: 1, y: 9 }),
        None
    );
}

#[test]
fn cross_monitor_crop_uses_virtual_desktop_physical_pixels() {
    let frames = vec![
        FrozenMonitor::new(
            "left",
            PhysicalRect {
                x: -2,
                y: 0,
                width: 2,
                height: 2,
            },
            1.25,
            solid(2, 2, [255, 0, 0, 255]),
        )
        .unwrap(),
        FrozenMonitor::new(
            "right",
            PhysicalRect {
                x: 0,
                y: 0,
                width: 2,
                height: 2,
            },
            2.0,
            solid(2, 2, [0, 0, 255, 255]),
        )
        .unwrap(),
    ];
    let png = compose_selection(
        &frames,
        PhysicalRect {
            x: -1,
            y: 0,
            width: 3,
            height: 2,
        },
    )
    .unwrap();
    let image = ImageReader::new(Cursor::new(png))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap()
        .to_rgba8();
    assert_eq!(image.dimensions(), (3, 2));
    assert_eq!(image.get_pixel(0, 0).0, [255, 0, 0, 255]);
    assert_eq!(image.get_pixel(1, 0).0, [0, 0, 255, 255]);
    assert_eq!(image.get_pixel(2, 0).0, [0, 0, 255, 255]);
}

#[test]
fn png_normalization_preserves_png_and_shared_limits() {
    let image = solid(8_100, 4, [20, 40, 60, 255]);
    let mut source = Cursor::new(Vec::new());
    image.write_to(&mut source, ImageFormat::Png).unwrap();
    let normalized = normalize_png(&source.into_inner()).unwrap();
    let decoded = ImageReader::new(Cursor::new(&normalized))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();
    assert!(decoded.width() <= 8_000);
    assert!(decoded.height() <= 8_000);
    assert!(normalized.len().div_ceil(3) * 4 <= 8 * 1024 * 1024);
}
