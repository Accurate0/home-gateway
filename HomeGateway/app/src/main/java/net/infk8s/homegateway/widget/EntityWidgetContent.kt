package net.infk8s.homegateway.widget

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.datastore.preferences.core.Preferences
import androidx.glance.ColorFilter
import androidx.glance.GlanceModifier
import androidx.glance.GlanceTheme
import androidx.glance.Image
import androidx.glance.ImageProvider
import androidx.glance.LocalSize
import androidx.glance.action.actionParametersOf
import androidx.glance.action.actionStartActivity
import androidx.glance.action.clickable
import androidx.glance.appwidget.action.actionRunCallback
import androidx.glance.appwidget.cornerRadius
import androidx.glance.background
import androidx.glance.color.ColorProvider
import androidx.glance.layout.Alignment
import androidx.glance.layout.Box
import androidx.glance.layout.Column
import androidx.glance.layout.Row
import androidx.glance.layout.Spacer
import androidx.glance.layout.fillMaxSize
import androidx.glance.layout.fillMaxWidth
import androidx.glance.layout.height
import androidx.glance.layout.padding
import androidx.glance.layout.size
import androidx.glance.layout.width
import androidx.glance.text.FontWeight
import androidx.glance.text.Text
import androidx.glance.text.TextStyle
import androidx.glance.unit.ColorProvider
import net.infk8s.homegateway.MainActivity

@Composable
fun EntityWidgetContent(preferences: Preferences) {
    val model = WidgetModel.read(preferences)
    val error = preferences[EntityWidgets.ERROR]

    val action = model?.command?.let { command ->
        actionRunCallback<EntityWidgetAction>(
            actionParametersOf(EntityWidgetAction.ID to model.id, EntityWidgetAction.COMMAND to command.name),
        )
    } ?: actionStartActivity<MainActivity>()

    Box(
        GlanceModifier
            .fillMaxSize()
            .background(containerColor(model))
            .cornerRadius(20.dp)
            .clickable(action)
            .padding(12.dp),
    ) {
        when {
            model == null -> Placeholder(
                if (preferences[EntityWidgets.ENTITY_KEY] == null) "Choose a device" else error ?: "Loading…",
            )

            else -> {
                val size = LocalSize.current
                when {
                    size.height >= EntityWidget.SQUARE.height && size.width >= EntityWidget.LARGE.width ->
                        LargeLayout(model, error)
                    size.height >= EntityWidget.SQUARE.height -> SquareLayout(model, error)
                    size.width >= EntityWidget.WIDE.width -> WideLayout(model, error)
                    size.width >= EntityWidget.SMALL.width -> SmallLayout(model, error)
                    else -> TinyLayout(model)
                }
            }
        }
    }
}

@Composable
private fun TinyLayout(model: WidgetModel) {
    Column(
        GlanceModifier.fillMaxSize(),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        IconBox(model, 28.dp)
        Spacer(GlanceModifier.height(4.dp))
        Label(model.headline ?: model.status, 11.sp, FontWeight.Medium)
    }
}

@Composable
private fun SmallLayout(model: WidgetModel, error: String?) {
    Row(GlanceModifier.fillMaxSize(), verticalAlignment = Alignment.CenterVertically) {
        IconBox(model, 36.dp)
        Spacer(GlanceModifier.width(10.dp))
        Column(GlanceModifier.defaultWeight()) {
            Label(model.name, 13.sp, FontWeight.Medium)
            Secondary(error ?: model.headline ?: model.status, error != null)
        }
    }
}

@Composable
private fun WideLayout(model: WidgetModel, error: String?) {
    Row(GlanceModifier.fillMaxSize(), verticalAlignment = Alignment.CenterVertically) {
        IconBox(model, 36.dp)
        Spacer(GlanceModifier.width(12.dp))
        Column(GlanceModifier.defaultWeight()) {
            Label(model.name, 14.sp, FontWeight.Medium)
            Secondary(error ?: model.subtitle, error != null)
        }
        Spacer(GlanceModifier.width(8.dp))
        Column(horizontalAlignment = Alignment.End) {
            Label(model.headline ?: model.status, if (model.headline != null) 20.sp else 13.sp, FontWeight.Bold)
            model.battery?.let { Secondary(it) }
        }
    }
}

