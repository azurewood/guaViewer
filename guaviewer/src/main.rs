mod app_run;

fn main() -> Result<(), slint::PlatformError> {
    app_run::run_app()
}

// use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
// use std::cell::RefCell;
// use std::rc::Rc;
// use std::sync::Arc;
// use std::sync::mpsc;
// use std::thread;

// // tcptdx library types (ensure your Cargo.toml includes the tcptdx crate)
// use guaviewer::model::{AppError, Candle, Cursor, FetchRequest, FetchResult};
// use tcptdx::{client::FeedClient, commands::KLine};

// slint::include_modules!();

// /// Parse a `u16` from a Slint string field.
// ///
// /// This returns a structured `AppError::ParseU16` which retains field context.
// fn parse_u16(field: &'static str, text: slint::SharedString) -> Result<u16, AppError> {
//     let trimmed = text.trim();
//     trimmed.parse::<u16>().map_err(|_| AppError::ParseU16 {
//         field,
//         value: text.to_string(),
//     })
// }

// /// Validate stock code input.
// ///
// /// Rules:
// /// - trimmed
// /// - exactly 6 characters
// /// - digits only (0-9)
// fn validate_code(raw: &str) -> Result<String, AppError> {
//     let code = raw.trim();

//     if code.len() != 6 {
//         return Err(AppError::InvalidInput {
//             message: "code must be exactly 6 digits (e.g. 000001)".into(),
//         });
//     }

//     if !code.chars().all(|c| c.is_ascii_digit()) {
//         return Err(AppError::InvalidInput {
//             message: "code must contain digits only (0–9)".into(),
//         });
//     }

//     Ok(code.to_string())
// }

// /// Convert an SVG string into a raster `slint::Image` using the provided scale factor.
// ///
// /// Slint's built-in SVG support is limited depending on backend; rasterizing via `resvg`
// /// gives consistent rendering while still allowing you to implement zoom/pan by re-rendering
// /// at different scales.
// fn svg_to_image_with_scale(svg_content: &str, scale: f32) -> Result<Image, AppError> {
//     // 1. Create and configure the font database first
//     // println!("{}", scale);
//     let mut fontdb = fontdb::Database::new();
//     fontdb.load_system_fonts(); // Now we can call &mut methods safely

//     // 2. Initialize Options with the prepared fontdb
//     let opt = usvg::Options {
//         fontdb: Arc::new(fontdb),
//         ..usvg::Options::default()
//     };
//     // let tree = usvg::Tree::from_data(svg_content.as_bytes(), &opt)
//     //     .map_err(|e| format!("Invalid SVG: {e}"))?;
//     let tree =
//         usvg::Tree::from_data(svg_content.as_bytes(), &opt).map_err(|e| AppError::InvalidSvg {
//             message: e.to_string(),
//         })?;

//     let size = tree.size();
//     let width = (size.width() * scale) as u32;
//     let height = (size.height() * 1.0f32) as u32;

//     if width == 0 || height == 0 {
//         // return Err("SVG produced an empty image".to_string());
//         return Err(AppError::EmptySvgImage);
//     }

//     // let mut pixmap = tiny_skia::Pixmap::new(width, height)
//     //     .ok_or_else(|| "Failed to create pixmap".to_string())?;
//     let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or(AppError::PixmapAllocFailed)?;

//     // Apply scaling transform
//     let transform = tiny_skia::Transform::from_scale(scale, 1.0f32);
//     resvg::render(&tree, transform, &mut pixmap.as_mut());

//     // resvg outputs premultiplied BGRA; Slint expects RGBA
//     let mut rgba_bytes: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);
//     for chunk in pixmap.data().chunks_exact(4) {
//         rgba_bytes.push(chunk[0]); // R
//         rgba_bytes.push(chunk[1]); // G
//         rgba_bytes.push(chunk[2]); // B
//         rgba_bytes.push(chunk[3]); // A
//     }

//     let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(&rgba_bytes, width, height);
//     Ok(Image::from_rgba8(buffer))
// }

// /// Backward-compatible helper for older call sites.
// /// `position_x` is currently ignored because panning is handled at the UI layer.
// fn render_svg_to_image(svg_content: &str, scale: f32, _position_x: f32) -> Result<Image, AppError> {
//     svg_to_image_with_scale(svg_content, scale)
// }

// /// Fetch KLine data from tcptdx and convert it into candles.
// ///
// /// - `market`: market id (0 = Shenzhen, 1 = Shanghai)
// /// - `code`: 6-digit stock code (e.g. "000001")
// /// - `category`: kline category (e.g. 9 = daily)
// /// - `start`: start index
// /// - `count`: number of records (tcptdx max is typically 800)
// fn fetch_kline_as_candles(
//     market: u16,
//     code: &str,
//     category: u16,
//     start: u16,
//     count: u16,
// ) -> Result<Vec<Candle>, AppError> {
//     let mut req = KLine::new(market, code, category, start, count);
//     // let mut client =
//     //     FeedClient::new_default().map_err(|e| format!("FeedClient init failed: {e}"))?;
//     let mut client = FeedClient::new_default().map_err(|e| AppError::ClientInit {
//         message: e.to_string(),
//     })?;

//     // client
//     //     .send(&mut req)
//     //     .map_err(|e| format!("KLine fetch failed: {e}"))?;
//     client.send(&mut req).map_err(|e| AppError::FetchKLine {
//         message: e.to_string(),
//     })?;

