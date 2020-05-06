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
async fn invalid_groups() -> Result<(), Error> {
    reset()?;
    let app = App::new().service(test_app().await);
    let mut app = test::init_service(app).await;
    let creator = Soa::from(&basic_user(1, true)).creator().aal_medium();

    let res = post(
        &mut app,
        "/groups/api/v1/groups",
        json!({ "name": "Nda", "description": "the nda group" }),
        &creator,
    )
    .await;
    assert_eq!(res.status().as_u16(), 400);
    assert_eq!(read_json(res).await["error"], "invalid_group_name");

    let res = post(
        &mut app,
        "/groups/api/v1/groups",
        json!({ "name": "nda", "description": "foobar".repeat(100) }),
        &creator,
    )
    .await;
    assert_eq!(res.status().as_u16(), 400);
    assert_eq!(read_json(res).await["error"], "invalid_group_data");

    Ok(())
}
