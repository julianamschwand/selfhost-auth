db:
  rm -r data/dev.sqlite3 && sqlite3 data/dev.sqlite3 < src/db/schema.sql
