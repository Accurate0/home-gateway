package net.infk8s.homegateway.notifications

import android.util.Log
import net.infk8s.homegateway.graphql.ApolloProvider
import net.infk8s.homegateway.graphql.RecordNotificationInteractionMutation
import net.infk8s.homegateway.graphql.type.NotificationInteractionKind

object NotificationInteractions {
    private const val TAG = "NotificationInteractions"

    suspend fun record(notificationId: String, kind: NotificationInteractionKind) {
        try {
            val response = ApolloProvider.client
                .mutation(RecordNotificationInteractionMutation(notificationId, kind))
                .execute()

            if (response.hasErrors()) {
                Log.w(TAG, "recordNotificationInteraction($notificationId, $kind) returned errors: ${response.errors}")
            }
        } catch (e: Exception) {
            Log.e(TAG, "recordNotificationInteraction($notificationId, $kind) failed", e)
        }
    }
}
