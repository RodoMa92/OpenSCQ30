// Automatically generated by flapigen
package com.oppzippy.openscq30.libbindings;


public final class SetSoundModeOkPacket {

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