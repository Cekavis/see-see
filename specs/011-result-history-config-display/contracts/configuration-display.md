# Contract: Configuration Display

## Analysis snapshot

- Attached snapshots include `modelConfigName` and `promptConfigName` for the current attempt.
- Streaming and terminal events preserve the current names through frontend state updates.
- Retry reset replaces both names before the new started event is emitted.
- Initial frontend state may use empty names only until attachment completes.

## Result window

- When both names are available, render `模型配置：{name}` and `提示词配置：{name}` in compact header metadata.
- Do not render empty configuration labels before attachment.
- Long names wrap without hiding the status, always-on-top control, result content, or footer actions.

## History list

- Metadata includes status, saved model configuration name, saved prompt configuration name, and start time.
- When present, the original image is the first card child; records without images have no reserved image region.
- The textual content and actions follow below the image region.
- Images use intrinsic aspect-preserving presentation, container-width and maximum-height bounds, and are not cropped.
- The initial query uses `limit: 10` and no cursor.
- Page-size choices are 10, 20, and 50; changing size resets to page 1 and queries with the selected limit.
- Next uses the current response cursor; previous uses the stored cursor for the preceding visited page.
- Previous is disabled on page 1 and next is disabled when no next cursor exists.
