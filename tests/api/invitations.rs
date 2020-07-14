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
async fn invite_order() -> Result<(), Error> {
    reset()?;
    let app = App::new().service(test_app().await);
    let mut app = test::init_service(app).await;

    let host_user = basic_user(1, true);
    let invite_user_1 = basic_user(2, true);
    let invite_user_2 = basic_user(3, true);
    let host = Soa::from(&host_user).creator().aal_medium();

    let res = post(
        &mut app,
        "/groups/api/v1/groups",
        json!({ "name": "invite-test", "description": "a group" }),
        &host,
    )
    .await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/groups", &host).await;
    assert!(res.status().is_success());
    assert_eq!(read_json(res).await["groups"][0]["typ"], "Closed");

    let res = post(
        &mut app,
        "/groups/api/v1/invitations/invite-test",
        json!({ "user_uuid": user_uuid(&invite_user_1), "invitation_expiration": 7 }),
        &host,
    )
    .await;
    assert!(res.status().is_success());
    let res = post(
        &mut app,
        "/groups/api/v1/invitations/invite-test",
        json!({ "user_uuid": user_uuid(&invite_user_2), "invitation_expiration": 2 }),
        &host,
    )
    .await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/invitations/invite-test", &host).await;
    assert!(res.status().is_success());
    let invitations = read_json(res).await;
    assert_eq!(invitations[0]["user_uuid"], user_uuid(&invite_user_2));
    assert_eq!(invitations[1]["user_uuid"], user_uuid(&invite_user_1));

    Ok(())
}

#[actix_rt::test]
async fn invitation_text() -> Result<(), Error> {
    reset()?;
    let app = App::new().service(test_app().await);
    let mut app = test::init_service(app).await;

    let host_user = basic_user(1, true);
    let host = Soa::from(&host_user).creator().aal_medium();

    let res = post(
        &mut app,
        "/groups/api/v1/groups",
        json!({ "name": "invitation-text-test", "description": "a group" }),
        &host,
    )
    .await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/groups", &host).await;
    assert!(res.status().is_success());
    assert_eq!(read_json(res).await["groups"][0]["typ"], "Closed");

    let res = post(
        &mut app,
        "/groups/api/v1/invitations/invitation-text-test/email",
        json!({ "body": "some copy" }),
        &host,
    )
    .await;
    assert!(res.status().is_success());

    let res = get(
        &mut app,
        "/groups/api/v1/invitations/invitation-text-test/email",
        &host,
    )
    .await;
    assert!(res.status().is_success());
    let invitations = read_json(res).await;
    assert_eq!(invitations["body"], "some copy");

    let res = post(
        &mut app,
        "/groups/api/v1/invitations/invitation-text-test/email",
        json!({ "body": null }),
        &host,
    )
    .await;
    assert!(res.status().is_success());

    let res = get(
        &mut app,
        "/groups/api/v1/invitations/invitation-text-test/email",
        &host,
    )
    .await;
    assert!(res.status().is_success());
    let invitations = read_json(res).await;
    assert_eq!(invitations["body"], serde_json::Value::Null);

    Ok(())
}
