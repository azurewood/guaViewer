use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use std::cell::RefCell;
use std::rc::Rc;

// --- tcptdx integration ---
//
// This Slint app is intended to *consume* the `tcptdx` library.
// The exact import path depends on how you expose items from the library.
// The simplest setup is to re-export `FeedClient` and `KLine` from `tcptdx`'s lib.rs.
//
// If your paths differ, adjust the `use ...` lines below.
use tcptdx::{client::FeedClient, commands::KLine};

slint::include_modules!();

/// A single OHLCV candle used by the SVG renderer.
///
/// This is deliberately UI-agnostic so we can later plug in `tcptdx::KLine` data
/// without changing the rendering logic.
#[derive(Clone, Debug)]
struct Candle {
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    /// Optional label (e.g. date). Only used by the "full" renderer.
    label: String,
}

// /// Generate a small deterministic dataset so the UI can render *something*
// /// before we wire the real `FeedClient` network fetch.
// fn sample_candles(count: usize) -> Vec<Candle> {
//     let mut out = Vec::with_capacity(count);
//     let mut last_close = 100.0_f64;

//     for i in 0..count {
//         // A simple, deterministic pseudo-series (no randomness).
//         let wave = ((i as f64) / 5.0).sin() * 2.0;
//         let drift = (i as f64) * 0.05;
//         let close = 100.0 + drift + wave;
//         let open = last_close;
//         let high = open.max(close) + 1.0 + (((i as f64) / 3.0).cos().abs() * 0.8);
//         let low = open.min(close) - 1.0 - (((i as f64) / 4.0).sin().abs() * 0.8);
//         let volume = 1000.0 + ((i as f64) * 37.0).sin().abs() * 800.0;

//         out.push(Candle {
//             open,
//             high,
//             low,
//             close,
//             volume,
//             label: format!("#{}", i),
//         });

//         last_close = close;
//     }

//     out
// }

// /// Fetch KLine data from the TDX server using `tcptdx`, then map it into candles.
// ///
// /// Notes:
// /// - `count` is limited by the protocol (usually max 800). We clamp defensively.
// /// - The `label` field is only used by the "full" SVG renderer.
// fn fetch_kline_as_candles(
//     market: u16,
//     code: &str,
//     category: u16,
//     start: u16,
//     count: u16,
// ) -> Result<Vec<Candle>, String> {
//     // Defensive clamp (protocol max is commonly 800, see your `KLine` docs).
//     let count = count.min(800);

//     // Build request (your `KLine` is owned-string based, so this is cheap).
//     let mut req = KLine::new(market, code.to_string(), category, start, count);

//     // Create a client and perform the request.
//     let mut client =
//         FeedClient::new_default().map_err(|e| format!("FeedClient::new_default: {e}"))?;
//     client
//         .send(&mut req)
//         .map_err(|e| format!("FeedClient::send: {e}"))?;

//     // Map tcptdx's `KLineData` into our UI-agnostic candle type.
//     // We keep a human-readable label for the X-axis in the full renderer.
//     let mut out = Vec::with_capacity(req.data.len());
//     for d in req.data.iter() {
//         out.push(Candle {
//             open: d.open,
//             high: d.high,
//             low: d.low,
//             close: d.close,
//             volume: d.vol,
//             label: format!("{:?}", d.dt),
//         });
//     }
//     Ok(out)
// }
//
// /// Fetch KLine data from tcptdx and convert it into Candle structs.
///
/// # Arguments
/// * `market`   - Market ID (e.g. 0 = Shenzhen, 1 = Shanghai)
/// * `code`     - Stock code (e.g. "000001")
/// * `category` - KLine category (e.g. 9 = daily)
/// * `start`    - Start index
/// * `count`    - Number of KLine records to fetch
///
/// # Returns
/// * `Ok(Vec<Candle>)` on success
/// * `Err(String)` if network or protocol errors occur
fn fetch_kline_as_candles(
    market: u16,
    code: &str,
    category: u16,
    start: u16,
    count: u16,
) -> Result<Vec<Candle>, String> {
    let mut req = KLine::new(market, code, category, start, count);
    let mut client =
        FeedClient::new_default().map_err(|e| format!("FeedClient init failed: {e}"))?;

    client
        .send(&mut req)
        .map_err(|e| format!("KLine fetch failed: {e}"))?;

    Ok(req
        .data
        .iter()
        .map(|rec| Candle {
            open: rec.open as f64,
            high: rec.high as f64,
            low: rec.low as f64,
            close: rec.close as f64,
            volume: rec.vol as f64,
            label: format!("{:?}", rec.dt).to_owned(), //rec.dt,
        })
        .collect())
}

