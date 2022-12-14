use actix_cors::Cors;
use actix_web::{
    dev::HttpServiceFactory,
    http::{self, Uri},
    middleware::DefaultHeaders,
    web::{self},
};

use keygate_core::config::Environment;
use utoipa::OpenApi;

mod identity;
mod meta;
mod process_login;
mod process_signup;
mod session;

use crate::{errors::KeygateErrorResponse, KG};

#[derive(OpenApi)]
#[openapi(
    paths(
        // /api/v1/session
        session::refresh,
        // /api/v1/process/login
        process_login::create_login_process,
        process_login::login_process_password,
        // /api/v1/process/signup
        process_signup::create_signup_process,
        process_signup::signup_process_password,
        // /api/v1/meta
        meta::meta 
    ),
    components(schemas(
        // /api/v1/session
        session::RefreshResponse,
        // /api/v1/process/login
        process_login::LoginProcessRequest,
        process_login::LoginProcessResponse,
        process_login::LoginPasswordRequest,
        process_login::LoginPasswordResponse,
        process_login::LoginProcessStep,

        // /api/v1/process/signup
        process_signup::SignupProcessRequest,
        process_signup::SignupProcessResponse,
        process_signup::SignupPasswordRequest,
        process_signup::SignupPasswordResponse,
        process_signup::SignupProcessStep,

        // /api/v1/meta
        meta::MetaResonse,

        // /api/v1/*
        KeygateErrorResponse
    ))
)]
pub struct PublicApiDoc;

pub fn service(scope: &str, kg: KG) -> impl HttpServiceFactory {
    let session = session::service("/session");
    let identity = identity::service("/identity");
    let process_login = process_login::service("/process/login");
    let process_signup = process_signup::service("/process/signup");
    let meta = meta::service("/meta");

    let environment = { kg.config.read().unwrap().environment.clone() };

    let mut default_headers = DefaultHeaders::new()
        .add(("x-xss-protection", "1; mode=block"))
        .add(("x-content-type-options", "nosniff"))
        .add(("x-frame-options", "DENY"))
        .add(("referrer-policy", "no-referrer"))
        .add(("Content-Security-Policy", "default-src 'none'"));

    if environment == Environment::Production {
        default_headers = default_headers.add(("Strict-Transport-Security", "max-age=31536000; includeSubDomains"));
    }

    let cors = match environment {
        Environment::Production => Cors::default(),
        Environment::Development => Cors::default()
            .allowed_origin_fn(|origin, _req_head| {
                let Ok(Ok(uri)) = origin.to_str().map(|s| s.parse::<Uri>()) else {
                    return false;
                };

                if let Some(host) = uri.host() {
                    return host == "localhost" || host.ends_with(".localhost") || host == "127.0.0.1";
                }

                false
            })
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::CONTENT_TYPE,
            ])
            .max_age(3600),
    };

    web::scope(scope)
        .wrap(cors)
        .wrap(default_headers)
        .service(session)
        .service(identity)
        .service(process_login)
        .service(process_signup)
        .service(meta)
        .route("/ping", web::get().to(pong))
    // .service(process_recovery)
}

async fn pong() -> &'static str {
    "pong"
}
