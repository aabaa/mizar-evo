//! Resolver-local recovered-syntax helpers.
//!
//! `mizar-syntax` owns recovery production and vocabulary. The resolver only
//! consumes the represented flags and keeps degraded semantic facts from
//! producing follow-on semantic diagnostics.

use crate::resolved_ast::SemanticOrigin;
use mizar_syntax::SurfaceNodeView;

pub(crate) fn surface_contains_recovery(view: SurfaceNodeView<'_>) -> bool {
    view.is_recovered()
        || view.as_recovery().is_some()
        || view.child_views().any(surface_contains_recovery)
}

pub(crate) const fn suppress_dependent_diagnostic_for_recovered_origin(
    origin: &SemanticOrigin,
) -> bool {
    origin.is_recovered()
}

pub(crate) const fn suppress_dependent_diagnostic_for_recovered_shell(recovered: bool) -> bool {
    recovered
}
