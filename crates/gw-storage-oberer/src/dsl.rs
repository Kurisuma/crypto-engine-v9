#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::unwrap_used)]

//! Zero-Raw-SQL Schema DSL module.
//! Exposes oberer-orm's native compile-time type-enforced schema, index, view, and trigger builders.

pub use oberer_orm::query::{BoundExpr, Column, Expr, TablePredicate};
pub use oberer_orm::schema::table_builder::{TableExpr, TableSchemaQueryBuilder};
pub use oberer_orm::schema::trigger_builder::{OrmTriggerDef, TriggerQueryBuilder};
pub use oberer_orm::schema::trigger_dsl::{
    New, Old, TriggerAction, TriggerDelete, TriggerExpr, TriggerInsert, TriggerStatement,
    TriggerUpdate,
};
pub use oberer_orm::schema::view_builder::{OrmViewDef, ViewQueryBuilder, ViewSourceKind};
pub use oberer_orm::conn::HybridConn;
pub use oberer_orm::{OrmResultExt, Table, TableCached, TableOps, TableTrigger, TableView};

/// Type alias for TableSchemaQueryBuilder adhering to the `Schema(struct)` pattern.
pub type Schema<T> = TableSchemaQueryBuilder<T>;

/// Type alias for ViewQueryBuilder adhering to the `ViewSchema(struct)` pattern.
pub type ViewSchema<T, P = T> = ViewQueryBuilder<T, P>;

/// Type alias for TriggerQueryBuilder adhering to the `TriggerSchema(struct)` pattern.
pub type TriggerSchema<T> = TriggerQueryBuilder<T>;
