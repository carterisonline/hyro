cfg_if::cfg_if! {
    if #[cfg(feature = "framework-axum")] {
        mod framework_axum;
        pub use framework_axum::*;
        pub mod prelude {
            pub use super::framework_axum::RouterExt;
        }
    } else if #[cfg(feature = "framework-trillium")] {
        mod framework_trillium;
        pub use framework_trillium::*;
        pub mod prelude {
            pub use super::framework_trillium::RouterExt;
            pub use super::framework_trillium::ConnExt;
        }
    }
}
