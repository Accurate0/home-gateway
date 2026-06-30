package net.infk8s.homegateway

import android.Manifest
import android.content.pm.PackageManager
import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.viewModels
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.ListItem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.core.content.ContextCompat
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.google.firebase.messaging.FirebaseMessaging
import kotlin.concurrent.thread
import net.infk8s.homegateway.graphql.EntitiesUiState
import net.infk8s.homegateway.graphql.EntitiesViewModel
import net.infk8s.homegateway.graphql.EntityUi
import net.infk8s.homegateway.ui.theme.HomeGatewayTheme

class MainActivity : ComponentActivity() {
    private val requestNotificationPermission =
        registerForActivityResult(ActivityResultContracts.RequestPermission()) { /* no-op */ }

    private val entitiesViewModel: EntitiesViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        ensureNotificationPermission()
        registerPushToken()
        enableEdgeToEdge()
        setContent {
            HomeGatewayTheme {
                Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
                    val state by entitiesViewModel.state.collectAsStateWithLifecycle()
                    EntitiesScreen(state, Modifier.padding(innerPadding))
                }
            }
        }
    }

    private fun ensureNotificationPermission() {
        val granted = ContextCompat.checkSelfPermission(
            this,
            Manifest.permission.POST_NOTIFICATIONS,
        ) == PackageManager.PERMISSION_GRANTED
        if (!granted) {
            requestNotificationPermission.launch(Manifest.permission.POST_NOTIFICATIONS)
        }
    }

    private fun registerPushToken() {
        FirebaseMessaging.getInstance().token.addOnCompleteListener { task ->
            if (!task.isSuccessful) {
                Log.e("MainActivity", "failed to fetch fcm token", task.exception)
                return@addOnCompleteListener
            }
            val token = task.result
            // Listener runs on the main thread; register off it to avoid network-on-main.
            thread { PushTokenRegistrar.register(token) }
        }
    }
}

@Composable
fun EntitiesScreen(state: EntitiesUiState, modifier: Modifier = Modifier) {
    when (state) {
        is EntitiesUiState.Loading -> Box(modifier.fillMaxSize(), Alignment.Center) {
            CircularProgressIndicator()
        }

        is EntitiesUiState.Error -> Box(modifier.fillMaxSize(), Alignment.Center) {
            Text(state.message, color = MaterialTheme.colorScheme.error)
        }

        is EntitiesUiState.Loaded -> LazyColumn(modifier.fillMaxSize()) {
            items(state.items, key = { it.id }) { entity ->
                EntityRow(entity)
                HorizontalDivider()
            }
        }
    }
}

@Composable
fun EntityRow(entity: EntityUi) {
    ListItem(
        headlineContent = { Text(entity.name) },
        supportingContent = { Text(entity.details()) },
        trailingContent = {
            Text(
                entity.typeLabel(),
                style = MaterialTheme.typography.labelMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        },
    )
}

private fun EntityUi.typeLabel(): String = when (this) {
    is EntityUi.Light -> "Light"
    is EntityUi.Door -> "Door"
    is EntityUi.Presence -> "Presence"
    is EntityUi.Environment -> "Environment"
}

private fun EntityUi.details(): String = when (this) {
    is EntityUi.Light -> on?.let { if (it) "On" else "Off" } ?: "Unknown"
    is EntityUi.Door -> open?.let { if (it) "Open" else "Closed" } ?: "Unknown"
    is EntityUi.Presence -> present?.let { if (it) "Present" else "Away" } ?: "Unknown"
    is EntityUi.Environment -> buildList {
        temperature?.let { add("%.1f°C".format(it)) }
        humidity?.let { add("%.0f%%".format(it)) }
        pressure?.let { add("%.0f hPa".format(it)) }
        lux?.let { add("%.0f lx".format(it)) }
        uvIndex?.let { add("UV %.1f".format(it)) }
    }.joinToString(" · ").ifEmpty { "Unknown" }
}
