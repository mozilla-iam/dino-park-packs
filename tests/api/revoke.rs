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
async fn revoke_nda() -> Result<(), Error> {
    reset()?;
    let app = App::new().service(test_app().await);
    let mut app = test::init_service(app).await;

    let host_user = basic_user(1, true);
    let add_staff_user = basic_user(2, true);
    let add_normal_user = basic_user(11, false);
    let host = Soa::from(&host_user).admin().aal_medium();

    let res = post(
        &mut app,
        "/groups/api/v1/groups",
        json!({ "name": "nda", "description": "the nda group" }),
        &host,
    )
    .await;
    assert!(res.status().is_success());

    let res = post(
        &mut app,
        "/groups/api/v1/groups",
        json!({ "name": "revoke-test", "description": "a group", "trust": "Ndaed" }),
        &host,
    )
    .await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/groups", &host).await;
    assert!(res.status().is_success());
    assert_eq!(read_json(res).await["groups"][0]["typ"], "Closed");

    let res = post(
        &mut app,
        "/groups/api/v1/sudo/member/nda",
        json!({ "user_uuid": user_uuid(&add_staff_user) }),
        &host,
    )
    .await;
    assert!(res.status().is_success());
    let res = post(
        &mut app,
        "/groups/api/v1/sudo/member/nda",
        json!({ "user_uuid": user_uuid(&add_normal_user) }),
        &host,
    )
    .await;
    assert!(res.status().is_success());


    let res = post(
        &mut app,
        "/groups/api/v1/sudo/member/revoke-test",
        json!({ "user_uuid": user_uuid(&add_staff_user) }),
        &host,
    )
    .await;
    assert!(res.status().is_success());
    let res = post(
        &mut app,
        "/groups/api/v1/sudo/member/revoke-test",
        json!({ "user_uuid": user_uuid(&add_normal_user) }),
        &host,
    )
    .await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/members/revoke-test", &host).await;
    assert!(res.status().is_success());
    let members = read_json(res).await;
    assert_eq!(members["members"][0]["user_uuid"], user_uuid(&host_user));
    assert_eq!(
        members["members"][1]["user_uuid"],
        user_uuid(&add_normal_user)
    );
    assert_eq!(
        members["members"][2]["user_uuid"],
        user_uuid(&add_staff_user)
    );

    let res = delete(
        &mut app,
        &format!(
            "/groups/api/v1/members/nda/{}",
            user_uuid(&add_staff_user)
        ),
        &host,
    )
    .await;
    assert!(res.status().is_success());
    let res = delete(
        &mut app,
        &format!(
            "/groups/api/v1/members/nda/{}",
            user_uuid(&add_normal_user)
        ),
        &host,
    )
    .await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/members/revoke-test", &host).await;
    assert!(res.status().is_success());
    let members = read_json(res).await;
    assert_eq!(members["members"][0]["user_uuid"], user_uuid(&host_user));
    assert_eq!(
        members["members"][1]["user_uuid"],
        user_uuid(&add_staff_user)
    );
    assert_eq!(
        members["members"][2],
        json!(null)
    );
    

    Ok(())
}
