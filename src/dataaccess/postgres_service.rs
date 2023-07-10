use std::sync::Arc;

use serenity::prelude::TypeMapKey;
use tokio::sync::Mutex;
use tokio_postgres::{Client, Error};
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use postgres_openssl::MakeTlsConnector;

pub struct PostgresService {
    pub client: Client,
}

#[derive(Debug)]
pub struct User {
    pub user_id: i32,
    pub username: Option<String>,
    pub discord_username: Option<String>,
    pub discord_mention: Option<String>,
    pub discord_id: i64,
    pub friendly_name: Option<String>,
}

impl TypeMapKey for PostgresService {
    type Value = Arc<Mutex<PostgresService>>;
}

// TODO DONT HAVE ALL TABLES IN THIS ONE STRUCT BUT IDC RIGHT NOW
impl PostgresService {
    pub async fn new(conn_str: &str) -> Result<PostgresService, Error> {
        let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
        builder.set_verify(SslVerifyMode::NONE); // This line disables certificate verification
        let connector = MakeTlsConnector::new(builder.build());

        let (client, connection) = tokio_postgres::connect(conn_str, connector).await?;

        // The connection object performs the actual communication with the PostgresService,
        // so spawn it off to run on its own.
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        Ok(PostgresService { client })
    }

    pub async fn _get_users(&self) -> Result<Vec<User>, Error> {
        let rows = self
            .client
            .query("SELECT * FROM public.\"Users\"", &[])
            .await?;

        let mut users = Vec::new();
        for row in rows {
            users.push(PostgresService::row_to_user(&row));
        }

        Ok(users)
    }

    pub async fn get_user_by_discord_mention(&self, discord_mention: &str) -> Result<Option<User>, Box<dyn std::error::Error + Send + Sync>> {
        let rows = self
            .client
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

    pub async fn add_bbp_to_user(&self, target: &User, issuer: &User, description: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let timestamp = chrono::Utc::now().naive_utc();
    
        let _ = self
            .client
            .query(
                "INSERT INTO public.\"Bbps\" (\"UserID\", \"Value\", \"Description\", \"Timestamp\", \"IssuerID\") VALUES ($1, 1, $2, $3, $4)",
                &[&target.user_id, &description, &timestamp, &issuer.user_id]
            )
            .await?;
    
        Ok(())
    }

    pub async fn add_gbp_to_user(&self, target: &User, issuer: &User, description: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let timestamp = chrono::Utc::now().naive_utc();
    
        let _ = self
            .client
            .query(
                "INSERT INTO public.\"Gbps\" (\"UserID\", \"Value\", \"Description\", \"Timestamp\", \"IssuerID\") VALUES ($1, 1, $2, $3, $4)",
                &[&target.user_id, &description, &timestamp, &issuer.user_id]
            )
            .await?;
    
        Ok(())
    }

    fn row_to_user(row: &tokio_postgres::Row) -> User {
        User {
            user_id: row.get("UserID"),
            username: row.try_get("Username").ok(),
            discord_username: row.try_get("DiscordUsername").ok(),
            discord_mention: row.try_get("DiscordMention").ok(),
            discord_id: 0, // todo figure out how to deserialize a numeric column to i64
            friendly_name: row.try_get("FriendlyName").ok(),
        }
    }

}
