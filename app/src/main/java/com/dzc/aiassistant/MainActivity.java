package com.dzc.aiassistant;

import android.annotation.SuppressLint;
import android.app.AlertDialog;
import android.app.Activity;
import android.content.ContentValues;
import android.content.ClipData;
import android.content.ClipboardManager;
import android.content.Context;
import android.graphics.Color;
import android.net.Uri;
import android.os.Bundle;
import android.os.Environment;
import android.provider.MediaStore;
import android.view.View;
import android.view.Window;
import android.view.WindowInsetsController;
import android.view.WindowManager;
import android.webkit.CookieManager;
import android.webkit.JavascriptInterface;
import android.webkit.WebSettings;
import android.webkit.WebView;
import android.webkit.WebViewClient;
import android.widget.Toast;

import org.json.JSONArray;
import org.json.JSONObject;

import java.io.BufferedReader;
import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.io.OutputStream;
import java.net.HttpURLConnection;
import java.net.URL;
import java.nio.charset.StandardCharsets;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

public class MainActivity extends Activity {
    private WebView webView;
    private final ExecutorService executor = Executors.newCachedThreadPool();

    @SuppressLint({"SetJavaScriptEnabled", "AddJavascriptInterface"})
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        configureEdgeToEdge();

        webView = new WebView(this);
        webView.setBackgroundColor(Color.TRANSPARENT);
        webView.setLayerType(View.LAYER_TYPE_HARDWARE, null);
        webView.setOverScrollMode(View.OVER_SCROLL_NEVER);
        setContentView(webView);

        WebSettings settings = webView.getSettings();
        settings.setJavaScriptEnabled(true);
        settings.setDomStorageEnabled(true);
        settings.setDatabaseEnabled(false);
        settings.setCacheMode(WebSettings.LOAD_NO_CACHE);
        settings.setAllowFileAccess(true);
        settings.setAllowContentAccess(true);
        settings.setMediaPlaybackRequiresUserGesture(false);
        settings.setUseWideViewPort(true);
        settings.setLoadWithOverviewMode(true);
        settings.setTextZoom(100);
        settings.setSaveFormData(false);
        settings.setSupportZoom(false);
        settings.setBuiltInZoomControls(false);
        settings.setDisplayZoomControls(false);

        CookieManager.getInstance().setAcceptCookie(false);
        webView.clearCache(true);
        webView.clearHistory();

