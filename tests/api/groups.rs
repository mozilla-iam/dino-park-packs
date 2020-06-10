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
async fn list() -> Result<(), Error> {
    reset()?;
    let app = App::new().service(test_app().await);
    let mut app = test::init_service(app).await;
    let user = basic_user(1, true);
    let creator = Soa::from(&user).creator().aal_medium();
    let res = get(&mut app, "/groups/api/v1/groups", &creator).await;
    assert!(res.status().is_success());
    assert_eq!(read_json(res).await, json!({ "groups": [], "next": null }));

    let res = post(
        &mut app,
        "/groups/api/v1/groups",
        json!({ "name": "group1", "description": "a group" }),
        &creator,
    )
    .await;
    assert!(res.status().is_success());

    let res = post(
        &mut app,
        "/groups/api/v1/groups",
        json!({ "name": "group3", "description": "a group" }),
        &creator,
    )
    .await;
    assert!(res.status().is_success());

    let res = post(
        &mut app,
        "/groups/api/v1/groups",
        json!({ "name": "group2", "description": "a group" }),
        &creator,
    )
    .await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/groups", &creator).await;
    assert!(res.status().is_success());
    let j = read_json(res).await;
    assert_eq!(j["groups"][0]["name"], "group1");
    assert_eq!(j["groups"][1]["name"], "group2");
    assert_eq!(j["groups"][2]["name"], "group3");

    let res = get(&mut app, "/groups/api/v1/groups?s=2&by=NameDesc", &creator).await;
    assert!(res.status().is_success());
    let j = read_json(res).await;
    assert_eq!(j["groups"][0]["name"], "group3");
    assert_eq!(j["groups"][1]["name"], "group2");
    let res = get(
        &mut app,
        "/groups/api/v1/groups?s=2&n=2&by=NameDesc",
        &creator,
    )
    .await;
    assert!(res.status().is_success());
    let j = read_json(res).await;
    assert_eq!(j["groups"][0]["name"], "group1");

    let admin = Soa::from(&user).admin().aal_medium();
    let user2 = basic_user(2, true);
    let res = post(
        &mut app,
        "/groups/api/v1/sudo/member/group2",
        json!({ "user_uuid": user_uuid(&user2), "group_expiration": 2 }),
        &admin,
    )
    .await;
    assert!(res.status().is_success());

    let res = get(
        &mut app,
        "/groups/api/v1/groups?s=2&by=MemberCountDesc",
        &creator,
    )
    .await;
    assert!(res.status().is_success());
    let j = read_json(res).await;
    assert_eq!(j["groups"][0]["name"], "group2");
    assert_eq!(j["groups"][1]["name"], "group1");
    let res = get(
        &mut app,
        "/groups/api/v1/groups?s=2&n=2&by=MemberCountDesc",
        &creator,
    )
    .await;
    assert!(res.status().is_success());
    let j = read_json(res).await;
    assert_eq!(j["groups"][0]["name"], "group3");
    Ok(())
}
