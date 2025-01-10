use std::{sync::Arc, time::Duration};

use bb8_postgres::{PostgresConnectionManager, bb8::Pool};
use serenity::prelude::TypeMapKey;
use tokio::sync::Mutex;
use tokio_postgres::Error;
use openssl::ssl::{SslConnector, SslMethod};
use postgres_openssl::MakeTlsConnector;

pub struct PostgresService {
    pub pool: Pool<PostgresConnectionManager<MakeTlsConnector>>,
}

#[derive(Debug)]
pub struct User {
    pub user_id: i32,
    pub username: Option<String>,
    pub discord_username: Option<String>,
    pub discord_mention: Option<String>,
    pub discord_id: i64,
    pub friendly_name: Option<String>,
    pub points: i32,
    pub bbps_issued: i32,
    pub gbps_issued: i32,
    pub rank: Option<i64>
}

#[derive(Debug)]
pub struct LeaderboardUser {
    pub user_id: i32,
    pub discord_username: Option<String>,
    pub discord_mention: Option<String>,
    pub discord_id: i64,
    pub friendly_name: Option<String>,
    pub points: i32,
    pub bbps_issued: i32,
    pub gbps_issued: i32,
    pub rank: i64
}

#[derive(Debug)]
pub  struct HistoryRecord {
    pub  issuer_friendly_name: String,
    pub  description: String,
    pub  timestamp: chrono::NaiveDateTime,
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.user_id == other.user_id
    }
}

impl TypeMapKey for PostgresService {
    type Value = Arc<Mutex<PostgresService>>;
}

impl PostgresService {
    pub async fn new(conn_str: &str) -> Result<PostgresService, Error> {
        let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
        builder.set_verify(openssl::ssl::SslVerifyMode::NONE);

        let manager = PostgresConnectionManager::new(conn_str.parse()?, MakeTlsConnector::new(builder.build()));
        let pool = Pool::builder()
            .retry_connection(true)
            .idle_timeout(Some(Duration::from_secs(86400)))
            .max_size(15)
            .build(manager)
            .await?;

        Ok(PostgresService { pool })
    }

    pub async fn get_user_by_discord_id(&self, discord_id: i64) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>> {
        // Get a connection from the pool
        let conn = self.pool.get().await?;
        if conn.is_closed() {
            print!("Attempted to use a connection that is closed.")
        }
        
        let rows = conn
            .query("SELECT * FROM public.\"Users\" WHERE \"DiscordID\" = $1", &[&discord_id])
            .await?;

        Self::handle_query_result(&rows)
    }

    pub async fn get_user_by_discord_id_with_rank(&self, discord_mention: i64) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.pool.get().await?;
        
        let rows = conn
            .query("SELECT * FROM (
                        SELECT *, RANK() OVER (ORDER BY \"Points\" DESC) AS \"Rank\"
                        FROM public.\"Users\"
                    ) ranked_users
                    WHERE \"DiscordID\" = $1", &[&discord_mention])
            .await?;

