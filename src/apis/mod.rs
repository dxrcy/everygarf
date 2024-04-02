use chrono::{Datelike, NaiveDate};

use crate::dates::{date_month_to_string, date_to_string};

#[derive(Clone, Copy, Debug)]
pub enum SourceAPI {
    Gocomics,
    Fandom,
}

impl SourceAPI {
    pub fn get_page_url(&self, date: NaiveDate) -> String {
        match &self {
            Self::Gocomics => {
                let date_string = date_to_string(date, "/", false);
                format!("https://www.gocomics.com/garfield/{}", date_string)
            }

            Self::Fandom => {
                let month_string = date_month_to_string(date);
                let date_string = date_to_string(date, "-", false);
                format!(
                    "https://garfield.fandom.com/wiki/Garfield,_{month}_{year}_comic_strips?file={date}.gif",
                    month = month_string,
                    year = date.year(),
                    date = date_string,
                )
            }
        }
    }

    pub fn find_image_url<'a>(&self, body: &'a str) -> Option<&'a str> {
        match &self {
            Self::Gocomics => {
                // make this consistent with Self::Fandom
                let char_index = body.find("https://assets.amuniversal.com")?;
                body.get(char_index..char_index + 63)
            }

            Self::Fandom => {
                const IMAGE_URL_BASE: &str = "https://static.wikia.nocookie.net";

                let char_index_left = body.find(IMAGE_URL_BASE)?;

                let mut char_index_right = char_index_left + IMAGE_URL_BASE.len() + 1;
                let mut chars = body.chars().skip(char_index_right);

                while chars.next().is_some_and(|ch| ch != '"') {
                    char_index_right += 1;
                }

                body.get(char_index_left..char_index_right)
            }
        }
    }
}
