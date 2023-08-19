// Automatically generated by flapigen
package com.oppzippy.openscq30.lib.bindings;
import androidx.annotation.NonNull;

public final class SetSoundModeOkPacket {
    @Override
    public boolean equals(Object obj) {
        if (obj instanceof SetSoundModeOkPacket)
            return ((SetSoundModeOkPacket)obj).rustEq(this);
        return false;
    }


    private final boolean rustEq(@NonNull SetSoundModeOkPacket other) {
        long a0 = other.mNativeObj;
        boolean ret = do_rustEq(mNativeObj, a0);

        JNIReachabilityFence.reachabilityFence1(other);

        return ret;
    }
    private static native boolean do_rustEq(long self, long other);

    public SetSoundModeOkPacket() throws Exception {
        mNativeObj = init();
    }
    private static native long init() throws Exception;

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
    /*package*/ SetSoundModeOkPacket(InternalPointerMarker marker, long ptr) {
        assert marker == InternalPointerMarker.RAW_PTR;
        this.mNativeObj = ptr;
    }
    /*package*/ long mNativeObj;
}