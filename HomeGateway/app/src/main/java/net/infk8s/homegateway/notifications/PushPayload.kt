package net.infk8s.homegateway.notifications

import org.json.JSONArray

data class PushPayload(
    val title: String,
    val body: String,
    val category: NotificationCategory,
    val tag: String,
    val actions: List<PushAction>,
    val rawActions: String,
    val notificationId: String?,
) {
    companion object {
        const val KEY_NOTIFICATION_ID = "notification_id"

        fun from(data: Map<String, String>): PushPayload? {
            val body = data["body"] ?: return null

            return PushPayload(
                title = data["title"] ?: "Home Gateway",
                body = body,
                category = NotificationCategory.from(data["category"]),
                tag = data["tag"] ?: body,
                actions = parseActions(data["actions"]),
                rawActions = data["actions"].orEmpty(),
                notificationId = data[KEY_NOTIFICATION_ID]?.ifEmpty { null },
            )
        }

        private fun parseActions(raw: String?): List<PushAction> {
            if (raw.isNullOrBlank()) return emptyList()

            val array = runCatching { JSONArray(raw) }.getOrNull() ?: return emptyList()

            return (0 until array.length()).mapNotNull { index ->
                val item = array.optJSONObject(index) ?: return@mapNotNull null
                val label = item.optString("label").ifEmpty { null } ?: return@mapNotNull null

                when (item.optString("type")) {
                    "run_workflow" ->
                        item.optString("slug").ifEmpty { null }?.let { PushAction.RunWorkflow(label, it) }

                    "snooze" -> PushAction.Snooze(label, item.optLong("seconds", 0L))
                    "dismiss" -> PushAction.Dismiss(label)
                    "acknowledge" -> PushAction.Acknowledge(label)
                    else -> null
                }
            }
        }
    }
}

sealed interface PushAction {
    val label: String

    data class RunWorkflow(override val label: String, val slug: String) : PushAction

    data class Snooze(override val label: String, val seconds: Long) : PushAction

    data class Dismiss(override val label: String) : PushAction

    data class Acknowledge(override val label: String) : PushAction
}
