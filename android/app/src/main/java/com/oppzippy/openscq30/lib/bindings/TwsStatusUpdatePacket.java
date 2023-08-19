// Automatically generated by flapigen
package com.oppzippy.openscq30.lib.bindings;
import androidx.annotation.NonNull;

public final class TwsStatusUpdatePacket {
    @Override
    public boolean equals(Object obj) {
        if (obj instanceof TwsStatusUpdatePacket)
            return ((TwsStatusUpdatePacket)obj).rustEq(this);
        return false;
    }


    private final boolean rustEq(@NonNull TwsStatusUpdatePacket other) {
        long a0 = other.mNativeObj;
        boolean ret = do_rustEq(mNativeObj, a0);

        JNIReachabilityFence.reachabilityFence1(other);

        return ret;
    }
    private static native boolean do_rustEq(long self, long other);

    public TwsStatusUpdatePacket() throws Exception {
        mNativeObj = init();
    }
    private static native long init() throws Exception;

    public final short hostDevice() {
        short ret = do_hostDevice(mNativeObj);

        return ret;
    }
    private static native short do_hostDevice(long self);

    public final boolean twsStatus() {
        boolean ret = do_twsStatus(mNativeObj);

        return ret;
    }
    private static native boolean do_twsStatus(long self);

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
    /*package*/ TwsStatusUpdatePacket(InternalPointerMarker marker, long ptr) {
        assert marker == InternalPointerMarker.RAW_PTR;
        this.mNativeObj = ptr;
    }
    /*package*/ long mNativeObj;
}