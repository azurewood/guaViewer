use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

// tcptdx library types (ensure your Cargo.toml includes the tcptdx crate)
use tcptdx::{client::FeedClient, commands::KLine};

slint::include_modules!();

/// Application-level errors with structured variants.
///
/// This keeps error handling explicit and avoids passing around ad-hoc `String`s.
/// In UI code we format these errors via `Display` and show them in `svg-status`.
#[derive(Debug)]
enum AppError {
    /// A user input field could not be parsed into the expected type.
    ParseU16 { field: &'static str, value: String },

    /// The SVG string could not be parsed by `usvg`.
    InvalidSvg { message: String },

    /// The SVG produced a zero-sized output (width or height became 0).
    EmptySvgImage,

    /// Failed to allocate the raster pixmap buffer.
    PixmapAllocFailed,

    /// Failed to initialize the TCP TDX client.
    ClientInit { message: String },

    /// Failed to fetch KLine data over the network.
    FetchKLine { message: String },

    /// The server returned no data for the given query.
    NoKLineData,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::ParseU16 { field, value } => {
                write!(f, "{field} must be a u16 (got '{value}')")
            }
            AppError::InvalidSvg { message } => write!(f, "Invalid SVG: {message}"),
            AppError::EmptySvgImage => write!(f, "SVG produced an empty image"),
            AppError::PixmapAllocFailed => write!(f, "Failed to allocate pixmap buffer"),
            AppError::ClientInit { message } => write!(f, "FeedClient init failed: {message}"),
            AppError::FetchKLine { message } => write!(f, "KLine fetch failed: {message}"),
            AppError::NoKLineData => write!(f, "No KLine data returned"),
        }
    }
}

impl std::error::Error for AppError {}

/// Parse a `u16` from a Slint string field.
///
/// This returns a structured `AppError::ParseU16` which retains field context.
fn parse_u16(field: &'static str, text: slint::SharedString) -> Result<u16, AppError> {
    let trimmed = text.trim();
    trimmed.parse::<u16>().map_err(|_| AppError::ParseU16 {
        field,
        value: text.to_string(),
    })
}

/// Convert an SVG string into a raster `slint::Image` using the provided scale factor.
///
/// Slint's built-in SVG support is limited depending on backend; rasterizing via `resvg`
/// gives consistent rendering while still allowing you to implement zoom/pan by re-rendering
/// at different scales.
fn svg_to_image_with_scale(svg_content: &str, scale: f32) -> Result<Image, AppError> {
    let opt = usvg::Options::default();
    // let tree = usvg::Tree::from_data(svg_content.as_bytes(), &opt)
    //     .map_err(|e| format!("Invalid SVG: {e}"))?;
    let tree =
        usvg::Tree::from_data(svg_content.as_bytes(), &opt).map_err(|e| AppError::InvalidSvg {
            message: e.to_string(),
        })?;

    let size = tree.size();
    let width = (size.width() * scale) as u32;
    let height = (size.height() * scale) as u32;

    if width == 0 || height == 0 {
        // return Err("SVG produced an empty image".to_string());
        return Err(AppError::EmptySvgImage);
    }

    // let mut pixmap = tiny_skia::Pixmap::new(width, height)
    //     .ok_or_else(|| "Failed to create pixmap".to_string())?;
    let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or(AppError::PixmapAllocFailed)?;

    // Apply scaling transform
    let transform = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // resvg outputs premultiplied BGRA; Slint expects RGBA
    let mut rgba_bytes: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);
    for chunk in pixmap.data().chunks_exact(4) {
        rgba_bytes.push(chunk[0]); // R
        rgba_bytes.push(chunk[1]); // G
        rgba_bytes.push(chunk[2]); // B
        rgba_bytes.push(chunk[3]); // A
    }

    let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(&rgba_bytes, width, height);
    Ok(Image::from_rgba8(buffer))
}

/// A minimal candle model used by the SVG renderer.
#[derive(Debug, Clone)]
struct Candle {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    /// Render-friendly date label (already formatted).
    label: String,
}

