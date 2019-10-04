use uuid::Uuid;

pub struct User {
    pub user_uuid: Uuid,
}

impl Default for User {
    fn default() -> Self {
        User {
            user_uuid: Uuid::nil(),
        }
    }
}