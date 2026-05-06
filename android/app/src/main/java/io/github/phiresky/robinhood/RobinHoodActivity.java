package io.github.phiresky.robinhood;

public class RobinHoodActivity extends android.app.NativeActivity {
    static {
        System.loadLibrary("robin_rs");
    }

    private static native void nativeOnBackPressed();

    private boolean backCallbackRegistered = false;

    private final android.window.OnBackInvokedCallback backCallback =
            new android.window.OnBackInvokedCallback() {
                @Override
                public void onBackInvoked() {
                    android.util.Log.i("RobinHoodActivity", "onBackInvoked");
                    nativeOnBackPressed();
                }
            };

    @Override
    protected void onCreate(android.os.Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        android.util.Log.i("RobinHoodActivity", "onCreate");
        registerBackCallback();
    }

    @Override
    protected void onResume() {
        super.onResume();
        android.util.Log.i("RobinHoodActivity", "onResume");
        getWindow().getDecorView().post(new Runnable() {
            @Override
            public void run() {
                registerBackCallback();
            }
        });
    }

    private void registerBackCallback() {
        if (android.os.Build.VERSION.SDK_INT >= 33) {
            if (backCallbackRegistered) {
                getOnBackInvokedDispatcher().unregisterOnBackInvokedCallback(backCallback);
                backCallbackRegistered = false;
            }
            getOnBackInvokedDispatcher().registerOnBackInvokedCallback(
                    android.window.OnBackInvokedDispatcher.PRIORITY_OVERLAY,
                    backCallback);
            backCallbackRegistered = true;
            android.util.Log.i("RobinHoodActivity", "registered back callback");
        }
    }

    @Override
    public boolean dispatchKeyEvent(android.view.KeyEvent event) {
        if (event.getKeyCode() == android.view.KeyEvent.KEYCODE_BACK
                && event.getAction() == android.view.KeyEvent.ACTION_UP) {
            android.util.Log.i("RobinHoodActivity", "dispatchKeyEvent BACK");
            nativeOnBackPressed();
            return true;
        }
        return super.dispatchKeyEvent(event);
    }

    @Override
    public void onBackPressed() {
        android.util.Log.i("RobinHoodActivity", "onBackPressed");
        nativeOnBackPressed();
    }

    public void finishFromNative(int exitCode) {
        android.util.Log.i("RobinHoodActivity", "finishFromNative exitCode=" + exitCode);
        if (android.os.Build.VERSION.SDK_INT >= 21) {
            finishAndRemoveTask();
        } else {
            finish();
        }
    }
}
