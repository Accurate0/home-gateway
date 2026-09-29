package net.infk8s.homegateway

import android.Manifest
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.viewModels
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.material3.Icon
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.core.content.ContextCompat
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import com.google.firebase.messaging.FirebaseMessaging
import kotlin.concurrent.thread
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.launch
import net.infk8s.homegateway.auth.SignInScreen
import net.infk8s.homegateway.dashboard.DashboardScreen
import net.infk8s.homegateway.dashboard.DashboardTopBar
import net.infk8s.homegateway.dashboard.SectionOrderSheet
import net.infk8s.homegateway.graphql.EntitiesUiState
import net.infk8s.homegateway.graphql.EntitiesViewModel
import net.infk8s.homegateway.graphql.type.NotificationInteractionKind
import net.infk8s.homegateway.notifications.NotificationInteractions
import net.infk8s.homegateway.notifications.NotificationsScreen
import net.infk8s.homegateway.notifications.NotificationsViewModel
import net.infk8s.homegateway.notifications.PushPayload
import net.infk8s.homegateway.ui.theme.HomeGatewayTheme

class MainActivity : ComponentActivity() {
    private val requestNotificationPermission =
        registerForActivityResult(ActivityResultContracts.RequestPermission()) { /* no-op */ }

    private val signIn =
        registerForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
            completeSignIn(result.data)
        }

    private val entitiesViewModel: EntitiesViewModel by viewModels()

    private val notificationsViewModel: NotificationsViewModel by viewModels()

    private var selectedTab by mutableStateOf(AppTab.HOME)

    private var signInBusy by mutableStateOf(false)

    private var signInError by mutableStateOf<String?>(null)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        if (savedInstanceState == null) {
            reportNotificationOpened(intent)
        }
        ensureNotificationPermission()
        registerPushToken()
        enableEdgeToEdge()
        setContent {
            HomeGatewayTheme {
                val signedIn by gateway.auth.signedIn.collectAsStateWithLifecycle()

                if (signedIn) {
                    SignedInContent()
                } else {
                    SignInScreen(
                        busy = signInBusy,
                        error = signInError,
                        onSignIn = ::startSignIn,
                        modifier = Modifier.safeDrawingPadding(),
                    )
                }
            }
        }
    }

    @Composable
    private fun SignedInContent() {
        var editing by rememberSaveable { mutableStateOf(false) }
        var reorderingSections by rememberSaveable { mutableStateOf(false) }

        val entities by entitiesViewModel.state.collectAsStateWithLifecycle()

        Scaffold(
            modifier = Modifier.fillMaxSize(),
            topBar = {
                if (selectedTab == AppTab.HOME) {
                    DashboardTopBar(
                        editing = editing,
                        onToggleEditing = { editing = !editing },
                        onReorderSections = { reorderingSections = true },
                        onSignOut = { lifecycleScope.launch { gateway.auth.signOut() } },
                    )
                }
            },
            bottomBar = {
                NavigationBar {
                    AppTab.entries.forEach { tab ->
                        NavigationBarItem(
                            selected = selectedTab == tab,
                            onClick = { selectedTab = tab },
                            icon = { Icon(painterResource(tab.icon), contentDescription = null) },
                            label = { Text(tab.label) },
                        )
                    }
                }
            },
        ) { innerPadding ->
            when (selectedTab) {
                AppTab.HOME -> {
                    // Run the live WebSocket only while the UI is at least STARTED.
                    // repeatOnLifecycle cancels run() when the app is backgrounded or the
                    // phone locks, and restarts it on return — which re-fetches a fresh
                    // snapshot so the list is correct after time away.
                    val lifecycleOwner = LocalLifecycleOwner.current
                    LaunchedEffect(lifecycleOwner) {
                        lifecycleOwner.repeatOnLifecycle(Lifecycle.State.STARTED) {
                            entitiesViewModel.run()
                        }
                    }

                    DashboardScreen(
                        state = entities,
                        controls = entitiesViewModel.controls(),
                        lightLevels = entitiesViewModel.lightLevels,
                        editing = editing,
                        onMoveTile = entitiesViewModel::moveTile,
                        modifier = Modifier.padding(innerPadding),
                    )

                    val loaded = entities as? EntitiesUiState.Loaded
                    if (reorderingSections && loaded != null) {
                        SectionOrderSheet(
                            sections = loaded.sections,
                            onMove = entitiesViewModel::moveSection,
                            onDismiss = { reorderingSections = false },
                        )
                    }
                }

                AppTab.NOTIFICATIONS -> {
                    val state by notificationsViewModel.state.collectAsStateWithLifecycle()
                    val lifecycleOwner = LocalLifecycleOwner.current
                    LaunchedEffect(lifecycleOwner) {
                        lifecycleOwner.repeatOnLifecycle(Lifecycle.State.STARTED) {
                            notificationsViewModel.refresh()
                        }
                    }
                    NotificationsScreen(
                        state,
                        onRefresh = notificationsViewModel::refresh,
                        onAcknowledge = notificationsViewModel::acknowledge,
                        modifier = Modifier.padding(innerPadding),
                    )
                }
            }
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        reportNotificationOpened(intent)
    }

    private fun startSignIn() {
        signInBusy = true
        signInError = null

        lifecycleScope.launch {
            try {
                signIn.launch(gateway.auth.authorizationIntent())
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                Log.w("MainActivity", "failed to start sign-in", e)
                signInError = e.message ?: "Sign-in failed"
                signInBusy = false
            }
        }
    }

    private fun completeSignIn(data: Intent?) {
        if (data == null) {
            signInBusy = false
            signInError = "Sign-in was cancelled"
            return
        }

        lifecycleScope.launch {
            try {
                gateway.auth.completeAuthorization(data)
                signInError = null
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                Log.w("MainActivity", "sign-in failed", e)
                signInError = e.message ?: "Sign-in failed"
            } finally {
                signInBusy = false
            }
        }
    }

    private fun reportNotificationOpened(intent: Intent?) {
        val notificationId = intent?.getStringExtra(PushPayload.KEY_NOTIFICATION_ID) ?: return
        intent.removeExtra(PushPayload.KEY_NOTIFICATION_ID)
        selectedTab = AppTab.NOTIFICATIONS

        lifecycleScope.launch {
            NotificationInteractions.record(gateway.apollo, notificationId, NotificationInteractionKind.OPENED)
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
