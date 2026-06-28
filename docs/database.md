DATABASE SCHEMA

Database

PostgreSQL

---

Table: links

Purpose: Stores the mapping between a subdomain and its destination URL.

| Column          | Type | Description          |
| --------------- | ---- | -------------------- |
| id              | UUID | Primary key          |
| subdomain       | TEXT | Unique subdomain     |
| destination_url | TEXT | Redirect destination |

SQL

sql
CREATE TABLE links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subdomain TEXT UNIQUE NOT NULL,
    destination_url TEXT NOT NULL
);

---

Table: click_events

Purpose: Stores analytics for every redirect.

| Column      | Type        | Description           |
| ----------- | ----------- | --------------------- |
| id          | UUID        | Primary key           |
| subdomain   | TEXT        | Requested subdomain   |
| clicked_at  | TIMESTAMPTZ | Click timestamp       |
| ip_address  | TEXT        | Visitor IP            |
| referrer    | TEXT        | Referrer URL          |
| device_type | TEXT        | Mobile/Desktop/Tablet |

SQL

sql
CREATE TABLE click_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subdomain TEXT NOT NULL,
    clicked_at TIMESTAMPTZ NOT NULL,
    ip_address TEXT,
    referrer TEXT,
    device_type TEXT
);

---

Relationship

links -> subdomain -> click_events

One link can have many click events.

---

me
├── click #1
├── click #2
└── click #3

info
├── click #1
└── click #2
