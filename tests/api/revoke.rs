use crate::helpers::api::*;
use crate::helpers::db::get_pool;
use crate::helpers::db::reset;
use crate::helpers::misc::read_json;
use crate::helpers::misc::test_app_and_cis;
use crate::helpers::misc::Soa;
use crate::helpers::sudo::add_to_group;
use crate::helpers::users::basic_user;
use crate::helpers::users::user_uuid;
use actix_web::test;
use actix_web::App;
use dino_park_packs::db::operations::users::update_user_cache;
use failure::Error;
use serde_json::json;
use std::sync::Arc;

#[actix_rt::test]
async fn revoke_nda() -> Result<(), Error> {
    reset()?;
    let (service, cis_client) = test_app_and_cis().await;
    let cis_client = Arc::new(cis_client);
    let app = App::new().service(service);
    let mut app = test::init_service(app).await;

    let host_user = basic_user(1, true);
    let staff_user_1 = basic_user(2, true);
    let mut staff_user_2 = basic_user(3, true);
    let normal_user_1 = basic_user(11, false);
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
        json!({ "name": "revoke-test", "description": "a group", "trust": "Ndaed" }),
        &host.clone().creator(),
    )
    .await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/groups", &host).await;
    assert!(res.status().is_success());
    assert_eq!(read_json(res).await["groups"][0]["typ"], "Closed");

    add_to_group(&mut app, &host, &staff_user_1, "nda").await;
    add_to_group(&mut app, &host, &staff_user_2, "nda").await;
    add_to_group(&mut app, &host, &normal_user_1, "nda").await;

    add_to_group(&mut app, &host, &staff_user_1, "revoke-test").await;
    add_to_group(&mut app, &host, &staff_user_2, "revoke-test").await;
    add_to_group(&mut app, &host, &normal_user_1, "revoke-test").await;

    let res = get(&mut app, "/groups/api/v1/members/revoke-test", &host).await;
    assert!(res.status().is_success());
    let members = read_json(res).await;
    assert_eq!(members["members"].as_array().map(|a| a.len()), Some(4));

    let res = delete(
        &mut app,
        &format!("/groups/api/v1/members/nda/{}", user_uuid(&staff_user_2)),
        &host,
    )
    .await;
    assert!(res.status().is_success());
    let res = delete(
        &mut app,
        &format!("/groups/api/v1/members/nda/{}", user_uuid(&normal_user_1)),
        &host,
    )
    .await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/members/revoke-test", &host).await;
    assert!(res.status().is_success());
    let members = read_json(res).await;
    assert_eq!(members["members"].as_array().map(|a| a.len()), Some(3));

    let pool = get_pool();
    staff_user_2.staff_information.staff.value = Some(false);
    update_user_cache(&pool, &staff_user_2, Arc::clone(&cis_client)).await?;

    let res = get(&mut app, "/groups/api/v1/members/revoke-test", &host).await;
    assert!(res.status().is_success());
    let members = read_json(res).await;
    assert_eq!(members["members"].as_array().map(|a| a.len()), Some(2));

    Ok(())
}

#[actix_rt::test]
async fn revoke_staff() -> Result<(), Error> {
    reset()?;
    let (service, cis_client) = test_app_and_cis().await;
    let cis_client = Arc::new(cis_client);
    let app = App::new().service(service);
    let mut app = test::init_service(app).await;

    let host_user = basic_user(1, true);
    let mut staff_user_1 = basic_user(2, true);
    let mut staff_user_2 = basic_user(3, true);
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
        json!({ "name": "revoke-test", "description": "a group", "trust": "Staff" }),
        &host.clone().creator(),
    )
    .await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/groups", &host).await;
    assert!(res.status().is_success());
    assert_eq!(read_json(res).await["groups"][0]["typ"], "Closed");

    add_to_group(&mut app, &host, &staff_user_1, "nda").await;

    add_to_group(&mut app, &host, &staff_user_1, "revoke-test").await;
    add_to_group(&mut app, &host, &staff_user_2, "revoke-test").await;

    let res = get(&mut app, "/groups/api/v1/members/revoke-test", &host).await;
    assert!(res.status().is_success());
    let members = read_json(res).await;
    assert_eq!(members["members"].as_array().map(|a| a.len()), Some(3));

    let pool = get_pool();
    staff_user_2.staff_information.staff.value = Some(false);
    update_user_cache(&pool, &staff_user_2, Arc::clone(&cis_client)).await?;

    let res = get(&mut app, "/groups/api/v1/members/revoke-test", &host).await;
    assert!(res.status().is_success());
    let members = read_json(res).await;
    assert_eq!(members["members"].as_array().map(|a| a.len()), Some(2));

    staff_user_1.staff_information.staff.value = Some(false);
    update_user_cache(&pool, &staff_user_1, Arc::clone(&cis_client)).await?;

    let res = get(&mut app, "/groups/api/v1/members/revoke-test", &host).await;
    assert!(res.status().is_success());
    let members = read_json(res).await;
    assert_eq!(members["members"].as_array().map(|a| a.len()), Some(1));

    Ok(())
}
