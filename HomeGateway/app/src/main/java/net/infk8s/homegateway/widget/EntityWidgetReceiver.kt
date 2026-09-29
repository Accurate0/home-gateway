package net.infk8s.homegateway.widget

import android.content.Context
import androidx.glance.appwidget.GlanceAppWidget
import androidx.glance.appwidget.GlanceAppWidgetReceiver

class EntityWidgetReceiver : GlanceAppWidgetReceiver() {
    override val glanceAppWidget: GlanceAppWidget = EntityWidget()

    override fun onEnabled(context: Context) {
        super.onEnabled(context)
        EntityWidgets.schedule(context)
    }

    override fun onDisabled(context: Context) {
        super.onDisabled(context)
        EntityWidgets.cancel(context)
    }
}
