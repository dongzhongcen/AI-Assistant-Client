package com.dzc.aiassistant;

import android.annotation.SuppressLint;
import android.app.Activity;
import android.graphics.Color;
import android.os.Bundle;
import android.view.View;
import android.webkit.CookieManager;
import android.webkit.JavascriptInterface;
import android.webkit.WebSettings;
import android.webkit.WebView;
import android.webkit.WebViewClient;

import org.json.JSONArray;
import org.json.JSONObject;

import java.io.BufferedReader;
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

        webView = new WebView(this);
        webView.setBackgroundColor(Color.rgb(246, 247, 249));
        webView.setLayerType(View.LAYER_TYPE_HARDWARE, null);
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
}