//     // Map tcptdx records into our chart candles.
//     let candles = req
//         .data
//         .iter()
//         .map(|rec| Candle {
//             open: rec.open as f64,
//             high: rec.high as f64,
//             low: rec.low as f64,
//             close: rec.close as f64,
//             volume: rec.vol as f64,
//             label: format!(
//                 "{:04}-{:02}-{:02} {:02}:{:02}",
//                 rec.dt.year, rec.dt.month, rec.dt.day, rec.dt.hour, rec.dt.minute
//             ),
//         })
//         .collect::<Vec<_>>();

//     if candles.is_empty() {
//         // return Err("No KLine data returned".to_string());
//         return Err(AppError::NoKLineData);
//     }

//     Ok(candles)
// }

// /// Render a simple OHLC candlestick chart into an SVG string.
// ///
// /// This version draws:
// /// - Candle wick (high-low)
// /// - Candle body (open-close)
// ///
// /// It intentionally omits:
// /// - Grid / axes
// /// - Labels
// /// - Volume

// fn auto_price_precision(min_price: f64, max_price: f64, tick_count: usize) -> usize {
//     let ticks = tick_count.max(2) as f64;
//     let range = (max_price - min_price).abs().max(1e-12);
//     let step = (range / (ticks - 1.0)).abs().max(1e-12);

//     let mut prec: i32 = if step >= 1.0 {
//         0
//     } else {
//         let p = -step.log10().floor();
//         p as i32
//     };

//     if prec < 0 {
//         prec = 0;
//     }
//     if prec > 6 {
//         prec = 6;
//     }
//     prec as usize
// }

// fn fmt_price(price: f64, precision: usize) -> String {
//     format!("{:.*}", precision, price)
// }

// /// Render candles, then (optionally) overlay a cursor (vertical+horizontal line + price label).
// fn render_svg_with_cursor(
//     candles: &[Candle],
//     width: i32,
//     height: i32,
//     full_render: bool,
//     cursor: Option<Cursor>,
// ) -> String {
//     let mut svg = if full_render {
//         render_svg_candles_full(candles, width, height)
//     } else {
//         render_svg_candles_simple(candles, width, height)
//     };

//     let Some(cursor) = cursor else { return svg };

//     // Insert overlay just before </svg>
//     let overlay = cursor_overlay_svg(candles, width, height, full_render, cursor);
//     // println!("{}", overlay);
//     if let Some(idx) = svg.rfind("</svg>") {
//         svg.insert_str(idx, &overlay);
//     }
//     // println!();
//     // println!("{}", svg);
//     svg
// }

// /// Build cursor overlay SVG elements.
// fn cursor_overlay_svg(
//     candles: &[Candle],
//     width: i32,
//     height: i32,
//     full_render: bool,
//     cursor: Cursor,
// ) -> String {
//     if candles.is_empty() {
//         return String::new();
//     }

//     let w = width as f64;
//     let h = height as f64;

//     let min_price = candles.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
//     let max_price = candles
//         .iter()
//         .map(|c| c.high)
//         .fold(f64::NEG_INFINITY, f64::max);
//     let price_range = (max_price - min_price).max(1e-9);

//     let (pad_left, _pad_right, pad_top, _pad_bottom, plot_h, plot_w, y_top, y_bottom) =
//         if full_render {
//             let pad_left = 60.0;
//             let pad_right = 20.0;
//             let pad_top = 20.0;
//             let vol_h = (h * 0.22).max(80.0);
//             let chart_h = (h - vol_h).max(200.0);
//             let plot_h = (chart_h - pad_top - 10.0).max(1.0);
//             let y_top = pad_top;
//             let y_bottom = pad_top + plot_h;
//             (
//                 pad_left,
//                 pad_right,
//                 pad_top,
//                 30.0,
//                 plot_h,
//                 (w - pad_left - pad_right).max(1.0),
//                 y_top,
//                 y_bottom,
//             )
//         } else {
//             let pad_left = 20.0;
//             let pad_right = 20.0;
//             let pad_top = 20.0;
//             let pad_bottom = 20.0;
//             let plot_h = (h - pad_top - pad_bottom).max(1.0);
//             let y_top = pad_top;
//             let y_bottom = pad_top + plot_h;
//             (
//                 pad_left,
//                 pad_right,
//                 pad_top,
//                 pad_bottom,
//                 plot_h,
//                 (w - pad_left - pad_right).max(1.0),
//                 y_top,
//                 y_bottom,
//             )
//         };

//     let n = candles.len().max(1) as f64;
//     let step = (plot_w / n).max(1.0);

//     let idx = cursor.idx.min(candles.len().saturating_sub(1));
//     let x_center = pad_left + (idx as f64 + 0.5) * step;

//     // price -> y (clamped into price plot area)
//     let mut y = pad_top + (max_price - cursor.price) / price_range * plot_h;
//     if y < y_top {
//         y = y_top;
//     }
//     if y > y_bottom {
//         y = y_bottom;
//     }

//     let precision = auto_price_precision(min_price, max_price, 6);

//     // --- Horizontal cursor: show the price label just LEFT of the crossing.
//     let cross_label = fmt_price(cursor.price, precision);
//     let cross_label_h = 18.0;
//     let cross_label_w = (cross_label.len() as f64 * 7.0 + 12.0).max(44.0);
//     let cross_label_y = y - cross_label_h / 2.0;
//     // cross_label_y = cross_label_y.clamp(y_top, y_bottom - cross_label_h);
//     let mut cross_label_x = x_center - 8.0 - cross_label_w;
//     cross_label_x = cross_label_x.clamp(pad_left, pad_left + plot_w - cross_label_w);

