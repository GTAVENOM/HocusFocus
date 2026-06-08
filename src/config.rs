use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config{
    pub rules: Vec<Rule>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Rule{
    pub name: String,
    pub domains: Vec<String>,
    pub schedules: Vec<Schedule>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Schedule{
    pub days: Vec<String>,
    pub start: String,
    pub end: String,
}

impl Config{
    pub fn load_or_create(path: &Path)->Result<Self, Box<dyn Error>>{
        if !path.exists(){
            if let Some(parent)= path.parent(){
                fs::create_dir_all(parent)?;
            }
            let default_config= Self::default_template();
            let toml_string=toml::to_string_pretty(&default_config)?;
            fs::write(path,toml_string)?;
            return Ok(default_config);
        }

        let content=fs::read_to_string(path)?;
        let config:Config=toml::from_str(&content)?;
        Ok(config)
    }

    fn default_template()-> Self{
        Config{
            rules: vec![
                Rule{
                    name: "Social Media Block".to_string(),
                    domains: vec![
                        "facebook.com".to_string(),
                        "www.facebook.com".to_string(),
                        "twitter.com".to_string(),
                        "www.twitter.com".to_string(),
                        "x.com".to_string(),
                        "www.x.com".to_string(),
                        "instagram.com".to_string(),
                        "www.instagram.com".to_string(),
                        "reddit.com".to_string(),
                        "www.reddit.com".to_string(),
                    ],
                    schedules: vec![
                        Schedule {
                            days: vec![
                                "Mon".to_string(),
                                "Tue".to_string(),
                                "Wed".to_string(),
                                "Thu".to_string(),
                                "Fri".to_string(),
                            ],
                            start: "09:00".to_string(),
                            end: "17:00".to_string(),
                        }
                    ],
                }
            ],
        }
    }
}

pub fn get_default_config_path()-> Option<PathBuf>{
    dirs::home_dir().map(|mut p|{
        p.push(".config");
        p.push("hocus-focus");
        p.push("config.toml");
        p
    })
}