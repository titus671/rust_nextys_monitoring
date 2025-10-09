use sqlx::{Postgres, Transaction, postgres::PgPoolOptions};

use crate::config::Config;
use crate::nextys::meters::Meters;
use crate::nextys::settings::Settings;
use chrono::Utc;

/// Initialize database connection
pub async fn initialize_connection(config: Config) -> Result<sqlx::Pool<Postgres>, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(
            format!(
                "postgres://{}:{}@{}/{}",
                config.timescaledb.timescaledb_user,
                config.timescaledb.timescaledb_pass,
                config.timescaledb.timescaledb_host,
                config.timescaledb.timescaledb_db
            )
            .as_str(),
        )
        .await?;
    Ok(pool)
}
/// Initialize tables, this should only need to be called once at database creation.
pub async fn initialize_tables(pool: sqlx::Pool<Postgres>) -> Result<(), sqlx::Error> {
    let meta_exists: (bool,) = sqlx::query_as(
        "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables 
                WHERE table_name = 'sensor_metadata')",
    )
    .fetch_one(&pool)
    .await?;

    let hypertable_exists: (bool,) = sqlx::query_as(
        "SELECT EXISTS (
                SELECT 1 FROM _timescaledb_catalog.hypertable
                WHERE table_name = 'sensor_data')",
    )
    .fetch_one(&pool)
    .await?;

    match meta_exists.0 {
        true => {}
        false => {
            let mut tx: Transaction<Postgres> = pool.begin().await?;
            sqlx::query(
                "CREATE TABLE sensor_metadata (
                        id SERIAL PRIMARY KEY,
                        ip_address INET,
                        sysName VARCHAR(50),
                        location VARCHAR(50),
                        batt_low INT,
                        ac_down INT,
                        batt_type INTEGER,
                        charge_voltage REAL,
                        charge_current REAL,
                        float_voltage REAL,
                        low_voltage REAL,
                        deep_discharge_voltage REAL,
                        max_discharge_current REAL,
                        batt_capacity REAL,
                        DCDC_OUTPUT_MODE INTEGER
                        );",
            )
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            println!("Created metadata table");
        }
    }
    // check for hypertable
    match hypertable_exists.0 {
        true => {}
        false => {
            let mut tx: Transaction<Postgres> = pool.begin().await?;
            // Create data table
            sqlx::query(
                "CREATE TABLE sensor_data (
                        time TIMESTAMPTZ NOT NULL,
                        sensor_id INTEGER,
                        input_voltage_min REAL,
                        input_voltage_avg REAL,
                        input_voltage_max REAL,
                        input_current_min REAL,
                        input_current_avg REAL,
                        input_current_max REAL,
                        output_voltage_min REAL,
                        output_voltage_avg REAL,
                        output_voltage_max REAL,
                        output_current_min REAL,
                        output_current_avg REAL,
                        output_current_max REAL,
                        batt_voltage_min REAL,
                        batt_voltage_avg REAL,
                        batt_voltage_max REAL,
                        batt_current_min REAL,
                        batt_current_avg REAL,
                        batt_current_max REAL,
                        batt_soc REAL,
                        batt_int_resistance REAL,
                        batt_charge_capacity REAL,
                        operating_time INTEGER,
                        batt_operating_time INTEGER
                    );",
            )
            .execute(&mut *tx)
            .await?;
            println!("Created data table");
            // create index
            sqlx::query("CREATE INDEX ON sensor_data (sensor_id, time DESC);")
                .execute(&mut *tx)
                .await?;
            // Convert to hypertable
            sqlx::query(
                "SELECT create_hypertable('sensor_data', 'time');
            ",
            )
            .execute(&mut *tx)
            .await?;
            println!("Converted to hypertable");
            // set compression
            sqlx::query(
                "
                ALTER TABLE sensor_data SET (timescaledb.compress,
                    timescaledb.compress_segmentby = 'sensor_id');",
            )
            .execute(&mut *tx)
            .await?;
            // set compression policy
            sqlx::query("SELECT add_compression_policy('sensor_data', INTERVAL '2 days');")
                .execute(&mut *tx)
                .await?;

            tx.commit().await?;
        }
    }
    println!("Meta existence: {:#?}", meta_exists.0);
    println!("Hypertable existence: {:#?}", hypertable_exists.0);
    Ok(())
}
//get/set id
pub async fn get_id(
    pool: sqlx::Pool<sqlx::Postgres>,
    config: &mut Config,
) -> Result<i32, sqlx::Error> {
    match config.device_id {
        Some(id) => Ok(id),
        None => {
            let id: (i32,) = sqlx::query_as(
                "INSERT INTO sensor_metadata (
                    ip_address,
                    sysname,
                    location
                    )
                VALUES ($1, $2, $3)
                RETURNING id;",
            )
            .bind(&config.ip_address)
            .bind(&config.sys_name)
            .bind(&config.location)
            .fetch_one(&pool)
            .await?;
            config.device_id = Some(id.0);
            Ok(id.0)
        }
    }
}
/// upload settings
pub async fn upload_settings(
    pool: sqlx::Pool<sqlx::Postgres>,
    config: &Config,
    settings: &Settings,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "
INSERT INTO sensor_metadata (
    id,
    ip_address,
    sysname,
    location,
    batt_type,
    charge_voltage,
    charge_current,
    float_voltage,
    low_voltage,
    deep_discharge_voltage,
    max_discharge_current,
    batt_capacity
    )
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
    ON CONFLICT (id) DO UPDATE
        SET
            ip_address = $2,
            sysname = $3,
            location = $4,
            batt_type = $5,
            charge_voltage = $6,
            charge_current = $7,
            float_voltage = $8,
            low_voltage = $9,
            deep_discharge_voltage = $10,
            max_discharge_current = $11,
            batt_capacity = $12
",
    )
    .bind(config.device_id)
    .bind(config.ip_address)
    .bind(config.sys_name.as_str())
    .bind(config.location.as_str())
    .bind(settings.batt_type_int)
    .bind(settings.batt_charge_voltage)
    .bind(settings.batt_charge_current)
    .bind(settings.batt_float_voltage)
    .bind(settings.batt_low_voltage)
    .bind(settings.batt_deep_discharge_voltage)
    .bind(settings.batt_max_discharge_current)
    .bind(settings.batt_capacity)
    .execute(&pool)
    .await?;
    Ok(())
}
/// upload metrics
pub async fn upload_metrics(
    pool: &sqlx::Pool<sqlx::Postgres>,
    config: &Config,
    meters: &Meters,
) -> Result<(), sqlx::Error> {
    let now = Utc::now().timestamp_millis();
    sqlx::query(
        "
INSERT INTO sensor_data (
    time,
    sensor_id,
    input_voltage_avg,
    input_current_avg,
    output_voltage_avg,
    output_current_avg,
    batt_voltage_avg,
    batt_current_avg,
    batt_soc,
    batt_int_resistance
    )
VALUES (to_timestamp($1), $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(now)
    .bind(config.device_id)
    .bind(meters.input_voltage)
    .bind(meters.input_current)
    .bind(meters.output_voltage)
    .bind(meters.output_current)
    .bind(meters.batt_voltage)
    .bind(meters.batt_current)
    .bind(meters.batt_soc)
    .bind(meters.batt_int_resistance)
    .execute(pool)
    .await?;
    // Check for ac_down/batt_low
    let ac_down: i32;
    let batt_low: i32;
    if meters.input_voltage <= config.ac_down_threshold {
        ac_down = 1;
    } else {
        ac_down = 0;
    }
    if meters.batt_voltage <= config.low_batt_threshold {
        batt_low = 1;
    } else {
        batt_low = 0;
    }
    sqlx::query(
        "
    UPDATE sensor_metadata
        SET batt_low = $1, ac_down = $2
    WHERE id = $3",
    )
    .bind(batt_low)
    .bind(ac_down)
    .bind(config.device_id)
    .execute(pool)
    .await?;
    Ok(())
}
