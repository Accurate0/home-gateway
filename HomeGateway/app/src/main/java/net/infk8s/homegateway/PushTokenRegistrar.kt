package net.infk8s.homegateway

import android.util.Log
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import org.json.JSONObject
import java.util.concurrent.TimeUnit

/// Registers this device's FCM token with the gateway so it can receive pushes.
/// Mirrors the auth/URL pattern used by [AlarmWorker].
object PushTokenRegistrar {
    fun register(token: String) {
        val payload = mapOf("token" to token)

        val client = OkHttpClient.Builder()
            .callTimeout(30, TimeUnit.SECONDS)
            .build()

        val apiKey = BuildConfig.HOME_GATEWAY_INGEST_API_SECRET
        var url = "https://home.anurag.sh/v1/ingest/home/push-token"
        if (BuildConfig.DEBUG) {
            url = "http://192.168.0.104:8000/v1/ingest/home/push-token"
        }

        val request = Request.Builder()
            .post(
                JSONObject(payload).toString().toRequestBody("application/json".toMediaType())
            )
            .header("X-Webhook-Secret", apiKey)
            .url(url)
            .build()

        try {
            client.newCall(request).execute().use { response ->
                if (response.isSuccessful) {
                    Log.i("PushTokenRegistrar", "registered push token")
                } else {
                    Log.e("PushTokenRegistrar", "failed to register token ${response.code}")
                }
            }
        } catch (e: Exception) {
            Log.e("PushTokenRegistrar", "error registering token", e)
        }
    }
}
