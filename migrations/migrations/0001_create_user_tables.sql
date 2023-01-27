CREATE TABLE users (
    username string primary key,
    hash string
);
CREATE TABLE tokens (
    token string,
    username string REFERENCES users (username)
);
