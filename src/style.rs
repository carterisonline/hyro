use std::path::Path;

pub use lightningcss;
pub use lightningcss::css_modules::{Config, Pattern};
pub use lightningcss::stylesheet::{ParserFlags, ParserOptions};
pub use lightningcss::targets::{Browsers, Features, Targets};

use lightningcss::bundler::{Bundler, FileProvider};
use lightningcss::stylesheet::PrinterOptions;
use once_cell::sync::{Lazy, OnceCell};

pub static CARTERS_PARSER_OPTIONS: Lazy<ParserOptions> = Lazy::new(|| ParserOptions {
    flags: ParserFlags::NESTING | ParserFlags::CUSTOM_MEDIA,
    ..Default::default()
});

pub static CARTERS_TARGET_OPTIONS: Lazy<Targets> = Lazy::new(|| Targets {
    browsers: Some(Browsers {
        android: None,
        chrome: Some(6160384),
        edge: Some(6291456),
        firefox: Some(6160384),
        ie: Some(720896),
        ios_saf: Some(786944),
        opera: Some(5308416),
        safari: Some(852224),
        samsung: None,
    }),
    ..Default::default()
});

pub(crate) static STYLE_FILE_PROVIDER: Lazy<FileProvider> = Lazy::new(FileProvider::new);
#[cfg(not(debug_assertions))]
pub(crate) static MAIN_CSS: OnceCell<String> = OnceCell::new();

#[cfg(debug_assertions)]
pub(crate) static PATH: OnceCell<String> = OnceCell::new();
#[cfg(debug_assertions)]
pub(crate) static PARSER_OPTIONS: OnceCell<ParserOptions> = OnceCell::new();
#[cfg(debug_assertions)]
pub(crate) static TARGETS: OnceCell<Targets> = OnceCell::new();

#[derive(Debug)]
pub(crate) enum TransformCSSError<'a> {
    BundleError(lightningcss::bundler::BundleErrorKind<'a, std::io::Error>),
    PrinterError(lightningcss::error::PrinterErrorKind),
}

/// Utility function for bundling and minifying CSS.
pub(crate) fn transform_css<'a>(
    path: &str,
    options: ParserOptions<'a, 'a>,
    browserslist: Targets,
) -> Result<String, TransformCSSError<'a>> {
    // 1: Initialize the bundler state
    let mut bundler = Bundler::new(&*STYLE_FILE_PROVIDER, None, options);

    // 2: Bundle the CSS by following @import statements
    match bundler.bundle(Path::new(path)) {
        Ok(stylesheet) => {
            // 3: Since step 2 produced a rust-native stylesheet structure, we convert it back to CSS.
            let printed = stylesheet.to_css(PrinterOptions {
                minify: true,
                targets: browserslist,
                ..Default::default()
            });

            // 4: Only return the serialized CSS in the .code field
            match printed {
                Ok(printed) => Ok(printed.code),
                Err(e) => Err(TransformCSSError::PrinterError(e.kind)),
            }
        }
        Err(e) => Err(TransformCSSError::BundleError(e.kind)),
    }
}
