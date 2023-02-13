-- SPDX-FileCopyrightText: 2023 Jonathan Frere
--
-- SPDX-License-Identifier: MPL-2.0

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