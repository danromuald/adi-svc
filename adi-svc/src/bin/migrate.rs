/// Database migration tool
/// 
/// Runs SQL migrations to set up the database schema.

use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://adi_user:adi_password@localhost:5432/adi_db".to_string());
    
    println!("Connecting to database: {}", database_url.replace(|c: char| c.is_ascii_digit() && database_url.contains("password"), "*"));
    
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    
    println!("Running migrations...");
    
    // Create operations table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS operations (
            operation_id VARCHAR(255) PRIMARY KEY,
            status VARCHAR(50) NOT NULL,
            model_type VARCHAR(100) NOT NULL,
            created_at TIMESTAMPTZ NOT NULL,
            last_updated TIMESTAMPTZ NOT NULL
        )
        "#
    )
    .execute(&pool)
    .await?;
    
    println!("✓ Created operations table");
    
    // Create results table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS results (
            operation_id VARCHAR(255) PRIMARY KEY REFERENCES operations(operation_id) ON DELETE CASCADE,
            model_id VARCHAR(255) NOT NULL,
            api_version VARCHAR(50) NOT NULL,
            content TEXT NOT NULL,
            pages_data JSONB,
            tables_data JSONB,
            key_value_pairs_data JSONB,
            documents_data JSONB,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#
    )
    .execute(&pool)
    .await?;
    
    println!("✓ Created results table");
    
    // Create indexes for better performance
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_operations_status ON operations(status)"
    )
    .execute(&pool)
    .await?;
    
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_operations_created_at ON operations(created_at DESC)"
    )
    .execute(&pool)
    .await?;
    
    println!("✓ Created indexes");
    
    println!("✅ All migrations completed successfully!");
    
    Ok(())
}

