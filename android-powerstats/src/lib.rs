use binder::binder_impl::Deserialize;

#[path = "android/hardware/power/stats/mod.rs"]
pub mod powerstats;

pub(crate) mod mangled {
    pub(crate) use super::powerstats::mangled::*;

    pub(crate) type _7_android_2_os_14_ResultReceiver = super::ResultReceiver;
}

// https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/core/java/android/os/ResultReceiver.aidl;l=20;drc=b741c646c69ebdcfbc3287297a312a4ee1aeb5fe
// ERROR: android-frameworks-base/core/java/android/os/ResultReceiver.aidl:20.37-52: Refusing to generate code with unstructured parcelables. Declared parcelables should be in their own file and/or cannot be used with --structured interfaces.
// Unstructured parcel only has a definition in Java:
// https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/core/java/android/os/ResultReceiver.java
pub(crate) struct ResultReceiver {
    //TODO: Type to IResultReceiver
    // https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/core/java/com/android/internal/os/IResultReceiver.aidl?q=IResultReceiver.aidl&ss=android%2Fplatform%2Fsuperproject%2Fmain
    binder: binder::SpIBinder,
}

impl binder::binder_impl::Serialize for ResultReceiver {
    fn serialize(
        &self,
        parcel: &mut binder::binder_impl::BorrowedParcel<'_>,
    ) -> Result<(), binder::StatusCode> {
        // https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/core/java/android/os/ResultReceiver.java;l=125;drc=9e8f83db6d969f1e1f47ffa0b0390d867491235b
        self.binder.serialize(parcel)
    }
}

impl binder::binder_impl::Deserialize for ResultReceiver {
    fn deserialize(
        parcel: &binder::binder_impl::BorrowedParcel<'_>,
    ) -> Result<Self, binder::StatusCode> {
        binder::SpIBinder::deserialize(parcel).map(|binder| Self { binder })
    }
}

#[path = "android/os/IPowerStatsService.rs"]
pub mod powerstatsservice;

pub fn android_hardware_power_powerstats() -> binder::Result<()> {
    use powerstats::IPowerStats;
    let descriptor = <IPowerStats::BpPowerStats as IPowerStats::IPowerStats>::get_descriptor();
    let i =
        binder::get_interface::<dyn IPowerStats::IPowerStats>(&format!("{}/default", descriptor))?;
    dbg!(&i);

    dbg!(i.getPowerEntityInfo())?;
    let meters = dbg!(i.getEnergyMeterInfo())?;
    let meter_ids = meters.iter().map(|m| m.id).collect::<Vec<_>>();
    let consumed = i.getEnergyConsumed(&meter_ids)?;
    dbg!(consumed);
    Ok(())
}

pub fn android_os_powerstatsservice() -> binder::Result<()> {
    let i = binder::get_interface::<dyn powerstatsservice::IPowerStatsService>("powerstats")?;
    dbg!(&i);

    // i.getSupportedPowerMonitors();

    Ok(())
}

pub fn pull_data() -> binder::Result<()> {
    // android_hardware_power_powerstats()?;
    android_os_powerstatsservice()?;
    Ok(())
}
