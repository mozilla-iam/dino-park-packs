use crate::helpers::api::*;
use crate::helpers::db::reset;
use crate::helpers::misc::create_nda;
use crate::helpers::misc::test_app_and_cis;
use crate::helpers::misc::Soa;
use crate::helpers::sudo::add_to_group;
use crate::helpers::users::basic_user;
use crate::helpers::users::user_uuid;
use actix_web::test;
use actix_web::App;
use failure::Error;
use serde_json::json;
use std::sync::Arc;

#[actix_rt::test]
async fn join_nda() -> Result<(), Error> {
    reset()?;
    let (service, cis_client) = test_app_and_cis().await;
    let cis_client = Arc::new(cis_client);
    let app = App::new().service(service);
    let mut app = test::init_service(app).await;

    let host_user = basic_user(1, true);
    let staff_user_1 = basic_user(2, true);
    let nda_user_1 = basic_user(11, false);
    let normal_user_1 = basic_user(12, false);
    let host = Soa::from(&host_user).aal_medium();

    create_nda(Arc::clone(&cis_client)).await?;
    add_to_group(&mut app, &host, &nda_user_1, "nda").await;

    let res = post(
        &mut app,
        "/groups/api/v1/groups",
        json!({ "name": "trust-nda", "description": "a group", "trust": "Ndaed" }),
        &host.clone().creator(),
    )
    .await;
    assert!(res.status().is_success());

    let res = post(
        &mut app,
        "/groups/api/v1/invitations/trust-nda",
        json!({ "user_uuid": user_uuid(&normal_user_1) }),
        &host,
    )
    .await;
    assert!(!res.status().is_success());

    let res = post(
        &mut app,
        "/groups/api/v1/invitations/trust-nda",
        json!({ "user_uuid": user_uuid(&nda_user_1) }),
        &host,
    )
    .await;
    assert!(res.status().is_success());

    let res = post(
        &mut app,
        "/groups/api/v1/invitations/trust-nda",
        json!({ "user_uuid": user_uuid(&staff_user_1) }),
        &host,
    )
    .await;
    assert!(res.status().is_success());

    Ok(())
}

#[actix_rt::test]
async fn join_staff() -> Result<(), Error> {
    reset()?;
    let (service, cis_client) = test_app_and_cis().await;
    let cis_client = Arc::new(cis_client);
    let app = App::new().service(service);
    let mut app = test::init_service(app).await;

    let host_user = basic_user(1, true);
    let staff_user_1 = basic_user(2, true);
    let nda_user_1 = basic_user(11, false);
    let normal_user_1 = basic_user(12, false);
    let host = Soa::from(&host_user).aal_medium();

    create_nda(Arc::clone(&cis_client)).await?;
    add_to_group(&mut app, &host, &nda_user_1, "nda").await;

    let res = post(
        &mut app,
        "/groups/api/v1/groups",
        json!({ "name": "trust-staff", "description": "a group", "trust": "Staff" }),
        &host.clone().creator(),
    )
    .await;
    assert!(res.status().is_success());

    let res = post(
        &mut app,
        "/groups/api/v1/invitations/trust-staff",
        json!({ "user_uuid": user_uuid(&normal_user_1) }),
        &host,
    )
    .await;
    assert!(!res.status().is_success());

    let res = post(
        &mut app,
        "/groups/api/v1/invitations/trust-staff",
        json!({ "user_uuid": user_uuid(&nda_user_1) }),
        &host,
    )
    .await;
    assert!(!res.status().is_success());

    let res = post(
        &mut app,
        "/groups/api/v1/invitations/trust-staff",
        json!({ "user_uuid": user_uuid(&staff_user_1) }),
        &host,
    )
    .await;
    assert!(res.status().is_success());

    Ok(())
}