//     // --- Vertical cursor: show candle OHLC + date in a compact box on the RIGHT side.
//     let c = &candles[idx];
//     let lines = [
//         c.label.clone(),
//         format!("O {}", fmt_price(c.open, precision)),
//         format!("H {}", fmt_price(c.high, precision)),
//         format!("L {}", fmt_price(c.low, precision)),
//         format!("C {}", fmt_price(c.close, precision)),
//         format!("V {}", fmt_price(c.volume, precision)),
//     ];
//     let max_chars = lines.iter().map(|s| s.chars().count()).max().unwrap_or(1) as f64;
//     let info_pad = 10.0;
//     let info_line_h = 14.0;
//     let info_box_w = (max_chars * 7.0 + info_pad * 2.0).max(120.0);
//     let info_box_h = info_pad + info_line_h * (lines.len() as f64) + 2.0;
//     // let info_x = (w - 6.0 - info_box_w).clamp(pad_left, w - 6.0 - info_box_w);
//     let info_y = (y_top + 1.0).clamp(2.0, (h - 2.0 - info_box_h).max(2.0));

//     // Build multi-line <text> using <tspan> so it works reliably in SVG.
//     let mut tspans = String::new();
//     for (i, line) in lines.iter().enumerate() {
//         if i == 0 {
//             tspans.push_str(&format!(
//                 r#"<tspan x="{tx:.2}" dy="0" fill="{color}">{line}</tspan>"#,
//                 tx = x_center + info_box_w + 1.0, //w - 10.0,
//                 line = line,
//                 color = "#374151"
//             ));
//         } else {
//             tspans.push_str(&format!(
//                 r#"<tspan x="{tx:.2}" dy="{dy:.2}" fill="{color}">{line}</tspan>"#,
//                 tx = x_center + info_box_w + 1.0, //w - 10.0,
//                 dy = info_line_h,
//                 line = line,
//                 color = "#374151"
//             ));
//         }
//     }

//     format!(
//         r#"<g id="cursor-overlay">
//             <line x1="{x:.2}" y1="{y1:.2}" x2="{x:.2}" y2="{y2:.2}" stroke="{c_gray}" stroke-width="1" stroke-dasharray="4 3" />
//             <line x1="{x0:.2}" y1="{y:.2}" x2="{x1:.2}" y2="{y:.2}" stroke="{c_gray}" stroke-width="1" stroke-dasharray="4 3" />

//             <!-- Horizontal cursor price label (left of crossing) -->
//             <rect x="{clx:.2}" y="{cly:.2}" width="{clw:.2}" height="{clh:.2}" rx="3" ry="3" fill="{c_white}" opacity="0.62" stroke="{c_border}" />
//             <text x="{cltx:.2}" y="{clty:.2}" font-size="11" text-anchor="end" fill="{c_text}">{cross}</text>

//             <!-- Candle OHLC + date (right side) -->
//             <rect x="{ix:.2}" y="{iy:.2}" width="{iw:.2}" height="{ih:.2}" rx="4" ry="4" fill="{c_white}" opacity="0.72" stroke="{c_border}" />
//             <text x="{itx:.2}" y="{ity:.2}" font-size="12" text-anchor="end" fill="{c_text}" font-family="monospace">{tspans}</text>
//         </g>"#,
//         x = x_center,
//         y1 = y_top,
//         y2 = y_bottom,
//         x0 = pad_left,
//         x1 = pad_left + plot_w,
//         y = y,
//         clx = cross_label_x,
//         cly = cross_label_y,
//         clw = cross_label_w,
//         clh = cross_label_h,
//         cltx = cross_label_x + cross_label_w - 6.0,
//         clty = y + 4.0,
//         cross = cross_label,
//         ix = x_center + 6.0, //info_x,
//         iy = info_y,
//         iw = info_box_w,
//         ih = info_box_h,
//         itx = x_center + info_box_w - 10.0, //w - 10.0,
//         ity = info_y + info_pad + 2.0,
//         tspans = tspans,
//         c_gray = "#888",
//         c_white = "#fff",
//         c_border = "#aaa",
//         c_text = "#333",
//     )
// }

// /// Map a click/tap inside the view (in view pixels) onto the nearest candle index and a price.
// fn cursor_from_view_click(
//     candles: &[Candle],
//     svg_width: i32,
//     svg_height: i32,
//     full_render: bool,
//     x_view: f32,  // self.mouse-x from Slint TouchArea
//     y_view: f32,  // self.mouse-y from Slint TouchArea
//     view_w: f32,  // self.width of the Image element
//     view_h: f32,  // self.height of the Image element
//     scale_x: f32, // Your custom zoom factor
//     pan_x: f32,   // Your horizontal pan offset
// ) -> Option<Cursor> {
//     if candles.is_empty() || view_w < 1.0 || view_h < 1.0 {
//         return None;
//     }

//     // println!("{} {} {} {}", x_view, y_view, scale_x, pan_x);
//     // 1. Dimensions of the generated raster buffer
//     let s_x = (scale_x as f64).max(0.001);
//     let raster_w = svg_width as f64 * s_x;
//     // let raster_h = svg_height as f64;

//     // // 2. Slint 'image-fit: contain' calculation
//     // // Slint finds the scale factor that preserves aspect ratio while fitting the image.
//     // let fit_scale = view_w as f64 / raster_w; //(view_w as f64 / raster_w).min(view_h as f64 / raster_h);
//     // println!(
//     //     "fit_scale = {}, svg_width = {}, raster_w = {}",
//     //     fit_scale, svg_width, raster_w
//     // );
//     // // 3. Reverse Letterboxing and Pan
//     // // The pan_x is applied to the Image's position in view coordinates.
//     // // Subtract pan and letterbox offset BEFORE dividing by fit_scale.
//     // let letterbox_x = (view_w as f64 - (raster_w * fit_scale)) * 0.5;
//     // let letterbox_y = (view_h as f64 - (raster_h * fit_scale)) * 0.5;

