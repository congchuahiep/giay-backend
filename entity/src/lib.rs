//! SeaORM entities.
//!
//! This crate is intentionally a placeholder until entities are generated from
//! the migrated database schema.
//!
//! ```sh
//! sea-orm-cli generate entity \
//!   --database-url postgres://postgres:postgres@localhost:5432/giay \
//!   --output-dir entity/src \
//!   --entity-format dense
//! ```

pub mod prelude;

pub mod ext;
pub mod sea_orm_active_enums;
pub mod user;
pub mod user_session;
