package net.infk8s.homegateway.widget

import android.content.Context
import androidx.compose.ui.unit.DpSize
import androidx.compose.ui.unit.dp
import androidx.datastore.preferences.core.Preferences
import androidx.glance.GlanceId
import androidx.glance.GlanceTheme
import androidx.glance.appwidget.GlanceAppWidget
import androidx.glance.appwidget.SizeMode
import androidx.glance.appwidget.provideContent
import androidx.glance.currentState

class EntityWidget : GlanceAppWidget() {
    override val sizeMode = SizeMode.Responsive(setOf(TINY, SMALL, SQUARE, WIDE, LARGE))

    override suspend fun provideGlance(context: Context, id: GlanceId) {
        provideContent {
            GlanceTheme {
                EntityWidgetContent(currentState<Preferences>())
            }
        }
    }

    companion object {
        val TINY = DpSize(57.dp, 57.dp)
        val SMALL = DpSize(130.dp, 57.dp)
        val SQUARE = DpSize(130.dp, 130.dp)
        val WIDE = DpSize(270.dp, 57.dp)
        val LARGE = DpSize(270.dp, 130.dp)
    }
}
