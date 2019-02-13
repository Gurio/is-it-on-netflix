extern crate serde_derive;
extern crate actix_web;
extern crate futures;

use serde_derive::Deserialize;
use std::collections::HashMap;
use futures::Future;
use actix_web::{
    client, HttpMessage, 
};

// the format of UNOG response is real stupid
// you get an array of movies
// and each movie itself is an array of infos related to it
// each info might be a String or a map,
// and there is no indicator which is which (except, maybe a position)
#[derive(Debug)]
#[derive(Deserialize)]
#[serde(untagged)]
pub enum Record {
    SomeData(String),
    CountryMap(HashMap<String,String>),
}

#[derive(Debug)]
#[derive(Deserialize)]
pub struct UnogResponse {
    #[serde(rename = "COUNT")]
    pub count: String,
    #[serde(rename = "ITEMS")]
    pub items: Vec<Vec<Record>>,
}

pub fn get_json_body(response: &client::ClientResponse) -> UnogResponse {
    let parsed_response: UnogResponse = response.json().wait().expect("Parsing json failed");
    parsed_response
}