//     // let x_raster = (x_view as f64 - letterbox_x - pan_x as f64) / fit_scale;
//     // let y_raster = (y_view as f64 - letterbox_y) / fit_scale;

//     // // 4. Reverse the SVG-to-Raster scale
//     // let x_svg = x_raster / s_x;
//     // let y_svg = y_raster;

//     // 5. Map to Chart Data (Must match your SVG padding exactly)
//     let (_pad_left, _pad_right, pad_top, plot_h) = if full_render {
//         // SVG dimensions are always the fixed base svg_width/height
//         let chart_h = (svg_height as f64 - (svg_height as f64 * 0.22).max(80.0)).max(200.0);
//         (0f64, 0f64, 20.0, (chart_h - 20.0 - 10.0).max(1.0))
//     } else {
//         (0f64, 0f64, 20.0, (svg_height as f64 - 40.0).max(1.0))
//     };

//     let plot_w = raster_w as f64; // - pad_left - pad_right;
//     let n = candles.len() as f64;
//     let step = plot_w / n;

//     let x_svg = (x_view - pan_x + svg_width as f32 * 0.5f32 * (scale_x - 1.0f32)) as f64;
//     let y_svg = y_view as f64; // / fit_scale;
//     // println!("x_svg = {}", x_svg);
//     // Use floor() to find which 'bin' the cursor is in
//     let mut idx_f = x_svg / step; // (x_svg - pad_left) / step;
//     if idx_f >= step / 2.0f64 {
//         idx_f += 1.0f64;
//     } else {
//         idx_f -= 1.0f64;
//     }
//     let idx = (idx_f.floor() as isize).clamp(0, (candles.len() - 1) as isize) as usize;

//     // Price Mapping
//     let min_p = candles.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
//     let max_p = candles
//         .iter()
//         .map(|c| c.high)
//         .fold(f64::NEG_INFINITY, f64::max);
//     let p_range = (max_p - min_p).max(1e-9);

//     let y_rel = (y_svg - pad_top).clamp(0.0, plot_h);
//     let price = max_p - (y_rel / plot_h) * p_range;

//     Some(Cursor { idx, price })
// }

// fn render_svg_candles_simple(candles: &[Candle], width: i32, height: i32) -> String {
//     let pad_left = 20.0;
//     let pad_right = 20.0;
//     let pad_top = 20.0;
//     let pad_bottom = 20.0;

//     let w = width as f64;
//     let h = height as f64;

//     let min_price = candles.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
//     let max_price = candles
//         .iter()
//         .map(|c| c.high)
//         .fold(f64::NEG_INFINITY, f64::max);

//     let price_range = (max_price - min_price).max(1e-9);

//     let plot_w = (w - pad_left - pad_right).max(1.0);
//     let plot_h = (h - pad_top - pad_bottom).max(1.0);

//     let n = candles.len().max(1) as f64;
//     let step = plot_w / n;
//     let body_w = (step * 0.65).max(1.0);

//     let y_of = |p: f64| -> f64 {
//         // higher price -> smaller y
//         pad_top + (max_price - p) / price_range * plot_h
//     };

//     let mut out = String::new();
//     out.push_str(&format!(
//         r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" fill="{color}">"#,
//         width = width,
//         height = height,
//         color = "#374151"
//     ));

//     // Background
//     out.push_str(r#"<rect x="0" y="0" width="100%" height="100%" fill="white"/>"#);
//     out.push_str(r#"<g id="viewport">"#);

//     for (i, c) in candles.iter().enumerate() {
//         let x_center = pad_left + (i as f64 + 0.5) * step;
//         let x0 = x_center - body_w / 2.0;

//         let y_high = y_of(c.high);
//         let y_low = y_of(c.low);
//         let y_open = y_of(c.open);
//         let y_close = y_of(c.close);

//         let bullish = c.close >= c.open;
//         let body_top = y_open.min(y_close);
//         let body_bot = y_open.max(y_close);
//         let body_h = (body_bot - body_top).max(1.0);

//         let fill = if bullish { "#2f9e44" } else { "#e03131" };

//         // Wick
//         out.push_str(&format!(
//             r#"<line x1="{xc:.2}" y1="{yh:.2}" x2="{xc:.2}" y2="{yl:.2}" stroke="{color}" stroke-width="1"/>"#,
//             xc = x_center, yh = y_high, yl = y_low, color = "#333"
//         ));

//         // Body
//         out.push_str(&format!(
//             r#"<rect x="{x0:.2}" y="{y:.2}" width="{w:.2}" height="{h:.2}" fill="{f}"/>"#,
//             x0 = x0,
//             y = body_top,
//             w = body_w,
//             h = body_h,
//             f = fill
//         ));

//         // // X-Axis Labels (Every 10th candle)
//         // if i % 10 == 0 {
//         //     out.push_str(&format!(
//         //         r#"<text x="{xc:.2}" y="{ty:.2}" font-size="11" text-anchor="middle" fill="{color}">{label}</text>"#,
//         //         xc = x_center, ty = h - 5.0, color = "#666", label = c.label
//         //     ));
//         // }
//     }

//     out.push_str(r#"</g>"#);
//     out.push_str("</svg>");
//     // println!("{:?}", out);
//     out
// }

// /// Render a fuller candlestick chart into an SVG string.
// ///
// /// Adds:
// /// - Price grid lines + Y labels
// /// - Volume bars
// /// - Sparse X labels
// fn render_svg_candles_full(candles: &[Candle], width: i32, height: i32) -> String {
//     let pad_left = 60.0;
//     let pad_right = 20.0;
//     let pad_top = 20.0;
//     let pad_bottom = 30.0;

//     let w = width as f64;
//     let h = height as f64;

