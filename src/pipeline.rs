#![allow(unused_must_use)]

use std::{fmt::Display, collections::HashMap};

use serde::{Serialize, Deserialize};
use tabled::{Tabled, Panel, Modify, object::{Rows}, Alignment, style::HorizontalLine, Style};

use crate::scraper::{Scraper, DOMElement, ScrapingResult};

#[derive(Clone, Debug, Serialize, Deserialize, Tabled)]
#[tabled(rename_all = "UPPERCASE")]
pub struct Target {

    #[serde(skip)]
    pub name: String,
    pub selector: String
}


impl Display for ActionData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        if let ActionData::ActionClick(selector) = self {
            write!(f, "{}", selector.selector);
        } else if let ActionData::ActionWait(selector) = self {
            write!(f, "{}", selector.duration);
        } else if let ActionData::ActionType(selector) = self {
            write!(f, "{}", selector.text);
        } else if let ActionData::ActionScreenshot(selector) = self {
            write!(f, "{}", selector.target);
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionClick {

    pub selector: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionWait {

    pub duration: u32
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionType {

    pub target: String,
    pub text: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionScreenshot {

    pub target: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActionData {
    ActionClick(ActionClick),
    ActionScreenshot(ActionScreenshot),
    ActionWait(ActionWait),
    ActionType(ActionType),
    // Other possible response types here...
}

#[derive(Clone, Serialize, Deserialize, Tabled)]
#[tabled(rename_all = "UPPERCASE")]
pub struct Action {

    #[serde(skip)]
    pub name: String,
    pub class: String,
    #[serde(flatten)]
    pub data: ActionData
}

pub trait PipelineObject {
    
    fn add(&mut self, scraper: &mut Scraper);
}

impl PipelineObject for Target {

    fn add(&mut self, scraper: &mut Scraper) {
        
        let n = self.name.clone();
        let s = self.selector.clone();

        if s.starts_with("/") {
            scraper.find_elements_by_xpath(n, s);
        } else {
            scraper.find_elements_by_css(n, s);
  
        }
        
    }
}

impl PipelineObject for Action {

    fn add(&mut self, scraper: &mut Scraper) {
        
        let n = self.name.clone();
        
        if let ActionData::ActionClick(a) = &self.data {
            scraper.click(n, a.selector.to_string());
        } else if let ActionData::ActionWait(a) = &self.data {
            scraper.sleep(a.duration as u64);
        } else if let ActionData::ActionType(a) = &self.data {
            scraper.type_into(n, a.target.to_string(), a.text.to_string());
        } else if let ActionData::ActionScreenshot(a) = &self.data {
            scraper.screenshot(n, a.target.to_string());
        }
        
    }
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

    pub targets: HashMap<String, Target>,
    pub actions: HashMap<String, Action>,

    pub steps: Vec<String>
}

pub struct ScrapingPipeline {

    pipeline_config: PipelineConfig,
    scraper: Scraper

}

impl ScrapingPipeline {
    
    pub fn from_file(config_source: &str, scraper: Scraper) -> ScrapingPipeline {

        let f = std::fs::File::open(config_source).unwrap();
        let mut pipeline_config: PipelineConfig = serde_yaml::from_reader(f).unwrap();

        for (k, t) in &mut pipeline_config.targets {
            t.name = k.to_string();
        }

        for (k, t) in &mut pipeline_config.actions {
            t.name = k.to_string();
        }

        ScrapingPipeline { pipeline_config, scraper }
    }

    pub fn get_steps(&self) -> Vec<String> {

        self.pipeline_config.steps.clone()
    }

    pub fn run(&mut self) -> ScrapingResult {


        self.scraper.navigate_to(self.pipeline_config.pipeline.url.to_string());

        let targets = &self.pipeline_config.targets;

        let actions = &self.pipeline_config.actions;

        for step in &self.pipeline_config.steps {
            
            println!("{}", step);
            
            if targets.contains_key(step) {
                let t = &mut (targets.get(step).unwrap().clone());
                t.add(&mut self.scraper);
            } else if actions.contains_key(step) {

                let a = &mut (actions.get(step).unwrap().clone());
                a.add(&mut self.scraper)
                
            } else {
                println!("{} not implemented.", step);
            }
            
        }

        self.scraper.collect()
    }

}

impl Display for ScrapingPipeline {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        let pipeline = &self.pipeline_config;

        //let separator = str::repeat("-", (80 - 4));

        let mut targets_data: Vec<&Target> = Vec::default();

        for (_, target) in &pipeline.targets {

            targets_data.push(target);
        }

        let mut actions_data: Vec<&Action> = Vec::default();

        for (_, target) in &pipeline.actions {

            actions_data.push(target);
        }

        let mut table_targets = tabled::Table::new(targets_data);
        
        table_targets.with(Panel::header("TARGETS"));
        table_targets.with(tabled::Style::sharp().horizontals(
            [HorizontalLine::new(1, Style::modern().get_horizontal())
                    .main(Some('═'))
                    .intersection(None),
                    HorizontalLine::new(2, Style::modern().get_horizontal())
                    .main(Some('═'))
                    .intersection(None)
                ]
        ));
        table_targets.with(Modify::new(Rows::new(0..=1)).with(Alignment::center()));
        table_targets.with(tabled::Style::correct_spans());

        let mut table_actions = tabled::Table::new(actions_data);
        
        table_actions.with(Panel::header("ACTIONS"));
        table_actions.with(tabled::Style::sharp().horizontals(
            [HorizontalLine::new(1, Style::modern().get_horizontal())
                    .main(Some('═'))
                    .intersection(None),
                    HorizontalLine::new(2, Style::modern().get_horizontal())
                    .main(Some('═'))
                    .intersection(None)
                ]
        ));
        table_actions.with(Modify::new(Rows::new(0..=1)).with(Alignment::center()));
        table_actions.with(tabled::Style::correct_spans());

        writeln!(f, "{}", table_targets);
        writeln!(f, "{}", table_actions);

        
        Ok(())
    }
}