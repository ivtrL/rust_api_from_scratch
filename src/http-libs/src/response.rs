use serde::{ Deserialize, Serialize };
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub version: String,
    pub status_code: u16,
    pub status_message: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}
