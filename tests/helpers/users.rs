use cis_profile::schema::Display;
use cis_profile::schema::Profile;
use uuid::Uuid;

pub fn basic_user(n: u64, staff: bool) -> Profile {
    let uuid = Uuid::new_v5(&Uuid::NAMESPACE_OID, n.to_string().as_bytes());
    let mut p = Profile::default();
    p.uuid.value = Some(uuid.to_hyphenated().to_string());
    p.uuid.metadata.display = Some(Display::Public);
    p.user_id.value = Some(format!("fire{}", n));
    p.user_id.metadata.display = Some(Display::Public);
    p.primary_username.value = Some(format!("Hans{}", n));
    p.primary_username.metadata.display = Some(Display::Public);
    p.active.value = Some(true);
    p.active.metadata.display = Some(Display::Public);
    p.first_name.value = Some(format!("Hans{}", n));
    p.first_name.metadata.display = Some(Display::Public);
    p.last_name.value = Some(format!("Knall{}", n));
    p.last_name.metadata.display = Some(Display::Public);
    p.primary_email.value = Some(format!("hans{}@knall.org", n));
    p.primary_email.metadata.display = Some(Display::Public);
    if staff {
        p.staff_information.staff.value = Some(true);
        p.staff_information.staff.metadata.display = Some(Display::Public);
    }
    p
}

pub fn user_uuid(p: &Profile) -> String {
    p.uuid.value.clone().unwrap()
}

pub fn user_email(p: &Profile) -> String {
    p.primary_email.value.clone().unwrap()
}
