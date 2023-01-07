// Automatically generated by flapigen
package com.oppzippy.openscq30.lib;
import androidx.annotation.NonNull;

public final class SetEqualizerPacket {

    public SetEqualizerPacket(@NonNull EqualizerConfiguration configuration) {
        long a0 = configuration.mNativeObj;
        configuration.mNativeObj = 0;

        mNativeObj = init(a0);
        JNIReachabilityFence.reachabilityFence1(configuration);
    }
    private static native long init(long configuration);

    public final short [] bytes() {
        short [] ret = do_bytes(mNativeObj);

        return ret;
    }
    private static native short [] do_bytes(long self);

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
    /*package*/ SetEqualizerPacket(InternalPointerMarker marker, long ptr) {
        assert marker == InternalPointerMarker.RAW_PTR;
        this.mNativeObj = ptr;
    }
    /*package*/ long mNativeObj;
}