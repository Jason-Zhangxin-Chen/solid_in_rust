// Run an algorithm over each item in a collection of data to create a new item,
// thus creating a whole new collection.
//
// The etymology here is unclear to me. The terms ‘fold’ and ‘folder’ are used in the Rust compiler,
// although it appears to me to be more like a map than a fold in the usual sense.
// See the discussion below for more details.


// The result of running the Renamer on an AST is a new AST identical to the old one, but with
// every name changed to foo. A real life folder might have some state preserved between nodes
// in the struct itself.
//
// A folder can also be defined to map one data structure to a different (but usually similar)
// data structure. For example, we could fold an AST into a HIR tree
// (HIR stands for high-level intermediate representation).

// The data we will fold, a simple AST.
mod ast {
    pub enum Stmt {
        Expr(Box<Expr>),
        Let(Box<Name>, Box<Expr>),
    }

    pub struct Name {
        pub(crate) value: String,
    }

    pub enum Expr {
        IntLit(i64),
        Add(Box<Expr>, Box<Expr>),
        Sub(Box<Expr>, Box<Expr>),
    }
}

// The abstract folder
mod fold {
    use crate::design_patterns::fold::ast::{Expr, Name, Stmt};

    pub trait Folder {
        // A leaf node just returns the node itself. In some cases, we can do this
        // to inner nodes too.
        fn fold_name(&mut self, n: Box<Name>) -> Box<Name> { n }
        // Create a new inner node by folding its children.
        fn fold_stmt(&mut self, s: Box<Stmt>) -> Box<Stmt> {
            match *s {
                Stmt::Expr(e) => Box::new(Stmt::Expr(self.fold_expr(e))),
                Stmt::Let(n, e) => Box::new(Stmt::Let(self.fold_name(n), self.fold_expr(e))),
            }
        }
        fn fold_expr(&mut self, e: Box<Expr>) -> Box<Expr> { Box::new(Expr::IntLit(0)) /*...*/ }
    }
}

use fold::*;
use ast::*;

// An example concrete implementation - renames every name to 'foo'.
struct Renamer;
impl Folder for Renamer {
    fn fold_name(&mut self, n: Box<Name>) -> Box<Name> {
        Box::new(Name { value: "foo".to_owned() })
    }
    // Use the default methods for the other nodes.
}