fn f64_min_max(values: impl Iterator<Item = f64>) -> Option<(f64, f64)> {
    let mut it = values;
    let first = it.next()?;
    let mut min_v = first;
    let mut max_v = first;
    for v in it {
        if v < min_v {
            min_v = v;
        }
        if v > max_v {
            max_v = v;
        }
    }
    Some((min_v, max_v))
}

/// Render a simple OHLC candlestick chart as SVG.
///
/// This renderer draws:
/// - Candle bodies (open/close)
/// - Wicks (high/low)
///
/// It intentionally excludes:
/// - Grid lines
/// - Volume bars
/// - Axis labels
///
/// This keeps the SVG lightweight and fast for transforms.
fn render_svg_candles_simple(candles: &[Candle], width: u32, height: u32) -> String {
    let (min_p, max_p) =
        f64_min_max(candles.iter().flat_map(|c| [c.low, c.high].into_iter())).unwrap_or((0.0, 1.0));

    // Keep a small padding so wicks/bodies are not clipped.
    let pad = 12.0_f64;
    let w = width as f64;
    let h = height as f64;

    // Avoid division by zero if all prices are identical.
    let span = (max_p - min_p).max(1e-9);
    let price_to_y = |p: f64| -> f64 {
        // SVG y grows downward.
        pad + (max_p - p) / span * (h - pad * 2.0)
    };

    // Candle sizing.
    let n = candles.len().max(1) as f64;
    let step_x = (w - pad * 2.0) / n;
    let body_w = (step_x * 0.65).max(2.0);

    let mut svg = String::new();
    svg.push_str(&format!(
        r#"<svg xmlns="www.w3.org" width="{width}" height="{height}" viewBox="0 0 {width} {height}" fill="{color}">\n"#,
        width = width,
        height = height,
        color = "#374151"
    ));
    svg.push_str(r#"<rect x="0" y="0" width="100%" height="100%" fill="white"/>\n"#);

    for (i, c) in candles.iter().enumerate() {
        let cx = pad + (i as f64 + 0.5) * step_x;
        let y_high = price_to_y(c.high);
        let y_low = price_to_y(c.low);
        let y_open = price_to_y(c.open);
        let y_close = price_to_y(c.close);

        let is_up = c.close >= c.open;
        let stroke = if is_up { "#16a34a" } else { "#dc2626" };
        let fill = if is_up { "#22c55e" } else { "#ef4444" };

        // Wick
        svg.push_str(&format!(
            r#"<line x1="{cx:.2}" y1="{y_high:.2}" x2="{cx:.2}" y2="{y_low:.2}" stroke="{stroke}" stroke-width="1"/>\n"#
        ));

        // Body
        let body_top = y_open.min(y_close);
        let body_h = (y_open - y_close).abs().max(1.0);
        let x = cx - body_w / 2.0;
        svg.push_str(&format!(
            r#"<rect x="{x:.2}" y="{body_top:.2}" width="{body_w:.2}" height="{body_h:.2}" fill="{fill}" stroke="{stroke}" stroke-width="1"/>\n"#
        ));
    }

    svg.push_str("</svg>\n");
    svg
}

/// Render a candlestick chart with:
/// - price grid
/// - volume bars
/// - basic labels
fn render_svg_candles_full(candles: &[Candle], width: u32, height: u32) -> String {
    let w = width as f64;
    let h = height as f64;
    let pad = 16.0_f64;
    let label_pad_left = 44.0_f64; // room for y-axis labels
    let label_pad_bottom = 22.0_f64;

    let chart_left = pad + label_pad_left;
    let chart_right = w - pad;
    let chart_top = pad;
    let chart_bottom = h - pad - label_pad_bottom;

    // Split into price + volume areas.
    let price_h = (chart_bottom - chart_top) * 0.72;
    let vol_h = (chart_bottom - chart_top) - price_h - 10.0;
    let price_top = chart_top;
    let price_bottom = price_top + price_h;
    let vol_top = price_bottom + 10.0;
    let vol_bottom = vol_top + vol_h;

    let (min_p, max_p) =
        f64_min_max(candles.iter().flat_map(|c| [c.low, c.high].into_iter())).unwrap_or((0.0, 1.0));
    let span_p = (max_p - min_p).max(1e-9);
    let price_to_y =
        |p: f64| -> f64 { price_top + (max_p - p) / span_p * (price_bottom - price_top) };

    let (min_v, max_v) = f64_min_max(candles.iter().map(|c| c.volume)).unwrap_or((0.0, 1.0));
    let span_v = (max_v - min_v).max(1e-9);
    let vol_to_h = |v: f64| -> f64 { (v - min_v) / span_v * (vol_bottom - vol_top) };

    let n = candles.len().max(1) as f64;
    let step_x = (chart_right - chart_left) / n;
    let body_w = (step_x * 0.65).max(2.0);

    let mut svg = String::new();
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">\n"#
    ));
    svg.push_str(r#"<rect x="0" y="0" width="100%" height="100%" fill="white"/>\n"#);

    // Grid (price area)
    let grid_lines = 5;
    for i in 0..=grid_lines {
        let t = i as f64 / grid_lines as f64;
        let y = price_top + t * (price_bottom - price_top);
        svg.push_str(&format!(
            r#"<line x1="{chart_left:.2}" y1="{y:.2}" x2="{chart_right:.2}" y2="{y:.2}" stroke="{color}" stroke-width="1"/>"#,
            chart_left = chart_left,
            chart_right = chart_right,
            y = y,
            color = "#e5e7eb"
        ));

        // Y-axis label
        let p = max_p - t * span_p;
        svg.push_str(&format!(
            r#"<text x="{x:.2}" y="{y:.2}" font-size="11" text-anchor="end" dominant-baseline="middle" fill="{color}">{p:.2}</text>"#,
            x = chart_left - 6.0,
            y = y,
            p = p,
            color = "#374151"
        ));
    }

    // Outer borders for price and volume panels.
    svg.push_str(&format!(
        r#"<rect x="{chart_left:.2}" y="{price_top:.2}" width="{pw:.2}" height="{ph:.2}" fill="none" stroke="{gray}" stroke-width="1"/>"#,
        chart_left = chart_left,
        price_top = price_top,
        pw = chart_right - chart_left,
        ph = price_bottom - price_top,
        gray = "#d1d5db"
    ));

    svg.push_str(&format!(
        r#"<rect x="{chart_left:.2}" y="{vol_top:.2}" width="{pw:.2}" height="{vh:.2}" fill="none" stroke="{gray}" stroke-width="1"/>"#,
        chart_left = chart_left,
        vol_top = vol_top,
        pw = chart_right - chart_left,
        vh = vol_bottom - vol_top,
        gray = "#d1d5db"
    ));

    // Candles + volume
    for (i, c) in candles.iter().enumerate() {
        let cx = chart_left + (i as f64 + 0.5) * step_x;
        let y_high = price_to_y(c.high);
        let y_low = price_to_y(c.low);
        let y_open = price_to_y(c.open);
        let y_close = price_to_y(c.close);

        let is_up = c.close >= c.open;
        let stroke = if is_up { "#16a34a" } else { "#dc2626" };
        let fill = if is_up { "#22c55e" } else { "#ef4444" };

        // Wick
        svg.push_str(&format!(
            r#"<line x1="{cx:.2}" y1="{y_high:.2}" x2="{cx:.2}" y2="{y_low:.2}" stroke="{stroke}" stroke-width="1"/>\n"#
        ));

        // Body
        let body_top = y_open.min(y_close);
        let body_h = (y_open - y_close).abs().max(1.0);
        let x = cx - body_w / 2.0;
        svg.push_str(&format!(
            r#"<rect x="{x:.2}" y="{body_top:.2}" width="{body_w:.2}" height="{body_h:.2}" fill="{fill}" stroke="{stroke}" stroke-width="1"/>\n"#
        ));

        // Volume bar
        let v_h = vol_to_h(c.volume);
        let v_y = vol_bottom - v_h;
        svg.push_str(&format!(
            r#"<rect x="{x:.2}" y="{v_y:.2}" width="{body_w:.2}" height="{v_h:.2}" fill="{fill_color}" opacity="0.7"/>"#,
            x = x,
            v_y = v_y,
            body_w = body_w,
            v_h = v_h,
            fill_color = "#9ca3af"
        ));

        // X labels (sparse)
        let every = (candles.len() / 6).max(1);
        if i % every == 0 {
            svg.push_str(&format!(
                r#"<text x="{cx:.2}" y="{y:.2}" font-size="11" text-anchor="middle" fill="{fill}">{label}</text>"#,
                cx = cx,
                y = h - pad,
                label = c.label,
                fill = "#374151"
            ));
        }
    }

    // Title-like caption
    svg.push_str(&format!(
        r#"<text x="{x:.2}" y="{y:.2}" font-size="13" font-weight="600" fill="{color}">Candles (price + volume)</text>"#,
        x = chart_left,
        y = 14.0,
        color = "#111827"
    ));

    svg.push_str("</svg>\n");
    svg
}

