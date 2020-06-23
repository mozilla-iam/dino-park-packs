use crate::helpers::api::*;
use crate::helpers::db::reset;
use crate::helpers::misc::read_json;
use crate::helpers::misc::test_app;
use crate::helpers::misc::Soa;
use crate::helpers::users::basic_user;
use crate::helpers::users::user_email;
use crate::helpers::users::user_uuid;
use actix_web::test;
use actix_web::App;
use failure::Error;
use serde_json::json;

#[actix_rt::test]
async fn curator_emails() -> Result<(), Error> {
    reset()?;
    let app = App::new().service(test_app().await);
    let mut app = test::init_service(app).await;

    let host_user = basic_user(1, true);
    let add_user_1 = basic_user(2, true);
    let add_user_2 = basic_user(3, true);
    let host = Soa::from(&host_user).admin().aal_medium();

    let res = post(
        &mut app,
        "/groups/api/v1/groups",
        json!({ "name": "sudo-test", "description": "a group" }),
        &host,
    )
    .await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/groups", &host).await;
    assert!(res.status().is_success());
    assert_eq!(read_json(res).await["groups"][0]["typ"], "Closed");

    let res = post(
        &mut app,
        "/groups/api/v1/curators/sudo-test",
        json!({ "member_uuid": user_uuid(&add_user_1) }),
        &host,
    )
    .await;
    assert!(res.status().is_success());
    let res = post(
        &mut app,
        "/groups/api/v1/sudo/member/sudo-test",
        json!({ "user_uuid": user_uuid(&add_user_2), "group_expiration": 2 }),
        &host,
    )
    .await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/members/sudo-test", &host).await;
    assert!(res.status().is_success());
    let members = read_json(res).await;
    assert_eq!(members["members"][0]["user_uuid"], user_uuid(&host_user));
    assert_eq!(members["members"][1]["user_uuid"], user_uuid(&add_user_1));
    assert_eq!(members["members"][2]["user_uuid"], user_uuid(&add_user_2));

    let res = get(&mut app, "/groups/api/v1/sudo/curators/sudo-test", &host).await;
    assert!(res.status().is_success());
    let emails = read_json(res).await;
    assert_eq!(emails[0], user_email(&host_user));
    assert_eq!(emails[1], user_email(&add_user_1));
    assert!(emails.as_array().map(|a| a.len() == 2).unwrap_or_default());

    let no_admin = Soa::from(&host_user).creator().aal_medium();
    let res = get(
        &mut app,
        "/groups/api/v1/sudo/curators/sudo-test",
        &no_admin,
    )
    .await;
    assert!(res.status().is_client_error());

    Ok(())
}

#[actix_rt::test]
async fn inactive_group() -> Result<(), Error> {
    reset()?;
    let app = App::new().service(test_app().await);
    let mut app = test::init_service(app).await;
    let admin = Soa::from(&basic_user(1, true)).admin().aal_medium();

    let res = post(
        &mut app,
        "/groups/api/v1/groups",
        json!({ "name": "some", "description": "some group" }),
        &admin,
    )
    .await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/groups", &admin).await;
    assert_eq!(read_json(res).await["groups"][0]["name"], "some");

    let res = delete(&mut app, "/groups/api/v1/groups/some", &admin).await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/groups", &admin).await;
    assert_eq!(read_json(res).await, json!({ "groups": [], "next": null }));

    let res = get(&mut app, "/groups/api/v1/sudo/groups/inactive", &admin).await;
    assert_eq!(read_json(res).await[0]["name"], "some");

    let res = delete(&mut app, "/groups/api/v1/sudo/groups/inactive/some", &admin).await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/sudo/groups/inactive", &admin).await;
    assert_eq!(read_json(res).await, json!([]));

    Ok(())
}
