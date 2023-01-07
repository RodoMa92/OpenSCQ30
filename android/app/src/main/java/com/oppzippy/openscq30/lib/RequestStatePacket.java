// Automatically generated by flapigen
package com.oppzippy.openscq30.lib;
import androidx.annotation.NonNull;

public final class RequestStatePacket {

    public RequestStatePacket() {
        mNativeObj = init();
    }
    private static native long init();

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
    /*package*/ RequestStatePacket(InternalPointerMarker marker, long ptr) {
        assert marker == InternalPointerMarker.RAW_PTR;
        this.mNativeObj = ptr;
    }
    /*package*/ long mNativeObj;
}