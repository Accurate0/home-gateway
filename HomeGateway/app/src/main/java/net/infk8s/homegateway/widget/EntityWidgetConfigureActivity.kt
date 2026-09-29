package net.infk8s.homegateway.widget

import android.appwidget.AppWidgetManager
import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.glance.appwidget.GlanceAppWidgetManager
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.launch
import net.infk8s.homegateway.graphql.EntityUi
import net.infk8s.homegateway.picker.EntityPickerScreen
import net.infk8s.homegateway.ui.theme.HomeGatewayTheme

class EntityWidgetConfigureActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val appWidgetId = intent.extras
            ?.getInt(AppWidgetManager.EXTRA_APPWIDGET_ID, AppWidgetManager.INVALID_APPWIDGET_ID)
            ?: AppWidgetManager.INVALID_APPWIDGET_ID
        if (appWidgetId == AppWidgetManager.INVALID_APPWIDGET_ID) {
            finish()
            return
        }

        setResult(RESULT_CANCELED, resultIntent(appWidgetId))
        enableEdgeToEdge()
        setContent {
            HomeGatewayTheme {
                EntityPickerScreen(
                    title = "Choose a device",
                    onPick = { assign(appWidgetId, it) },
                    onCancel = ::finish,
                )
            }
        }
    }

    private fun assign(appWidgetId: Int, entity: EntityUi) {
        lifecycleScope.launch {
            val glanceId = GlanceAppWidgetManager(this@EntityWidgetConfigureActivity).getGlanceIdBy(appWidgetId)
            EntityWidgets.assign(this@EntityWidgetConfigureActivity, glanceId, entity)
            EntityWidgets.schedule(this@EntityWidgetConfigureActivity)

            setResult(RESULT_OK, resultIntent(appWidgetId))
            finish()
        }
    }

    private fun resultIntent(appWidgetId: Int) =
        Intent().putExtra(AppWidgetManager.EXTRA_APPWIDGET_ID, appWidgetId)
}