//     let vol_h = (h * 0.22).max(80.0);
//     let chart_h = (h - vol_h).max(200.0);

//     let min_price = candles.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
//     let max_price = candles
//         .iter()
//         .map(|c| c.high)
//         .fold(f64::NEG_INFINITY, f64::max);

//     let price_range = (max_price - min_price).max(1e-9);

//     let max_vol = candles
//         .iter()
//         .map(|c| c.volume)
//         .fold(0.0_f64, f64::max)
//         .max(1e-9);

//     let plot_w = (w - pad_left - pad_right).max(1.0);
//     let plot_h = (chart_h - pad_top - 10.0).max(1.0);

//     let n = candles.len().max(1) as f64;
//     let step = plot_w / n;
//     let body_w = (step * 0.65).max(1.0);

//     let y_of = |p: f64| -> f64 { pad_top + (max_price - p) / price_range * plot_h };

//     let mut out = String::new();
//     out.push_str(&format!(
//         r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">"#
//     ));

//     // Background
//     out.push_str(r#"<rect x="0" y="0" width="100%" height="100%" fill="white"/>"#);
//     out.push_str(r#"<g id="viewport">"#);

//     // Price grid
//     let grid_lines = 5;
//     let price_prec = auto_price_precision(min_price, max_price, (grid_lines as usize) + 1);
//     for i in 0..=grid_lines {
//         let t = i as f64 / grid_lines as f64;
//         let price = max_price - t * price_range;
//         let y = y_of(price);

//         // 1. First Line
//         out.push_str(&format!(
//             r#"<line x1="{x1:.2}" y1="{y:.2}" x2="{x2:.2}" y2="{y:.2}" stroke="{color}" stroke-width="1"/>"#,
//             x1 = pad_left,
//             x2 = w - pad_right,
//             y = y,
//             color = "#eee"
//         ));
//         // 2. Second Line
//         let price_label = fmt_price(price, price_prec);
//         out.push_str(&format!(
//             r#"<text x="{tx:.2}" y="{ty:.2}" font-size="11" text-anchor="end" fill="{color}">{label}</text>"#,
//             tx = pad_left - 8.0,
//             ty = y + 4.0,
//             label = price_label,
//             color = "#666"
//         ));
//     }

//     // Candles
//     for (i, c) in candles.iter().enumerate() {
//         let x_center = pad_left + (i as f64 + 0.5) * step;
//         let x0 = x_center - body_w / 2.0;

//         let y_high = y_of(c.high);
//         let y_low = y_of(c.low);
//         let y_open = y_of(c.open);
//         let y_close = y_of(c.close);

//         let bullish = c.close >= c.open;
//         let body_top = y_open.min(y_close);
//         let body_bot = y_open.max(y_close);
//         let body_h = (body_bot - body_top).max(1.0);

//         let fill = if bullish { "#2f9e44" } else { "#e03131" };

//         // Wick
//         out.push_str(&format!(
//             r#"<line x1="{x_center:.2}" y1="{y_high:.2}" x2="{x_center:.2}" y2="{y_low:.2}" stroke="{color}" stroke-width="1"/>"#,
//             x_center = x_center,
//             y_high = y_high,
//             y_low = y_low,
//             color = "#333"
//         ));

//         // Body
//         out.push_str(&format!(
//             r#"<rect x="{x0:.2}" y="{body_top:.2}" width="{body_w:.2}" height="{body_h:.2}" fill="{fill}"/>"#
//         ));
//     }

//     // Volume bars
//     let vol_top = chart_h;
//     let vol_base = h - pad_bottom;
//     for (i, c) in candles.iter().enumerate() {
//         let x_center = pad_left + (i as f64 + 0.5) * step;
//         let x0 = x_center - body_w / 2.0;

//         let vh = (c.volume / max_vol) * (vol_base - vol_top);
//         let y = vol_base - vh;

//         // let bullish = c.close >= c.open;
//         let fill = "#d1d5db"; //if bullish { "#2f9e44" } else { "#e03131" };

//         out.push_str(&format!(
//             r#"<rect x="{x0:.2}" y="{y:.2}" width="{body_w:.2}" height="{vh:.2}" fill="{fill}" opacity="0.35"/>"#
//         ));
//     }

//     // Sparse X labels
//     let label_step = (candles.len() / 6).max(1);
//     for i in (0..candles.len()).step_by(label_step) {
//         let x_center = pad_left + (i as f64 + 0.5) * step;
//         let text_y = h - 10.0;
//         out.push_str(&format!(
//             r#"<text x="{x:.2}" y="{y:.2}" font-size="11" text-anchor="middle" fill="{color}">{label}</text>"#,
//             x = x_center,
//             y = text_y,
//             color = "#666",
//             label = candles[i].label
//         ));
//     }

//     out.push_str(r#"</g>"#);
//     out.push_str("</svg>");
//     out
// }

// // /// Parse a `u16` from a Slint `string` field, with a nice error message.
// // ///
// // /// This keeps UI parsing code tidy and ensures all numeric input errors look consistent.
// // fn parse_u16(field_name: &str, text: slint::SharedString) -> Result<u16, String> {
// //     text.trim()
// //         .parse::<u16>()
// //         .map_err(|_| format!("{field_name} must be a u16 (got '{text}')"))
// // }

// fn main() -> Result<(), slint::PlatformError> {
//     let ui = AppWindow::new()?;

//     // Channels to be used by worker thread
//     let (tx_req, rx_req) = mpsc::channel::<FetchRequest>();
//     let (tx_res, rx_res) = mpsc::channel::<FetchResult>();

