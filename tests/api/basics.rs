use crate::helpers::api::*;
use crate::helpers::db::reset;
use crate::helpers::misc::read_json;
use crate::helpers::misc::test_app;
use crate::helpers::misc::Soa;
use crate::helpers::users::basic_user;
use actix_web::test;
use actix_web::App;
use dino_park_trust::GroupsTrust;
use dino_park_trust::Trust;
use failure::Error;
use serde_json::json;

#[actix_rt::test]
async fn basic() -> Result<(), Error> {
    reset()?;
    let app = App::new().service(test_app().await);
    let mut app = test::init_service(app).await;
    let req = test::TestRequest::get().uri("/healthz").to_request();
    let res = test::call_service(&mut app, req).await;
    assert!(res.status().is_success());
    Ok(())
}

#[actix_rt::test]
async fn create() -> Result<(), Error> {
    reset()?;
    let app = App::new().service(test_app().await);
    let mut app = test::init_service(app).await;
    let scope = Soa::from(&basic_user(1, true)).creator();
    let res = get(&mut app, "/groups/api/v1/groups", &scope).await;
    assert!(res.status().is_success());
    assert_eq!(read_json(res).await, json!({ "groups": [], "next": null }));

    let scope = Soa::new("nobody", Trust::Public, GroupsTrust::None);
    let res = get(&mut app, "/groups/api/v1/groups", &scope).await;
    assert_eq!(res.status().as_u16(), 403);

    let scope = Soa::from(&basic_user(1, true)).creator();
    let res = post(
        &mut app,
        "/groups/api/v1/groups",
        json!({ "name": "nda", "description": "the nda group" }),
        &scope,
    )
    .await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/groups", &scope).await;
    assert!(res.status().is_success());
    assert_eq!(read_json(res).await["groups"][0]["name"], "nda");

    Ok(())
}
