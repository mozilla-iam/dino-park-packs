use crate::helpers::api::*;
use crate::helpers::db::reset;
use crate::helpers::misc::read_json;
use crate::helpers::misc::test_app;
use crate::helpers::misc::Soa;
use crate::helpers::users::basic_user;
use actix_web::test;
use actix_web::App;
use failure::Error;
use serde_json::json;

#[actix_rt::test]
async fn details() -> Result<(), Error> {
    reset()?;
    let app = App::new().service(test_app().await);
    let mut app = test::init_service(app).await;
    let creator = Soa::from(&basic_user(1, true)).creator().aal_medium();

    let scope = Soa::from(&basic_user(1, true)).creator();
    let res = post(
        &mut app,
        "/groups/api/v1/groups",
        json!({ "name": "dino1", "description": "a group" }),
        &creator,
    )
    .await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/groups/dino1/details", &scope).await;
    assert!(res.status().is_success());
    let j = read_json(res).await;
    assert_eq!(j["group"]["name"], "dino1");
    assert_eq!(j["group"]["expiration"], 0);
    assert_eq!(j["member_count"], 1);
    assert_eq!(j["invitation_count"], 0);
    assert_eq!(j["renewal_count"], json!(null));
    assert_eq!(j["request_count"], json!(null));

    Ok(())
}
