package net.infk8s.homegateway.notifications

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import androidx.core.content.getSystemService
import net.infk8s.homegateway.MainActivity
import net.infk8s.homegateway.R

object Notifications {
    private const val SUMMARY_ID_OFFSET = 1

    fun createChannels(context: Context) {
        val manager = context.getSystemService<NotificationManager>() ?: return

        NotificationCategory.entries.forEach { category ->
            manager.createNotificationChannel(
                NotificationChannel(category.id, category.displayName, category.importance)
            )
        }
    }

    fun notificationId(tag: String): Int = tag.hashCode()

    fun show(context: Context, payload: PushPayload) {
        val manager = NotificationManagerCompat.from(context)
        if (!manager.areNotificationsEnabled()) return

        val id = notificationId(payload.tag)

        val activityIntent = Intent(context, MainActivity::class.java)
            .addFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_SINGLE_TOP)

        payload.notificationId?.let {
            activityIntent.putExtra(PushPayload.KEY_NOTIFICATION_ID, it)
        }

        val contentIntent = PendingIntent.getActivity(
            context,
            id,
            activityIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )

        val builder = NotificationCompat.Builder(context, payload.category.id)
            .setSmallIcon(R.drawable.ic_stat_notification)
            .setContentTitle(payload.title)
            .setContentText(payload.body)
            .setStyle(NotificationCompat.BigTextStyle().bigText(payload.body))
            .setPriority(NotificationCompat.PRIORITY_HIGH)
            .setGroup(payload.category.groupKey)
            .setContentIntent(contentIntent)
            .setAutoCancel(true)

        if (payload.notificationId != null) {
            builder.setDeleteIntent(NotificationActionReceiver.deletePendingIntent(context, id, payload))
        }

        payload.actions.forEachIndexed { index, action ->
            builder.addAction(
                NotificationCompat.Action.Builder(
                    0,
                    action.label,
                    NotificationActionReceiver.pendingIntent(context, id, payload, index, action),
                ).build()
            )
        }

        try {
            manager.notify(id, builder.build())
            manager.notify(summaryId(payload.category), summary(context, payload.category))
        } catch (e: SecurityException) {
            android.util.Log.w("Notifications", "notification permission missing", e)
        }
    }

    private fun summaryId(category: NotificationCategory): Int =
        category.groupKey.hashCode() + SUMMARY_ID_OFFSET

    private fun summary(context: Context, category: NotificationCategory) =
        NotificationCompat.Builder(context, category.id)
            .setSmallIcon(R.drawable.ic_stat_notification)
            .setContentTitle(category.displayName)
            .setGroup(category.groupKey)
            .setGroupSummary(true)
            .setAutoCancel(true)
            .build()
}
