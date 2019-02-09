extern crate actix;
extern crate actix_web;
extern crate env_logger;

extern crate askama;
extern crate futures;
extern crate strfmt;
extern crate serde_derive;

use std::collections::HashMap;

use actix_web::{
    server, App, client, HttpMessage, HttpResponse, Query, Result,
};

//use std::io::Read;
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


#[derive(Debug)]
#[derive(Deserialize)]
#[serde(untagged)]
enum Record {
    someData(String),
    langMap(HashMap<String,String>),
}

#[derive(Debug)]
#[derive(Deserialize)]
struct UnogResponse {
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

fn get_repsonse_body(response: &mut client::ClientResponse) -> String {
    let mut body = String::new(); 
    response.body().and_then(|resp| {  // <- complete body
            body = format!("{:?}", resp);
            Ok(())
        }).wait().unwrap();
    body
}

fn get_json_body(response: &client::ClientResponse) -> UnogResponse {
    let json: UnogResponse = response.json().wait().expect("Parsing json failed");
    println!("{:#?}", json);
    json
}

fn index(query: Query<HashMap<String, String>>) -> Result<HttpResponse> {
    let s = match (query.get("title"), query.get("year")){
        (Some(title), Some(year)) => {
            let req_uri = get_request_uri(&title[..], &year[..]).expect("Request String formatting error");
            let response = get_unog_response(&req_uri[..]);
            let body = "Oh hello"; //get_repsonse_body(&mut response);
            let json = get_json_body(&response);
            UserTemplate {
                title: title,
                year: year,
                response: &body[..],
            }.render().unwrap()
        },
        _ => Index.render().unwrap(),
    };
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

fn main() {
    let sys = actix::System::new("template-askama");

    // start http server
    server::new(move || {
        App::new().resource("/", |r| r.method(actix_web::http::Method::GET).with(index))
    }).bind("127.0.0.1:8080")
        .unwrap()
        .start();

    println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}