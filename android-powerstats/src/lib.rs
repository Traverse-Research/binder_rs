mod android_hardware_power_stats;
mod android_os_powerstatsservice;
mod bundle;
mod result_receiver;

pub(crate) mod mangled {
    pub(crate) use super::android_hardware_power_stats::mangled::*;
    pub(crate) use super::bundle::mangled::*;
    pub(crate) use super::result_receiver::mangled::*;
}

pub fn pull_data() -> binder::Result<()> {
    // android_hardware_power_stats::run()?;
    android_os_powerstatsservice::run()?;
    Ok(())
}
