CREATE TABLE store.otp_codes
(
    id         SERIAL      PRIMARY KEY,
    email      TEXT        NOT NULL,
    code       TEXT        NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);