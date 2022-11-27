#![allow(unused_must_use)]

use std::{fmt::Display, collections::HashMap};

use rayon::prelude::IntoParallelRefIterator;
use serde::{Serialize, Deserialize};
use tabled::{Tabled, Panel, Modify, object::{Rows}, Alignment, style::HorizontalLine, Style, Disable};
use rayon::prelude::*;

use crate::{scraping::chrome::{Scraper, ScrapingResult, ScreenshotFormat}, ScraperBuilder};

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
        } else if let ActionData::ActionTypeInto(selector) = self {
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
pub struct ActionTypeInto {

    pub target: String,
    pub text: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionScreenshot {

    pub target: String,
    pub format: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionSave {

    pub targets: Vec<String>,
    pub flatten: bool
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActionData {
    ActionClick(ActionClick),
    ActionScreenshot(ActionScreenshot),
    ActionWait(ActionWait),
    ActionTypeInto(ActionTypeInto),
    ActionSave(ActionSave)
    // Other possible response types here...
}

#[derive(Clone, Serialize, Deserialize, Tabled)]
#[tabled(rename_all = "UPPERCASE")]
pub struct Action {

    #[serde(skip)]
    pub name: String,
    #[serde(flatten)]
    pub data: ActionData
}

#[derive(Serialize, Deserialize)]
pub struct Pipeline {

    pub name: String,
    pub url: String,
    pub headless: bool,
    pub out_dir: String,
    pub timeout: u64
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

    pub pipeline_config: PipelineConfig,
    scraper: Scraper

}

impl ScrapingPipeline {
    
    pub fn from_file(config_source: &str) -> ScrapingPipeline {

        let f = std::fs::File::open(config_source).unwrap();
        let mut pipeline_config: PipelineConfig = serde_yaml::from_reader(f).unwrap();

        for (k, t) in &mut pipeline_config.targets {
            t.name = k.to_string();
        }

        for (k, t) in &mut pipeline_config.actions {
            t.name = k.to_string();
        }

        let mut builder = ScraperBuilder::default();
        let scraper = builder
            .set_headless(pipeline_config.pipeline.headless)
            .set_default_timeout(pipeline_config.pipeline.timeout)
            .set_save_dir(pipeline_config.pipeline.out_dir.clone())
            .build();

        ScrapingPipeline { pipeline_config, scraper:scraper }
    }

    pub fn get_steps(&self) -> Vec<String> {

        self.pipeline_config.steps.clone()
    }

    pub fn run(&mut self, url: &String) -> ScrapingResult {


        if url == "DEFAULT" {
            self.scraper.navigate_to(self.pipeline_config.pipeline.url.to_string());
        } else {
            self.scraper.navigate_to(url);
        }

        let targets = &mut self.pipeline_config.targets;

        let actions = &mut self.pipeline_config.actions;

        for step in &mut self.pipeline_config.steps {
            
            let mut step_name = step.to_string();

            if step.contains(" from ") {

                let (new_name, from_name) = step.split_once(" from ").unwrap();

                if targets.contains_key(from_name) {

                    let mut new_t = targets.get(from_name).unwrap().clone();
                    new_t.name = new_name.to_string();

                    step_name = new_t.name.clone();
                    targets.insert(step_name.clone(), new_t);
                } else if actions.contains_key(from_name) {
                    let mut new_t = actions.get(from_name).unwrap().clone();
                    new_t.name = new_name.to_string();

                    step_name = new_t.name.clone();
                    actions.insert(step_name.clone(), new_t);
                } else {
                    println!("Invalid alias: {}", step_name);
                    continue;
                }

                *step = new_name.to_string();
            }
            
        }

        let targets = &self.pipeline_config.targets.clone();
        let actions = &self.pipeline_config.actions.clone();

        for step in self.get_steps() {
            
            let step_name = step.to_string();

            if targets.contains_key(&step_name) {
                let t = &(targets.get(&step_name).unwrap().clone());
                self.register_target(t);

            } else if actions.contains_key(&step_name) {
    
                let a = &(actions.get(&step_name).unwrap().clone());
                self.register_action(a);
            } else {
                println!("{} not implemented.", &step_name);
            }
        }
        
        self.scraper.collect()
    }

    fn register_target(&mut self, target: &Target) {

        let n = target.name.clone();
        let s = target.selector.clone();

        if s.starts_with("/") { // TODO: xpath and css validation
            self.scraper.find_elements_by_xpath(n, s);
        } else {
            self.scraper.find_elements_by_css(n, s);
  
        }
    }

    fn register_action(&mut self, action: &Action) {
        let n = action.name.clone();
        
        match &action.data {
            ActionData::ActionClick(a) => {self.scraper.click(n, a.selector.to_string());},
            ActionData::ActionScreenshot(a) => {
                
                let format = match a.format.as_str() {
                    "JPEG" => ScreenshotFormat::JPEG,
                    "PNG" => ScreenshotFormat::PNG,
                    _ => ScreenshotFormat::PNG,
                };
                self.scraper.screenshot(n, a.target.to_string(), format);
            },
            ActionData::ActionWait(a) => {self.scraper.sleep(a.duration as u64);},
            ActionData::ActionTypeInto(a) => {self.scraper.type_into(n, a.target.to_string(), a.text.to_string());}
            ActionData::ActionSave(a) => {

                /* let save_path = a.save_path.clone();
                let save_path = save_path.replace("$URL$", action.name.as_str()); */

                self.scraper.save(&a.targets,  &a.flatten);
            
            },
        };
    }
}

pub struct PipelineRunner {

}

impl PipelineRunner {
    pub fn go( pipeline_file: String, urls: &Vec<String>) -> Vec<ScrapingResult> {

        
        let mut scraping_results: Vec<ScrapingResult> = vec![];

        urls.par_iter().enumerate().map(move |(_i, s)| {
            
            let mut pipeline = ScrapingPipeline::from_file(&pipeline_file.clone());

            /* let parts = s.split("/").collect::<Vec<&str>>();
            let id = parts.last().unwrap().to_string().replace(".html", "");
            pipeline.pipeline_config.pipeline.name = format!("{}",  id); */
            
            let res = pipeline.run(&s);
            res
    
        }).collect_into_vec(&mut scraping_results);
        return scraping_results
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

        let steps = pipeline.steps.clone();

        let steps: Vec<String> = steps.iter().enumerate().map(|(i, s)| format!("{}. {}", i+1, s)).collect();

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

        let w = table_targets.total_width();
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

        table_actions.with(tabled::Width::increase(w).priority::<tabled::peaker::PriorityMax>());

        let mut table_steps = tabled::Table::new(steps);
        table_steps.with(Panel::header("STEPS"));
        table_steps.with(Disable::row(Rows::single(1)));
        
        table_steps.with(tabled::Style::sharp());

        table_steps.with(tabled::Style::sharp().horizontals(
            [HorizontalLine::new(1, Style::modern().get_horizontal())
                    .main(Some('═'))
                    .intersection(None),
                ]
        ));
        table_steps.with(Modify::new(Rows::first()).with(Alignment::center()));
        table_steps.with(tabled::Width::increase(w).priority::<tabled::peaker::PriorityMax>());

        writeln!(f, "{}", table_targets);
        writeln!(f, "{}", table_actions);
        writeln!(f, "{}", table_steps);
        
        Ok(())
    }
}
