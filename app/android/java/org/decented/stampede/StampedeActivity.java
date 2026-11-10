package org.decented.stampede;

import android.app.NativeActivity;
import android.content.Intent;
import android.os.Bundle;
import dev.forgesworn.release.Updater;
import java.io.File;

/**
 * NativeActivity that keeps the newest intent. Stock NativeActivity drops
 * onNewIntent, so a link opened while the game runs (singleTask) would leave
 * getIntent() on the launch intent. The native side reads getIntent() over JNI
 * (app/src/platform/android.rs) and consumes the link with setData(null).
 *
 * It also hands itself to {@link Updater} (the release kit's: in-app updates and the More games
 * installs need an Activity to show Android's install prompt) and tells it when
 * the game comes back to the front.
 */
public class StampedeActivity extends NativeActivity {
    private TiltInput tilt;
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        Updater.attach(this);
        dropOldUpdaterCache();
        tilt = new TiltInput(this);
        super.onCreate(savedInstanceState);
    }

    /**
     * Before the release kit, the updater downloaded into cacheDir/updates (an APK is ~66 MB).
     * The kit uses its own directory and never looks there, so clear what an update made by
     * the old updater may have left behind.
     */
    private void dropOldUpdaterCache() {
        try {
            File dir = new File(getCacheDir(), "updates");
            File[] old = dir.listFiles();
            if (old != null) {
                for (File f : old) {
                    f.delete();
                }
            }
            dir.delete();
        } catch (Throwable t) {
            // Only disk space is at stake; never let this stop the game starting.
        }
    }

    @Override
    protected void onResume() {
        super.onResume();
        tilt.resume();
        Updater.onResume();
    }

    @Override
    protected void onPause() {
        tilt.pause();
        super.onPause();
    }

    public void setTiltEnabled(boolean enabled) {
        runOnUiThread(() -> tilt.enable(enabled));
    }

    public float[] readTilt() { return tilt.read(); }

    @Override
    protected void onNewIntent(Intent intent) {
        super.onNewIntent(intent);
        setIntent(intent);
    }
}
