use diesel::{ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl};
use rand::Rng;

use crate::email;
use crate::forms;

table! {
    sessions {
        id -> Integer,
        title -> Text,
        start_time -> Text,
        remaining -> Integer,
    }
}

table! {
    requests (email) {
        session_id -> Integer,
        email -> Text,
        name -> Text,
        identifier -> Integer,
    }
}

table! {
    registrations (email) {
        session_id -> Integer,
        email -> Text,
        name -> Text,
    }
}

#[derive(Debug, Insertable, Queryable, Serialize)]
pub struct Session {
    pub id: i32,
    pub title: String,
    pub start_time: String,
    pub remaining: i32,
}

#[derive(Debug, Insertable, Queryable, Serialize)]
pub struct Request {
    pub session_id: i32,
    pub email: String,
    pub name: String,
    pub identifier: i32,
}

#[derive(Debug, Insertable, Queryable, Serialize)]
pub struct Registration {
    pub session_id: i32,
    pub email: String,
    pub name: String,
}

impl Session {
    pub fn get_results(conn: &diesel::SqliteConnection) -> QueryResult<Vec<Self>> {
        sessions::dsl::sessions.get_results::<Self>(conn)
    }

    pub fn find(id: i32, conn: &diesel::SqliteConnection) -> QueryResult<Self> {
        sessions::dsl::sessions.find(id).first::<Session>(conn)
    }

    pub fn decrement_remaining(id: i32, conn: &diesel::SqliteConnection) -> QueryResult<usize> {
        let current = Self::find(id, conn)?.remaining;

        diesel::update(sessions::dsl::sessions.filter(sessions::dsl::id.eq(&id)))
            .set(sessions::dsl::remaining.eq(current - 1))
            .execute(conn)
    }
}

impl Request {
    pub fn create(data: forms::Register) -> Self {
        Self {
            session_id: data.session_id,
            email: data.email.0,
            name: data.name,
            identifier: rand::thread_rng().gen::<i32>().abs(),
        }
    }

    pub fn insert(&self, conn: &diesel::SqliteConnection) -> QueryResult<usize> {
        // Ensure the session has spaces
        let session = Session::find(self.session_id, conn)?;

        if session.remaining == 0 {
            return Err(diesel::result::Error::NotFound);
        }

        // Email the user
        email::confirm_email_address(&self, &session);

        diesel::insert_into(requests::table)
            .values(self)
            .execute(conn)
    }

    pub fn verify(identifier: i32, conn: &diesel::SqliteConnection) -> QueryResult<usize> {
        // Find the request
        let request: Self = requests::dsl::requests
            .filter(requests::dsl::identifier.eq(&identifier))
            .first(conn)?;
        let session = Session::find(request.session_id, conn)?;

        let registration = Registration::create(request);
        registration.insert(conn)?;

        email::send_confirmation_email(&registration, &session);
        Ok(0)
    }
}

impl Registration {
    pub fn create(data: Request) -> Self {
        Self {
            session_id: data.session_id,
            email: data.email,
            name: data.name,
        }
    }

    pub fn insert(&self, conn: &diesel::SqliteConnection) -> QueryResult<usize> {
        // Ensure the session has spaces
        let remaining = Session::find(self.session_id, conn)?.remaining;

        if remaining == 0 {
            return Err(diesel::result::Error::NotFound);
        }

        diesel::insert_into(registrations::table)
            .values(self)
            .execute(conn)?;

        Session::decrement_remaining(self.session_id, conn)
    }

    pub fn count(session_id: i32, conn: &diesel::SqliteConnection) -> QueryResult<i64> {
        registrations::dsl::registrations
            .filter(registrations::dsl::session_id.eq(&session_id))
            .count()
            .get_result(conn)
    }
}
