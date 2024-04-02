use std::{
    sync::mpsc::{channel, Receiver, Sender},
    time::Duration,
};

use binder::{
    binder_impl::{BorrowedParcel, Deserialize},
    Parcelable, StatusCode,
};

use crate::{
    bundle::{parcel_read_string8, Bundle, Object},
    result_receiver::{iresultreceiver::IResultReceiver, ResultReceiver},
};

#[path = "android/os/IPowerStatsService.rs"]
pub mod powerstatsservice;

// pub(crate) mod mangled {
//     pub(crate) use super::powerstatsservice::mangled::*;
// }

pub use powerstatsservice::IPowerStatsService;

/// Java-only parcelable
/// <https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/core/java/android/os/PowerMonitor.java;l=40;drc=82bdcd7ff7ba4962274f1d88caac0594ae964bef>
#[derive(Clone, Debug, Default)]
pub(crate) struct PowerMonitor {
    pub(crate) index: i32,
    pub(crate) r#type: PowerMonitorType,
    pub(crate) name: String,
}

impl Parcelable for PowerMonitor {
    fn write_to_parcel(&self, parcel: &mut BorrowedParcel<'_>) -> Result<(), StatusCode> {
        todo!()
    }

    fn read_from_parcel(&mut self, parcel: &BorrowedParcel<'_>) -> Result<(), StatusCode> {
        todo!()
        // self.index = parcel.read()?;
        // self.r#type = parcel.read()?;
        // self.name = parcel.read()?;
        // Ok(())
    }
}

impl Deserialize for PowerMonitor {
    fn deserialize(parcel: &BorrowedParcel<'_>) -> Result<Self, StatusCode> {
        Ok(Self {
            index: parcel.read()?,
            r#type: match parcel.read::<i32>()? {
                x if x == PowerMonitorType::POWER_MONITOR_TYPE_CONSUMER as i32 => {
                    PowerMonitorType::POWER_MONITOR_TYPE_CONSUMER
                }
                x if x == PowerMonitorType::POWER_MONITOR_TYPE_MEASUREMENT as i32 => {
                    PowerMonitorType::POWER_MONITOR_TYPE_MEASUREMENT
                }
                x => todo!("Unknown PowerMonitorType {x:?}"),
            },
            name: parcel_read_string8(parcel)?,
        })
    }
}
// impl_deserialize_for_parcelable!(PowerMonitor);

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) enum PowerMonitorType {
    /**
     * Power monitor corresponding to a subsystem. The energy value may be a direct pass-through
     * power rail measurement, or modeled in some fashion.  For example, an energy consumer may
     * represent a combination of multiple rails or a portion of a rail shared between subsystems,
     * e.g. WiFi and Bluetooth are often handled by the same chip, powered by a shared rail.
     * Some consumer names are standardized, others are not.
     */
    #[default]
    POWER_MONITOR_TYPE_CONSUMER = 0,

    /**
     * Power monitor corresponding to a directly measured power rail. Rails are device-specific:
     * no assumptions can be made about the source of those measurements across different devices,
     * even if they have the same name.
     */
    POWER_MONITOR_TYPE_MEASUREMENT = 1,
}

pub(crate) struct ReceiveSupportedPowerMonitors(Sender<Vec<PowerMonitor>>);
impl ReceiveSupportedPowerMonitors {
    pub(crate) fn new() -> (Self, Receiver<Vec<PowerMonitor>>) {
        let (s, r) = channel();
        (Self(s), r)
    }
}
impl binder::Interface for ReceiveSupportedPowerMonitors {}
impl IResultReceiver for ReceiveSupportedPowerMonitors {
    fn r#send(&self, code: i32, data: &Bundle) -> binder::Result<()> {
        assert_eq!(code, 0);
        let Object::ParcelableArray(monitors) = &data.0[powerstatsservice::KEY_MONITORS] else {
            panic!("Must have ParcelableArray")
        };

        let result = monitors
            .iter()
            .map(|monitor| {
                // TODO: Need owned box or Any for downcast
                // TODO: This is unsafe and wrong to cast a dynamic fat-pointer to a struct pointer
                let monitor: &Box<PowerMonitor> = unsafe { std::mem::transmute(monitor) };
                *monitor.clone()
                // unsafe {
                //     std::mem::transmute_copy::<dyn Parcelable, PowerMonitor>(monitor.as_ref())
                // }
            })
            .collect::<Vec<_>>();

        self.0.send(result).unwrap();

        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct PowerReading {
    pub(crate) timestampMs: i64,
    pub(crate) energyUws: i64,
}

pub(crate) struct ReceivePowerMonitorReadings(Sender<Vec<PowerReading>>);
impl ReceivePowerMonitorReadings {
    pub(crate) fn new() -> (Self, Receiver<Vec<PowerReading>>) {
        let (s, r) = channel();
        (Self(s), r)
    }
}
impl binder::Interface for ReceivePowerMonitorReadings {}
impl IResultReceiver for ReceivePowerMonitorReadings {
    fn r#send(&self, code: i32, data: &Bundle) -> binder::Result<()> {
        assert_eq!(code, 0);
        let Object::LongArray(timestamps) = &data.0[powerstatsservice::KEY_TIMESTAMPS] else {
            panic!("Must have LongArray")
        };
        let Object::LongArray(energy) = &data.0[powerstatsservice::KEY_ENERGY] else {
            panic!("Must have LongArray")
        };

        let result = timestamps
            .iter()
            .zip(energy)
            .map(|(t, e)| PowerReading {
                timestampMs: *t,
                energyUws: *e,
            })
            .collect::<Vec<_>>();

        self.0.send(result).unwrap();

        Ok(())
    }
}

pub fn run() -> binder::Result<()> {
    let i = binder::get_interface::<dyn powerstatsservice::IPowerStatsService>("powerstats")?;
    dbg!(&i);

    let (receiver, chan) = ReceiveSupportedPowerMonitors::new();
    let receiver = ResultReceiver::new(receiver);
    // TODO: Since we pass a borrow, can we get access to the contents again?
    i.getSupportedPowerMonitors(&receiver)?;
    let monitors = chan.recv().unwrap();

    dbg!(&monitors);
    let monitor_ids = monitors
        .iter()
        .filter(|m| m.name.ends_with(":GPU") || m.name.starts_with("GPU/"))
        .inspect(|m| println!("Querying power stats for {}", m.name))
        .map(|m| m.index)
        .collect::<Vec<_>>();

    let (receiver, chan) = ReceivePowerMonitorReadings::new();
    let receiver = ResultReceiver::new(receiver);
    for _ in 0..10 {
        i.getPowerMonitorReadings(&monitor_ids, &receiver)?;
        let readings = chan.recv().unwrap();
        assert_eq!(monitor_ids.len(), readings.len());
        dbg!(readings);
        std::thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}
