-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

create table posts (
    id uuid Not NULL,
    PRIMARY KEY (id),
    username varchar not null,
    img_url varchar not null unique,
    caption varchar,
    likes integer not null default 0,
    created_at timestamp not null default current_timestamp
);