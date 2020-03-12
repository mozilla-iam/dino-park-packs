use crate::cis::CisFakeClient;
use crate::helpers::get_pool;
use actix_web::middleware::Logger;
use actix_web::test;
use actix_web::web;
use actix_web::App;
use failure::Error;
use std::sync::Arc;

use dino_park_packs::*;

#[actix_rt::test]
async fn basic() -> Result<(), Error> {
    let cis_client = Arc::new(CisFakeClient::new());
    let pool = get_pool();
    let app = App::new()
        .data(Arc::clone(&cis_client))
        .data(pool.clone())
        .wrap(Logger::default().exclude("/healthz"))
        .service(healthz::healthz_app())
        .service(api::internal::internal_app::<CisFakeClient>())
        .service(
            web::scope("/groups/api/v1/")
                .service(api::groups::groups_app::<CisFakeClient>())
                .service(api::members::members_app::<CisFakeClient>())
                .service(api::current::current_app::<CisFakeClient>())
                .service(api::invitations::invitations_app())
                .service(api::terms::terms_app())
                .service(api::users::users_app())
                .service(api::admins::admins_app::<CisFakeClient>())
                .service(api::sudo::sudo_app::<CisFakeClient>()),
        );
    let mut app = test::init_service(app).await;
    let req = test::TestRequest::get().uri("/healthz").to_request();
    let res = test::call_service(&mut app, req).await;
    assert!(res.status().is_success());
    Ok(())
}
