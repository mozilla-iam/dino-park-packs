use crate::helpers::api::*;
use crate::helpers::db::get_pool;
use crate::helpers::db::reset;
use crate::helpers::misc::read_json;
use crate::helpers::misc::test_app_and_cis;
use crate::helpers::misc::Soa;
use crate::helpers::sudo::add_to_group;
use crate::helpers::users::basic_user;
use crate::helpers::users::user_id;
use actix_web::test;
use actix_web::App;
use cis_client::getby::GetBy;
use cis_client::AsyncCisClientTrait;
use dino_park_packs::db::operations::users::_update_user_cache;
use dino_park_packs::db::operations::users::update_user_cache;
use failure::Error;
use serde_json::json;
use std::sync::Arc;

#[actix_rt::test]
async fn update_inactive() -> Result<(), Error> {
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
        json!({ "name": "inactive-test", "description": "a group", "trust": "Staff" }),
        &host.clone().creator(),
    )
    .await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/groups", &host).await;
    assert!(res.status().is_success());
    assert_eq!(read_json(res).await["groups"][0]["typ"], "Closed");

    add_to_group(&mut app, &host, &staff_user_1, "inactive-test").await;
    add_to_group(&mut app, &host, &staff_user_2, "inactive-test").await;

    let res = get(&mut app, "/groups/api/v1/members/inactive-test", &host).await;
    assert!(res.status().is_success());
    let members = read_json(res).await;
    assert_eq!(members["members"].as_array().map(|a| a.len()), Some(3));

    let pool = get_pool();
    staff_user_2.active.value = Some(false);
    update_user_cache(&pool, &staff_user_2, Arc::clone(&cis_client)).await?;

    let res = get(&mut app, "/groups/api/v1/members/inactive-test", &host).await;
    assert!(res.status().is_success());
    let members = read_json(res).await;
    assert_eq!(members["members"].as_array().map(|a| a.len()), Some(2));

    // updating an inactive profile must not fail
    update_user_cache(&pool, &staff_user_2, Arc::clone(&cis_client)).await?;

    let mut staff_user_2_reactivated = cis_client
        .get_user_by(&user_id(&staff_user_2), &GetBy::Uuid, None)
        .await?;
    // enabling a user again with groups resets groups to db state
    staff_user_2_reactivated.active.value = Some(true);
    update_user_cache(&pool, &staff_user_2_reactivated, Arc::clone(&cis_client)).await?;
    assert_eq!(
        cis_client
            .get_user_by(&user_id(&staff_user_2), &GetBy::Uuid, None)
            .await?
            .access_information
            .mozilliansorg
            .values
            .map(|kv| kv.0.is_empty()),
        Some(true)
    );

    staff_user_1.active.value = Some(false);
    let connection = pool.get()?;
    _update_user_cache(&connection, &staff_user_1)?;

    let res = get(&mut app, "/groups/api/v1/members/inactive-test", &host).await;
    assert!(res.status().is_success());
    let members = read_json(res).await;
    assert_eq!(members["members"].as_array().map(|a| a.len()), Some(2));

    let res = delete(
        &mut app,
        "/groups/api/v1/sudo/user/inactive",
        &host.clone().admin(),
    )
    .await;
    assert!(res.status().is_success());

    let res = get(&mut app, "/groups/api/v1/members/inactive-test", &host).await;
    assert!(res.status().is_success());
    let members = read_json(res).await;
    assert_eq!(members["members"].as_array().map(|a| a.len()), Some(1));

    Ok(())
}
