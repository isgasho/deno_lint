// Copyright 2020 the Deno authors. All rights reserved. MIT license.
use super::Context;
use super::LintRule;

use swc_common::comments::Comment;
use swc_common::comments::CommentKind;
use swc_common::Span;

pub struct BanTsComment;

impl BanTsComment {
  fn report(&self, context: &mut Context, span: Span) {
    context.add_diagnostic(
      span,
      "ban-ts-comment",
      "ts directives are not allowed",
    );
  }
}

impl LintRule for BanTsComment {
  fn new() -> Box<Self> {
    Box::new(BanTsComment)
  }

  fn tags(&self) -> &[&'static str] {
    &["recommended"]
  }

  fn code(&self) -> &'static str {
    "ban-ts-comment"
  }

  fn lint_module(
    &self,
    context: &mut Context,
    _module: &swc_ecmascript::ast::Module,
  ) {
    let mut violated_comment_spans = Vec::new();

    violated_comment_spans.extend(
      context.leading_comments.values().flatten().filter_map(|c| {
        if check_comment(c) {
          Some(c.span)
        } else {
          None
        }
      }),
    );
    violated_comment_spans.extend(
      context
        .trailing_comments
        .values()
        .flatten()
        .filter_map(|c| if check_comment(c) { Some(c.span) } else { None }),
    );

    for span in violated_comment_spans {
      self.report(context, span);
    }
  }
}

/// Returns `true` if the comment should be reported.
fn check_comment(comment: &Comment) -> bool {
  if comment.kind != CommentKind::Line {
    return false;
  }

  lazy_static! {
    static ref BTC_REGEX: regex::Regex =
      regex::Regex::new(r#"^/*\s*@ts-(expect-error|ignore|nocheck)$"#).unwrap();
  }

  BTC_REGEX.is_match(&comment.text)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test_util::*;

  #[test]
  fn ban_ts_comment_valid() {
    assert_lint_ok_n::<BanTsComment>(vec![
      r#"// just a comment containing @ts-expect-error somewhere"#,
      r#"/* @ts-expect-error */"#,
      r#"/** @ts-expect-error */"#,
      r#"/*
// @ts-expect-error in a block
*/
"#,
    ]);

    assert_lint_ok_n::<BanTsComment>(vec![
      r#"// just a comment containing @ts-ignore somewhere"#,
      r#"/* @ts-ignore */"#,
      r#"/** @ts-ignore */"#,
      r#"/*
// @ts-ignore in a block
*/
"#,
    ]);

    assert_lint_ok_n::<BanTsComment>(vec![
      r#"// just a comment containing @ts-nocheck somewhere"#,
      r#"/* @ts-nocheck */"#,
      r#"/** @ts-nocheck */"#,
      r#"/*
// @ts-nocheck in a block
*/
"#,
    ]);

    assert_lint_ok_n::<BanTsComment>(vec![
      r#"// just a comment containing @ts-check somewhere"#,
      r#"/* @ts-check */"#,
      r#"/** @ts-check */"#,
      r#"/*
// @ts-check in a block
*/
"#,
    ]);

    assert_lint_ok::<BanTsComment>(
      r#"if (false) {
// @ts-ignore: Unreachable code error
console.log('hello');
}"#,
    );
    assert_lint_ok::<BanTsComment>(
      r#"if (false) {
// @ts-expect-error: Unreachable code error
console.log('hello');
}"#,
    );
    assert_lint_ok::<BanTsComment>(
      r#"if (false) {
// @ts-nocheck: Unreachable code error
console.log('hello');
}"#,
    );

    assert_lint_ok::<BanTsComment>(
      r#"// @ts-expect-error: Suppress next line"#,
    );
    assert_lint_ok::<BanTsComment>(r#"// @ts-ignore: Suppress next line"#);
    assert_lint_ok::<BanTsComment>(r#"// @ts-nocheck: Suppress next line"#);
  }

  #[test]
  fn ban_ts_comment_invalid() {
    assert_lint_err::<BanTsComment>(r#"// @ts-expect-error"#, 0);
    assert_lint_err::<BanTsComment>(r#"// @ts-ignore"#, 0);
    assert_lint_err::<BanTsComment>(r#"// @ts-nocheck"#, 0);
  }
}
