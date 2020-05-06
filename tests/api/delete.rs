use crate::helpers::api::*;
use crate::helpers::db::reset;
use crate::helpers::misc::read_json;
use crate::helpers::misc::test_app;
use crate::helpers::misc::Soa;
use crate::helpers::users::basic_user;
use crate::helpers::users::user_uuid;
use actix_web::test;
use actix_web::App;
use failure::Error;
use serde_json::json;

#[actix_rt::test]
async fn delete_group() -> Result<(), Error> {
    reset()?;
    let app = App::new().service(test_app().await);
    let mut app = test::init_service(app).await;
    let creator = Soa::from(&basic_user(1, true)).creator().aal_medium();

    let res = post(
        &mut app,
        "/groups/api/v1/groups",
        json!({ "name": "some", "description": "some group" }),
        &creator,
    )
    .await;
    assert!(res.status().is_success());

    let user2 = basic_user(2, true);
    let res = post(
        &mut app,
        "/groups/api/v1/curators/some",
        json!({ "member_uuid": user_uuid(&user2) }),
        &creator,
    )
    .await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/members/some", &creator).await;
    assert!(res.status().is_success());
    assert_eq!(
        read_json(res).await["members"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or_default(),
        2
    );

    let res = get(&mut app, "/groups/api/v1/groups", &creator).await;
    assert_eq!(read_json(res).await["groups"][0]["name"], "some");

    let res = delete(&mut app, "/groups/api/v1/groups/some", &creator).await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/groups", &creator).await;
    assert_eq!(read_json(res).await, json!({ "groups": [], "next": null }));

    Ok(())
}
