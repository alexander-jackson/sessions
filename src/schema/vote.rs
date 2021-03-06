//! Allows modifications of the `votes` table in the database.

use std::collections::HashMap;

use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, QueryResult, RunQueryDsl,
};

use crate::schema::candidate::candidates;
use crate::schema::nomination::nominations;

table! {
    /// Represents the schema for `votes`.
    votes (warwick_id, position_id, candidate_id) {
        /// The Warwick identifier of the user voting.
        warwick_id -> Integer,
        /// The position identifier they voted for.
        position_id -> Integer,
        /// The identifier of the candidate they voted for.
        candidate_id -> Integer,
        /// The ranking they gave it.
        ranking -> Integer,
    }
}

/// Represents a row in the `votes` table.
#[derive(Debug, Insertable, Queryable, Serialize)]
pub struct Vote {
    /// The Warwick identifier of the user voting.
    pub warwick_id: i32,
    /// The position identifier they voted for.
    pub position_id: i32,
    /// The identifier of the candidate they voted for.
    pub candidate_id: i32,
    /// The ranking they gave them.
    pub ranking: i32,
}

impl Vote {
    /// Inserts the [`Vote`] into the database.
    pub fn insert(&self, conn: &diesel::PgConnection) -> QueryResult<usize> {
        diesel::insert_into(votes::table).values(self).execute(conn)
    }

    /// Inserts a group of [`Vote`]s into the database.
    pub fn insert_all(
        user_id: i32,
        position_id: i32,
        map: &HashMap<i32, i32>,
        conn: &diesel::PgConnection,
    ) -> QueryResult<usize> {
        // Delete all previous votes to avoid clashes
        diesel::delete(
            votes::dsl::votes.filter(
                votes::dsl::warwick_id
                    .eq(user_id)
                    .and(votes::dsl::position_id.eq(position_id)),
            ),
        )
        .execute(conn)?;

        log::trace!(
            "Deleted all votes for user_id={}, position_id={}",
            user_id,
            position_id
        );

        let votes: Vec<Self> = map
            .iter()
            .map(|(ranking, candidate_id)| Vote {
                warwick_id: user_id,
                position_id,
                candidate_id: *candidate_id,
                ranking: *ranking,
            })
            .collect();

        log::trace!(
            "user_id={} cast the following votes: {:?} for position_id={}",
            user_id,
            map,
            position_id
        );

        diesel::insert_into(votes::table)
            .values(votes)
            .execute(conn)
    }

    /// Gets all [`Vote`] entries in the database.
    pub fn get_results(conn: &diesel::PgConnection) -> QueryResult<Vec<Self>> {
        votes::dsl::votes.get_results::<Self>(conn)
    }

    /// Gets a user's current ballot state, if they have voted.
    pub fn get_current_ballot(
        user_id: i32,
        position_id: i32,
        conn: &diesel::PgConnection,
    ) -> QueryResult<Option<Vec<String>>> {
        // Get their votes for this position
        let votes = votes::dsl::votes
            .filter(
                votes::dsl::warwick_id
                    .eq(user_id)
                    .and(votes::dsl::position_id.eq(position_id)),
            )
            .inner_join(
                nominations::dsl::nominations.on(nominations::dsl::warwick_id
                    .eq(votes::dsl::candidate_id)
                    .and(nominations::dsl::position_id.eq(votes::dsl::position_id))),
            )
            .inner_join(
                candidates::dsl::candidates
                    .on(candidates::dsl::warwick_id.eq(nominations::dsl::warwick_id)),
            )
            .order_by(votes::dsl::ranking)
            .select(candidates::dsl::name)
            .get_results(conn);

        votes.map(|v| if v.is_empty() { None } else { Some(v) })
    }
}

impl From<(i32, i32, i32, i32)> for Vote {
    fn from((position_id, warwick_id, candidate_id, ranking): (i32, i32, i32, i32)) -> Self {
        Self {
            warwick_id,
            position_id,
            candidate_id,
            ranking,
        }
    }
}
