use binder::binder_impl::{Deserialize, Serialize};

pub(crate) mod mangled {

    pub(crate) type _7_android_2_os_6_Bundle = super::Bundle;
}

/// https://cs.android.com/android/platform/superproject/main/+/main:frameworks/base/core/java/android/os/BaseBundle.java
#[derive(Debug)]
pub(crate) struct Bundle {}
impl Serialize for Bundle {
    fn serialize(
        &self,
        parcel: &mut binder::binder_impl::BorrowedParcel<'_>,
    ) -> Result<(), binder::StatusCode> {
        todo!()
    }
}

impl Deserialize for Bundle {
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