fn svg_to_image_with_scale(svg_content: &str, scale: f32) -> Option<Image> {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(svg_content.as_bytes(), &opt).ok()?;

    let size = tree.size();
    let width = (size.width() * scale) as u32;
    let height = (size.height() * scale) as u32;

    // Ensure minimum size
    if width == 0 || height == 0 {
        return None;
    }

    let mut pixmap = tiny_skia::Pixmap::new(width, height)?;

    // Apply scaling transform
    let transform = tiny_skia::Transform::from_scale(scale, scale);
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // Convert BGRA to RGBA - create vec of u8 values directly
    let mut rgba_bytes: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);
    for chunk in pixmap.data().chunks_exact(4) {
        rgba_bytes.push(chunk[0]); // R
        rgba_bytes.push(chunk[1]); // G
        rgba_bytes.push(chunk[2]); // B
        rgba_bytes.push(chunk[3]); // A
    }

    let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(&rgba_bytes, width, height);
    Some(Image::from_rgba8(buffer))
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;

    let default_svg = r#"<svg width="200" height="200" xmlns="http://www.w3.org/2000/svg">
        <circle cx="100" cy="100" r="80" fill="blue" stroke="black" stroke-width="2"/>
        <rect x="70" y="70" width="60" height="60" fill="yellow" opacity="0.7"/>
        <polygon points="100,40 120,80 80,80" fill="red"/>
    </svg>"#;

    // Store the current scale factor, position, and SVG content
    let scale = Rc::new(RefCell::new(1.0f32));
    let position_x = Rc::new(RefCell::new(0.0f32));
    let current_svg_content = Rc::new(RefCell::new(default_svg.to_string()));

    // Set initial SVG content in the text area and set up monitoring
    ui.set_svg_input(default_svg.into());

    // Monitor for changes in the SVG input using a timer-based approach
    {
        let ui_weak = ui.as_weak();
        let scale = scale.clone();
        let current_svg_content = current_svg_content.clone();
        let last_svg_content = Rc::new(RefCell::new(default_svg.to_string()));

        let timer = slint::Timer::default();
        timer.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_millis(500),
            move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let new_svg = ui.get_svg_input().to_string();
                    let last_content = last_svg_content.borrow().clone();

                    // Only update if content has changed
                    if new_svg != last_content && !new_svg.trim().is_empty() {
                        let current_scale = *scale.borrow();

                        match svg_to_image_with_scale(&new_svg, current_scale) {
                            Some(image) => {
                                // Update the stored SVG content
                                *current_svg_content.borrow_mut() = new_svg.clone();
                                *last_svg_content.borrow_mut() = new_svg;

                                // Update image with current scale
                                ui.set_svg_image(image);
                                ui.set_svg_status("SVG updated".into());
                            }
                            None => {
                                *last_svg_content.borrow_mut() = new_svg;
                                ui.set_svg_status("Error: Invalid SVG".into());
                            }
                        }
                    } else if new_svg.trim().is_empty() && new_svg != last_content {
                        *last_svg_content.borrow_mut() = new_svg;
                        ui.set_svg_status("Empty SVG content".into());
                    }
                }
            },
        );
    }

    // Set initial image
    if let Some(image) = svg_to_image_with_scale(&current_svg_content.borrow(), *scale.borrow()) {
        ui.set_svg_image(image);
    }

    // Set initial scale display
    ui.set_current_scale(format!("Scale: {:.1}x", *scale.borrow()).into());
    ui.set_image_x_offset((*position_x.borrow() as f32) * 1.0);

    // Handle SVG content changes manually with update button
    {
        let ui_weak = ui.as_weak();
        let scale = scale.clone();
        let current_svg_content = current_svg_content.clone();

        ui.on_svg_content_changed(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let new_svg = ui.get_svg_input().to_string();

                // Only update if content is not empty
                if !new_svg.trim().is_empty() {
                    let current_scale = *scale.borrow();

                    match svg_to_image_with_scale(&new_svg, current_scale) {
                        Some(image) => {
                            // Update the stored SVG content
                            *current_svg_content.borrow_mut() = new_svg;

                            // Update image with current scale
                            ui.set_svg_image(image);
                            ui.set_svg_status("SVG updated manually".into());
                        }
                        None => {
                            ui.set_svg_status("Error: Invalid SVG".into());
                        }
                    }
                } else {
                    ui.set_svg_status("Empty SVG content".into());
                }
            }
        });
    }

    // Handle scale up button
    {
        let ui_weak = ui.as_weak();
        let scale = scale.clone();
        let current_svg_content = current_svg_content.clone();
        ui.on_scale_up(move || {
            let mut current_scale = scale.borrow_mut();
            *current_scale = (*current_scale * 1.2).min(5.0); // Increase by 20%, max 5x

            if let Some(ui) = ui_weak.upgrade() {
                if let Some(image) =
                    svg_to_image_with_scale(&current_svg_content.borrow(), *current_scale)
                {
                    ui.set_svg_image(image);
                    ui.set_current_scale(format!("Scale: {:.1}x", *current_scale).into());
                }
            }
        });
    }

    // Handle scale down button
    {
        let ui_weak = ui.as_weak();
        let scale = scale.clone();
        let current_svg_content = current_svg_content.clone();
        ui.on_scale_down(move || {
            let mut current_scale = scale.borrow_mut();
            *current_scale = (*current_scale / 1.2).max(0.2); // Decrease by 20%, min 0.2x

            if let Some(ui) = ui_weak.upgrade() {
                if let Some(image) =
                    svg_to_image_with_scale(&current_svg_content.borrow(), *current_scale)
                {
                    ui.set_svg_image(image);
                    ui.set_current_scale(format!("Scale: {:.1}x", *current_scale).into());
                }
            }
        });
    }

    // Handle reset button
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
                if let Some(image) =
                    svg_to_image_with_scale(&current_svg_content.borrow(), *current_scale)
                {
                    ui.set_svg_image(image);
                    ui.set_current_scale(format!("Scale: {:.1}x", *current_scale).into());
                    ui.set_image_x_offset(*current_pos_x);
                    ui.set_target_x_offset(*current_pos_x);
                }
            }
        });
    }

    // Handle swipe left
    {
        let ui_weak = ui.as_weak();
        let position_x = position_x.clone();
        ui.on_swipe_left(move || {
            let mut current_pos_x = position_x.borrow_mut();
            *current_pos_x -= 100.0; // Move left by 100px

            if let Some(ui) = ui_weak.upgrade() {
                ui.set_target_x_offset(*current_pos_x);
            }
        });
    }

    // Handle swipe right
    {
        let ui_weak = ui.as_weak();
        let position_x = position_x.clone();
        ui.on_swipe_right(move || {
            let mut current_pos_x = position_x.borrow_mut();
            *current_pos_x += 100.0; // Move right by 100px

            if let Some(ui) = ui_weak.upgrade() {
                ui.set_target_x_offset(*current_pos_x);
            }
        });
    }

    // Handle KLine reload/refresh button (fetch + SVG generation, end-to-end)
    {
        let ui_weak = ui.as_weak();
        ui.on_reload_kline(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let market = ui.get_market_input().trim().parse::<u16>();
                let category = ui.get_category_input().trim().parse::<u16>();
                let start = ui.get_start_input().trim().parse::<u16>();
                let count = ui.get_count_input().trim().parse::<u16>();
                let code = ui.get_code_input().trim().to_string();
                let full_render = ui.get_full_render();

                match (market, category, start, count) {
                    (Ok(market), Ok(category), Ok(start), Ok(count)) => {
                        // Fetch real KLine data via tcptdx.
                        // match fetch_kline_as_candles(market, &code, category, start, count) {
                        //     Ok(candles) => {
                        //         if candles.is_empty() {
                        //             ui.set_svg_status("No data returned".into());
                        //             return;
                        //         }

                        //         let svg = if full_render {
                        //             render_svg_candles_full(&candles, 900, 520)
                        //         } else {
                        //             render_svg_candles_simple(&candles, 900, 420)
                        //         };

                        //         // Setting svg-input will trigger the existing timer-based renderer.
                        //         ui.set_svg_input(svg.into());
                        //         ui.set_svg_status(
                        //             format!(
                        //                 "Rendered {} candles (market={} code={} category={} start={} count={})",
                        //                 candles.len(), market, code, category, start, count
                        //             )
                        //             .into(),
                        //         );
                        //     }
                        //     Err(err) => {
                        //         // Fallback: keep the UI responsive even if networking fails.
                        //         // You can remove this fallback once your network layer is stable.
                        //         let candles = sample_candles(count as usize);
                        //         let svg = if full_render {
                        //             render_svg_candles_full(&candles, 900, 520)
                        //         } else {
                        //             render_svg_candles_simple(&candles, 900, 420)
                        //         };
                        //         ui.set_svg_input(svg.into());
                        //         ui.set_svg_status(format!("Fetch failed: {err} (showing sample data)").into());
                        //     }
                        // }
                        match fetch_kline_as_candles(market, &code, category, start, count) {
                            Ok(candles) => {
                                let svg = if full_render {
                                    render_svg_candles_full(&candles, 900, 520)
                                } else {
                                    render_svg_candles_simple(&candles, 900, 420)
                                };
                                ui.set_svg_input(svg.into());
                                ui.set_svg_status(
                                    format!("Loaded {} candles", candles.len()).into(),
                                );
                            }
                            Err(err) => {
                                ui.set_svg_input("".into());
                                ui.set_svg_status(format!("Error: {err}").into());
                            }
                        }
                    }
                    _ => {
                        ui.set_svg_status(
                            "Invalid input: market/category/start/count must be u16".into(),
                        );
                    }
                }
            }
        });
    }

    ui.run()
}
