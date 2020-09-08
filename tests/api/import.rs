use crate::helpers::api::*;
use crate::helpers::db::get_pool;
use crate::helpers::db::reset;
use crate::helpers::misc::read_json;
use crate::helpers::misc::test_app_and_cis;
use crate::helpers::misc::Soa;
use crate::helpers::users::basic_user;
use actix_web::test;
use actix_web::App;
use csv::ReaderBuilder;
use dino_park_packs::db::types::TrustType;
use dino_park_packs::import::ops::*;
use dino_park_packs::import::tsv::MozilliansGroup;
use dino_park_packs::import::tsv::MozilliansGroupCurator;
use dino_park_packs::import::tsv::MozilliansGroupMembership;
use failure::format_err;
use failure::Error;
use std::sync::Arc;

fn get_members() -> Result<Vec<MozilliansGroupMembership>, Error> {
    let mut rdr = ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path("tests/data/import-test/m.tsv")?;

    rdr.deserialize()
        .collect::<Result<Vec<MozilliansGroupMembership>, csv::Error>>()
        .map_err(Into::into)
}

fn get_curators() -> Result<Vec<MozilliansGroupCurator>, Error> {
    let mut rdr = ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path("tests/data/import-test/c.tsv")?;

    rdr.deserialize()
        .collect::<Result<Vec<MozilliansGroupCurator>, csv::Error>>()
        .map_err(Into::into)
}

fn get_group() -> Result<MozilliansGroup, Error> {
    let mut rdr = ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path("tests/data/import-test/g.tsv")?;

    rdr.deserialize()
        .collect::<Result<Vec<MozilliansGroup>, csv::Error>>()?
        .pop()
        .ok_or_else(|| format_err!(""))
}

#[actix_rt::test]
async fn import() -> Result<(), Error> {
    reset()?;
    let (service, cis_client) = test_app_and_cis().await;
    let cis_client = Arc::new(cis_client);
    let app = App::new().service(service);
    let mut app = test::init_service(app).await;
    let creator = Soa::from(&basic_user(1, true)).creator().aal_medium();
    let group = get_group()?;
    let curators = get_curators()?;
    let members = get_members()?;
    let connection = get_pool().get()?;
    import_group(&connection, group, TrustType::Authenticated)?;
    import_curators(
        &connection,
        "import-test",
        curators,
        TrustType::Authenticated,
        cis_client.clone(),
    )
    .await?;

    let res = get(&mut app, "/groups/api/v1/groups", &creator).await;
    assert!(res.status().is_success());
    assert_eq!(read_json(res).await["groups"][0]["name"], "import-test");

    let res = get(
        &mut app,
        "/groups/api/v1/groups/import-test/details",
        &creator,
    )
    .await;
    assert!(res.status().is_success());
    let j = read_json(res).await;
    assert_eq!(j["group"]["terms"], true);
    assert_eq!(
        j["group"]["description"],
        "import test group\n\n**Website:** [https://example.com/](https://example.com/)"
    );

    let res = get(
        &mut app,
        "/groups/api/v1/invitations/import-test/email",
        &creator,
    )
    .await;
    assert!(res.status().is_success());
    assert_eq!(read_json(res).await["body"], "some \ninvitation email");

    let res = get(
        &mut app,
        "/groups/api/v1/members/import-test?r=Curator",
        &creator,
    )
    .await;
    assert!(res.status().is_success());
    assert_eq!(
        read_json(res).await["members"].as_array().map(|a| a.len()),
        Some(3)
    );

    let res = get(
        &mut app,
        "/groups/api/v1/members/import-test?r=Member",
        &creator,
    )
    .await;
    assert!(res.status().is_success());
    assert_eq!(
        read_json(res).await["members"].as_array().map(|a| a.len()),
        Some(0)
    );

    import_members(
        &connection,
        "import-test",
        members,
        TrustType::Authenticated,
        cis_client.clone(),
    )
    .await?;

    let res = get(
        &mut app,
        "/groups/api/v1/members/import-test?r=Curator",
        &creator,
    )
    .await;
    assert!(res.status().is_success());
    assert_eq!(
        read_json(res).await["members"].as_array().map(|a| a.len()),
        Some(3)
    );

    let res = get(
        &mut app,
        "/groups/api/v1/members/import-test?r=Member",
        &creator,
    )
    .await;
    assert!(res.status().is_success());
    assert_eq!(
        read_json(res).await["members"].as_array().map(|a| a.len()),
        Some(5)
    );

    let res = get(
        &mut app,
        "/groups/api/v1/groups/import-test/details",
        &creator,
    )
    .await;
    assert!(res.status().is_success());
    let j = read_json(res).await;
    assert_eq!(j["group"]["created"], "2011-03-22T00:00:00Z");

    Ok(())
}

