use std::time::SystemTime;

use diesel::prelude::*;

use crate::schema::{colors, messages, users};
use libs::password::{hash_password, verify_password};

#[derive(Queryable, Selectable)]
#[diesel(table_name = colors)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Color {
    pub id: i32,
    pub r: i16,
    pub g: i16,
    pub b: i16,
}

impl Color {
    pub fn read(
        connection: &mut PgConnection,
        color_id: i32,
    ) -> Result<Color, diesel::result::Error> {
        use crate::schema::colors::dsl::*;

        Ok(colors
            .filter(id.eq(color_id))
            .limit(1)
            .select(Color::as_select())
            .load(connection)
            .unwrap()
            .pop()
            .unwrap())
    }
}

#[derive(Insertable)]
#[diesel(table_name = colors)]
pub struct ColorNew {
    pub r: i16,
    pub g: i16,
    pub b: i16,
}

impl ColorNew {
    pub fn insert(&self, connection: &mut PgConnection) -> Result<Color, diesel::result::Error> {
        diesel::insert_into(colors::table)
            .values(self)
            .returning(Color::as_returning())
            .get_result(connection)
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Message {
    pub id: i32,
    pub user_id: i32,
    pub text: String,
    pub created_at: SystemTime,
}

impl Message {
    pub fn read(
        connection: &mut PgConnection,
        limit: i64,
    ) -> Result<Vec<Message>, diesel::result::Error> {
        use crate::schema::messages::dsl::*;

        messages
            .limit(limit)
            .select(Message::as_select())
            .load(connection)
    }
}

#[derive(Insertable)]
#[diesel(table_name = messages)]
pub struct MessageNew {
    pub user_id: i32,
    pub text: String,
}

impl MessageNew {
    pub fn insert(&self, connection: &mut PgConnection) -> Result<Message, diesel::result::Error> {
        diesel::insert_into(messages::table)
            .values(self)
            .returning(Message::as_returning())
            .get_result(connection)
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub color_id: Option<i32>,
}

impl User {
    pub fn read(
        connection: &mut PgConnection,
        usernamee: &str,
    ) -> Result<User, diesel::result::Error> {
        use crate::schema::users::dsl::*;

        let x = users
            .filter(username.eq(usernamee))
            .limit(1)
            .select(User::as_select())
            .load(connection);

        match x {
            Ok(mut val) => {
                return Ok(val.pop().unwrap());
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub fn read_by_id(
        connection: &mut PgConnection,
        user_id: i32,
    ) -> Result<User, diesel::result::Error> {
        use crate::schema::users::dsl::*;

        let x = users
            .filter(id.eq(user_id))
            .limit(1)
            .select(User::as_select())
            .load(connection);

        match x {
            Ok(mut val) => {
                return Ok(val.pop().unwrap());
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub fn login(
        connection: &mut PgConnection,
        username: &str,
        password: &str,
    ) -> Result<User, ()> {
        let user = Self::read(connection, username).unwrap();

        if verify_password(password, user.password.as_str()).unwrap() {
            return Ok(user);
        }

        Err(())
    }
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct UserNew {
    username: String,
    password: String,
    color_id: i32,
}

impl UserNew {
    pub fn register(
        connection: &mut PgConnection,
        username: &str,
        password: &str,
        r: u8,
        g: u8,
        b: u8,
    ) -> User {
        let new_color = ColorNew {
            r: r as i16,
            g: g as i16,
            b: b as i16,
        };

        let color = new_color.insert(connection).unwrap();

        let password_hash = hash_password(password).unwrap();

        let new_user = UserNew {
            username: String::from(username),
            password: password_hash,
            color_id: color.id,
        };

        new_user.insert(connection).unwrap()
    }

    fn insert(&self, connection: &mut PgConnection) -> Result<User, diesel::result::Error> {
        diesel::insert_into(users::table)
            .values(self)
            .returning(User::as_returning())
            .get_result(connection)
    }
}
