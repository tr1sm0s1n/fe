pub mod attr;
pub mod body;
pub mod expr;
pub mod ident;
pub mod item;
pub mod params;
pub mod pat;
pub mod path;
pub mod prim_ty;
pub mod scope_graph;
pub mod stmt;
pub mod types;
pub mod use_tree;

pub(crate) mod module_tree;

pub use attr::*;
pub use body::*;
use common::{input::IngotKind, InputIngot};
pub use expr::*;
pub use ident::*;
pub use item::*;
pub use module_tree::*;
pub use params::*;
pub use pat::*;
pub use path::*;
pub use stmt::*;
pub use types::*;
pub use use_tree::*;

use num_bigint::BigUint;

use crate::{external_ingots_impl, HirDb};

#[salsa::tracked]
pub struct IngotId {
    inner: InputIngot,
}
impl IngotId {
    pub fn module_tree(self, db: &dyn HirDb) -> &ModuleTree {
        module_tree_impl(db, self.inner(db))
    }

    pub fn root_mod(self, db: &dyn HirDb) -> TopLevelMod {
        self.module_tree(db).root_data().top_mod
    }

    pub fn external_ingots(self, db: &dyn HirDb) -> &[(IdentId, TopLevelMod)] {
        external_ingots_impl(db, self.inner(db)).as_slice()
    }

    pub fn kind(self, db: &dyn HirDb) -> IngotKind {
        self.inner(db).kind(db.as_input_db())
    }
}

#[salsa::interned]
pub struct IntegerId {
    #[return_ref]
    pub data: BigUint,
}

#[salsa::interned]
pub struct StringId {
    /// The text of the string literal, without the quotes.
    #[return_ref]
    pub data: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LitKind {
    Int(IntegerId),
    String(StringId),
    Bool(bool),
}

/// `Partial<T>` is a type that explicitly indicates the possibility that an HIR
/// node cannot be generated due to syntax errors in the source file.
///
/// If a node is `Partial::Absent`, it means that the corresponding AST either
/// does not exist or is erroneous. When a `Partial::Absent` is generated, the
/// relevant error is always generated by the parser, so in Analysis phases, it
/// can often be ignored.
///
/// This type is clearly distinguished from `Option<T>`. The
/// `Option<T>` type is used to hold syntactically valid optional nodes, while
/// `Partial<T>` means that a syntactically required element may be missing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Partial<T> {
    Present(T),
    Absent,
}

impl<T> Partial<T> {
    pub fn unwrap(&self) -> &T {
        match self {
            Self::Present(value) => value,
            Self::Absent => panic!("unwrap called on absent value"),
        }
    }

    pub fn to_opt(self) -> Option<T> {
        match self {
            Self::Present(value) => Some(value),
            Self::Absent => None,
        }
    }
}

impl<T> Default for Partial<T> {
    fn default() -> Self {
        Self::Absent
    }
}

impl<T> From<Option<T>> for Partial<T> {
    fn from(value: Option<T>) -> Self {
        if let Some(value) = value {
            Self::Present(value)
        } else {
            Self::Absent
        }
    }
}

impl<T> From<Partial<T>> for Option<T> {
    fn from(value: Partial<T>) -> Option<T> {
        value.to_opt()
    }
}
