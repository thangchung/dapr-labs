-- COUNTER

START TRANSACTION;

CREATE SCHEMA IF NOT EXISTS "order";

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE
    "order".orders (
        id uuid NOT NULL DEFAULT (uuid_generate_v4()),
        order_source integer NOT NULL,
        loyalty_member_id uuid NOT NULL,
        order_status integer NOT NULL,
        updated timestamp
        with
            time zone NULL,
            CONSTRAINT pk_orders PRIMARY KEY (id)
    );

CREATE TABLE
    "order".line_items (
        id uuid NOT NULL DEFAULT (uuid_generate_v4()),
        item_type integer NOT NULL,
        name text NOT NULL,
        price numeric NOT NULL,
        item_status integer NOT NULL,
        is_barista_order boolean NOT NULL,
        order_id uuid NULL,
        created timestamp
        with
            time zone NOT NULL DEFAULT (now()),
            updated timestamp
        with
            time zone NULL,
            CONSTRAINT pk_line_items PRIMARY KEY (id),
            CONSTRAINT fk_line_items_orders_order_temp_id FOREIGN KEY (order_id) REFERENCES "order".orders (id)
    );

CREATE UNIQUE INDEX ix_line_items_id ON "order".line_items (id);

CREATE INDEX ix_line_items_order_id ON "order".line_items (order_id);

CREATE UNIQUE INDEX ix_orders_id ON "order".orders (id);

COMMIT;

--  BARISTA
START TRANSACTION;

CREATE SCHEMA IF NOT EXISTS "barista";

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE
    barista.barista_orders (
        id uuid NOT NULL DEFAULT (uuid_generate_v4()),
        item_type integer NOT NULL,
        item_name text NOT NULL,
        time_up timestamp
        with
            time zone NOT NULL,
            created timestamp
        with
            time zone NOT NULL DEFAULT (now()),
            updated timestamp
        with
            time zone NULL,
            CONSTRAINT pk_barista_orders PRIMARY KEY (id)
    );

CREATE UNIQUE INDEX ix_barista_orders_id ON barista.barista_orders (id);

COMMIT;

-- KITCHEN
START TRANSACTION;

CREATE SCHEMA IF NOT EXISTS "kitchen";

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE
    kitchen.kitchen_orders (
        id uuid NOT NULL DEFAULT (uuid_generate_v4()),
        order_id uuid NOT NULL,
        item_type integer NOT NULL,
        item_name text NOT NULL,
        time_up timestamp
        with
            time zone NOT NULL,
            created timestamp
        with
            time zone NOT NULL DEFAULT (now()),
            updated timestamp
        with
            time zone NULL,
            CONSTRAINT pk_kitchen_orders PRIMARY KEY (id)
    );

CREATE UNIQUE INDEX ix_kitchen_orders_id ON kitchen.kitchen_orders (id);

COMMIT;