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
    let service = test_app().await;
    let app = App::new().service(service);
    let mut app = test::init_service(app).await;

    let host_user = basic_user(1, true);
    let normal_user_1 = basic_user(11, false);
    let host = Soa::from(&host_user).aal_medium();

    let res = post(
        &mut app,
        "/groups/api/v1/groups",
        json!({ "name": "exp-test", "description": "a group", "trust": "Authenticated" }),
        &host.clone().creator(),
    )
    .await;
    assert!(res.status().is_success());

    let res = post(
        &mut app,
        "/groups/api/v1/sudo/member/exp-test",
        json!({ "user_uuid": user_uuid(&normal_user_1), "group_expiration": 1 }),
        &host.clone().admin(),
    )
    .await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/members/exp-test?r=Member", &host).await;
    assert!(res.status().is_success());
    let members = read_json(res).await;
    assert_eq!(members["members"].as_array().map(|a| a.len()), Some(1));
    assert_eq!(members["members"][0]["expiration"].as_null(), None);

    let res = post(
        &mut app,
        &format!(
            "/groups/api/v1/members/exp-test/{}/renew",
            user_uuid(&normal_user_1)
        ),
        json!({ "group_expiration": 0 }),
        &host,
    )
    .await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/members/exp-test?r=Member", &host).await;
    assert!(res.status().is_success());
    let members = read_json(res).await;
    assert_eq!(members["members"].as_array().map(|a| a.len()), Some(1));
    assert_eq!(members["members"][0]["expiration"].as_null(), Some(()));

    Ok(())
}