//     // Store current scale, position, and SVG content (so zoom/pan can re-render consistently).
//     let scale = Rc::new(RefCell::new(1.0f32));
//     let position_x = Rc::new(RefCell::new(0.0f32));
//     let current_svg_content = Rc::new(RefCell::new(String::new()));
//     let current_candles: Rc<RefCell<Vec<Candle>>> = Rc::new(RefCell::new(Vec::new()));
//     let cursor: Rc<RefCell<Option<Cursor>>> = Rc::new(RefCell::new(None));
//     let last_full_render = Rc::new(RefCell::new(false));

//     // Set initial scale display.
//     ui.set_current_scale(format!("Scale: {:.1}x", *scale.borrow()).into());
//     ui.set_image_x_offset((*position_x.borrow() as f32) * 1.0);
//     // Reload / Refresh: fetch KLine -> render SVG -> rasterize -> show
//     {
//         let ui_weak = ui.as_weak();
//         // let scale = scale.clone();
//         // let current_svg_content = current_svg_content.clone();
//         // let current_candles = current_candles.clone();
//         // let cursor = cursor.clone();
//         // let last_full_render = last_full_render.clone();

//         ui.on_reload_kline(move || {
//             let Some(ui) = ui_weak.upgrade() else {
//                 return;
//             };
//             ui.set_svg_status("Loading…".into());

//             // Parse inputs
//             let market = match parse_u16("market", ui.get_market_input()) {
//                 Ok(v) => v,
//                 Err(e) => {
//                     ui.set_svg_status(format!("Error: {e}").into());
//                     return;
//                 }
//             };

//             // let code = ui.get_code_input().to_string();
//             // if code.len() != 6 {
//             //     ui.set_svg_status("Error: code must be 6 characters (e.g. 000001)".into());
//             //     return;
//             // }
//             let code = match validate_code(&ui.get_code_input()) {
//                 Ok(v) => v,
//                 Err(e) => {
//                     ui.set_svg_status(format!("Error: {e}").into());
//                     return;
//                 }
//             };

//             let category = match parse_u16("category", ui.get_category_input()) {
//                 Ok(v) => v,
//                 Err(e) => {
//                     ui.set_svg_status(format!("Error: {e}").into());
//                     return;
//                 }
//             };

//             let start = match parse_u16("start", ui.get_start_input()) {
//                 Ok(v) => v,
//                 Err(e) => {
//                     ui.set_svg_status(format!("Error: {e}").into());
//                     return;
//                 }
//             };

//             let count = match parse_u16("count", ui.get_count_input()) {
//                 Ok(v) => v,
//                 Err(e) => {
//                     ui.set_svg_status(format!("Error: {e}").into());
//                     return;
//                 }
//             };

//             // Enforce protocol limit (tcptdx typically caps at 800)
//             let count = count.min(800);

//             // // Fetch
//             // let candles = match fetch_kline_as_candles(market, &code, category, start, count) {
//             //     Ok(c) => c,
//             //     Err(e) => {
//             //         ui.set_svg_status(format!("Error: {e}").into());
//             //         return;
//             //     }
//             // };
//             let _ = tx_req.send(FetchRequest {
//                 market,
//                 code,
//                 category,
//                 start,
//                 count,
//             });

//             // // Render SVG
//             // let full_render = ui.get_full_render();
//             // *last_full_render.borrow_mut() = full_render;
//             // *current_candles.borrow_mut() = candles.clone();
//             // // Reset cursor on reload.
//             // *cursor.borrow_mut() = None;

//             // let svg_w = 960;
//             // let svg_h = if full_render { 520 } else { 420 };
//             // let svg = render_svg_with_cursor(&candles, svg_w, svg_h, full_render, *cursor.borrow());

//             // // Store the SVG so zoom can re-render at the new scale
//             // *current_svg_content.borrow_mut() = svg;

//             // // Rasterize at current scale and show
//             // let current_scale = *scale.borrow();
//             // match svg_to_image_with_scale(&current_svg_content.borrow(), current_scale) {
//             //     Ok(image) => {
//             //         ui.set_svg_image(image);
//             //         ui.set_svg_status(format!("Loaded {} candles", candles.len()).into());
//             //     }
//             //     Err(e) => {
//             //         ui.set_svg_status(format!("Error: {e}").into());
//             //     }
//             // }
//         });
//     }

//     // Scale Up
//     {
//         let ui_weak = ui.as_weak();
//         let scale = scale.clone();
//         let current_svg_content = current_svg_content.clone();
//         ui.on_scale_up(move || {
//             let mut current_scale = scale.borrow_mut();
//             *current_scale = (*current_scale * 1.2).min(5.0);

//             if let Some(ui) = ui_weak.upgrade() {
//                 if current_svg_content.borrow().is_empty() {
//                     ui.set_svg_status("No chart loaded".into());
//                     ui.set_current_scale(format!("Scale: {:.1}x", *current_scale).into());
//                     return;
//                 }

//                 match svg_to_image_with_scale(&current_svg_content.borrow(), *current_scale) {
//                     Ok(image) => {
//                         ui.set_svg_image(image);
//                         ui.set_current_scale(format!("Scale: {:.1}x", *current_scale).into());
//                     }
//                     Err(e) => ui.set_svg_status(format!("Error: {e}").into()),
//                 }
//             }
//         });
//     }

//     // Scale Down
//     {
//         let ui_weak = ui.as_weak();
//         let scale = scale.clone();
//         let current_svg_content = current_svg_content.clone();
//         ui.on_scale_down(move || {
//             let mut current_scale = scale.borrow_mut();
//             *current_scale = (*current_scale / 1.2).max(0.2);

