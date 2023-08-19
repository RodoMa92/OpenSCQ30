// Automatically generated by flapigen
package com.oppzippy.openscq30.lib.bindings;
import androidx.annotation.NonNull;

public final class StereoVolumeAdjustments {
    @Override
    public boolean equals(Object obj) {
        if (obj instanceof StereoVolumeAdjustments)
            return ((StereoVolumeAdjustments)obj).rustEq(this);
        return false;
    }


    private final boolean rustEq(@NonNull StereoVolumeAdjustments other) {
        long a0 = other.mNativeObj;
        boolean ret = do_rustEq(mNativeObj, a0);

        JNIReachabilityFence.reachabilityFence1(other);

        return ret;
    }
    private static native boolean do_rustEq(long self, long other);

    public StereoVolumeAdjustments(@NonNull VolumeAdjustments left, @NonNull VolumeAdjustments right) {
        long a0 = left.mNativeObj;
        left.mNativeObj = 0;

        long a1 = right.mNativeObj;
        right.mNativeObj = 0;

        mNativeObj = init(a0, a1);
        JNIReachabilityFence.reachabilityFence2(left, right);
    }
    private static native long init(long left, long right);

    public final @NonNull VolumeAdjustments left() {
        long ret = do_left(mNativeObj);
        VolumeAdjustments convRet = new VolumeAdjustments(InternalPointerMarker.RAW_PTR, ret);

        return convRet;
    }
    private static native long do_left(long self);

    public final @NonNull VolumeAdjustments right() {
        long ret = do_right(mNativeObj);
        VolumeAdjustments convRet = new VolumeAdjustments(InternalPointerMarker.RAW_PTR, ret);

        return convRet;
    }
    private static native long do_right(long self);

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
    /*package*/ StereoVolumeAdjustments(InternalPointerMarker marker, long ptr) {
        assert marker == InternalPointerMarker.RAW_PTR;
        this.mNativeObj = ptr;
    }
    /*package*/ long mNativeObj;
}