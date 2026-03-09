use std::str::FromStr;

use axum::{
    Router,
    error_handling::HandleErrorLayer,
    http::Uri,
    response::IntoResponse,
    routing::{any, get},
};
use axum_oidc::handle_oidc_redirect;
use axum_oidc::{
    EmptyAdditionalClaims, OidcAuthLayer, OidcClaims, OidcClient, OidcLoginLayer,
    OidcRpInitiatedLogout, error::MiddlewareError,
};
use clap::Command;
use dioxus::prelude::*;
use mize::Mize;
use mize::MizeResult;
use openidconnect::Scope;
use openidconnect::{ClientId, ClientSecret, IssuerUrl};
use tower::ServiceBuilder;
use tower_sessions::{
    Expiry, MemoryStore, SessionManagerLayer,
    cookie::{SameSite, time::Duration},
};

pub fn server(mize: &mut Mize) -> MizeResult<()> {
    let mut cli = mize.get_part_native::<marts::CliPart>("cli")?;
    mize.add_name_only_part("ppc.server");

    mize.new_opt("auth.issuer");
    mize.new_opt("auth.client_id");
    mize.new_opt("auth.client_secret");
    mize.new_opt("auth.redirect");
    mize.new_opt("auth.cookie_key");

    let mut mize = mize.clone();
    cli.subcommand(Command::new("server"), move |_, _| {
        // Create tokio runtime for async server
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async { start_server(&mut mize).await })?;

        Ok(())
    });

    Ok(())
}

pub async fn start_server(mize: &mut Mize) -> MizeResult<()> {
    dioxus::logger::initialize_default();

    let issuer = mize.get_config("auth.issuer")?.value_string()?;
    let client_id = mize.get_config("auth.client_id")?.value_string()?;
    let client_secret = mize.get_config("auth.client_secret")?.value_string()?;
    let url = mize.get_config("web.url")?.value_string()?;
    let redirect_url = format!("{url}/oidc");

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::seconds(120)));

    let oidc_login_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: MiddlewareError| async {
            dbg!(&e);
            e.into_response()
        }))
        .layer(OidcLoginLayer::<EmptyAdditionalClaims>::new());

    let oidc_client = OidcClient::<EmptyAdditionalClaims>::builder()
        .with_default_http_client()
        .with_redirect_url(Uri::from_str(redirect_url.as_str())?)
        .with_client_id(ClientId::new(client_id))
        .add_scope(Scope::new("profile".into()))
        .add_scope(Scope::new("email".into()))
        // Optional: add untrusted audiences. If the `aud` claim contains any of these audiences, the token is rejected.
        //.add_untrusted_audience(Audience::new("123456789".to_string()))
        .with_client_secret(ClientSecret::new(client_secret))
        .discover(IssuerUrl::new(issuer.into()).expect("Invalid IssuerUrl"))
        .await
        .unwrap()
        .build();

    let oidc_auth_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: MiddlewareError| async {
            dbg!(&e);
            e.into_response()
        }))
        .layer(OidcAuthLayer::new(oidc_client));

    let app = Router::new()
        .route("/foo", get(authenticated))
        .route("/logout", get(logout))
        .layer(oidc_login_service)
        .route("/bar", get(maybe_authenticated))
        .route("/oidc", any(handle_oidc_redirect::<EmptyAdditionalClaims>))
        .layer(oidc_auth_service)
        .layer(session_layer)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        // not authenticated routes
        .route("/", get(ppc_main_page));

    tracing::info!("Running on http://localhost:3000");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn authenticated(claims: OidcClaims<EmptyAdditionalClaims>) -> impl IntoResponse {
    format!("Hello {}", claims.subject().as_str())
}

async fn ppc_main_page() -> impl IntoResponse {
    "hi"
}

#[axum::debug_handler]
async fn maybe_authenticated(
    claims: Result<OidcClaims<EmptyAdditionalClaims>, axum_oidc::error::ExtractorError>,
) -> impl IntoResponse {
    if let Ok(claims) = claims {
        format!(
            "Hello {}! You are already logged in from another Handler.",
            claims.subject().as_str()
        )
    } else {
        "Hello anon!".to_string()
    }
}

async fn logout(logout: OidcRpInitiatedLogout) -> impl IntoResponse {
    logout.with_post_logout_redirect(Uri::from_static("https://localhost:3000"))
}
