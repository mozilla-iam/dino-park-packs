use crate::db::db::Pool;
use crate::db::model::*;
use crate::db::schema;
use crate::db::types::*;
use diesel::prelude::*;
use failure::Error;
use uuid::Uuid;

pub fn member_role(pool: &Pool, user_uuid: &Uuid, group_name: &str) -> Result<Role, Error> {
    let connection = pool.get()?;
    schema::memberships::table
        .filter(schema::memberships::user_uuid.eq(user_uuid))
        .inner_join(schema::groups::table)
        .filter(schema::groups::name.eq(group_name))
        .inner_join(schema::roles::table)
        .get_result::<(Membership, Group, Role)>(&connection)
        .map(|(_, _, r)| r)
        .map_err(Into::into)
}
