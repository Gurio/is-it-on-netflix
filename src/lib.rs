extern crate actix;
extern crate actix_web;
extern crate env_logger;

extern crate askama;
extern crate futures;
extern crate strfmt;
extern crate serde_derive;

use std::collections::HashMap;

use actix_web::{
    client, HttpMessage, HttpResponse, Query, Result,
};

use std::error::Error;
use askama::Template;
use futures::Future;
use serde_derive::Deserialize;
use strfmt::{strfmt, FmtError};


#[derive(Template)]
#[template(path = "user.html")]
struct UserTemplate<'a> {
    title: &'a str,
    year: &'a str,
    response: &'a str,
}

#[derive(Template)]
#[template(path = "index.html")]
struct Index;

// the format of UNOG response is real stupid
// you get an array of movies
// and each movie itself is an array of infos related to it
// each info might be a String or a map,
// and there is no indicator which is which (except, maybe a position)
#[derive(Debug)]
#[derive(Deserialize)]
#[serde(untagged)]
enum Record {
    SomeData(String),
    LangMap(HashMap<String,String>),
}

#[derive(Debug)]
#[derive(Deserialize)]
struct ParsedResponse {
    #[serde(rename = "COUNT")]
    count: String,
    #[serde(rename = "ITEMS")]
    items: Vec<Vec<Record>>,
}

static REQUEST_STR: &'static str = "http://unogs.com/nf.cgi?u=5unogs&q={title}-!{year},{year}-!0,5-!0,10-!0,10-!Any-!Any-!Any-!Any-!I%20Don&t=ns&cl=21,23,26,29,33,307,45,39,327,331,334,337,336,269,267,357,65,67,392,400,402,408,412,348,270,73,34,425,46,78&st=adv&ob=Relevance&p=1&l=100&";
//static REFERER_STR: &'static str = "http://unogs.com/\\?q\\={title}-\\!{year},{year}-\\!0,5-\\!0,10-\\!0,10-\\!Any-\\!Any-\\!Any-\\!Any-\\!I%20Don\\&cl\\=21,23,26,29,33,307,45,39,327,331,334,337,336,269,267,357,65,67,392,400,402,408,412,348,270,73,34,425,46,78\\&st\\=adv\\&ob\\=Relevance\\&p\\=1\\&ao\\=and";

fn get_request_uri(title: &str, year: &str) -> Result<String, FmtError> {
    let mut vars: HashMap<String, &str> = HashMap::new();
    vars.insert("title".to_string(), title);
    vars.insert("year".to_string(), year);
    strfmt(&REQUEST_STR, &vars)
}

fn get_unog_response(req_uri: &str) -> client::ClientResponse {
    client::get(req_uri)
        .no_default_headers()
        .set_header("Host", "unogs.com")
        .header("Accept", "*/*")
        .header("User-Agent", "curl/7.61.0") 
        .header("Referer", req_uri)
        .finish().unwrap()
        .send()
        .wait().unwrap()
}

fn get_json_body(response: &client::ClientResponse) -> ParsedResponse {
    let parsed_response: ParsedResponse = response.json().wait().expect("Parsing json failed");
    parsed_response
}

fn get_lang_map(parsed_response: &ParsedResponse, target_name: &String) -> HashMap<String, String> {
    let mut is_target_found = false;
    let mut result = HashMap::new();
    'movies: for movie_info in parsed_response.items.iter() {
        if is_target_found { break; } // if we found title name, but LangMap was not found - just stop
        'records: for record in movie_info {
            match record {
                Record::SomeData(data) => {
                    if data.to_lowercase().contains(&target_name.to_lowercase()) {
                        is_target_found = true;
                    }
                },
                Record::LangMap(map) => {
                    if is_target_found {
                        result = map.clone();
                        break 'movies;
                    }
                },
            }
        }
    };
    result
}

pub fn index(query: Query<HashMap<String, String>>) -> Result<HttpResponse> {
    let s = match (query.get("title"), query.get("year")){
        (Some(title), Some(year)) => {
            let req_uri = get_request_uri(&title[..], &year[..]).expect("Request String formatting error");
            let response = get_unog_response(&req_uri[..]);
            let json_response = get_json_body(&response);
            let languages = get_lang_map(&json_response, &title);
            println!("{:#?}", languages);
            UserTemplate {
                title: title,
                year: year,
                response: &format!("{:#?}", languages)[..],
            }.render().unwrap()
        },
        _ => Index.render().unwrap(),
    };
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}