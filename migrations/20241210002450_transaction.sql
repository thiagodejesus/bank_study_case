-- Add migration script here
CREATE TABLE
    transaction (
        id SERIAL PRIMARY KEY,
        account_id UUID REFERENCES account (id),
        amount BIGINT,
        type VARCHAR(255),
        created_at TIMESTAMP
        WITH
            TIME ZONE DEFAULT NOW ()
    );