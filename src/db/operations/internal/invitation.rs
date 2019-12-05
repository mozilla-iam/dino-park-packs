use crate::db::db::Pool;
use crate::db::model::*;
use crate::db::operations::internal;
use crate::db::schema;
use crate::db::schema::groups::dsl as groups;
use crate::user::User;
use chrono::NaiveDateTime;
use diesel::dsl::count;
use diesel::prelude::*;
use failure::Error;

pub fn invite(
    pool: &Pool,
    group_name: &str,
    host: User,
    member: User,
    invitation_expiration: Option<NaiveDateTime>,
    group_expiration: Option<NaiveDateTime>,
) -> Result<(), Error> {
    let connection = pool.get()?;
    let group = groups::groups
        .filter(groups::name.eq(group_name))
        .first::<Group>(&*connection)?;
    let invitation = Invitation {
        user_uuid: member.user_uuid,
        group_id: group.id.clone(),
        invitation_expiration,
        group_expiration,
        added_by: host.user_uuid,
    };
    diesel::insert_into(schema::invitations::table)
        .values(&invitation)
        .execute(&*connection)
        .map(|_| ())
        .map_err(Error::from)
}

pub fn pending_count(pool: &Pool, group_name: &str) -> Result<i64, Error> {
    let connection = pool.get()?;
    let count = schema::invitations::table
        .inner_join(groups::groups)
        .filter(groups::name.eq(group_name))
        .select(count(schema::invitations::user_uuid))
        .first(&connection)?;
    Ok(count)
}

pub fn pending(pool: &Pool, group_name: &str) -> Result<Vec<Invitation>, Error> {
    let connection = pool.get()?;
    let group = groups::groups
        .filter(groups::name.eq(group_name))
        .first::<Group>(&*connection)?;
    Invitation::belonging_to(&group)
        .get_results(&connection)
        .map_err(Into::into)
}

pub fn pending_for_user(pool: &Pool, user: &User) -> Result<Vec<Invitation>, Error> {
    let connection = pool.get()?;
    schema::invitations::table
        .filter(schema::invitations::user_uuid.eq(user.user_uuid))
        .get_results(&connection)
        .map_err(Into::into)
}

pub fn accept(pool: &Pool, group_name: &str, member: &User) -> Result<(), Error> {
    let connection = pool.get()?;
    let group = groups::groups
        .filter(groups::name.eq(group_name))
        .first::<Group>(&*connection)?;
    let invitation = schema::invitations::table
        .filter(
            schema::invitations::user_uuid
                .eq(member.user_uuid)
                .and(schema::invitations::group_id.eq(group.id)),
        )
        .first::<Invitation>(&connection)?;
    let role = internal::member::member_role(pool, group_name)?;
    let membership = InsertMembership {
        group_id: invitation.group_id,
        user_uuid: invitation.user_uuid,
        role_id: role.id,
        expiration: invitation.group_expiration,
        added_by: invitation.added_by,
    };
    diesel::insert_into(schema::memberships::table)
        .values(&membership)
        .on_conflict((
            schema::memberships::user_uuid,
            schema::memberships::group_id,
        ))
        .do_update()
        .set(&membership)
        .execute(&*connection)?;
    diesel::delete(schema::invitations::table)
        .filter(
            schema::invitations::user_uuid
                .eq(member.user_uuid)
                .and(schema::invitations::group_id.eq(group.id)),
        )
        .execute(&connection)?;
    Ok(())
}