        webView.setWebViewClient(new WebViewClient());
        webView.addJavascriptInterface(new AndroidBridge(), "AndroidBridge");
        webView.loadUrl("file:///android_asset/index.html");
    }

    private void configureEdgeToEdge() {
        Window window = getWindow();
        window.setStatusBarColor(Color.TRANSPARENT);
        window.setNavigationBarColor(Color.TRANSPARENT);
        window.getDecorView().setSystemUiVisibility(
                View.SYSTEM_UI_FLAG_LAYOUT_STABLE
                        | View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION
                        | View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
                        | View.SYSTEM_UI_FLAG_LIGHT_STATUS_BAR
        );
        if (android.os.Build.VERSION.SDK_INT >= 28) {
            WindowManager.LayoutParams params = window.getAttributes();
            params.layoutInDisplayCutoutMode = WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_SHORT_EDGES;
            window.setAttributes(params);
        }
        if (android.os.Build.VERSION.SDK_INT >= 30) {
            WindowInsetsController controller = window.getInsetsController();
            if (controller != null) {
                controller.setSystemBarsBehavior(WindowInsetsController.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE);
            }
        }
    }

    @Override
    protected void onDestroy() {
        executor.shutdownNow();
        if (webView != null) {
            webView.stopLoading();
            webView.clearCache(true);
            webView.clearHistory();
            webView.removeAllViews();
            webView.destroy();
        }
        clearDirectory(getCacheDir());
        super.onDestroy();
    }

    private void clearDirectory(java.io.File directory) {
        if (directory == null || !directory.exists()) {
            return;
        }
        java.io.File[] files = directory.listFiles();
        if (files == null) {
            return;
        }
        for (java.io.File file : files) {
            if (file.isDirectory()) {
                clearDirectory(file);
            }
            file.delete();
        }
    }

    private class AndroidBridge {
        @JavascriptInterface
        public void toast(String message) {
            runOnUiThread(new Runnable() {
                @Override
                public void run() {
                    Toast.makeText(MainActivity.this, message, Toast.LENGTH_SHORT).show();
                }
            });
        }

        @JavascriptInterface
        public String pasteClipboard() {
            ClipboardManager clipboard = (ClipboardManager) getSystemService(Context.CLIPBOARD_SERVICE);
            if (clipboard == null || !clipboard.hasPrimaryClip()) {
                return "";
            }
            ClipData data = clipboard.getPrimaryClip();
            if (data == null || data.getItemCount() == 0) {
                return "";
            }
            CharSequence text = data.getItemAt(0).coerceToText(MainActivity.this);
            return text == null ? "" : text.toString();
        }

        @JavascriptInterface
        public void confirmClear() {
            runOnUiThread(new Runnable() {
                @Override
                public void run() {
                    new AlertDialog.Builder(MainActivity.this)
                            .setTitle("清空会话")
                            .setMessage("确认清空所有本地会话？")
                            .setNegativeButton("取消", null)
                            .setPositiveButton("清空", new android.content.DialogInterface.OnClickListener() {
                                @Override
                                public void onClick(android.content.DialogInterface dialog, int which) {
                                    webView.evaluateJavascript("window.__clearLocalConversations && window.__clearLocalConversations();", null);
                                }
                            })
                            .show();
                }
            });
        }

        @JavascriptInterface
        public void exportConversations(String payload, String filename) {
            executor.execute(new Runnable() {
                @Override
                public void run() {
                    try {
                        String safeName = sanitizeFilename(filename);
                        if (android.os.Build.VERSION.SDK_INT >= 29) {
                            ContentValues values = new ContentValues();
                            values.put(MediaStore.Downloads.DISPLAY_NAME, safeName);
                            values.put(MediaStore.Downloads.MIME_TYPE, "application/json");
                            values.put(MediaStore.Downloads.RELATIVE_PATH, Environment.DIRECTORY_DOWNLOADS);
                            Uri uri = getContentResolver().insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, values);
                            if (uri == null) {
                                throw new Exception("无法创建下载文件");
                            }
                            try (OutputStream stream = getContentResolver().openOutputStream(uri)) {
                                if (stream == null) {
                                    throw new Exception("无法写入下载文件");
                                }
                                stream.write(payload.getBytes(StandardCharsets.UTF_8));
                            }
                        } else {
                            File dir = Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS);
                            if (!dir.exists()) {
                                dir.mkdirs();
                            }
                            File output = new File(dir, safeName);
                            try (FileOutputStream stream = new FileOutputStream(output)) {
                                stream.write(payload.getBytes(StandardCharsets.UTF_8));
                            }
                        }
                        toast("已导出到 Downloads/" + safeName);
                    } catch (Exception error) {
                        toast("导出失败：" + (error.getMessage() == null ? "未知错误" : error.getMessage()));
                    }
                }
            });
        }

        @JavascriptInterface
        public void chatCompletions(String rawRequest) {
            executor.execute(new Runnable() {
                @Override
                public void run() {
                String requestId = "";
                try {
                    JSONObject request = new JSONObject(rawRequest);
                    requestId = request.getString("requestId");
                    String baseUrl = trimTrailingSlash(request.optString("baseUrl", "https://api.openai.com/v1"));
                    String apiKey = request.getString("apiKey");

                    JSONObject payload = new JSONObject();
                    payload.put("model", request.getString("model"));
                    payload.put("temperature", request.optDouble("temperature", 0.7));
                    payload.put("stream", true);
                    payload.put("messages", request.getJSONArray("messages"));

                    HttpURLConnection connection = (HttpURLConnection) new URL(baseUrl + "/chat/completions").openConnection();
                    connection.setRequestMethod("POST");
                    connection.setConnectTimeout(30000);
                    connection.setReadTimeout(0);
                    connection.setDoOutput(true);
                    connection.setRequestProperty("Content-Type", "application/json; charset=utf-8");
                    connection.setRequestProperty("Authorization", "Bearer " + apiKey);

                    try (OutputStream output = connection.getOutputStream()) {
                        output.write(payload.toString().getBytes(StandardCharsets.UTF_8));
                    }

                    int status = connection.getResponseCode();
                    InputStream stream = status >= 200 && status < 300
                            ? connection.getInputStream()
                            : connection.getErrorStream();

                    if (status < 200 || status >= 300) {
                        String error = readAll(stream);
                        emitError(requestId, error.isEmpty() ? "HTTP " + status : error);
                        return;
                    }

                    readEventStream(requestId, stream);
                    emitDone(requestId);
                } catch (Exception error) {
                    emitError(requestId, error.getMessage() == null ? "Unknown error" : error.getMessage());
                }
                }
            });
        }
    }

    private void readEventStream(String requestId, InputStream stream) throws Exception {
        try (BufferedReader reader = new BufferedReader(new InputStreamReader(stream, StandardCharsets.UTF_8))) {
            String line;
            while ((line = reader.readLine()) != null) {
                line = line.trim();
                if (!line.startsWith("data:")) {
                    continue;
                }
                String data = line.substring(5).trim();
                if ("[DONE]".equals(data)) {
                    return;
                }
                JSONObject event = new JSONObject(data);
                JSONArray choices = event.optJSONArray("choices");
                if (choices == null || choices.length() == 0) {
                    continue;
                }
                JSONObject delta = choices.getJSONObject(0).optJSONObject("delta");
                if (delta == null || !delta.has("content")) {
                    continue;
                }
                emitDelta(requestId, delta.optString("content", ""));
            }
        }
    }

    private String readAll(InputStream stream) throws Exception {
        if (stream == null) {
            return "";
        }
        StringBuilder builder = new StringBuilder();
        try (BufferedReader reader = new BufferedReader(new InputStreamReader(stream, StandardCharsets.UTF_8))) {
            String line;
            while ((line = reader.readLine()) != null) {
                builder.append(line).append('\n');
            }
        }
        return builder.toString().trim();
    }

    private void emitDelta(String requestId, String delta) {
        evaluateCallback(requestId, "onDelta", JSONObject.quote(delta));
    }

    private void emitDone(String requestId) {
        evaluateCallback(requestId, "onDone", "");
    }

    private void emitError(String requestId, String message) {
        evaluateCallback(requestId, "onError", JSONObject.quote(message));
    }

    private void evaluateCallback(String requestId, String method, String argument) {
        if (requestId == null || requestId.isEmpty()) {
            return;
        }
        String script = "window.__androidChatCallbacks && window.__androidChatCallbacks['"
                + escapeJs(requestId)
                + "'] && window.__androidChatCallbacks['"
                + escapeJs(requestId)
                + "']." + method + "(" + argument + ");";
        runOnUiThread(new Runnable() {
            @Override
            public void run() {
                webView.evaluateJavascript(script, null);
            }
        });
    }

    private String trimTrailingSlash(String value) {
        while (value.endsWith("/")) {
            value = value.substring(0, value.length() - 1);
        }
        return value;
    }

    private String escapeJs(String value) {
        return value.replace("\\", "\\\\").replace("'", "\\'");
    }

    private String sanitizeFilename(String value) {
        return value.replaceAll("[\\\\/:*?\"<>|]", "_");
    }
}
