//! Allows modifications of the `registrations` table in the database.

use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl};

use crate::email;
use crate::forms;
use crate::guards::AuthorisedUser;
use crate::schema::{custom_types, sessions, Session};
use crate::session_window::SessionWindow;

table! {
    /// Represents the schema for `registrations`.
    registrations (warwick_id) {
        /// The identifier for the session.
        session_id -> Integer,
        /// The user's Warwick ID.
        warwick_id -> Integer,
        /// The user's name.
        name -> Text,
    }
}

/// Represents a row in the `registrations` table.
#[derive(Debug, Insertable, Queryable, Serialize)]
pub struct Registration {
    /// The identifier for the session
    pub session_id: i32,
    /// The user's Warwick ID
    pub warwick_id: i32,
    /// The user's name
    pub name: String,
}

impl Registration {
    /// Inserts the [`Registration`] into the database.
    ///
    /// This fails if the session has no remaining places, and sends the user a confirmation email
    /// upon success. It also decrements the number of remaining places for the given session.
    pub fn insert(&self, conn: &diesel::SqliteConnection) -> QueryResult<usize> {
        // Ensure the session has spaces
        let session = Session::find(self.session_id, conn)?;

        if session.remaining == 0 {
            return Err(diesel::result::Error::NotFound);
        }

        diesel::insert_into(registrations::table)
            .values(self)
            .execute(conn)?;

        email::send_confirmation(&self, &session);

        Session::decrement_remaining(self.session_id, conn)
    }

    /// Deletes a user's registration from the database if it exists.
    pub fn cancel(
        warwick_id: i32,
        session_id: i32,
        conn: &diesel::SqliteConnection,
    ) -> QueryResult<usize> {
        let filter = registrations::dsl::warwick_id
            .eq(warwick_id)
            .and(registrations::dsl::session_id.eq(session_id));

        Session::increment_remaining(session_id, conn)?;

        diesel::delete(registrations::table.filter(filter)).execute(conn)
    }

    /// Gets the number of registrations for a given session.
    pub fn count(session_id: i32, conn: &diesel::SqliteConnection) -> QueryResult<i64> {
        registrations::dsl::registrations
            .filter(registrations::dsl::session_id.eq(&session_id))
            .count()
            .get_result(conn)
    }

    /// Gets the session data and names of those registered for all sessions in the database.
    pub fn get_registration_list(
        conn: &diesel::SqliteConnection,
        window: SessionWindow,
    ) -> QueryResult<Vec<(i32, custom_types::DateTime, String, String)>> {
        let columns = (
            sessions::dsl::id,
            sessions::dsl::start_time,
            sessions::dsl::title,
            registrations::dsl::name,
        );
        let ordering = (sessions::dsl::start_time, sessions::dsl::id);
        let filter = sessions::dsl::start_time
            .gt(window.start)
            .and(sessions::dsl::start_time.lt(window.end));

        registrations::dsl::registrations
            .inner_join(sessions::dsl::sessions)
            .select(columns)
            .filter(filter)
            .order_by(ordering)
            .load(conn)
    }

    /// Gets all the sessions that a user has booked.
    pub fn get_user_bookings(
        id: i32,
        window: SessionWindow,
        conn: &diesel::SqliteConnection,
    ) -> QueryResult<Vec<Session>> {
        let columns = sessions::all_columns;
        let ordering = (sessions::dsl::start_time, sessions::dsl::id);
        let filter = sessions::dsl::start_time
            .gt(window.start)
            .and(sessions::dsl::start_time.lt(window.end))
            .and(registrations::dsl::warwick_id.eq(id));

        registrations::dsl::registrations
            .inner_join(sessions::dsl::sessions)
            .select(columns)
            .filter(filter)
            .order_by(ordering)
            .load(conn)
    }
}

impl From<(AuthorisedUser, forms::Register)> for Registration {
    /// Creates a new [`Registration`] assuming the user is already verified.
    fn from(data: (AuthorisedUser, forms::Register)) -> Self {
        Self {
            session_id: data.1.session_id,
            warwick_id: data.0.id,
            name: data.0.name,
        }
    }
}
