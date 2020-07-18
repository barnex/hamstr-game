use crate::prelude::*;

/// Check if a file exists. E.g.:
///     check_exists(f)?
#[must_use]
pub fn check_exists<P: AsRef<Path>>(p: P) -> Result<()> {
	if !p.as_ref().exists() {
		GenError::new(format!(
			"no such file or directory: {}",
			p.as_ref().to_string_lossy()
		))
	} else {
		Ok(())
	}
}
