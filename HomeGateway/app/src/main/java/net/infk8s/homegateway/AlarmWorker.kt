package net.infk8s.homegateway
import android.app.AlarmManager
import android.content.Context
import android.util.Log
import androidx.core.content.getSystemService
import androidx.work.CoroutineWorker
import androidx.work.WorkerParameters
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import org.json.JSONObject
import java.text.SimpleDateFormat
import java.util.Calendar
import java.util.Date
import java.util.GregorianCalendar
import java.util.Locale
import java.util.TimeZone
import java.util.concurrent.TimeUnit

class AlarmWorker(appContext: Context, workerParams: WorkerParameters):
    CoroutineWorker(appContext, workerParams) {

    override suspend fun doWork(): Result {
        val alarmManager = this.applicationContext.getSystemService<AlarmManager>()!!
        val alarmClockTriggerTime = alarmManager.nextAlarmClock.triggerTime

        val cal: Calendar = GregorianCalendar()
        cal.timeInMillis = alarmClockTriggerTime

        val dateFormat = "yyyy-MM-dd'T'HH:mm:ssXXX"
        val sdf = SimpleDateFormat(dateFormat, Locale.getDefault())
        sdf.timeZone = TimeZone.getTimeZone("Australia/Perth")

        val localTime = sdf.format(Date(alarmClockTriggerTime))

        val payload = mapOf(
            "local_time" to localTime,
        )

        val client = OkHttpClient.Builder()
            .callTimeout(30, TimeUnit.SECONDS)
            .build()

        val apiKey = BuildConfig.HOME_GATEWAY_INGEST_API_SECRET
        var url = "https://home.anurag.sh/v1/ingest/home/alarm"
        if(BuildConfig.DEBUG) {
            url = "http://192.168.0.104:8000/v1/ingest/home/alarm"
        }

        val request = Request.Builder()
            .post(
                JSONObject(payload).toString().toRequestBody("application/json".toMediaType())
            )
            .header("X-Webhook-Secret", apiKey)
            .url(url)
            .build()

        client.newCall(request).execute().use { response ->
            if (response.isSuccessful) {
                Log.i("AlarmWorker", "sent alarm details via url")
            } else {
                Log.e("AlarmWorker", String.format("failed to send alarm details %d", response.code))
            }
        }

        return Result.success()
    }
}
