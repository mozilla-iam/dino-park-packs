use crate::helpers::api::*;
use crate::helpers::db::reset;
use crate::helpers::misc::read_json;
use crate::helpers::misc::test_app;
use crate::helpers::misc::Soa;
use crate::helpers::sudo::add_to_group;
use crate::helpers::users::basic_user;
use actix_web::test;
use actix_web::App;
use failure::Error;
use serde_json::json;

#[actix_rt::test]
async fn upgrade_group_trust() -> Result<(), Error> {
    reset()?;
    let service = test_app().await;
    let app = App::new().service(service);
    let mut app = test::init_service(app).await;

    let host_user = basic_user(1, true);
    let staff_user_1 = basic_user(2, true);
    let nda_user_1 = basic_user(11, false);
    let nda_user_2 = basic_user(12, false);
    let normal_user_1 = basic_user(13, false);
    let normal_user_2 = basic_user(14, false);
    let host = Soa::from(&host_user).aal_medium();

    let res = post(
        &mut app,
        "/groups/api/v1/groups",
        json!({ "name": "nda", "description": "the nda group" }),
        &host.clone().creator(),
    )
    .await;
    assert!(res.status().is_success());

    let res = post(
        &mut app,
        "/groups/api/v1/groups",
        json!({ "name": "upgrade-test", "description": "a group", "trust": "Authenticated" }),
        &host.clone().creator(),
    )
    .await;
    assert!(res.status().is_success());

    add_to_group(&mut app, &host, &nda_user_1, "nda").await;
    add_to_group(&mut app, &host, &nda_user_2, "nda").await;

    add_to_group(&mut app, &host, &staff_user_1, "upgrade-test").await;
    add_to_group(&mut app, &host, &nda_user_1, "upgrade-test").await;
    add_to_group(&mut app, &host, &nda_user_2, "upgrade-test").await;
    add_to_group(&mut app, &host, &normal_user_1, "upgrade-test").await;
    add_to_group(&mut app, &host, &normal_user_2, "upgrade-test").await;

    let res = get(&mut app, "/groups/api/v1/members/upgrade-test", &host).await;
    assert!(res.status().is_success());
    let members = read_json(res).await;
    assert_eq!(members["members"].as_array().map(|a| a.len()), Some(6));

    let res = get(
        &mut app,
        "/groups/api/v1/groups/upgrade-test/details",
        &host,
    )
    .await;
    assert!(res.status().is_success());
    let group = read_json(res).await;
    assert_eq!(group["group"]["trust"], "Authenticated");

    let res = put(
        &mut app,
        "/groups/api/v1/sudo/trust/groups/upgrade-test",
        json!({ "trust": "Ndaed" }),
        &host.clone().admin(),
    )
    .await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/members/upgrade-test", &host).await;
    assert!(res.status().is_success());
    let members = read_json(res).await;
    assert_eq!(members["members"].as_array().map(|a| a.len()), Some(4));

    let res = get(
        &mut app,
        "/groups/api/v1/groups/upgrade-test/details",
        &host,
    )
    .await;
    assert!(res.status().is_success());
    let group = read_json(res).await;
    assert_eq!(group["group"]["trust"], "Ndaed");

    let res = put(
        &mut app,
        "/groups/api/v1/sudo/trust/groups/upgrade-test",
        json!({ "trust": "Staff"}),
        &host.clone().admin(),
    )
    .await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/members/upgrade-test", &host).await;
    assert!(res.status().is_success());
    let members = read_json(res).await;
    assert_eq!(members["members"].as_array().map(|a| a.len()), Some(2));

    let res = get(
        &mut app,
        "/groups/api/v1/groups/upgrade-test/details",
        &host,
    )
    .await;
    assert!(res.status().is_success());
    let group = read_json(res).await;
    assert_eq!(group["group"]["trust"], "Staff");
    Ok(())
}
