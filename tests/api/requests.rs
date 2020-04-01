use crate::helpers::api::*;
use crate::helpers::db::reset;
use crate::helpers::misc::read_json;
use crate::helpers::misc::test_app;
use crate::helpers::misc::Soa;
use crate::helpers::users::basic_user;
use crate::helpers::users::user_uuid;
use actix_web::test;
use actix_web::App;
use dino_park_trust::GroupsTrust;
use dino_park_trust::Trust;
use failure::Error;
use serde_json::json;

#[actix_rt::test]
async fn cancel_reviewed() -> Result<(), Error> {
    reset()?;
    let app = App::new().service(test_app().await);
    let mut app = test::init_service(app).await;
    let scope = Soa::from(&basic_user(1, true)).creator();
    let res = get(&mut app, "/groups/api/v1/groups", &scope).await;
    assert!(res.status().is_success());
    assert_eq!(read_json(res).await, json!({ "groups": [], "next": null }));

    let nobody = Soa::new("nobody", Trust::Public, GroupsTrust::None);
    let host_user = basic_user(1, true);
    let requester_user = basic_user(2, true);
    let host = Soa::from(&host_user).creator();
    let requester = Soa::from(&requester_user);

    let res = get(&mut app, "/groups/api/v1/groups", &nobody).await;
    assert_eq!(res.status().as_u16(), 403);

    let res = post(
        &mut app,
        "/groups/api/v1/groups",
        json!({ "name": "reviewed-test", "description": "a reviewed group", "type": "Reviewed" }),
        &host,
    )
    .await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/groups", &host).await;
    assert!(res.status().is_success());
    assert_eq!(read_json(res).await["groups"][0]["typ"], "Reviewed");

    let res = post(
        &mut app,
        "/groups/api/v1/self/requests/reviewed-test",
        json!(null),
        &requester,
    )
    .await;
    assert!(res.status().is_success());

    let res = delete(
        &mut app,
        "/groups/api/v1/self/requests/reviewed-test",
        &requester,
    )
    .await;
    assert!(res.status().is_success());

    Ok(())
}

#[actix_rt::test]
async fn reject_reviewed() -> Result<(), Error> {
    reset()?;
    let app = App::new().service(test_app().await);
    let mut app = test::init_service(app).await;
    let scope = Soa::from(&basic_user(1, true)).creator();
    let res = get(&mut app, "/groups/api/v1/groups", &scope).await;
    assert!(res.status().is_success());
    assert_eq!(read_json(res).await, json!({ "groups": [], "next": null }));

    let nobody = Soa::new("nobody", Trust::Public, GroupsTrust::None);
    let host_user = basic_user(1, true);
    let requester_user = basic_user(2, true);
    let host = Soa::from(&host_user).creator();
    let requester = Soa::from(&requester_user);

    let res = get(&mut app, "/groups/api/v1/groups", &nobody).await;
    assert_eq!(res.status().as_u16(), 403);

    let res = post(
        &mut app,
        "/groups/api/v1/groups",
        json!({ "name": "reviewed-test", "description": "a reviewed group", "type": "Reviewed" }),
        &host,
    )
    .await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/groups", &host).await;
    assert!(res.status().is_success());
    assert_eq!(read_json(res).await["groups"][0]["typ"], "Reviewed");

    let res = post(
        &mut app,
        "/groups/api/v1/self/requests/reviewed-test",
        json!(null),
        &requester,
    )
    .await;
    assert!(res.status().is_success());

    let res = delete(
        &mut app,
        &format!(
            "/groups/api/v1/requests/reviewed-test/{}",
            user_uuid(&requester_user)
        ),
        &host,
    )
    .await;
    assert!(res.status().is_success());

    Ok(())
}