//             if let Some(ui) = ui_weak.upgrade() {
//                 if current_svg_content.borrow().is_empty() {
//                     ui.set_svg_status("No chart loaded".into());
//                     ui.set_current_scale(format!("Scale: {:.1}x", *current_scale).into());
//                     return;
//                 }

//                 match svg_to_image_with_scale(&current_svg_content.borrow(), *current_scale) {
//                     Ok(image) => {
//                         ui.set_svg_image(image);
//                         ui.set_current_scale(format!("Scale: {:.1}x", *current_scale).into());
//                     }
//                     Err(e) => ui.set_svg_status(format!("Error: {e}").into()),
//                 }
//             }
//         });
//     }

//     // Reset View
//     {
//         let ui_weak = ui.as_weak();
//         let scale = scale.clone();
//         let position_x = position_x.clone();
//         let current_svg_content = current_svg_content.clone();
//         ui.on_reset_scale(move || {
//             let mut current_scale = scale.borrow_mut();
//             let mut current_pos_x = position_x.borrow_mut();
//             *current_scale = 1.0;
//             *current_pos_x = 0.0;

//             if let Some(ui) = ui_weak.upgrade() {
//                 ui.set_current_scale(format!("Scale: {:.1}x", *current_scale).into());
//                 ui.set_target_x_offset(*current_pos_x);

//                 if current_svg_content.borrow().is_empty() {
//                     ui.set_svg_status("No chart loaded".into());
//                     return;
//                 }

//                 match svg_to_image_with_scale(&current_svg_content.borrow(), *current_scale) {
//                     Ok(image) => ui.set_svg_image(image),
//                     Err(e) => ui.set_svg_status(format!("Error: {e}").into()),
//                 }
//             }
//         });
//     }

//     // Swipe Left
//     {
//         let ui_weak = ui.as_weak();
//         let position_x = position_x.clone();
//         ui.on_swipe_left(move || {
//             let mut current_pos_x = position_x.borrow_mut();
//             *current_pos_x -= 100.0;
//             if let Some(ui) = ui_weak.upgrade() {
//                 ui.set_target_x_offset(*current_pos_x);
//             }
//         });
//     }

//     // Swipe Right
//     {
//         let ui_weak = ui.as_weak();
//         let position_x = position_x.clone();
//         ui.on_swipe_right(move || {
//             let mut current_pos_x = position_x.borrow_mut();
//             *current_pos_x += 100.0;
//             if let Some(ui) = ui_weak.upgrade() {
//                 ui.set_target_x_offset(*current_pos_x);
//             }
//         });
//     }

//     // Initialize scale display
//     ui.set_current_scale("Scale: 1.0x".into());

//     // Click/Tap on kline view: position a cursor (snap to nearest candle) and show price label on Y axis.
//     {
//         let ui_weak = ui.as_weak();
//         let current_svg_content = current_svg_content.clone();
//         let scale = scale.clone();
//         let position_x = position_x.clone();
//         let current_candles = current_candles.clone();
//         let cursor = cursor.clone();
//         let last_full_render = last_full_render.clone();

//         ui.on_kline_click(move |x, y, view_w, view_h| {
//             let Some(ui) = ui_weak.upgrade() else {
//                 return;
//             };

//             let candles = current_candles.borrow();
//             if candles.is_empty() {
//                 return;
//             }

//             let full_render = *last_full_render.borrow();
//             let svg_w = 960;
//             let svg_h = if full_render { 520 } else { 420 };

//             let Some(cur) = cursor_from_view_click(
//                 &candles,
//                 svg_w,
//                 svg_h,
//                 full_render,
//                 x,
//                 y,
//                 view_w,
//                 view_h,
//                 (*scale.borrow()).max(0.1),
//                 ui.get_target_x_offset(),
//             ) else {
//                 return;
//             };

//             *cursor.borrow_mut() = Some(cur);

//             let svg = render_svg_with_cursor(&candles, svg_w, svg_h, full_render, Some(cur));
//             *current_svg_content.borrow_mut() = svg.clone();

//             match render_svg_to_image(&svg, (*scale.borrow() * 1.0).max(0.1), *position_x.borrow())
//             // match svg_to_image_with_scale(&svg, (*scale.borrow()).max(0.1))
//             {
//                 Ok(img) => {
//                     ui.set_svg_image(img);
//                     ui.set_svg_status(
//                         format!(
//                             "Cursor: idx={} price={}",
//                             cur.idx,
//                             fmt_price(
//                                 cur.price,
//                                 auto_price_precision(
//                                     candles.iter().map(|c| c.low).fold(f64::INFINITY, f64::min),
//                                     candles
//                                         .iter()
//                                         .map(|c| c.high)
//                                         .fold(f64::NEG_INFINITY, f64::max),
//                                     6
//                                 )
//                             )
//                         )
//                         .into(),
//                     );
//                 }
//                 Err(e) => ui.set_svg_status(format!("Error: {e}").into()),
//             }
//         });
//     }

//     // Arrow keys: move cursor left/right by one candle (step-based).
//     {
//         let ui_weak = ui.as_weak();
//         let current_svg_content = current_svg_content.clone();
//         let scale = scale.clone();
//         let position_x = position_x.clone();
//         let current_candles = current_candles.clone();
//         let cursor = cursor.clone();
//         let last_full_render = last_full_render.clone();

//         ui.on_move_cursor_by(move |delta| {
//             // println!("delta = {}", delta);
//             let Some(ui) = ui_weak.upgrade() else {
//                 return;
//             };

//             let candles = current_candles.borrow();
//             if candles.is_empty() {
//                 return;
//             }

//             let full_render = *last_full_render.borrow();
//             let svg_w = 960;
//             let svg_h = if full_render { 520 } else { 420 };

