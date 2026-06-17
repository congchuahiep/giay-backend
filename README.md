# giay-rs

Rust workspace for the Giấy backend using Axum and SeaORM.

## Naming convention

- Tables use plural `snake_case` names, for example `users`, `workspaces`, `workspace_users`, and `pages`.
- Columns use singular `snake_case` names, for example `user_id`, `workspace_id`, `created_at`, and `updated_at`.

This follows the practical default recommended by Bytebase and avoids reserved-word issues such as `user`.

## Setup

Copy the example environment file and adjust values as needed:

```sh
cp .env.example .env
```

## Run migrations

```sh
cargo run -p migration -- up
```

## Run the app

```sh
cargo run
```

The app currently exposes:

```text
GET /health
```

## Generate entities later

After applying migrations, generate SeaORM entities with:

```sh
sea-orm-cli generate entity \
  --database-url postgres://postgres:postgres@localhost:5432/giay \
  --output-dir entity/src \
  --entity-format dense
```

The `entity` crate is intentionally a placeholder until generated entities are added.
