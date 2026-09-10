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
import net.infk8s.homegateway.graphql.type.NotificationInteractionKind

class NotificationActionReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val id = intent.getIntExtra(EXTRA_ID, 0)
        val tag = intent.getStringExtra(EXTRA_TAG).orEmpty()
        val notificationId = intent.getStringExtra(EXTRA_NOTIFICATION_ID)
        val kind = intent.getStringExtra(EXTRA_KIND)

        if (kind != KIND_SWIPED) {
            NotificationManagerCompat.from(context).cancel(id)
        }

        when (kind) {
            KIND_RUN_WORKFLOW -> {
                val slug = intent.getStringExtra(EXTRA_SLUG) ?: return

                launchAsync {
                    try {
                        val response = ApolloProvider.client.mutation(RunWorkflowMutation(slug)).execute()
                        if (response.hasErrors()) {
                            Log.w(TAG, "runWorkflow($slug) returned errors: ${response.errors}")
                        }
                    } catch (e: Exception) {
                        Log.e(TAG, "runWorkflow($slug) failed", e)
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
                    .putString(SnoozeWorker.KEY_NOTIFICATION_ID, notificationId)
                    .build()

                WorkManager.getInstance(context).enqueue(
                    OneTimeWorkRequestBuilder<SnoozeWorker>()
                        .setInitialDelay(seconds, TimeUnit.SECONDS)
                        .setInputData(data)
                        .build()
                )

                report(notificationId, NotificationInteractionKind.SNOOZED)
            }

            KIND_DISMISS -> report(notificationId, NotificationInteractionKind.DISMISSED)

            KIND_ACKNOWLEDGE -> report(notificationId, NotificationInteractionKind.ACKNOWLEDGED)

            KIND_SWIPED -> report(notificationId, NotificationInteractionKind.SWIPED)

            else -> Unit
        }
    }

    private fun launchAsync(block: suspend () -> Unit) {
        val pending = goAsync()

        CoroutineScope(SupervisorJob() + Dispatchers.IO).launch {
            try {
                block()
            } finally {
                pending.finish()
            }
        }
    }

    private fun report(notificationId: String?, kind: NotificationInteractionKind) {
        if (notificationId == null) return

        launchAsync { NotificationInteractions.record(notificationId, kind) }
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
        const val EXTRA_NOTIFICATION_ID = "notification_id"

        const val KIND_RUN_WORKFLOW = "run_workflow"
        const val KIND_SNOOZE = "snooze"
        const val KIND_DISMISS = "dismiss"
        const val KIND_ACKNOWLEDGE = "acknowledge"
        const val KIND_SWIPED = "swiped"

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
                .putExtra(EXTRA_NOTIFICATION_ID, payload.notificationId)

            when (action) {
                is PushAction.RunWorkflow -> intent
                    .putExtra(EXTRA_KIND, KIND_RUN_WORKFLOW)
                    .putExtra(EXTRA_SLUG, action.slug)

                is PushAction.Snooze -> intent
                    .putExtra(EXTRA_KIND, KIND_SNOOZE)
                    .putExtra(EXTRA_SECONDS, action.seconds)

                is PushAction.Dismiss -> intent.putExtra(EXTRA_KIND, KIND_DISMISS)

                is PushAction.Acknowledge -> intent.putExtra(EXTRA_KIND, KIND_ACKNOWLEDGE)
            }

            return PendingIntent.getBroadcast(
                context,
                id * 31 + index,
                intent,
                PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
            )
        }

        fun deletePendingIntent(context: Context, id: Int, payload: PushPayload): PendingIntent {
            val intent = Intent(context, NotificationActionReceiver::class.java)
                .putExtra(EXTRA_ID, id)
                .putExtra(EXTRA_TAG, payload.tag)
                .putExtra(EXTRA_NOTIFICATION_ID, payload.notificationId)
                .putExtra(EXTRA_KIND, KIND_SWIPED)

            return PendingIntent.getBroadcast(
                context,
                id * 31 + payload.actions.size,
                intent,
                PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
            )
        }
    }
}
