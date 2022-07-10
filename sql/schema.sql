drop table reports;

CREATE TABLE if NOT exists accounts
(
    id          INT GENERATED ALWAYS AS IDENTITY,
    api_key     varchar(64) NOT NULL,
    valid_until timestamp   NOT NULL,
    primary key (id)
);



CREATE TABLE if NOT exists reports
(
    id          INT GENERATED ALWAYS AS IDENTITY,
    user_id     integer   NOT NULL,
    account_id  integer   NOT NULL,
    message     text      not null,
    reported_at timestamp NOT NULL default current_date,
    --maybe also banned_at ?
    is_banned   boolean,
    primary key (id),
    CONSTRAINT reported_by
        FOREIGN KEY (account_id) REFERENCES accounts (id)

);

