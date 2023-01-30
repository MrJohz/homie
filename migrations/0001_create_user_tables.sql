CREATE TABLE
    users (
        id integer primary key autoincrement,
        username text,
        hash text
    );

CREATE TABLE
    tokens (
        token text UNIQUE,
        id integer REFERENCES users (id)
    );