/// Fetch KLine data from tcptdx and convert it into candles.
///
/// - `market`: market id (0 = Shenzhen, 1 = Shanghai)
/// - `code`: 6-digit stock code (e.g. "000001")
/// - `category`: kline category (e.g. 9 = daily)
/// - `start`: start index
/// - `count`: number of records (tcptdx max is typically 800)
fn fetch_kline_as_candles(
    market: u16,
    code: &str,
    category: u16,
    start: u16,
    count: u16,
) -> Result<Vec<Candle>, AppError> {
    let mut req = KLine::new(market, code, category, start, count);
    // let mut client =
    //     FeedClient::new_default().map_err(|e| format!("FeedClient init failed: {e}"))?;
    let mut client = FeedClient::new_default().map_err(|e| AppError::ClientInit {
        message: e.to_string(),
    })?;

    // client
    //     .send(&mut req)
    //     .map_err(|e| format!("KLine fetch failed: {e}"))?;
    client.send(&mut req).map_err(|e| AppError::FetchKLine {
        message: e.to_string(),
    })?;

    // Map tcptdx records into our chart candles.
    let candles = req
        .data
        .iter()
        .map(|rec| Candle {
            open: rec.open as f64,
            high: rec.high as f64,
            low: rec.low as f64,
            close: rec.close as f64,
            volume: rec.vol as f64,
            label: format!("{:04}-{:02}-{:02}", rec.dt.year, rec.dt.month, rec.dt.day),
        })
        .collect::<Vec<_>>();

    if candles.is_empty() {
        // return Err("No KLine data returned".to_string());
        return Err(AppError::NoKLineData);
    }

    Ok(candles)
}

/// Render a simple OHLC candlestick chart into an SVG string.
///
/// This version draws:
/// - Candle wick (high-low)
/// - Candle body (open-close)
///
/// It intentionally omits:
/// - Grid / axes
/// - Labels
/// - Volume
fn render_svg_candles_simple(candles: &[Candle], width: i32, height: i32) -> String {
    let pad_left = 20.0;
    let pad_right = 20.0;
    let pad_top = 20.0;
    let pad_bottom = 20.0;

    let w = width as f64;
    let h = height as f64;

    let min_price = candles.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
    let max_price = candles
        .iter()
        .map(|c| c.high)
        .fold(f64::NEG_INFINITY, f64::max);

    let price_range = (max_price - min_price).max(1e-9);

    let plot_w = (w - pad_left - pad_right).max(1.0);
    let plot_h = (h - pad_top - pad_bottom).max(1.0);

    let n = candles.len().max(1) as f64;
    let step = plot_w / n;
    let body_w = (step * 0.65).max(1.0);

    let y_of = |p: f64| -> f64 {
        // higher price -> smaller y
        pad_top + (max_price - p) / price_range * plot_h
    };

    let mut out = String::new();
    out.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" fill="{color}">"#,
        width = width,
        height = height,
        color = "#374151"
    ));

    // Background
    out.push_str(r#"<rect x="0" y="0" width="100%" height="100%" fill="white"/>"#);

    for (i, c) in candles.iter().enumerate() {
        let x_center = pad_left + (i as f64 + 0.5) * step;
        let x0 = x_center - body_w / 2.0;

        let y_high = y_of(c.high);
        let y_low = y_of(c.low);
        let y_open = y_of(c.open);
        let y_close = y_of(c.close);

        let bullish = c.close >= c.open;
        let body_top = y_open.min(y_close);
        let body_bot = y_open.max(y_close);
        let body_h = (body_bot - body_top).max(1.0);

        let fill = if bullish { "#2f9e44" } else { "#e03131" };

        // Wick
        out.push_str(&format!(
            r#"<line x1="{xc:.2}" y1="{yh:.2}" x2="{xc:.2}" y2="{yl:.2}" stroke="{color}" stroke-width="1"/>"#,
            xc = x_center, yh = y_high, yl = y_low, color = "#333"
        ));

        // Body
        out.push_str(&format!(
            r#"<rect x="{x0:.2}" y="{y:.2}" width="{w:.2}" height="{h:.2}" fill="{f}"/>"#,
            x0 = x0,
            y = body_top,
            w = body_w,
            h = body_h,
            f = fill
        ));

        // // X-Axis Labels (Every 10th candle)
        // if i % 10 == 0 {
        //     out.push_str(&format!(
        //         r#"<text x="{xc:.2}" y="{ty:.2}" font-size="11" text-anchor="middle" fill="{color}">{label}</text>"#,
        //         xc = x_center, ty = h - 5.0, color = "#666", label = c.label
        //     ));
        // }
    }

    out.push_str("</svg>");
    // println!("{:?}", out);
    out
}