        Self::handle_query_result(&rows)
    }

    pub async fn add_user(&self, discord_id: i64, discord_username: &str, friendly_name: &str) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.pool.get().await?;
        let discord_mention = format!("<@{}>", discord_id);
        
        let rows = conn
            .query(
                "INSERT INTO public.\"Users\" (\"DiscordID\",\"DiscordUsername\",\"DiscordMention\",\"FriendlyName\") \
                VALUES ($1,$2,$3,$4)\
                RETURNING *", &[&discord_id, &discord_username, &discord_mention, &friendly_name])
            .await?;

        Self::handle_query_result(&rows)
    }

    pub async fn add_bbp_to_user(&self, target: &User, issuer: &User, description: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let timestamp = chrono::Utc::now().naive_utc();
        let conn = self.pool.get().await?;

        let _ = conn
            .query(
                "INSERT INTO public.\"Bbps\" (\"UserID\", \"Value\", \"Description\", \"Timestamp\", \"IssuerID\") VALUES ($1, 1, $2, $3, $4)",
                &[&target.user_id, &description, &timestamp, &issuer.user_id]
            )
            .await?;
    
        Ok(())
    }

    pub async fn add_gbp_to_user(&self, target: &User, issuer: &User, description: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let timestamp = chrono::Utc::now().naive_utc();
        let conn = self.pool.get().await?;

        let _ = conn
            .query(
                "INSERT INTO public.\"Gbps\" (\"UserID\", \"Value\", \"Description\", \"Timestamp\", \"IssuerID\") VALUES ($1, 1, $2, $3, $4)",
                &[&target.user_id, &description, &timestamp, &issuer.user_id]
            )
            .await?;
    
        Ok(())
    }

    pub async fn forgive_user(&self, target: &User, issuer: &User) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.pool.get().await?;

        let rows = conn
            .query(
                "WITH most_recent AS (
                    SELECT \"BbpID\"
                    FROM public.\"Bbps\"
                    WHERE \"IssuerID\" = $1 AND \"UserID\" = $2 AND \"Forgiven\" = false
                    ORDER BY \"Timestamp\" DESC
                    LIMIT 1
                )
                UPDATE public.\"Bbps\"
                SET \"Forgiven\" = true
                FROM most_recent
                WHERE \"Bbps\".\"BbpID\" = most_recent.\"BbpID\"
                RETURNING *;",
                &[&issuer.user_id, &target.user_id]
            )
            .await?;
    
        match rows.len() {
            0 => Ok(None),
            1 => match rows[0].try_get::<_, Option<String>>("Description") {
                    Ok(description) => Ok(description),
                    Err(_) => Err("Couldn't get the description for the forgive.".into())
                },
            _ => Err("Multiple users found for a single Discord mention".into()),
        }
    }

    pub async fn get_leaderboard(&self) -> Result<Vec<LeaderboardUser>, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.pool.get().await?;

        let rows = conn
            .query(
                "SELECT \"UserID\", \"DiscordUsername\", \"DiscordMention\", \"DiscordID\", \"FriendlyName\", \"Points\", \"BbpsIssued\", \"GbpsIssued\", \"Rank\"
                 FROM (
                     SELECT *, RANK() OVER (ORDER BY \"Points\" DESC) AS \"Rank\"
                     FROM public.\"Users\"
                 ) ranked_users
                 ORDER BY \"Rank\"
                 LIMIT 10",
                &[]
            )
            .await?;

        let leaderboard = rows.iter().map(|row| LeaderboardUser {
            user_id: row.get("UserID"),
            discord_username: row.try_get("DiscordUsername").ok(),
            discord_mention: row.try_get("DiscordMention").ok(),
            discord_id: row.get("DiscordID"),
            friendly_name: row.try_get("FriendlyName").ok(),
            points: row.get("Points"),
            bbps_issued: row.get("BbpsIssued"),
            gbps_issued: row.get("GbpsIssued"),
            rank: row.get("Rank"),
        }).collect();

        Ok(leaderboard)
    }

    pub async fn get_user_history(&self, discord_id: i64) -> Result<Vec<HistoryRecord>, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.pool.get().await?;

        // First, get the UserID from the Users table using the DiscordID
        let user_id_row = conn
            .query_one("SELECT \"UserID\" FROM public.\"Users\" WHERE \"DiscordID\" = $1", &[&discord_id])
            .await?;

        let user_id: i32 = user_id_row.get("UserID");

        // Then, get the history records for the UserID
        let rows = conn
            .query(
                "SELECT u.\"FriendlyName\", b.\"Description\", b.\"Timestamp\"
                 FROM public.\"Bbps\" b
                 JOIN public.\"Users\" u ON b.\"IssuerID\" = u.\"UserID\"
                 WHERE b.\"UserID\" = $1
                 ORDER BY b.\"Timestamp\" DESC
                 LIMIT 10",
                &[&user_id]
            )
            .await?;

        let history = rows.iter().map(|row| {
            HistoryRecord {
                issuer_friendly_name: row.get("FriendlyName"),
                description: row.get("Description"),
                timestamp: row.get("Timestamp"),
            }
        }).collect();

        Ok(history)
    }
    
    fn handle_query_result(rows: &[tokio_postgres::Row]) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>> {
        match rows.len() {
            0 => Ok(None),
            1 => {
                let user = PostgresService::row_to_user(&rows[0]);
                Ok(Some(user))
            },
            _ => Err("Multiple users found for a single Discord mention".into()),
        }
    }

    fn row_to_user(row: &tokio_postgres::Row) -> User {
        User {
            user_id: row.get("UserID"),
            username: None, // This columns isnt currently used
            discord_username: row.try_get("DiscordUsername").ok(),
            discord_mention: row.try_get("DiscordMention").ok(),
            discord_id: row.get("DiscordID"), 
            friendly_name: row.try_get("FriendlyName").ok(),
            points: row.get("Points"),
            bbps_issued: row.get("BbpsIssued"),
            gbps_issued: row.get("GbpsIssued"),
            rank: row.try_get("Rank").ok()
        }
    }

}