//             // Initialize cursor if user hasn't clicked yet.
//             let mut cur = match *cursor.borrow() {
//                 Some(c) => c,
//                 None => Cursor {
//                     idx: 0,
//                     price: candles[0].close,
//                 },
//             };

//             // Apply delta and clamp.
//             let len = candles.len();
//             let next = (cur.idx as i64) + (delta as i64);
//             let next = next.clamp(0, (len.saturating_sub(1)) as i64) as usize;
//             cur.idx = next;

//             *cursor.borrow_mut() = Some(cur);

//             let svg = render_svg_with_cursor(&candles, svg_w, svg_h, full_render, Some(cur));
//             *current_svg_content.borrow_mut() = svg.clone();

//             match render_svg_to_image(&svg, (*scale.borrow() * 1.0).max(0.1), *position_x.borrow())
//             {
//                 Ok(img) => {
//                     ui.set_svg_image(img);
//                     ui.set_svg_status(
//                         format!(
//                             "Cursor: idx={} price={}",
//                             cur.idx,
//                             fmt_price(
//                                 cur.price,
//                                 auto_price_precision(
//                                     candles.iter().map(|c| c.low).fold(f64::INFINITY, f64::min),
//                                     candles
//                                         .iter()
//                                         .map(|c| c.high)
//                                         .fold(f64::NEG_INFINITY, f64::max),
//                                     6
//                                 )
//                             )
//                         )
//                         .into(),
//                     );
//                 }
//                 Err(e) => ui.set_svg_status(format!("Error: {e}").into()),
//             }
//         });
//     }

//     // Deliver results back to UI thread
//     // Slint must be updated on the UI thread.
//     // Use a timer or event loop hook:

//     let ui_weak = ui.as_weak();

//     // IMPORTANT: keep this variable alive
//     let res_timer = slint::Timer::default();
//     res_timer.start(
//         slint::TimerMode::Repeated,
//         std::time::Duration::from_millis(50),
//         move || {
//             if let Ok(result) = rx_res.try_recv() {
//                 // println!("{:?}", result);
//                 if let Some(ui) = ui_weak.upgrade() {
//                     match result {
//                         FetchResult::Ok(candles) => {
//                             let scale = scale.clone();
//                             let current_svg_content = current_svg_content.clone();
//                             let current_candles = current_candles.clone();
//                             let cursor = cursor.clone();
//                             let last_full_render = last_full_render.clone();
//                             // ui.set_svg_content(svg.into());
//                             //
//                             // Render SVG
//                             let full_render = ui.get_full_render();
//                             *last_full_render.borrow_mut() = full_render;
//                             *current_candles.borrow_mut() = candles.clone();
//                             // Reset cursor on reload.
//                             *cursor.borrow_mut() = None;

//                             let svg_w = 960;
//                             let svg_h = if full_render { 520 } else { 420 };
//                             let svg = render_svg_with_cursor(
//                                 &candles,
//                                 svg_w,
//                                 svg_h,
//                                 full_render,
//                                 *cursor.borrow(),
//                             );

//                             // Store the SVG so zoom can re-render at the new scale
//                             *current_svg_content.borrow_mut() = svg;

//                             // Rasterize at current scale and show
//                             let current_scale = *scale.borrow();
//                             match svg_to_image_with_scale(
//                                 &current_svg_content.borrow(),
//                                 current_scale,
//                             ) {
//                                 Ok(image) => {
//                                     ui.set_svg_image(image);
//                                     ui.set_svg_status(
//                                         format!("Loaded {} candles", candles.len()).into(),
//                                     );
//                                 }
//                                 Err(e) => {
//                                     ui.set_svg_status(format!("Error: {e}").into());
//                                 }
//                             }
//                             ui.set_svg_status("OK".into());
//                         }
//                         FetchResult::Err(e) => {
//                             ui.set_svg_status(format!("Error: {e}").into());
//                         }
//                     }
//                 }
//             }
//         },
//     );

//     // Spawn background worker thread
//     thread::spawn(move || {
//         while let Ok(req) = rx_req.recv() {
//             let result =
//                 fetch_kline_as_candles(req.market, &req.code, req.category, req.start, req.count);

//             match result {
//                 Ok(candles) => {
//                     // println!("{:?}", candles);
//                     let _ = tx_res.send(FetchResult::Ok(candles));
//                 }
//                 Err(e) => {
//                     let _ = tx_res.send(FetchResult::Err(e));
//                 }
//             }
//         }
//     });

//     ui.run()
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     // Verify that Candle is Send
//     #[test]
//     fn candle_is_send() {
//         fn assert_send<T: Send>() {}
//         assert_send::<Candle>();
//     }

//     // Verify that Vec<Candle> is Send
//     #[test]
//     fn vec_candle_is_send() {
//         fn assert_send<T: Send>() {}
//         assert_send::<Vec<Candle>>();
//     }

//     #[test]
//     fn validate_code_accepts_valid_code() {
//         let code = validate_code("000001").unwrap();
//         assert_eq!(code, "000001");
//     }

//     #[test]
//     fn validate_code_trims_whitespace() {
//         let code = validate_code("  000001 \n").unwrap();
//         assert_eq!(code, "000001");
//     }

//     #[test]
//     fn validate_code_rejects_wrong_length() {
//         let err = validate_code("12345").unwrap_err();
//         assert!(err.to_string().contains("exactly 6"));
//     }

//     #[test]
//     fn validate_code_rejects_non_digits() {
//         let err = validate_code("12A001").unwrap_err();
//         assert!(err.to_string().contains("digits"));
//     }

//     #[test]
//     fn validate_code_rejects_empty_string() {
//         let err = validate_code("").unwrap_err();
//         assert!(err.to_string().contains("exactly 6"));
//     }
// }