/// Render a fuller candlestick chart into an SVG string.
///
/// Adds:
/// - Price grid lines + Y labels
/// - Volume bars
/// - Sparse X labels
fn render_svg_candles_full(candles: &[Candle], width: i32, height: i32) -> String {
    let pad_left = 60.0;
    let pad_right = 20.0;
    let pad_top = 20.0;
    let pad_bottom = 30.0;

    let w = width as f64;
    let h = height as f64;

    let vol_h = (h * 0.22).max(80.0);
    let chart_h = (h - vol_h).max(200.0);

    let min_price = candles.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
    let max_price = candles
        .iter()
        .map(|c| c.high)
        .fold(f64::NEG_INFINITY, f64::max);

    let price_range = (max_price - min_price).max(1e-9);

    let max_vol = candles
        .iter()
        .map(|c| c.volume)
        .fold(0.0_f64, f64::max)
        .max(1e-9);

    let plot_w = (w - pad_left - pad_right).max(1.0);
    let plot_h = (chart_h - pad_top - 10.0).max(1.0);

    let n = candles.len().max(1) as f64;
    let step = plot_w / n;
    let body_w = (step * 0.65).max(1.0);

    let y_of = |p: f64| -> f64 { pad_top + (max_price - p) / price_range * plot_h };

    let mut out = String::new();
    out.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">"#
    ));

    // Background
    out.push_str(r#"<rect x="0" y="0" width="100%" height="100%" fill="white"/>"#);

    // Price grid
    let grid_lines = 5;
    for i in 0..=grid_lines {
        let t = i as f64 / grid_lines as f64;
        let price = max_price - t * price_range;
        let y = y_of(price);

        // 1. First Line
        out.push_str(&format!(
            r#"<line x1="{x1:.2}" y1="{y:.2}" x2="{x2:.2}" y2="{y:.2}" stroke="{color}" stroke-width="1"/>"#,
            x1 = pad_left,
            x2 = w - pad_right,
            y = y,
            color = "#eee"
        ));

        // 2. Second Line
        out.push_str(&format!(
            r#"<text x="{tx:.2}" y="{ty:.2}" font-size="11" text-anchor="end" fill="{color}">{price:.2}</text>"#,
            tx = pad_left - 8.0,
            ty = y + 4.0,
            price = price,
            color = "#666"
        ));
    }

    // Candles
    for (i, c) in candles.iter().enumerate() {
        let x_center = pad_left + (i as f64 + 0.5) * step;
        let x0 = x_center - body_w / 2.0;

        let y_high = y_of(c.high);
        let y_low = y_of(c.low);
        let y_open = y_of(c.open);
        let y_close = y_of(c.close);

        let bullish = c.close >= c.open;
        let body_top = y_open.min(y_close);
        let body_bot = y_open.max(y_close);
        let body_h = (body_bot - body_top).max(1.0);

        let fill = if bullish { "#2f9e44" } else { "#e03131" };

        // Wick
        out.push_str(&format!(
            r#"<line x1="{x_center:.2}" y1="{y_high:.2}" x2="{x_center:.2}" y2="{y_low:.2}" stroke="{color}" stroke-width="1"/>"#,
            x_center = x_center,
            y_high = y_high,
            y_low = y_low,
            color = "#333"
        ));

        // Body
        out.push_str(&format!(
            r#"<rect x="{x0:.2}" y="{body_top:.2}" width="{body_w:.2}" height="{body_h:.2}" fill="{fill}"/>"#
        ));
    }

    // Volume bars
    let vol_top = chart_h;
    let vol_base = h - pad_bottom;
    for (i, c) in candles.iter().enumerate() {
        let x_center = pad_left + (i as f64 + 0.5) * step;
        let x0 = x_center - body_w / 2.0;

        let vh = (c.volume / max_vol) * (vol_base - vol_top);
        let y = vol_base - vh;

        // let bullish = c.close >= c.open;
        let fill = "#d1d5db"; //if bullish { "#2f9e44" } else { "#e03131" };

        out.push_str(&format!(
            r#"<rect x="{x0:.2}" y="{y:.2}" width="{body_w:.2}" height="{vh:.2}" fill="{fill}" opacity="0.35"/>"#
        ));
    }

    // Sparse X labels
    let label_step = (candles.len() / 6).max(1);
    for i in (0..candles.len()).step_by(label_step) {
        let x_center = pad_left + (i as f64 + 0.5) * step;
        let text_y = h - 10.0;
        out.push_str(&format!(
            r#"<text x="{x:.2}" y="{y:.2}" font-size="11" text-anchor="middle" fill="{color}">{label}</text>"#,
            x = x_center,
            y = text_y,
            color = "#666",
            label = candles[i].label
        ));
    }

    out.push_str("</svg>");
    out
}

