use parking_lot::MutexGuard;
use std::path::{Path, PathBuf};

use lightningcss::stylesheet::ParserOptions;
use lightningcss::targets::Targets;

use crate::render::ENVIRONMENT;
use crate::{TEMPLATE_DIR, TEMPLATE_EXTENSION};

pub fn set_template_dir<T: AsRef<Path>>(dir: T) -> Result<(), PathBuf> {
    let dir = dir.as_ref().to_path_buf();
    if !dir.exists() {
        error!("Template directory does not exist: {}", dir.display());
    } else if dir.is_file() {
        error!("Template directory is a file: {}", dir.display());
    }

    TEMPLATE_DIR.set(dir)
}

pub fn set_template_file_extension<S: AsRef<str>>(extension: S) -> Result<(), String> {
    let extension = extension.as_ref().trim_start_matches('.');
    TEMPLATE_EXTENSION.set(format!(".{}", extension))
}

pub fn set_style_options(options: ParserOptions<'static, 'static>) {
    crate::style::STYLE_OPTIONS.set(options).unwrap();
}

pub fn set_style_targets(targets: Targets) {
    crate::style::STYLE_TARGETS.set(targets).unwrap();
}

pub fn modify_template_env<F: FnOnce(&mut MutexGuard<'_, minijinja::Environment<'static>>)>(
    func: F,
) {
    func(&mut ENVIRONMENT.lock());
}
