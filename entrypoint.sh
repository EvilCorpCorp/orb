#!/bin/bash
set -e

# Function to execute SQL
function exec_sql() {
  psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" -c "$1"
}

wait_for_postgres() {
  until pg_isready -q; do
    echo "Waiting for PostgreSQL..."
    sleep 1
  done
}

if [ -z "$(ls -A $PGDATA)" ]; then
  echo "Data directory is empty. Initializing database..."
  initdb -D $PGDATA
else
  echo "Data directory already initialized. Skipping initdb."
fi

pg_ctl -D "$PGDATA" -o "-c listen_addresses=''" -w start
# Initialisation si nécessaire
if ! psql -lqt | cut -d \| -f 1 | grep -qw "$POSTGRES_DB"; then
  echo "Initializing PostgreSQL database..."
  # Start PostgreSQL temporarily to create the table

  wait_for_postgres
  # Get the table name from the environment variable
  TABLE_NAME="${DB_TABLE_NAME:-transactions}"

  createdb $POSTGRES_DB

  # Create the table
  exec_sql "CREATE TABLE IF NOT EXISTS $TABLE_NAME (
        id SERIAL PRIMARY KEY,
        from_address VARCHAR(255) NOT NULL,
        to_address VARCHAR(255) NOT NULL,
        value VARCHAR(255) NOT NULL,
        nonce BIGINT NOT NULL,
        gas_price VARCHAR(255) NOT NULL,
        gas_limit BIGINT NOT NULL,
        data TEXT,
        gas_priority_fee VARCHAR(255),
        max_fee_per_blob_gas VARCHAR(255)
    );"

  echo "Table $TABLE_NAME created successfully."

  # Stop PostgreSQL after creating the table

  # Modify postgresql.conf to listen on all addresses
  echo "listen_addresses='*'" >>"$PGDATA/postgresql.conf"

  # Modify pg_hba.conf to allow connections from all addresses
  echo "host all all 0.0.0.0/0 md5" >>"$PGDATA/pg_hba.conf"
fi
pg_ctl -D "$PGDATA" -m fast -w stop

# Start PostgreSQL in the foreground
echo "Starting PostgreSQL..."

exec postgres
