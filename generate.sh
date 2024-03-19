#! /usr/bin/bash

set -ex

P=android-hardware-interfaces/power/stats/aidl/aidl_api/android.hardware.power.stats/current/
aidl --structured --stability=vintf --lang=rust -I$P $P/android/hardware/power/stats/*.aidl -o android-powerstats/src/
