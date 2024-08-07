use cairo_lang_defs::plugin::PluginDiagnostic;
use cairo_lang_filesystem::span::TextSpan;
use cairo_lang_semantic::diagnostic::SemanticDiagnosticKind;
use cairo_lang_semantic::SemanticDiagnostic;
use cairo_lang_syntax::node::ast::{ExprMatch, Pattern};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::{SyntaxNode, TypedSyntaxNode};
use cairo_lang_utils::Upcast;

use crate::db::AnalysisDatabase;
use crate::plugin::{diagnostic_kind_from_message, CairoLintKind};

#[derive(Default)]
pub struct Fix {
    pub span: TextSpan,
    pub suggestion: String,
}

pub fn fix_semantic_diagnostic(db: &AnalysisDatabase, diag: &SemanticDiagnostic) -> String {
    match diag.kind {
        SemanticDiagnosticKind::UnusedVariable => {
            format!("_{}", diag.stable_location.syntax_node(db.upcast()).get_text(db.upcast()))
        }
        SemanticDiagnosticKind::PluginDiagnostic(ref diag) => Fixer.fix_plugin_diagnostic(db, diag),
        _ => "".to_owned(),
    }
}

#[derive(Default)]
pub struct Fixer;
impl Fixer {
    pub fn fix_if_let(&self, db: &dyn SyntaxGroup, node: SyntaxNode) -> String {
        let match_expr = ExprMatch::from_syntax_node(db, node.clone());
        let arms = match_expr.arms(db).elements(db);
        let first_arm = &arms[0];
        let second_arm = &arms[1];
        let (pattern, first_expr) =
            match (&first_arm.patterns(db).elements(db)[0], &second_arm.patterns(db).elements(db)[0]) {
                (Pattern::Underscore(_), Pattern::Enum(pat)) => {
                    (&pat.as_syntax_node().get_text_without_trivia(db), second_arm)
                }
                (Pattern::Enum(pat), Pattern::Underscore(_)) => {
                    (&pat.as_syntax_node().get_text_without_trivia(db), first_arm)
                }
                (Pattern::Underscore(_), Pattern::Struct(pat)) => {
                    (&pat.as_syntax_node().get_text_without_trivia(db), second_arm)
                }
                (Pattern::Struct(pat), Pattern::Underscore(_)) => {
                    (&pat.as_syntax_node().get_text_without_trivia(db), first_arm)
                }
                (_, _) => panic!("Incorrect diagnostic"),
            };
        format!(
            "{}if let {} = {} {{ {} }}",
            node.get_text(db).chars().take_while(|c| c.is_whitespace()).collect::<String>(),
            pattern,
            match_expr.expr(db).as_syntax_node().get_text(db),
            first_expr.expression(db).as_syntax_node().get_text_without_trivia(db),
        )
    }
    pub fn fix_plugin_diagnostic(&self, db: &AnalysisDatabase, diag: &PluginDiagnostic) -> String {
        match diagnostic_kind_from_message(&diag.message) {
            CairoLintKind::IfLet => self.fix_if_let(db, diag.stable_ptr.lookup(db.upcast())),
            _ => "".to_owned(),
        }
    }
}
