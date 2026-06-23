# giay-rs

Rust workspace for the Giấy backend using Axum and SeaORM.

> [!IMPORTANT]
> **Dành cho mọi lập trình viên và AI Agent:**
> Vui lòng luôn đọc các tài liệu sau trước khi đóng góp mã nguồn:
>
> - **[Kiến trúc tổng thể (Architecture)](./docs/architecture.md)**
> - **[Hướng dẫn sử dụng SeaORM](./docs/seaorm.md)**
> - **[Kế hoạch triển khai Tenant-Based](./docs/tenant-based-plan.md)**

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

## Generate entities later

After applying migrations, generate SeaORM entities with:

```sh
sea-orm-cli generate entity -o entity/src/models \
    --with-serde both \
    --enum-extra-derives 'utoipa::ToSchema' \
    --enum-extra-attributes 'serde(rename_all = "snake_case")'
```

The `entity` crate is intentionally a placeholder until generated entities are added.
