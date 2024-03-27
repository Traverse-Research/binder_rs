use std::time::Duration;

use powerstats::IPowerStats;

#[path = "android/hardware/power/stats/mod.rs"]
pub mod powerstats;

pub(crate) mod mangled {
    pub(crate) use super::powerstats::mangled::*;
}

pub fn run() -> binder::Result<()> {
    let descriptor = <IPowerStats::BpPowerStats as IPowerStats::IPowerStats>::get_descriptor();
    let i =
        binder::get_interface::<dyn IPowerStats::IPowerStats>(&format!("{}/default", descriptor))?;
    dbg!(&i);

    // dbg!(i.getPowerEntityInfo())?;
    let meters = dbg!(i.getEnergyMeterInfo())?;
    println!("We have {} meters", meters.len());
    // let meter_ids = meters.iter().map(|m| m.id).collect::<Vec<_>>();
    let gpu_meter_ids = meters
        .iter()
        .filter(|m| m.subsystem == "GPU")
        .map(|m| m.id)
        .collect::<Vec<_>>();
    // dbg!(&meter_ids);
    let mut start = dbg!(i.getEnergyConsumed(&gpu_meter_ids))?;
    // let last = start.clone();
    for _ in 0..10 {
        std::thread::sleep(Duration::from_secs(1));

        let consumed = i.getEnergyConsumed(&gpu_meter_ids)?;
        for (prev, cur) in start.iter().zip(&consumed) {
            assert_eq!(prev.id, cur.id);
            let dt = Duration::from_millis((cur.timestampMs - prev.timestampMs) as u64);
            let dp = cur.energyUWs - prev.energyUWs;
            let meter = meters.iter().find(|m| m.id == prev.id).unwrap();
            // TODO: Divide uWs by seconds to get the wattage
            println!("{} used {:.02}uWs in the past {:#02?}", meter.name, dp, dt);
        }
        start = consumed;
        // let consumed = dbg!(i.getEnergyConsumed(&[]))?;
        // dbg!(consumed);
    }
    Ok(())
}
