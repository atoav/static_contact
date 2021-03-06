//! static contact is a server side tool that relays post requests with form fields like name, email, phone and message to email via SMTP.
//!
//! This is meant to be a simple self-hosted solution for contact forms on static websites. You can connect multiple static websites to one single instance of the service
//! 
//! You can send a test request to static_contact via curl:
//! ```Bash
//! curl -sSL -D - --header "Origin: http://mysite.domain" --header "Content-Type: application/json" --request POST --data '{"name":"Mr. Foo Bar", "email":"mrfoo@bar.com", "phone":"+49012345678", "message":"Media is the massage", "identifier":"mysiteidentifier"}' http://localhost:8088
//! ```
//!
//! This emulates a user with the name "Mr. Foo Bar", the email "mrfoo@bar.com" and the phone number "+49012345678" writing a message with the content "Media is the massage".
//! Note the field `"identifier":"mysitename"` – this identifies the site against the server. If there is no endpoint with the identifier "mysiteidentifier" in the `config.toml`, the request is ignored.
//! This message is checked for type and length, and will be sent via SMTP to the according `endpoint.target` email adress specified in the config.
#[macro_use] extern crate failure;

use crate::form::FormData;
use crate::email::send_mail;
use actix_cors::Cors;
use serde::Serialize;
use actix_web::{http::header, error, web, FromRequest, HttpResponse, web::Json};
use actix_web::middleware::Logger;
use lazy_static::lazy_static;
use mime;

mod form;
mod email;
mod config;
use config::Config;

/// Generic Error type
type GenError = failure::Error;

/// Generic Result type
type GenResult<T> = Result<T, GenError>;



#[derive(Serialize)]
struct Status {
    status: String,
}

impl Status {
    fn into_response<S>(body: S) -> Option<GenResult<Json<Status>>> where S: Into<String> {
        let body = body.into();
        Some(Ok(Json(Status{status:body})))
    }
}


/// The index route deserializes [FormData] from the request's JSON body, whose maximum payload size is defined in the config. The following processing steps are taken:
/// 1. Check if the identifier provided by the Endpoint is found among the endpoints in the `config.toml`
/// 2. Check if the length of the files in the FormData is within the provided limits (some of which are hardcoded, some of which are setable via config)
/// 3. Check if the email is valid
/// 4. Send the email to the defined target
async fn index(mut data: web::Json<FormData>, data2: web::Data<Config>) -> GenResult<web::Json<Status>> {
    let mut response = None;

    println!("Received message");

    let config = data2.get_ref();
    let matching_endpoints = config.endpoints
                                   .iter()
                                   .find(|e| e.identifier == data.identifier);

    if matching_endpoints.is_none() {
        eprintln!("Error: The Endpoint with the identifier-value \"{}\" was not named in the config", data.identifier);
        response = Status::into_response(format!("Error: unregistered origin"));
    }

    // Check length of form data
    if response.is_none(){
        let result = data.check_length(matching_endpoints.unwrap());

        match result {
            Err(ref e) => response = Status::into_response(format!("Error while checking form data: {}", e)),
            _ => ()
        }
    }

    // Check email validity
    if response.is_none(){
        let result = data.check_existence(matching_endpoints.unwrap()).await;

        match result {
            Err(ref e) => response = Status::into_response(format!("Error while checking mail validity: {}", e)),
            Ok(_) => println!("Email is valid")
        }
    }

    if response.is_none(){
        match send_mail(&data, config, matching_endpoints.unwrap()){
            Ok(_) => {
                println!("Email relayed to target adress");
                response = Status::into_response(format!("success"));
            },
            Err(e) => response = Status::into_response(format!("Error while sending mail: {}", e))
        }    
    }
    response.unwrap()
}



#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{App, HttpServer};

    lazy_static! {
        static ref CONFIG: Config = Config::new();
    }
    println!("Read config from \"{:?}\"", CONFIG.path());
    let ip = &CONFIG.server.ip;
    let port = &CONFIG.server.port;

    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(move || {
        println!("Starting Worker Thread at {ip}:{port}",
            ip=CONFIG.server.ip, port=CONFIG.server.port);

        // Add each endpoint domain to be accepted via CORS
        let mut cors = Cors::new();
        for endpoint in Config::new().endpoints {
            cors = cors.allowed_origin(endpoint.domain.clone().as_str());
        }
        cors = cors.allowed_methods(vec!["GET", "POST"])
                   .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT, header::CONTENT_TYPE, header::ORIGIN])
                   .supports_credentials()
                   .max_age(3600);


        App::new()
            .data(Config::new())
            .wrap( 
                cors.finish()
            )
            .service(
                web::resource("/")
                    // change json extractor configuration
                    .app_data(web::Json::<FormData>::configure(|cfg| {
                        cfg.limit(CONFIG.server.max_payload)
                            .content_type(|mime| {  // <- accept text/plain content type
                                mime.type_() == mime::TEXT && mime.subtype() == mime::PLAIN
                            })
                            .error_handler(|err, _req| {
                                eprintln!("Error while decoding JSON: {:?}", err);
                                // create custom error response
                                error::InternalError::from_response(
                                    err,
                                    HttpResponse::Conflict().finish(),
                                )
                                .into()
                        })
                    }))
                    .route(web::post().to(index)),
            )
            .wrap(Logger::default())
    })
    .bind(format!("{ip}:{port}", ip=ip, port=port))?  
    .run()
    .await
}