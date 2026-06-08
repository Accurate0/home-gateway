create table android_push_tokens (
    token text primary key,
    created_at timestamptz not null default now(),
    updated_at timestamptz not null default now()
);
