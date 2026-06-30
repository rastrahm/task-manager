//! Configuración de PostgreSQL a partir de variables de entorno desglosadas.

use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgSslMode};
use sqlx::PgPool;

/// Parámetros de conexión leídos del entorno (`DB_*`).
#[derive(Debug, Clone)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

impl DbConfig {
    /// Carga la configuración desde variables de entorno.
    ///
    /// | Variable      | Obligatoria | Default        |
    /// |---------------|-------------|----------------|
    /// | `DB_HOST`     | no          | `localhost`    |
    /// | `DB_PORT`     | no          | `5432`         |
    /// | `DB_USER`     | no          | `postgres`     |
    /// | `DB_PASSWORD` | no          | `postgre`      |
    /// | `DB_NAME`     | no          | `tasks_db`     |
    pub fn from_env() -> Self {
        Self {
            host: env_or("DB_HOST", "localhost"),
            port: env_or("DB_PORT", "5432")
                .parse()
                .unwrap_or_else(|_| panic!("DB_PORT debe ser un número válido")),
            user: env_or("DB_USER", "postgres"),
            password: env_or("DB_PASSWORD", "postgre"),
            database: env_or("DB_NAME", "tasks_db"),
        }
    }

    fn connect_options(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .username(&self.user)
            .password(&self.password)
            .database(&self.database)
            .ssl_mode(PgSslMode::Prefer)
    }

    /// Abre el pool de conexiones usando los parámetros configurados.
    pub async fn connect_pool(max_connections: u32) -> PgPool {
        let config = Self::from_env();
        Self::connect_with_config(&config, max_connections).await
    }

    /// Pool contra la base de pruebas (`TEST_DB_NAME`, default `tasks_db_test`).
    #[cfg(any(test, feature = "test-utils"))]
    pub async fn connect_test_pool(max_connections: u32) -> PgPool {
        let mut config = Self::from_env();
        config.database = std::env::var("TEST_DB_NAME").unwrap_or_else(|_| "tasks_db_test".into());
        Self::connect_with_config(&config, max_connections).await
    }

    async fn connect_with_config(config: &Self, max_connections: u32) -> PgPool {
        println!(
            "Conectando a PostgreSQL en {}:{}/{} (usuario: {})",
            config.host, config.port, config.database, config.user
        );

        PgPoolOptions::new()
            .max_connections(max_connections)
            .connect_with(config.connect_options())
            .await
            .unwrap_or_else(|error| {
                panic!(
                    "No se pudo conectar a PostgreSQL en {}:{}/{}: {error}",
                    config.host, config.port, config.database
                )
            })
    }
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}
