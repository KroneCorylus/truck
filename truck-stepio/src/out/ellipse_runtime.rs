#[cfg(all(
    feature = "ellipse-reviewed-glibc",
    target_arch = "x86_64",
    target_os = "linux",
    target_env = "gnu"
))]
#[path = "ellipse_fenv.rs"]
mod fenv;

pub(super) fn eligible() -> bool {
    #[cfg(all(
        feature = "ellipse-reviewed-glibc",
        target_arch = "x86_64",
        target_os = "linux",
        target_env = "gnu"
    ))]
    {
        fenv::eligible()
    }
    #[cfg(not(all(
        feature = "ellipse-reviewed-glibc",
        target_arch = "x86_64",
        target_os = "linux",
        target_env = "gnu"
    )))]
    {
        false
    }
}

// Called only for exact built-in shell types. The closure writes to a private String.
// Arbitrary output callbacks run only after all numerical formatting has finished.
pub(super) fn buffered(
    output: &mut impl std::fmt::Write,
    render: impl FnOnce(&mut String) -> std::fmt::Result,
) -> std::fmt::Result {
    if !eligible() {
        return Err(std::fmt::Error);
    }
    let mut text = String::new();
    render(&mut text)?;
    if !eligible() {
        return Err(std::fmt::Error);
    }
    output.write_str(&text)
}

#[cfg(all(
    test,
    all(
        feature = "ellipse-reviewed-glibc",
        target_arch = "x86_64",
        target_os = "linux",
        target_env = "gnu"
    )
))]
pub(super) use fenv::{with_downward, DownwardGuard};
