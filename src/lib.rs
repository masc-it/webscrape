mod scraping;
mod pipeline;
mod utils;

pub mod proxy;
pub use crate::scraping::chrome::{ScraperBuilder};

pub use crate::pipeline::ScrapingPipeline;

pub use crate::utils::{img_to_base64, save_screenshot};

pub use crate::scraping::simple::SimpleScraper;