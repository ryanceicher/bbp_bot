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

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.user_id == other.user_id
    }
}

impl TypeMapKey for PostgresService {
    type Value = Arc<Mutex<PostgresService>>;
}

// TODO DONT HAVE ALL TABLES IN THIS ONE STRUCT BUT IDC RIGHT NOW
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

    pub async fn get_user_by_discord_mention(&self, discord_mention: &str) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>> {
        // Get a connection from the pool
        let conn = self.pool.get().await?;
        if conn.is_closed() {
            print!("Attempted to use a connection that is closed.")
        }
        
        let rows = conn
            .query("SELECT * FROM public.\"Users\" WHERE \"DiscordMention\" = $1", &[&discord_mention])
            .await?;
    
        match rows.len() {
            0 => Ok(None),
            1 => {
                let user = PostgresService::row_to_user(&rows[0]);
                Ok(Some(user))
            },
            _ => Err("Multiple users found for a single Discord mention".into()),
        }
    }

    pub async fn get_user_by_discord_mention_with_rank(&self, discord_mention: &str) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>> {
        let conn = self.pool.get().await?;
        
        let rows = conn
            .query("SELECT * FROM (
                        SELECT *, RANK() OVER (ORDER BY \"Points\" DESC) AS \"Rank\"
                        FROM public.\"Users\"
                    ) ranked_users
                    WHERE \"DiscordMention\" = $1", &[&discord_mention])
            .await?;
    
        match rows.len() {
            0 => Ok(None),
            1 => {
                let user = PostgresService::row_to_user(&rows[0]);
                Ok(Some(user))
            },
            _ => Err("Multiple users found for a single Discord mention".into()),
        }
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

    fn row_to_user(row: &tokio_postgres::Row) -> User {
        User {
            user_id: row.get("UserID"),
            username: None, // This columns isnt currently used
            discord_username: row.try_get("DiscordUsername").ok(),
            discord_mention: row.try_get("DiscordMention").ok(),
            discord_id: 0, // todo figure out how to deserialize a numeric column to i64
            friendly_name: row.try_get("FriendlyName").ok(),
            points: row.get("Points"),
            bbps_issued: row.get("BbpsIssued"),
            gbps_issued: row.get("GbpsIssued"),
            rank: row.try_get("Rank").ok()
        }
    }

}
