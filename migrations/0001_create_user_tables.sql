CREATE TABLE
    users (id text primary key, username text, hash text);

CREATE TABLE
    tokens (token text UNIQUE, id text REFERENCES users (id));