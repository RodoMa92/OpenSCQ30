// Automatically generated by flapigen
package com.oppzippy.openscq30.lib.bindings;
import androidx.annotation.NonNull;

public final class EqualizerConfiguration {
    @Override
    public boolean equals(Object obj) {
        if (obj instanceof EqualizerConfiguration)
            return ((EqualizerConfiguration)obj).rustEq(this);
        return false;
    }


    private final boolean rustEq(@NonNull EqualizerConfiguration other) {
        long a0 = other.mNativeObj;
        boolean ret = do_rustEq(mNativeObj, a0);

        JNIReachabilityFence.reachabilityFence1(other);

        return ret;
    }
    private static native boolean do_rustEq(long self, long other);

    public EqualizerConfiguration(@NonNull PresetEqualizerProfile preset_profile) {
        int a0 = preset_profile.getValue();
        mNativeObj = init(a0);
        JNIReachabilityFence.reachabilityFence1(preset_profile);
    }
    private static native long init(int preset_profile);

    public EqualizerConfiguration(@NonNull VolumeAdjustments volume_adjustments) {
        long a0 = volume_adjustments.mNativeObj;
        mNativeObj = init(a0);
        JNIReachabilityFence.reachabilityFence1(volume_adjustments);
    }
    private static native long init(long volume_adjustments);

    public final int profileId() {
        int ret = do_profileId(mNativeObj);

        return ret;
    }
    private static native int do_profileId(long self);

    public final @NonNull java.util.Optional<PresetEqualizerProfile> presetProfile() {
        int ret = do_presetProfile(mNativeObj);
        java.util.Optional<PresetEqualizerProfile> convRet;
        if (ret != -1) {
            convRet = java.util.Optional.of(PresetEqualizerProfile.fromInt(ret));
        } else {
            convRet = java.util.Optional.empty();
        }

        return convRet;
    }
    private static native int do_presetProfile(long self);

    public final @NonNull VolumeAdjustments volumeAdjustments() {
        long ret = do_volumeAdjustments(mNativeObj);
        VolumeAdjustments convRet = new VolumeAdjustments(InternalPointerMarker.RAW_PTR, ret);

        return convRet;
    }
    private static native long do_volumeAdjustments(long self);

    public final boolean equals(@NonNull EqualizerConfiguration other) {
        long a0 = other.mNativeObj;
        boolean ret = do_equals(mNativeObj, a0);

        JNIReachabilityFence.reachabilityFence1(other);

        return ret;
    }
    private static native boolean do_equals(long self, long other);

    public synchronized void delete() {
        if (mNativeObj != 0) {
            do_delete(mNativeObj);
            mNativeObj = 0;
       }
    }
    @Override
    protected void finalize() throws Throwable {
        try {
            delete();
        }
        finally {
             super.finalize();
        }
    }
    private static native void do_delete(long me);
    /*package*/ EqualizerConfiguration(InternalPointerMarker marker, long ptr) {
        assert marker == InternalPointerMarker.RAW_PTR;
        this.mNativeObj = ptr;
    }
    /*package*/ long mNativeObj;
}