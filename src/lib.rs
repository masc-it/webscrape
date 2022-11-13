mod scraper;
mod pipeline;
mod utils;
mod html_only;

pub mod proxy;
pub use crate::scraper::{ScraperBuilder};

pub use crate::pipeline::ScrapingPipeline;

pub use crate::utils::{img_to_base64, save_screenshot};

pub use crate::html_only::simple::SimpleScraper;