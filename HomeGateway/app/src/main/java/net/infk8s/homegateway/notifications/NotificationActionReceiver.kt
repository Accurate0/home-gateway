package net.infk8s.homegateway.notifications

import android.app.PendingIntent
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.util.Log
import androidx.core.app.NotificationManagerCompat
import androidx.work.Data
import androidx.work.OneTimeWorkRequestBuilder
import androidx.work.WorkManager
import java.util.concurrent.TimeUnit
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.launch
import net.infk8s.homegateway.graphql.ApolloProvider
import net.infk8s.homegateway.graphql.RunWorkflowMutation

class NotificationActionReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val id = intent.getIntExtra(EXTRA_ID, 0)
        val tag = intent.getStringExtra(EXTRA_TAG).orEmpty()

        NotificationManagerCompat.from(context).cancel(id)

        when (intent.getStringExtra(EXTRA_KIND)) {
            KIND_RUN_WORKFLOW -> {
                val slug = intent.getStringExtra(EXTRA_SLUG) ?: return
                val pending = goAsync()

                CoroutineScope(SupervisorJob() + Dispatchers.IO).launch {
                    try {
                        val response = ApolloProvider.client.mutation(RunWorkflowMutation(slug)).execute()
                        if (response.hasErrors()) {
                            Log.w(TAG, "runWorkflow($slug) returned errors: ${response.errors}")
                        }
                    } catch (e: Exception) {
                        Log.e(TAG, "runWorkflow($slug) failed", e)
                    } finally {
                        pending.finish()
                    }
                }
            }

            KIND_SNOOZE -> {
                val seconds = intent.getLongExtra(EXTRA_SECONDS, 0L)
                val data = Data.Builder()
                    .putString(SnoozeWorker.KEY_TITLE, intent.getStringExtra(EXTRA_TITLE))
                    .putString(SnoozeWorker.KEY_BODY, intent.getStringExtra(EXTRA_BODY))
                    .putString(SnoozeWorker.KEY_CATEGORY, intent.getStringExtra(EXTRA_CATEGORY))
                    .putString(SnoozeWorker.KEY_TAG, tag)
                    .putString(SnoozeWorker.KEY_ACTIONS, intent.getStringExtra(EXTRA_ACTIONS))
                    .build()

                WorkManager.getInstance(context).enqueue(
                    OneTimeWorkRequestBuilder<SnoozeWorker>()
                        .setInitialDelay(seconds, TimeUnit.SECONDS)
                        .setInputData(data)
                        .build()
                )
            }

            else -> Unit
        }
    }

    companion object {
        private const val TAG = "NotificationAction"

        const val EXTRA_ID = "id"
        const val EXTRA_TAG = "tag"
        const val EXTRA_KIND = "kind"
        const val EXTRA_SLUG = "slug"
        const val EXTRA_SECONDS = "seconds"
        const val EXTRA_TITLE = "title"
        const val EXTRA_BODY = "body"
        const val EXTRA_CATEGORY = "category"
        const val EXTRA_ACTIONS = "actions"

        const val KIND_RUN_WORKFLOW = "run_workflow"
        const val KIND_SNOOZE = "snooze"
        const val KIND_DISMISS = "dismiss"

        fun pendingIntent(
            context: Context,
            id: Int,
            payload: PushPayload,
            index: Int,
            action: PushAction,
        ): PendingIntent {
            val intent = Intent(context, NotificationActionReceiver::class.java)
                .putExtra(EXTRA_ID, id)
                .putExtra(EXTRA_TAG, payload.tag)
                .putExtra(EXTRA_TITLE, payload.title)
                .putExtra(EXTRA_BODY, payload.body)
                .putExtra(EXTRA_CATEGORY, payload.category.name)
                .putExtra(EXTRA_ACTIONS, payload.rawActions)

            when (action) {
                is PushAction.RunWorkflow -> intent
                    .putExtra(EXTRA_KIND, KIND_RUN_WORKFLOW)
                    .putExtra(EXTRA_SLUG, action.slug)

                is PushAction.Snooze -> intent
                    .putExtra(EXTRA_KIND, KIND_SNOOZE)
                    .putExtra(EXTRA_SECONDS, action.seconds)

                is PushAction.Dismiss -> intent.putExtra(EXTRA_KIND, KIND_DISMISS)
            }

            return PendingIntent.getBroadcast(
                context,
                id * 31 + index,
                intent,
                PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
            )
        }
    }
}
