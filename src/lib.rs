extern crate actix;
extern crate actix_web;
extern crate env_logger;

extern crate askama;
extern crate futures;
extern crate strfmt;

mod json;

use std::collections::HashMap;
use std::fs;
use std::io::Write;

use actix_web::{
    dev, error, client, multipart, HttpMessage, HttpRequest, HttpResponse, FutureResponse, AsyncResponder, Query, Result, Error,
};

//use std::error::Error;
use askama::Template;
use futures::{Future, Stream, future};
use strfmt::{strfmt, FmtError};

#[derive(Template)]
#[template(path = "movies.jinja")]
struct MoviesTemplate<'a> {
    titles: Vec<&'a String>,
    is_some: bool,
}

#[derive(Template)]
#[template(path = "user.jinja")]
struct UserTemplate<'a> {
    title: &'a str,
    year: &'a str,
    countries: Vec<&'a String>,
}

#[derive(Template)]
#[template(path = "index.jinja")]
struct Index;


static REQUEST_STR: &'static str = "http://unogs.com/nf.cgi?u=5unogs&q={title}-!{year},{year}-!0,5-!0,10-!0,10-!Any-!Any-!Any-!Any-!I%20Don&t=ns&cl=21,23,26,29,33,307,45,39,327,331,334,337,336,269,267,357,65,67,392,400,402,408,412,348,270,73,34,425,46,78&st=adv&ob=Relevance&p=1&l=100&";

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

fn get_country_map(parsed_response: &json::UnogResponse, target_name: &String) -> HashMap<String, String> {
    let mut is_target_found = false;
    let mut result = HashMap::new();
    'movies: for movie_info in parsed_response.items.iter() {
        if is_target_found { break; } // if we found title name, but CountryMap was not found - just stop
        'records: for record in movie_info {
            match record {
                json::Record::SomeData(data) => {
                    if data.to_lowercase().contains(&target_name.to_lowercase()) {
                        is_target_found = true;
                    }
                },
                json::Record::CountryMap(map) => {
                    if is_target_found {
                        result = map.clone();
                        break 'movies;
                    }
                },
            }
        }
    };
    if let Some(more) = result.get_mut("more") {
        *more = "...".to_string();
    };
    result
}

pub fn upload(req: HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    println!("in upload");
    req.body()
        .from_err()
        .and_then(|body| {
            println!("in future");
            let response_body = MoviesTemplate {
                        titles: Vec::new(),
                        is_some: false,
                    }.render().unwrap();
            println!("==== BODY ==== {:?}", body);
            Ok(HttpResponse::Ok()
                .content_type("text/html")
                .body(response_body))
        })
    .responder()
}

pub fn index(query: Query<HashMap<String, String>>) -> Result<HttpResponse> {
    let s = match (query.get("title"), query.get("year")){
        (Some(title), Some(year)) => {
            let req_uri = get_request_uri(&title[..], &year[..]).expect("Request String formatting error");
            let response = get_unog_response(&req_uri[..]);
            let json_response = json::get_json_body(&response);
            let countries = get_country_map(&json_response, &title);
            println!("{:#?}", countries);
            UserTemplate {
                title: title,
                year: year,
                countries: countries.values().collect(),
            }.render().unwrap()
        },
        _ => Index.render().unwrap(),
    };
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}
