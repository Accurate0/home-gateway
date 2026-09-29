package net.infk8s.homegateway.system.history

import androidx.compose.foundation.Canvas
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.unit.dp
import java.time.Instant

@Composable
fun HistoryChart(
    points: List<HistoryPoint>,
    since: Instant,
    valueRange: ClosedFloatingPointRange<Double>,
    guides: List<Double>,
    lineColour: Color,
    guideColour: Color,
    modifier: Modifier = Modifier,
) {
    Canvas(modifier) {
        val start = since.toEpochMilli().toFloat()
        val span = (Instant.now().toEpochMilli() - since.toEpochMilli()).coerceAtLeast(1).toFloat()
        val low = valueRange.start
        val height = (valueRange.endInclusive - low).coerceAtLeast(Double.MIN_VALUE)

        fun x(time: Instant) = ((time.toEpochMilli() - start) / span).coerceIn(0f, 1f) * size.width
        fun y(value: Double) = size.height - ((value - low) / height).toFloat().coerceIn(0f, 1f) * size.height

        val dash = PathEffect.dashPathEffect(floatArrayOf(4.dp.toPx(), 4.dp.toPx()))
        guides.forEach { guide ->
            drawLine(
                guideColour,
                Offset(0f, y(guide)),
                Offset(size.width, y(guide)),
                strokeWidth = 1.dp.toPx(),
                pathEffect = dash,
            )
        }

        if (points.size == 1) {
            drawCircle(lineColour, radius = 3.dp.toPx(), center = Offset(x(points[0].time), y(points[0].value)))
            return@Canvas
        }

        val path = Path()
        points.forEachIndexed { index, point ->
            if (index == 0) {
                path.moveTo(x(point.time), y(point.value))
            } else {
                path.lineTo(x(point.time), y(point.value))
            }
        }

        drawPath(
            path,
            lineColour,
            style = Stroke(width = 2.dp.toPx(), cap = StrokeCap.Round, join = StrokeJoin.Round),
        )
    }
}