#[actix_rt::test]
async fn import_staff_only() -> Result<(), Error> {
    reset()?;
    let (service, cis_client) = test_app_and_cis().await;
    let cis_client = Arc::new(cis_client);
    let app = App::new().service(service);
    let mut app = test::init_service(app).await;
    let creator = Soa::from(&basic_user(1, true)).creator().aal_medium();
    let group = get_group()?;
    let curators = get_curators()?;
    let members = get_members()?;
    let connection = get_pool().get()?;
    import_group(&connection, group, TrustType::Staff)?;
    import_curators(
        &connection,
        "import-test",
        curators,
        TrustType::Staff,
        cis_client.clone(),
    )
    .await?;

    let res = get(&mut app, "/groups/api/v1/groups", &creator).await;
    assert!(res.status().is_success());
    assert_eq!(read_json(res).await["groups"][0]["name"], "import-test");

    let res = get(
        &mut app,
        "/groups/api/v1/groups/import-test/details",
        &creator,
    )
    .await;
    assert!(res.status().is_success());
    let j = read_json(res).await;
    assert_eq!(j["group"]["terms"], true);
    assert_eq!(
        j["group"]["description"],
        "import test group\n\n**Website:** [https://example.com/](https://example.com/)"
    );

    let res = get(
        &mut app,
        "/groups/api/v1/invitations/import-test/email",
        &creator,
    )
    .await;
    assert!(res.status().is_success());
    assert_eq!(read_json(res).await["body"], "some \ninvitation email");

    let res = get(
        &mut app,
        "/groups/api/v1/members/import-test?r=Curator",
        &creator,
    )
    .await;
    assert!(res.status().is_success());
    assert_eq!(
        read_json(res).await["members"].as_array().map(|a| a.len()),
        Some(2)
    );

    let res = get(
        &mut app,
        "/groups/api/v1/members/import-test?r=Member",
        &creator,
    )
    .await;
    assert!(res.status().is_success());
    assert_eq!(
        read_json(res).await["members"].as_array().map(|a| a.len()),
        Some(0)
    );

    import_members(
        &connection,
        "import-test",
        members,
        TrustType::Staff,
        cis_client.clone(),
    )
    .await?;

    let res = get(
        &mut app,
        "/groups/api/v1/members/import-test?r=Curator",
        &creator,
    )
    .await;
    assert!(res.status().is_success());
    assert_eq!(
        read_json(res).await["members"].as_array().map(|a| a.len()),
        Some(2)
    );

    let res = get(
        &mut app,
        "/groups/api/v1/members/import-test?r=Member",
        &creator,
    )
    .await;
    assert!(res.status().is_success());
    assert_eq!(
        read_json(res).await["members"].as_array().map(|a| a.len()),
        Some(4)
    );

    let res = get(
        &mut app,
        "/groups/api/v1/groups/import-test/details",
        &creator,
    )
    .await;
    assert!(res.status().is_success());
    let j = read_json(res).await;
    assert_eq!(j["group"]["created"], "2011-03-22T00:00:00Z");

    let group = get_group()?;
    import_group(&connection, group, TrustType::Staff)?;
    Ok(())
}
