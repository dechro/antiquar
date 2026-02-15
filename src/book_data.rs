use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BookData {
    pub author: String,
    pub title: String,
    pub year: u16,
    pub cover: String,
    pub location: String,
    pub condition: u8,
    pub edition: String,
    pub publisher: String,
    pub category: u16,
    pub description: String,
    pub language: String,
    pub isbn: String,
    pub pages: String,
    pub format: String,
    pub weight: u16,
    pub price: u16,
    pub cover_url: String,
    pub keywords: Vec<String>,
    pub new: bool,
    pub first_edition: bool,
    pub signed: bool,
    pub unused: bool,
    pub personal_notice: String,
    pub unlimited: bool,
}
