CREATE TABLE users (
    id string primary key,
    username string,
    hash string
);
CREATE TABLE tokens (
    token string,
    id string REFERENCES users (id)
);
