#[path = "android/hardware/power/stats/mod.rs"]
pub mod powerstats;

pub(crate) mod mangled {
    pub(crate) use super::powerstats::mangled::*;
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

pub fn pull_data() -> binder::Result<()> {
    android_hardware_power_powerstats()?;
    // let i = binder::get_interface::<dyn powerstats::IPowerStats::IPowerStats>("powerstats")?;
    Ok(())
}
