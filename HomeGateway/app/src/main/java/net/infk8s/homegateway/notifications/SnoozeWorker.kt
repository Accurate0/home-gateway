package net.infk8s.homegateway.notifications

import android.content.Context
import androidx.work.CoroutineWorker
import androidx.work.WorkerParameters

class SnoozeWorker(context: Context, params: WorkerParameters) : CoroutineWorker(context, params) {
    override suspend fun doWork(): Result {
        val body = inputData.getString(KEY_BODY) ?: return Result.failure()
        val tag = inputData.getString(KEY_TAG) ?: body
        val rawActions = inputData.getString(KEY_ACTIONS).orEmpty()

        val data = buildMap {
            put("title", inputData.getString(KEY_TITLE) ?: "Home Gateway")
            put("body", body)
            put("category", inputData.getString(KEY_CATEGORY) ?: NotificationCategory.GENERAL.name)
            put("tag", tag)
            put("actions", rawActions)
            inputData.getString(KEY_NOTIFICATION_ID)?.let { put(PushPayload.KEY_NOTIFICATION_ID, it) }
        }

        val payload = PushPayload.from(data) ?: return Result.failure()

        Notifications.show(applicationContext, payload)

        return Result.success()
    }

    companion object {
        const val KEY_TITLE = "title"
        const val KEY_BODY = "body"
        const val KEY_CATEGORY = "category"
        const val KEY_TAG = "tag"
        const val KEY_ACTIONS = "actions"
        const val KEY_NOTIFICATION_ID = "notification_id"
    }
}
