use crate::helpers::api::*;
use crate::helpers::db::get_pool;
use crate::helpers::db::reset;
use crate::helpers::misc::test_app_and_cis;
use crate::helpers::misc::Soa;
use crate::helpers::users::basic_user;
use crate::helpers::users::user_uuid;
use actix_web::test;
use actix_web::App;
use cis_profile::schema::Display;
use dino_park_packs::db::operations::members::get_anonymous_member_emails;
use dino_park_packs::db::operations::users::update_user_cache;
use failure::Error;
use serde_json::json;
use std::sync::Arc;

#[actix_rt::test]
async fn hidden_member_emails() -> Result<(), Error> {
    reset()?;
    let (service, cis_client) = test_app_and_cis().await;
    let cis_client = Arc::new(cis_client);
    let app = App::new().service(service);
    let mut app = test::init_service(app).await;
    let user = basic_user(1, true);
    let creator = Soa::from(&user).creator().aal_medium();

    let res = post(
        &mut app,
        "/groups/api/v1/groups",
        json!({ "name": "anonymous", "description": "a group with a hidden member" }),
        &creator,
    )
    .await;
    assert!(res.status().is_success());

    let admin = Soa::from(&user).admin().aal_medium();
    let user2 = basic_user(2, true);
    let res = post(
        &mut app,
        "/groups/api/v1/sudo/member/anonymous",
        json!({ "user_uuid": user_uuid(&user2), "group_expiration": 2 }),
        &admin,
    )
    .await;
    assert!(res.status().is_success());

    let pool = get_pool();

    let emails = get_anonymous_member_emails(&pool, &admin.clone().into())?;
    assert!(emails.is_empty());

    let mut user3 = basic_user(3, true);
    let res = post(
        &mut app,
        "/groups/api/v1/sudo/member/anonymous",
        json!({ "user_uuid": user_uuid(&user3), "group_expiration": 2 }),
        &admin,
    )
    .await;
    assert!(res.status().is_success());

    user3.primary_email.metadata.display = Some(Display::Staff);
    update_user_cache(&pool, &user3, Arc::clone(&cis_client)).await?;
    let emails = get_anonymous_member_emails(&pool, &admin.clone().into())?;
    assert_eq!(emails, vec![String::from("hans3@knall.org")]);

    user3.primary_email.metadata.display = Some(Display::Ndaed);
    update_user_cache(&pool, &user3, Arc::clone(&cis_client)).await?;
    let emails = get_anonymous_member_emails(&pool, &admin.clone().into())?;
    assert!(emails.is_empty());

    user3.primary_email.metadata.display = Some(Display::Ndaed);
    user3.first_name.metadata.display = Some(Display::Staff);
    user3.last_name.metadata.display = Some(Display::Staff);
    update_user_cache(&pool, &user3, Arc::clone(&cis_client)).await?;
    let emails = get_anonymous_member_emails(&pool, &admin.clone().into())?;
    assert_eq!(emails, vec![String::from("hans3@knall.org")]);

    user3.primary_email.metadata.display = Some(Display::Ndaed);
    user3.first_name.metadata.display = Some(Display::Ndaed);
    user3.last_name.metadata.display = Some(Display::Staff);
    update_user_cache(&pool, &user3, Arc::clone(&cis_client)).await?;
    let emails = get_anonymous_member_emails(&pool, &admin.clone().into())?;
    assert!(emails.is_empty());

    user3.primary_email.metadata.display = Some(Display::Ndaed);
    user3.first_name.metadata.display = Some(Display::Staff);
    user3.last_name.metadata.display = Some(Display::Ndaed);
    update_user_cache(&pool, &user3, Arc::clone(&cis_client)).await?;
    let emails = get_anonymous_member_emails(&pool, &admin.into())?;
    assert!(emails.is_empty());

    Ok(())
}