@Composable
private fun SquareLayout(model: WidgetModel, error: String?) {
    Column(GlanceModifier.fillMaxSize()) {
        Row(GlanceModifier.fillMaxWidth()) {
            IconBox(model, 40.dp)
            Spacer(GlanceModifier.defaultWeight())
            model.battery?.let { Secondary(it) }
        }
        Spacer(GlanceModifier.defaultWeight())
        Label(model.name, 14.sp, FontWeight.Medium)
        Secondary(error ?: model.subtitle, error != null)
        Spacer(GlanceModifier.height(4.dp))
        if (model.headline != null) {
            Label(model.headline, 24.sp, FontWeight.Bold)
        } else {
            Label(model.status.uppercase(), 11.sp, FontWeight.Bold)
        }
    }
}

@Composable
private fun LargeLayout(model: WidgetModel, error: String?) {
    Column(GlanceModifier.fillMaxSize()) {
        Row(GlanceModifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            IconBox(model, 44.dp)
            Spacer(GlanceModifier.width(12.dp))
            Column(GlanceModifier.defaultWeight()) {
                Label(model.name, 16.sp, FontWeight.Medium)
                Secondary(error ?: model.subtitle, error != null)
            }
            model.battery?.let { Secondary(it) }
        }
        Spacer(GlanceModifier.defaultWeight())
        Row(GlanceModifier.fillMaxWidth(), verticalAlignment = Alignment.Bottom) {
            Box(GlanceModifier.defaultWeight()) {
                Label(model.headline ?: model.status, if (model.headline != null) 30.sp else 18.sp, FontWeight.Bold)
            }
            model.command?.let { Secondary("Tap: ${it.label.lowercase()}") }
        }
    }
}

@Composable
private fun Placeholder(text: String) {
    Box(GlanceModifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        Secondary(text)
    }
}

@Composable
private fun IconBox(model: WidgetModel, size: Dp) {
    val tone = model.tone

    Box(
        GlanceModifier
            .size(size)
            .background(tone?.let { ColorProvider(day = it.lightAccent, night = it.darkAccent) } ?: GlanceTheme.colors.secondaryContainer)
            .cornerRadius(size / 3),
        contentAlignment = Alignment.Center,
    ) {
        Image(
            provider = ImageProvider(model.glyph.drawable),
            contentDescription = null,
            modifier = GlanceModifier.size(size / 2),
            colorFilter = ColorFilter.tint(
                tone?.let { ColorProvider(it.onAccent) } ?: GlanceTheme.colors.onSecondaryContainer,
            ),
        )
    }
}

@Composable
private fun Label(text: String, size: TextUnit, weight: FontWeight) {
    Text(
        text,
        maxLines = 1,
        style = TextStyle(color = GlanceTheme.colors.onSurface, fontSize = size, fontWeight = weight),
    )
}

@Composable
private fun Secondary(text: String, error: Boolean = false) {
    Text(
        text,
        maxLines = 1,
        style = TextStyle(
            color = if (error) GlanceTheme.colors.error else GlanceTheme.colors.onSurfaceVariant,
            fontSize = 12.sp,
        ),
    )
}

@Composable
private fun containerColor(model: WidgetModel?) = model?.tone?.let { tone ->
    ColorProvider(
        day = tone.lightAccent.copy(alpha = 0.22f).compositeOver(LIGHT_SURFACE),
        night = tone.darkAccent.copy(alpha = 0.22f).compositeOver(DARK_SURFACE),
    )
} ?: GlanceTheme.colors.widgetBackground

private val LIGHT_SURFACE = Color(0xFFFFFBFE)
private val DARK_SURFACE = Color(0xFF1C1B1F)
