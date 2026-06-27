CREATE TABLE IF NOT EXISTS links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subdomain TEXT UNIQUE NOT NULL,
    destination_url TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS click_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subdomain TEXT NOT NULL,
    clicked_at TIMESTAMPTZ NOT NULL,
    ip_address TEXT NOT NULL,
    referrer TEXT,
    device_type TEXT NOT NULL
);

INSERT INTO links (subdomain, destination_url)
VALUES
('me', 'https://www.pablomendoza.site'),
('info', 'https://www.pablomendoza.site')
ON CONFLICT (subdomain)
DO UPDATE SET destination_url = EXCLUDED.destination_url;