use crate::helpers::api::*;
use crate::helpers::db::reset;
use crate::helpers::misc::test_app;
use crate::helpers::misc::Soa;
use crate::helpers::users::basic_user;
use actix_web::test;
use actix_web::App;
use failure::Error;
use serde_json::json;

#[actix_rt::test]
async fn create() -> Result<(), Error> {
    reset()?;
    let app = App::new().service(test_app().await);
    let mut app = test::init_service(app).await;
    let creator = Soa::from(&basic_user(1, true)).creator().aal_medium();

    let res = post(
        &mut app,
        "/groups/api/v1/groups",
        json!({ "name": "auth", "description": "an authenticated group", "trust": "Authenticated"}),
        &creator,
    )
    .await;
    assert!(res.status().is_success());

    let res = post(
        &mut app,
        "/groups/api/v1/groups",
        json!({ "name": "public", "description": "a public group", "trust": "Public"}),
        &creator,
    )
    .await;
    assert!(!res.status().is_success());

    Ok(())
}
