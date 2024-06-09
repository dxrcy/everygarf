mod fandom;

use std::fmt::Display;

use chrono::NaiveDate;
use clap::ValueEnum;
use reqwest::Client;

use crate::dates::date_to_string;

#[derive(Clone, Copy, Debug)]
pub struct Api<'a> {
    pub source: Source,
    pub proxy: Option<&'a str>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Source {
    Gocomics,
    Fandom,
}

pub async fn check_proxy_service(client: &Client, proxy: &str) -> Result<(), reqwest::Error> {
    client.get(proxy).send().await?.error_for_status()?;
    Ok(())
}

impl<'a> Api<'a> {
    pub fn get_page_url(&self, date: NaiveDate) -> String {
        let url = self.source.get_page_url(date);
        match self.proxy {
            None => url,
            Some(proxy) => proxy.to_string() + "?" + &url,
        }
    }
}

impl Default for Source {
    fn default() -> Self {
        Self::Gocomics
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_possible_value().unwrap().get_name())
    }
}

impl Source {
    fn get_page_url(&self, date: NaiveDate) -> String {
        // TODO: make consistent
        match &self {
            Self::Gocomics => {
                format!(
                    "https://www.gocomics.com/garfield/{}",
                    date_to_string(date, "/", false)
                )
            }

            Self::Fandom => {
                format!(
                    "https://garfield.fandom.com/wikia.php?controller=Lightbox&method=getMediaDetail&fileTitle={}",
                    fandom::get_file_title(date),
                )
            }
        }
    }

    pub fn find_image_url<'a>(&self, body: &'a str) -> Option<&'a str> {
        // TODO: make consistent
        let url = match &self {
            Self::Gocomics => {
                let char_index = body.find("https://assets.amuniversal.com")?;
                body.get(char_index..char_index + 63)
            }

            Self::Fandom => {
                const IMAGE_URL_BASE: &str = r#""imageUrl":""#;

                let char_index_left = body.find(IMAGE_URL_BASE)? + IMAGE_URL_BASE.len();

                let mut char_index_right = char_index_left;
                let mut chars = body.chars().skip(char_index_right);

                while chars.next().is_some_and(|ch| ch != '"') {
                    char_index_right += 1;
                }

                body.get(char_index_left..char_index_right)
            }
        };

        url.filter(|url| !url.is_empty())
    }
}
