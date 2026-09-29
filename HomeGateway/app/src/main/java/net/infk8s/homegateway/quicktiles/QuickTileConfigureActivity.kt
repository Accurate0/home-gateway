package net.infk8s.homegateway.quicktiles

import android.content.ComponentName
import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import net.infk8s.homegateway.gateway
import net.infk8s.homegateway.picker.EntityPickerScreen
import net.infk8s.homegateway.ui.theme.HomeGatewayTheme

class QuickTileConfigureActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val slot = intent.getIntExtra(QuickTiles.EXTRA_SLOT, -1).takeIf { it >= 0 }
            ?: intent.getParcelableExtra(Intent.EXTRA_COMPONENT_NAME, ComponentName::class.java)
                ?.let(QuickTiles::slotOf)
        if (slot == null) {
            finish()
            return
        }

        enableEdgeToEdge()
        setContent {
            HomeGatewayTheme {
                EntityPickerScreen(
                    title = "Quick settings tile ${slot + 1}",
                    onPick = { entity ->
                        gateway.quickTileAssignments.set(slot, entity)
                        QuickTiles.refresh(this, slot)
                        finish()
                    },
                    onCancel = ::finish,
                )
            }
        }
    }
}
