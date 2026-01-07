/// Declaratively creates a `RuleContext`.
///
/// # Syntax
/// ```rust
/// use shaum_rules::shaum_context;
/// use shaum_types::{Madhab, DaudStrategy};
///
/// let ctx = shaum_context! {
///     madhab: Madhab::Hanafi,
///     adjustment: 1,
///     strategy: DaudStrategy::Skip
/// };
/// ```
#[macro_export]
macro_rules! shaum_context {
    ( $($key:ident : $value:expr),* $(,)? ) => {
        {
            let mut ctx = $crate::RuleContext::new();
            $(
                ctx = $crate::shaum_context!(@apply ctx, $key, $value);
            )*
            ctx
        }
    };

    (@apply $ctx:ident, madhab, $v:expr) => { $ctx.madhab($v) };
    (@apply $ctx:ident, adjustment, $v:expr) => { $ctx.adjustment($v) };
    (@apply $ctx:ident, strategy, $v:expr) => { $ctx.daud_strategy($v) };
}
