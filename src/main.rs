extern crate actix_web;
use actix_web::{
    server, App,
};

extern crate is_it_on_netflix;
use is_it_on_netflix::{index, upload} ;

fn main() {
    let sys = actix::System::new("template-askama");

    // start http server
    server::new(move || {
        App::new()
            .resource("/",          |r| r.method(actix_web::http::Method::GET).with(index))
            .resource("/upload",    |r| r.method(actix_web::http::Method::POST).with(upload))
    }).bind("127.0.0.1:8080")
        .unwrap()
        .start();

    println!("Started http server: 127.0.0.1:8080");
    let _ = sys.run();
}