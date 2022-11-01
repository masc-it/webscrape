#![allow(unused_must_use)]

use std::fmt::Display;

use serde::{Serialize, Deserialize};

use crate::scraper::Scraper;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Target {

    pub name: String,
    pub selector: String
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ActionClick {

    pub selector: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionWait {

    pub duration: u32
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActionData {
    ActionClick(ActionClick),
    ActionWait(ActionWait)
    // Other possible response types here...
}

#[derive(Serialize, Deserialize)]
pub struct Action {
    pub name: String,
    pub class: String,
    #[serde(flatten)]
    pub data: ActionData
}

#[derive(Serialize, Deserialize)]
pub struct Pipeline {
    pub name: String,
    pub url: String
}

#[derive(Serialize, Deserialize)]
pub struct PipelineConfig {

    #[serde(flatten)]
    pub pipeline: Pipeline,

    pub targets: Vec<Target>,
    pub actions: Vec<Action>
}

pub struct ScrapingPipeline {

    pipeline_config: PipelineConfig,
    scraper: Scraper

}

impl ScrapingPipeline {
    
    pub fn from_file(config_source: &str, scraper: Scraper) -> ScrapingPipeline {

        let f = std::fs::File::open(config_source).unwrap();
        let pipeline_config: PipelineConfig = serde_yaml::from_reader(f).unwrap();

        ScrapingPipeline { pipeline_config, scraper }
    }

}

impl Display for ScrapingPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        let pipeline = &self.pipeline_config;

        writeln!(f, "TARGETS");
        for target in &pipeline.targets {

            writeln!(f, "{} - {}", target.name, target.selector);
        }

        writeln!(f, "\nACTIONS");
        for action in &pipeline.actions {

            writeln!(f, "{} - {}", action.name, action.class);
        }

        Ok(())
    }
}