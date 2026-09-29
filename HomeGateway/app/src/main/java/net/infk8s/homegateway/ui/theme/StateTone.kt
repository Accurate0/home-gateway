package net.infk8s.homegateway.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

enum class StateTone(
    val lightAccent: Color,
    val darkAccent: Color,
    val onAccent: Color,
) {
    LIGHT(Color(0xFFF2AF48), Color(0xFFF9B73F), Color(0xFF4D2800)),
    OPEN(Color(0xFFF0834E), Color(0xFFFF8E4D), Color(0xFF5A1C00)),
    PRESENT(Color(0xFF43C07A), Color(0xFF35CC86), Color(0xFF003A18)),
    INFO(Color(0xFF0284C7), Color(0xFF38BDF8), Color(0xFF082F49));

    val accent: Color
        @Composable get() = if (isSystemInDarkTheme()) darkAccent else lightAccent
}
