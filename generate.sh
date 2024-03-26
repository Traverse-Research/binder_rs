#! /usr/bin/bash

set -ex

# P=android-hardware-interfaces/power/stats/aidl/aidl_api/android.hardware.power.stats/current/
# aidl --structured --stability=vintf --lang=rust -I$P $P/android/hardware/power/stats/*.aidl -o android-powerstats/src/

P=android-frameworks-base/core/java
aidl --lang=rust -I$P $P/android/os/IPowerStatsService.aidl -o android-powerstats/src/
