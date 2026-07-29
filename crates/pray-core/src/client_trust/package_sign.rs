use crate::client_trust::home::effective_trust_home;
use crate::client_trust::policy::{best_rule, load_policy_or_default};
use crate::registry::RegistryPackageVersion;
use crate::{PrayError, PrayResult};

pub fn enforce_require_signed_packages(
    source_url: &str,
    package_name: &str,
    selected: &RegistryPackageVersion,
) -> PrayResult<()> {
    let home = effective_trust_home()?;
    let policy = load_policy_or_default(&home)?;
    let rule = best_rule(&policy, source_url);
    if !rule.require_signed_packages {
        return Ok(());
    }
    let Some(signature) = selected.signature.as_deref() else {
        return Err(PrayError::Integrity(format!(
            "trust policy requires signed packages, but {package_name} {} has no signature",
            selected.version
        )));
    };
    if signature.trim().is_empty() {
        return Err(PrayError::Integrity(format!(
            "trust policy requires signed packages, but {package_name} {} has an empty signature",
            selected.version
        )));
    }
    Ok(())
}
