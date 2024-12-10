-- Add migration script here
CREATE TABLE
    account (
        id UUID PRIMARY KEY,
        number BIGINT UNIQUE NOT NULL
    );
