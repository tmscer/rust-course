-- Your SQL goes here
CREATE TABLE "message"(
    "message_id" BIGSERIAL NOT NULL PRIMARY KEY,
    "public_id" UUID NOT NULL,
    "timestamp" TIMESTAMP NOT NULL,
    "user_nickname" VARCHAR NOT NULL,
    "user_ip" VARCHAR NOT NULL
);

CREATE TABLE "message_text"(
    "message_id" BIGINT NOT NULL PRIMARY KEY,
    "text" TEXT NOT NULL,
    FOREIGN KEY ("message_id") REFERENCES "message"("message_id")
);

CREATE TABLE "message_file"(
    "message_id" BIGINT NOT NULL PRIMARY KEY,
    "filename" VARCHAR NOT NULL,
    "filepath" VARCHAR NOT NULL,
    "length" BIGINT NOT NULL,
    "hash" VARCHAR NOT NULL,
    FOREIGN KEY ("message_id") REFERENCES "message"("message_id")
);