// /// Parse a `u16` from a Slint `string` field, with a nice error message.
// ///
// /// This keeps UI parsing code tidy and ensures all numeric input errors look consistent.
// fn parse_u16(field_name: &str, text: slint::SharedString) -> Result<u16, String> {
//     text.trim()
//         .parse::<u16>()
//         .map_err(|_| format!("{field_name} must be a u16 (got '{text}')"))
// }

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    // Store current scale, position, and SVG content (so zoom/pan can re-render consistently).
    let scale = Rc::new(RefCell::new(1.0f32));
    let position_x = Rc::new(RefCell::new(0.0f32));
    let current_svg_content = Rc::new(RefCell::new(String::new()));

    // Set initial scale display.
    ui.set_current_scale(format!("Scale: {:.1}x", *scale.borrow()).into());
    ui.set_image_x_offset((*position_x.borrow() as f32) * 1.0);
    // Reload / Refresh: fetch KLine -> render SVG -> rasterize -> show
    {
        let ui_weak = ui.as_weak();
        let scale = scale.clone();
        let current_svg_content = current_svg_content.clone();

        ui.on_reload_kline(move || {
            let Some(ui) = ui_weak.upgrade() else {
                return;
            };

            // Parse inputs
            let market = match parse_u16("market", ui.get_market_input()) {
                Ok(v) => v,
                Err(e) => {
                    ui.set_svg_status(format!("Error: {e}").into());
                    return;
                }
            };

            let code = ui.get_code_input().to_string();
            if code.len() != 6 {
                ui.set_svg_status("Error: code must be 6 characters (e.g. 000001)".into());
                return;
            }

            let category = match parse_u16("category", ui.get_category_input()) {
                Ok(v) => v,
                Err(e) => {
                    ui.set_svg_status(format!("Error: {e}").into());
                    return;
                }
            };

            let start = match parse_u16("start", ui.get_start_input()) {
                Ok(v) => v,
                Err(e) => {
                    ui.set_svg_status(format!("Error: {e}").into());
                    return;
                }
            };

            let count = match parse_u16("count", ui.get_count_input()) {
                Ok(v) => v,
                Err(e) => {
                    ui.set_svg_status(format!("Error: {e}").into());
                    return;
                }
            };

            // Enforce protocol limit (tcptdx typically caps at 800)
            let count = count.min(800);

            // Fetch
            let candles = match fetch_kline_as_candles(market, &code, category, start, count) {
                Ok(c) => c,
                Err(e) => {
                    ui.set_svg_status(format!("Error: {e}").into());
                    return;
                }
            };

            // Render SVG
            let full_render = ui.get_full_render();
            let svg = if full_render {
                render_svg_candles_full(&candles, 900, 520)
            } else {
                render_svg_candles_simple(&candles, 900, 420)
            };

            // Store the SVG so zoom can re-render at the new scale
            *current_svg_content.borrow_mut() = svg;

            // Rasterize at current scale and show
            let current_scale = *scale.borrow();
            match svg_to_image_with_scale(&current_svg_content.borrow(), current_scale) {
                Ok(image) => {
                    ui.set_svg_image(image);
                    ui.set_svg_status(format!("Loaded {} candles", candles.len()).into());
                }
                Err(e) => {
                    ui.set_svg_status(format!("Error: {e}").into());
                }
            }
        });
    }

    // Scale Up
    {
        let ui_weak = ui.as_weak();
        let scale = scale.clone();
        let current_svg_content = current_svg_content.clone();
        ui.on_scale_up(move || {
            let mut current_scale = scale.borrow_mut();
            *current_scale = (*current_scale * 1.2).min(5.0);

            if let Some(ui) = ui_weak.upgrade() {
                if current_svg_content.borrow().is_empty() {
                    ui.set_svg_status("No chart loaded".into());
                    ui.set_current_scale(format!("Scale: {:.1}x", *current_scale).into());
                    return;
                }

                match svg_to_image_with_scale(&current_svg_content.borrow(), *current_scale) {
                    Ok(image) => {
                        ui.set_svg_image(image);
                        ui.set_current_scale(format!("Scale: {:.1}x", *current_scale).into());
                    }
                    Err(e) => ui.set_svg_status(format!("Error: {e}").into()),
                }
            }
        });
    }

    // Scale Down
    {
        let ui_weak = ui.as_weak();
        let scale = scale.clone();
        let current_svg_content = current_svg_content.clone();
        ui.on_scale_down(move || {
            let mut current_scale = scale.borrow_mut();
            *current_scale = (*current_scale / 1.2).max(0.2);

            if let Some(ui) = ui_weak.upgrade() {
                if current_svg_content.borrow().is_empty() {
                    ui.set_svg_status("No chart loaded".into());
                    ui.set_current_scale(format!("Scale: {:.1}x", *current_scale).into());
                    return;
                }

                match svg_to_image_with_scale(&current_svg_content.borrow(), *current_scale) {
                    Ok(image) => {
                        ui.set_svg_image(image);
                        ui.set_current_scale(format!("Scale: {:.1}x", *current_scale).into());
                    }
                    Err(e) => ui.set_svg_status(format!("Error: {e}").into()),
                }
            }
        });
    }

    // Reset View
    {
        let ui_weak = ui.as_weak();
        let scale = scale.clone();
        let position_x = position_x.clone();
        let current_svg_content = current_svg_content.clone();
        ui.on_reset_scale(move || {
            let mut current_scale = scale.borrow_mut();
            let mut current_pos_x = position_x.borrow_mut();
            *current_scale = 1.0;
            *current_pos_x = 0.0;

            if let Some(ui) = ui_weak.upgrade() {
                ui.set_current_scale(format!("Scale: {:.1}x", *current_scale).into());
                ui.set_target_x_offset(*current_pos_x);

                if current_svg_content.borrow().is_empty() {
                    ui.set_svg_status("No chart loaded".into());
                    return;
                }

                match svg_to_image_with_scale(&current_svg_content.borrow(), *current_scale) {
                    Ok(image) => ui.set_svg_image(image),
                    Err(e) => ui.set_svg_status(format!("Error: {e}").into()),
                }
            }
        });
    }

    // Swipe Left
    {
        let ui_weak = ui.as_weak();
        let position_x = position_x.clone();
        ui.on_swipe_left(move || {
            let mut current_pos_x = position_x.borrow_mut();
            *current_pos_x -= 100.0;
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_target_x_offset(*current_pos_x);
            }
        });
    }

    // Swipe Right
    {
        let ui_weak = ui.as_weak();
        let position_x = position_x.clone();
        ui.on_swipe_right(move || {
            let mut current_pos_x = position_x.borrow_mut();
            *current_pos_x += 100.0;
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_target_x_offset(*current_pos_x);
            }
        });
    }

    // Initialize scale display
    ui.set_current_scale("Scale: 1.0x".into());

    ui.run()
}
