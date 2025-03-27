use rustc_ast::ast;
use rustc_ast::visit::Visitor;
use rustc_span::Symbol;
use rustc_span::sym;
use tracing::debug;

use crate::attr::MetaVisitor;
use crate::parse::macros::cfg_if::parse_cfg_if;
use crate::parse::macros::cfg_match::parse_cfg_match;
use crate::parse::session::ParseSess;

pub(crate) struct ModItem {
    pub(crate) item: ast::Item,
}

/// Traverse well-known macro names and fetch modules.
pub(crate) struct KnownMacroVisitor<'a> {
    psess: &'a ParseSess,
    mods: Vec<ModItem>,
}

impl<'a> KnownMacroVisitor<'a> {
    pub(crate) fn new(psess: &'a ParseSess) -> KnownMacroVisitor<'a> {
        KnownMacroVisitor {
            mods: vec![],
            psess,
        }
    }

    pub(crate) fn mods(self) -> Vec<ModItem> {
        self.mods
    }
}

impl<'a, 'ast: 'a> Visitor<'ast> for KnownMacroVisitor<'a> {
    fn visit_mac_call(&mut self, mac: &'ast ast::MacCall) {
        match self.visit_mac_inner(mac) {
            Ok(()) => (),
            Err(e) => debug!("{}", e),
        }
    }
}

impl<'a, 'ast: 'a> KnownMacroVisitor<'a> {
    fn visit_mac_inner(&mut self, mac: &'ast ast::MacCall) -> Result<(), &'static str> {
        fn is_macro_name(mac: &ast::MacCall, name: &str) -> bool {
            mac.path
                .segments
                .last()
                .map_or(false, |segment| segment.ident.name == Symbol::intern(name))
        }

        let items = if is_macro_name(mac, "cfg_if") {
            parse_cfg_if(&self.psess, mac)?
        } else if is_macro_name(mac, "cfg_match") {
            parse_cfg_match(&self.psess, mac)?
        } else {
            // ignore all other macros
            vec![]
        };

        self.mods
            .append(&mut items.into_iter().map(|item| ModItem { item }).collect());

        Ok(())
    }
}

/// Extracts `path = "foo.rs"` from attributes.
#[derive(Default)]
pub(crate) struct PathVisitor {
    /// A list of path defined in attributes.
    paths: Vec<String>,
}

impl PathVisitor {
    pub(crate) fn paths(self) -> Vec<String> {
        self.paths
    }
}

impl<'ast> MetaVisitor<'ast> for PathVisitor {
    fn visit_meta_name_value(
        &mut self,
        meta_item: &'ast ast::MetaItem,
        lit: &'ast ast::MetaItemLit,
    ) {
        if meta_item.has_name(sym::path) && lit.kind.is_str() {
            self.paths.push(meta_item_lit_to_str(lit));
        }
    }
}

#[cfg(not(windows))]
fn meta_item_lit_to_str(lit: &ast::MetaItemLit) -> String {
    match lit.kind {
        ast::LitKind::Str(symbol, ..) => symbol.to_string(),
        _ => unreachable!(),
    }
}

#[cfg(windows)]
fn meta_item_lit_to_str(lit: &ast::MetaItemLit) -> String {
    match lit.kind {
        ast::LitKind::Str(symbol, ..) => symbol.as_str().replace("/", "\\"),
        _ => unreachable!(),
    }
}
