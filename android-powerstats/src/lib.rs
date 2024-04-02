use std::{fmt, time::Duration};

use android_hardware_power_stats::IPowerStats::IPowerStats as _;
use android_os_powerstatsservice::{
    powerstatsservice::BpPowerStatsService, IPowerStatsService as _, PowerMonitorType,
};
use anyhow::Result;
use binder::{binder_impl::Proxy, Interface, Status, StatusCode};
use log::warn;

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
    android_hardware_power_stats::run()?;
    // android_os_powerstatsservice::run()?;
    Ok(())
}

enum Backend {
    VendorHardwareService(android_hardware_power_stats::powerstats::IPowerStats::BpPowerStats),
    SystemJavaService(android_os_powerstatsservice::powerstatsservice::BpPowerStatsService),
}

impl fmt::Debug for Backend {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut b = match self {
            Backend::VendorHardwareService(s) => s.as_binder(),
            Backend::SystemJavaService(s) => s.as_binder(),
        };
        f.debug_struct("Backend")
            .field("descriptor", &b.get_class().unwrap().get_descriptor())
            .finish_non_exhaustive()
        // f.write_str(match self {
        //     Backend::VendorHardwareService(_) => "android.hardware.power.stats.IPowerStats/default",
        //     Backend::SystemJavaService(_) => "powerstats",
        // })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendSelection {
    VendorHardwareService,
    SystemJavaService,
}

#[derive(Debug)]
pub struct PowerStats {
    backend: Backend,
}

impl PowerStats {
    /// Tries to talk to `android.hardware.power.stats.IPowerStats/default`, otherwise falls back to `powerstats`
    pub fn new() -> Result<Self> {
        match Self::new_with_backend(BackendSelection::SystemJavaService) {
            Ok(s) => Ok(s),
            Err(e) => {
                warn!("Failed to get `powerstats` service because of `{e:?}`. Falling back to vendor HAL");
                Self::new_with_backend(BackendSelection::VendorHardwareService).inspect_err(|e| {
                    warn!("Failed to get `android.hardware.power.stats.IPowerStats/default` because of `{e:?}`");
                    // type=1400 audit(0.0:419): avc:  denied  { call } for  scontext=u:r:untrusted_app_32:s0:c13,c257,c512,c768 tcontext=u:r:hal_power_stats_default:s0 tclass=binder permissive=1 app=...
                    warn!("If you see `denied {{ call }} for scontext=..untrusted_app.. tcontext=..hal_power_stats_default..` in `logcat`, issue `setenforce 0` from a root shell to allow access");
                })
            }
        }
    }

    pub fn new_with_backend(selection: BackendSelection) -> Result<Self> {
        match selection {
            BackendSelection::VendorHardwareService => {
                use android_hardware_power_stats::powerstats::IPowerStats;
                let descriptor =
                    <IPowerStats::BpPowerStats as IPowerStats::IPowerStats>::get_descriptor();
                // let i = binder::get_interface::<dyn IPowerStats::IPowerStats>()?;
                let i = binder::get_service(&format!("{}/default", descriptor))
                    .ok_or(StatusCode::NAME_NOT_FOUND)?;
                let i = android_hardware_power_stats::IPowerStats::BpPowerStats::from_binder(i)?;
                Ok(Self {
                    backend: Backend::VendorHardwareService(i),
                })
            }
            BackendSelection::SystemJavaService => {
                // let i = binder::get_interface::<
                //     // dyn android_os_powerstatsservice::powerstatsservice::IPowerStatsService,
                // >("powerstats");
                let i = binder::get_service("powerstats").ok_or(StatusCode::NAME_NOT_FOUND)?;
                let i = BpPowerStatsService::from_binder(i)?;
                Ok(Self {
                    backend: Backend::SystemJavaService(i),
                })
            }
        }
    }

    pub fn energy_meters(&self) -> Result<Vec<EnergyMeter>> {
        match &self.backend {
            Backend::VendorHardwareService(s) => {
                let meters = s.getEnergyMeterInfo()?;
                Ok(meters
                    .into_iter()
                    .map(|channel| EnergyMeter {
                        id: channel.id,
                        name: channel.name,
                        subsystem: Some(channel.subsystem),
                    })
                    .collect())
            }
            Backend::SystemJavaService(s) => {
                let (receiver, chan) =
                    android_os_powerstatsservice::ReceiveSupportedPowerMonitors::new();
                let receiver = result_receiver::ResultReceiver::new(receiver);
                // TODO: Since we pass a borrow, can we get access to the contents again?
                s.getSupportedPowerMonitors(&receiver)?;
                let monitors = chan.recv().unwrap();

                Ok(monitors
                    .into_iter()
                    // TODO: Filter on r#type?
                    .inspect(|pm| {
                        dbg!(pm);
                    })
                    // TODO: Also provide the aggregate (i.e. summation of two GPU regulators) "consumer" types?
                    .filter(|pm| pm.r#type == PowerMonitorType::POWER_MONITOR_TYPE_MEASUREMENT)
                    .map(|pm| EnergyMeter {
                        id: pm.index,
                        name: pm.name,
                        subsystem: None, // Typically wrapped in the name...
                    })
                    .collect())
            }
        }
    }

    // pub fn energy_reading(&self, meter_ids: &[i32]) -> Result<Vec<EnergyReading>> {
    //     match &self.backend {
    //         Backend::VendorHardwareService(s) => {
    //             let readings = s.readEnergyMeter()
    //             // TODO: This should be used with
    //             let readings = s.getEnergyConsumed()
    //         },
    //         Backend::SystemJavaService(_) => todo!(),
    //     }
    // }
}

#[doc(alias = "android.os.PowerMonitor")]
#[doc(alias = "android.hardware.power.stats.Channel")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnergyMeter {
    pub id: i32,
    pub name: String,
    /// Not available on [`Backend::SystemJavaService`], this is typically appended with a colon (`:`) to [`Self::name`].
    pub subsystem: Option<String>,
    // TODO: Add "optional" type based on per-subsystem aggregation that the system service does
    // (i.e. it sums all regulators feeding into the same CPU cluster, or into the GPU or other block)
}

#[doc(alias = "android.os.PowerMonitorReadings")]
#[doc(alias = "android.hardware.power.stats.EnergyConsumerResult")]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct EnergyReading {
    /// Monotonic timestamp since boot
    // pub timestamp_ms: i64,
    pub timestamp: Duration,
    /// Energy in `uWs` (uJ)
    pub energy_uws: i64,
}
