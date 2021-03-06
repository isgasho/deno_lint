// Copyright 2020 the Deno authors. All rights reserved. MIT license.
use super::{Context, LintRule};
use swc_common::Span;
use swc_ecmascript::ast::Module;
use swc_ecmascript::ast::Stmt::{Break, Continue, Return, Throw};
use swc_ecmascript::ast::TryStmt;
use swc_ecmascript::visit::{self, noop_visit_type, Node, Visit};

pub struct NoUnsafeFinally;

impl LintRule for NoUnsafeFinally {
  fn new() -> Box<Self> {
    Box::new(NoUnsafeFinally)
  }

  fn tags(&self) -> &[&'static str] {
    &["recommended"]
  }

  fn code(&self) -> &'static str {
    "no-unsafe-finally"
  }

  fn lint_module(&self, context: &mut Context, module: &Module) {
    let mut visitor = NoUnsafeFinallyVisitor::new(context);
    visitor.visit_module(module, module);
  }

  fn docs(&self) -> &'static str {
    r#"Disallows the use of control flow statements within `finally` blocks.

Use of the control flow statements (`return`, `throw`, `break` and `continue`) overrides the usage of any control flow statements that might have been used in the `try` or `catch` blocks, which is usually not the desired behaviour.

### Invalid:
```typescript
let foo = function() {
  try {
    return 1;
  } catch(err) {
    return 2;
  } finally {
    return 3;
  }
};
```
```typescript
let foo = function() {
  try {
    return 1;
  } catch(err) {
    return 2;
  } finally {
    throw new Error;
  }
};
```
### Valid:
```typescript
let foo = function() {
  try {
    return 1;
  } catch(err) {
    return 2;
  } finally {
    console.log("hola!");
  }
};
```"#
  }
}

struct NoUnsafeFinallyVisitor<'c> {
  context: &'c mut Context,
}

impl<'c> NoUnsafeFinallyVisitor<'c> {
  fn new(context: &'c mut Context) -> Self {
    Self { context }
  }

  fn add_diagnostic(&mut self, span: Span, stmt_type: &str) {
    self.context.add_diagnostic(
      span,
      "no-unsafe-finally",
      format!("Unsafe usage of {}Statement", stmt_type).as_str(),
    );
  }
}

impl<'c> Visit for NoUnsafeFinallyVisitor<'c> {
  noop_visit_type!();

  fn visit_try_stmt(&mut self, try_stmt: &TryStmt, parent: &dyn Node) {
    if let Some(finally_block) = &try_stmt.finalizer {
      for stmt in &finally_block.stmts {
        match stmt {
          Break(_) => self.add_diagnostic(finally_block.span, "Break"),
          Continue(_) => self.add_diagnostic(finally_block.span, "Continue"),
          Return(_) => self.add_diagnostic(finally_block.span, "Return"),
          Throw(_) => self.add_diagnostic(finally_block.span, "Throw"),
          _ => {}
        }
      }
    }
    visit::visit_try_stmt(self, try_stmt, parent);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_util::*;

  #[test]
  fn it_passes_when_there_are_no_disallowed_keywords_in_the_finally_block() {
    assert_lint_ok::<NoUnsafeFinally>(
      r#"
let foo = function() {
  try {
    return 1;
  } catch(err) {
    return 2;
  } finally {
    console.log("hola!");
  }
};
     "#,
    );
  }

  #[test]
  fn it_passes_for_a_return_within_a_function_in_a_finally_block() {
    assert_lint_ok::<NoUnsafeFinally>(
      r#"
let foo = function() {
  try {
    return 1;
  } catch(err) {
    return 2;
  } finally {
    let a = function() {
      return "hola!";
    }
  }
};
     "#,
    );
  }

  #[test]
  fn it_passes_for_a_break_within_a_switch_in_a_finally_block() {
    assert_lint_ok::<NoUnsafeFinally>(
      r#"
let foo = function(a) {
  try {
    return 1;
  } catch(err) {
    return 2;
  } finally {
    switch(a) {
      case 1: {
        console.log("hola!")
        break;
      }
    }
  }
};
      "#,
    );
  }

  #[test]
  fn it_fails_for_a_break_in_a_finally_block() {
    assert_lint_err_on_line::<NoUnsafeFinally>(
      r#"
let foo = function() {
  try {
    return 1;
  } catch(err) {
    return 2;
  } finally {
    break;
  }
};
     "#,
      7,
      12,
    );
  }

  #[test]
  fn it_fails_for_a_continue_in_a_finally_block() {
    assert_lint_err_on_line::<NoUnsafeFinally>(
      r#"
let foo = function() {
  try {
    return 1;
  } catch(err) {
    return 2;
  } finally {
    continue;
  }
};
     "#,
      7,
      12,
    );
  }

  #[test]
  fn it_fails_for_a_return_in_a_finally_block() {
    assert_lint_err_on_line::<NoUnsafeFinally>(
      r#"
let foo = function() {
  try {
    return 1;
  } catch(err) {
    return 2;
  } finally {
    return 3;
  }
};
          "#,
      7,
      12,
    );
  }

  #[test]
  fn it_fails_for_a_throw_in_a_finally_block() {
    assert_lint_err_on_line::<NoUnsafeFinally>(
      r#"
let foo = function() {
  try {
    return 1;
  } catch(err) {
    return 2;
  } finally {
    throw new Error;
  }
};
     "#,
      7,
      12,
    );
  }

  #[test]
  fn it_fails_for_a_throw_in_a_nested_finally_block() {
    assert_lint_err_on_line::<NoUnsafeFinally>(
      r#"
try {}
finally {
  try {}
  finally {
    throw new Error;
  }
}
     "#,
      5,
      10,
    );
  }
}
