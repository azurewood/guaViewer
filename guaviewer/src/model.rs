use std::fmt;

/// A minimal candle model used by the SVG renderer.
#[derive(Debug, Clone)]
pub struct Candle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    /// Render-friendly date label (already formatted).
    pub label: String,
}

#[derive(Clone, Copy, Debug)]
pub struct Cursor {
    /// Nearest candle index (snapped on click).
    pub idx: usize,
    /// Cursor price at the click Y position (mapped from screen Y to price).
    pub price: f64,
}

#[derive(Debug)]
pub struct FetchRequest {
    pub market: u16,
    pub code: String,
    pub category: u16,
    pub start: u16,
    pub count: u16,
}

#[derive(Debug)]
pub enum FetchResult {
    // Ok(String),  // SVG
    Ok(Vec<Candle>),
    Err(String), // error message
}

/// Application-level errors with structured variants.
///
/// This keeps error handling explicit and avoids passing around ad-hoc `String`s.
/// In UI code we format these errors via `Display` and show them in `svg-status`.
#[derive(Debug)]
pub enum AppError {
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

    /// Invalid stock code input.
    InvalidInput { message: String },
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
            AppError::InvalidInput { message } => write!(f, "Invalid stock code: {message}"),
        }
    }
}

impl std::error::Error for AppError {}
