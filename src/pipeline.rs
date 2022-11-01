#![allow(unused_must_use)]

use std::{sync::Arc};

use headless_chrome::browser::tab::Tab;

use serde::{Serialize, Deserialize};

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

pub trait Actionable {

    fn run(&self, tab: &Arc<Tab>);

    fn get_data(&self) -> String;

}

impl Actionable for ActionClick {
    fn get_data(&self) -> String {
        self.selector.to_owned()
    }

    fn run(&self, tab: &Arc<Tab>) {
        tab.wait_for_element(&self.selector).unwrap(); 

        let el = tab.find_elements(&self.selector).unwrap();
        let el = el.get(0).unwrap();

        el.click();
        println!("{} - {}", "Run click action...", self.selector);
    }
}

impl Actionable for ActionWait {
    fn get_data(&self) -> String {
        
        self.duration.to_string()
    }

    fn run(&self, tab: &Arc<Tab>) {
        
        println!("Run wait action for {} msecs", self.duration);
    }
}

impl Target {

    pub fn find_element(self, tab: &Arc<Tab>) -> String{

        if self.selector.starts_with("/") { // xpath

            let el = tab.find_element_by_xpath(&self.selector).unwrap();
            return el.get_inner_text().unwrap();
        } else {
            let el = tab.find_element(&self.selector).unwrap();
            return el.get_inner_text().unwrap();
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

    pub targets: Vec<Target>,
    pub actions: Vec<Action>
}