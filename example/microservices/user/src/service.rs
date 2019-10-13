use crate::user::User;

lazy_static! {
    pub(crate) static ref USER_SERVICE: User = User {
        database: "foo".into()
    };
}
