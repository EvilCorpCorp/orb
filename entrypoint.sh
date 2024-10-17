#!/bin/sh
set -e

if [ ! -d "$PATH_DATA" ]; then
echo "Initializing PostgreSQL database at $PATH_DATA $POSTGRES_USER..."
  su-exec postgres initdb "$PATH_DATA"

  # Create a PostgreSQL user and set the password
  echo "Creating PostgreSQL user: $POSTGRES_USER"
  psql --username=postgres <<-EOSQL
      CREATE USER "$POSTGRES_USER" WITH SUPERUSER PASSWORD '$POSTGRES_PASSWORD';
      CREATE DATABASE "$POSTGRES_DB" OWNER "$POSTGRES_USER";
EOSQL

  # Stop the PostgreSQL instance after initialization
  su-exec postgres pg_ctl "$PATH_DATA" -m fast -w stop

fi

exec su-exec postgres "$PATH_DATA"
