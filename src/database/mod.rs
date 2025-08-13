use sqlx::{Postgres, Transaction, postgres::PgPoolOptions};

pub async fn initialise_database() -> Result<sqlx::Pool<sqlx::Postgres>, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost/postgres")
        .await?;

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
    Ok(pool)
}
