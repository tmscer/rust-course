-- This file should undo anything in `up.sql`
ALTER TABLE
    "message_file" DROP CONSTRAINT "message_file_message_id_fkey";

ALTER TABLE
    "message_file"
ADD
    CONSTRAINT "message_file_message_id_fkey" FOREIGN KEY ("message_id") REFERENCES "message"("message_id");

ALTER TABLE
    "message_text" DROP CONSTRAINT "message_text_message_id_fkey";

ALTER TABLE
    "message_text"
ADD
    CONSTRAINT "message_text_message_id_fkey" FOREIGN KEY ("message_id") REFERENCES "message"("message_id");
