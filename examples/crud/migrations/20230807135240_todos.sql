CREATE TABLE IF NOT EXISTS todos (
    id INTEGER PRIMARY KEY NOT NULL,
    description TEXT NOT NULL,
    created TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    done BOOLEAN NOT NULL DEFAULT 0
);