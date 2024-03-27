use std::{any::Any, time::Duration};

use binder::{
    binder_impl::{Deserialize, Parcel},
    FromIBinder, Strong,
};

use crate::iresultreceiver::{BnResultReceiver, IResultReceiver};

#[path = "android/hardware/power/stats/mod.rs"]
pub mod powerstats;

pub(crate) mod mangled {
    pub(crate) use super::powerstats::mangled::*;

    pub(crate) use super::iresultreceiver::mangled::*;
    pub(crate) type _7_android_2_os_14_ResultReceiver = super::ResultReceiver;
    pub(crate) type _7_android_2_os_6_Bundle = super::Bundle;
}

/// https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/core/java/android/os/BaseBundle.java
#[derive(Debug)]
pub(crate) struct Bundle {}
impl binder::binder_impl::Serialize for Bundle {
    fn serialize(
        &self,
        parcel: &mut binder::binder_impl::BorrowedParcel<'_>,
    ) -> Result<(), binder::StatusCode> {
        todo!()
    }
}

impl binder::binder_impl::Deserialize for Bundle {
    fn deserialize(
        parcel: &binder::binder_impl::BorrowedParcel<'_>,
    ) -> Result<Self, binder::StatusCode> {
        dbg!(parcel.get_data_size());

        // Parse nullability because of writeTypedObject
        // https://cs.android.com/android/platform/superproject/main/+/main:out/soong/.intermediates/frameworks/base/framework-minus-apex-intdefs/android_common/e18b8e8d84cb9f664aa09a397b08c165/xref50/srcjars.xref/com/android/internal/os/IResultReceiver.java;l=118;drc=190beaa49a35da1d9dcf66be9cfccfd23b0eb467
        let is_set: i32 = parcel.read()?;
        assert!(is_set == 1);

        let length: i32 = parcel.read()?;
        dbg!(length);
        assert!(length >= 0, "Bad length {length}");
        if length == 0 {
            return Ok(Self {}); // Empty
        }

        // https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/core/java/android/os/BaseBundle.java;l=1877-1879;drc=190beaa49a35da1d9dcf66be9cfccfd23b0eb467
        const BUNDLE_MAGIC: i32 = 0x4C444E42; // 'B' 'N' 'D' 'L'
        const BUNDLE_MAGIC_NATIVE: i32 = 0x4C444E44; // 'B' 'N' 'D' 'N'
        let magic: i32 = parcel.read()?;
        let is_java_bundle = magic == BUNDLE_MAGIC;
        let is_native_bundle = magic == BUNDLE_MAGIC_NATIVE;
        dbg!(is_java_bundle, is_native_bundle);

        // https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/core/java/android/os/BaseBundle.java;l=459;drc=190beaa49a35da1d9dcf66be9cfccfd23b0eb467
        let count: i32 = parcel.read()?;
        dbg!(count);
        for i in 0..count {
            let str: String = parcel.read()?;
            dbg!(str);
            // https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/core/java/android/os/Parcel.java;l=4528;drc=190beaa49a35da1d9dcf66be9cfccfd23b0eb467
            const VAL_PARCELABLEARRAY: i32 = 16; // length-prefixed
            let t: i32 = parcel.read()?;
            dbg!(&t);
        }
        Ok(Self {})
    }
}

// https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/core/java/android/os/ResultReceiver.aidl;l=20;drc=b741c646c69ebdcfbc3287297a312a4ee1aeb5fe
// ERROR: android-frameworks-base/core/java/android/os/ResultReceiver.aidl:20.37-52: Refusing to generate code with unstructured parcelables. Declared parcelables should be in their own file and/or cannot be used with --structured interfaces.
// Unstructured parcel only has a definition in Java:
// https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/core/java/android/os/ResultReceiver.java
#[derive(Debug)]
pub(crate) struct ResultReceiver {
    // binder: Box<dyn IResultReceiver>,
    binder: Strong<dyn IResultReceiver>,
}

impl binder::binder_impl::Serialize for ResultReceiver {
    fn serialize(
        &self,
        parcel: &mut binder::binder_impl::BorrowedParcel<'_>,
    ) -> Result<(), binder::StatusCode> {
        // TODO: the service implementation should have called writeTypedObject(), which first writes an integer 0 or 1 to describe nullability:
        // https://cs.android.com/android/platform/superproject/main/+/main:out/soong/.intermediates/frameworks/base/framework-minus-apex-intdefs/android_common/e18b8e8d84cb9f664aa09a397b08c165/xref50/srcjars.xref/android/os/IPowerStatsService.java;l=92;drc=190beaa49a35da1d9dcf66be9cfccfd23b0eb467
        parcel.write(&1i32)?;
        // Only after that the contents of ResultReceiver should be passed:
        // https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/core/java/android/os/ResultReceiver.java;l=125;drc=9e8f83db6d969f1e1f47ffa0b0390d867491235b
        self.binder.as_binder().serialize(parcel)
    }
}

impl binder::binder_impl::Deserialize for ResultReceiver {
    fn deserialize(
        parcel: &binder::binder_impl::BorrowedParcel<'_>,
    ) -> Result<Self, binder::StatusCode> {
        use binder::FromIBinder;
        let is_set: i32 = parcel.read()?; // Same hack because of serialize()
        binder::SpIBinder::deserialize(parcel).map(|binder| Self {
            // Proxy
            binder: todo!(),
            // binder: Box::new(IResultReceiver::try_from(binder).unwrap()),
        })
    }
}

#[path = "com/android/internal/os/IResultReceiver.rs"]
pub mod iresultreceiver;
#[path = "android/os/IPowerStatsService.rs"]
pub mod powerstatsservice;

pub fn android_hardware_power_powerstats() -> binder::Result<()> {
    use powerstats::IPowerStats;
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

struct ReceiveSupportedPowerMonitors;

impl binder::Interface for ReceiveSupportedPowerMonitors {}
impl IResultReceiver for ReceiveSupportedPowerMonitors {
    fn r#send(&self, _arg_resultCode: i32, _arg_resultData: &Bundle) -> binder::Result<()> {
        dbg!(_arg_resultCode, _arg_resultData);
        Ok(())
    }
}

pub fn android_os_powerstatsservice() -> binder::Result<()> {
    let i = binder::get_interface::<dyn powerstatsservice::IPowerStatsService>("powerstats")?;
    dbg!(&i);

    let result = ReceiveSupportedPowerMonitors;
    let binder = BnResultReceiver::new_binder(
        result,
        binder::BinderFeatures {
            set_requesting_sid: true,
            _non_exhaustive: (),
        },
    );
    let receiver = ResultReceiver { binder };
    // let parcel = Parcel::new();
    // parcel.write_binder(receiver);
    i.getSupportedPowerMonitors(&receiver)?;
    // i.into_sync();
    // dbg!(result);
    dbg!(&receiver);

    Ok(())
}

pub fn pull_data() -> binder::Result<()> {
    // android_hardware_power_powerstats()?;
    android_os_powerstatsservice()?;
    Ok(())
}
