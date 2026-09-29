package net.infk8s.homegateway

import android.Manifest
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Bundle
import android.util.Log
import androidx.activity.ComponentActivity
import androidx.activity.compose.BackHandler
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.viewModels
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.saveable.rememberSaveableStateHolder
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
import net.infk8s.homegateway.dashboard.DashboardLayout
import net.infk8s.homegateway.dashboard.DashboardMode
import net.infk8s.homegateway.dashboard.DashboardScreen
import net.infk8s.homegateway.dashboard.DashboardTopBar
import net.infk8s.homegateway.dashboard.SectionNameDialog
import net.infk8s.homegateway.dashboard.SectionOrderSheet
import net.infk8s.homegateway.graphql.EntitiesUiState
import net.infk8s.homegateway.graphql.EntitiesViewModel
import net.infk8s.homegateway.graphql.type.NotificationInteractionKind
import net.infk8s.homegateway.modes.ModesScreen
import net.infk8s.homegateway.modes.ModesViewModel
import net.infk8s.homegateway.notifications.NotificationInteractions
import net.infk8s.homegateway.notifications.NotificationsScreen
import net.infk8s.homegateway.notifications.NotificationsViewModel
import net.infk8s.homegateway.notifications.PushPayload
import net.infk8s.homegateway.fuel.FuelScreen
import net.infk8s.homegateway.fuel.FuelViewModel
import net.infk8s.homegateway.jellyfin.JellyfinScreen
import net.infk8s.homegateway.jellyfin.JellyfinViewModel
import net.infk8s.homegateway.more.MorePage
import net.infk8s.homegateway.more.MoreScreen
import net.infk8s.homegateway.transperth.TransperthScreen
import net.infk8s.homegateway.transperth.TransperthViewModel
import net.infk8s.homegateway.woolworths.WoolworthsScreen
import net.infk8s.homegateway.woolworths.WoolworthsViewModel
import net.infk8s.homegateway.system.adhoc.AdhocTasksScreen
import net.infk8s.homegateway.system.adhoc.AdhocTasksViewModel
import net.infk8s.homegateway.system.batteries.BatteriesViewModel
import net.infk8s.homegateway.system.history.HistoryScreen
import net.infk8s.homegateway.system.history.HistoryViewModel
import net.infk8s.homegateway.system.homeassistant.HomeAssistantScreen
import net.infk8s.homegateway.system.homeassistant.HomeAssistantViewModel
import net.infk8s.homegateway.system.plants.PlantsViewModel
import net.infk8s.homegateway.ui.TitleTopBar
import net.infk8s.homegateway.ui.theme.HomeGatewayTheme
import net.infk8s.homegateway.workflows.RunsScreen
import net.infk8s.homegateway.workflows.RunsViewModel
import net.infk8s.homegateway.workflows.WorkflowsScreen
import net.infk8s.homegateway.workflows.WorkflowsViewModel

class MainActivity : ComponentActivity() {
    private val requestNotificationPermission =
        registerForActivityResult(ActivityResultContracts.RequestPermission()) { /* no-op */ }

    private val signIn =
        registerForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
            completeSignIn(result.data)
        }

    private val entitiesViewModel: EntitiesViewModel by viewModels()

    private val notificationsViewModel: NotificationsViewModel by viewModels()

    private val workflowsViewModel: WorkflowsViewModel by viewModels()

    private val runsViewModel: RunsViewModel by viewModels()

    private val modesViewModel: ModesViewModel by viewModels()

    private val batteriesViewModel: BatteriesViewModel by viewModels()

    private val plantsViewModel: PlantsViewModel by viewModels()

    private val adhocTasksViewModel: AdhocTasksViewModel by viewModels()

    private val homeAssistantViewModel: HomeAssistantViewModel by viewModels()

    private val transperthViewModel: TransperthViewModel by viewModels()

    private val fuelViewModel: FuelViewModel by viewModels()

    private val woolworthsViewModel: WoolworthsViewModel by viewModels()

    private val jellyfinViewModel: JellyfinViewModel by viewModels()

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
                    )
                }
            }
        }
    }

    @Composable
    private fun SignedInContent() {
        var editing by rememberSaveable { mutableStateOf(false) }
        var reorderingSections by rememberSaveable { mutableStateOf(false) }
        var addingSection by rememberSaveable { mutableStateOf(false) }
        var runsOpen by rememberSaveable { mutableStateOf(false) }
        var runsFilter by rememberSaveable { mutableStateOf<String?>(null) }
        var pendingRun by rememberSaveable { mutableStateOf<String?>(null) }
        var morePage by rememberSaveable { mutableStateOf<MorePage?>(null) }

        val entities by entitiesViewModel.state.collectAsStateWithLifecycle()
        val loaded = entities as? EntitiesUiState.Loaded

        fun openRuns(filter: String?, pending: String?) {
            runsFilter = filter
            pendingRun = pending
            runsOpen = true
        }

        BackHandler(enabled = selectedTab == AppTab.WORKFLOWS && runsOpen) { runsOpen = false }
        BackHandler(enabled = selectedTab == AppTab.MORE && morePage != null) { morePage = null }

        val snackbarHostState = remember { SnackbarHostState() }
        LaunchedEffect(Unit) {
            entitiesViewModel.commandFailures.collect { snackbarHostState.showSnackbar(it) }
        }

        val tabStates = rememberSaveableStateHolder()
        val tabKey = when {
            selectedTab == AppTab.WORKFLOWS && runsOpen -> "runs"
            selectedTab == AppTab.MORE && morePage != null -> "more:${morePage?.name}"
            else -> selectedTab.name
        }

        Scaffold(
            modifier = Modifier.fillMaxSize(),
            snackbarHost = { SnackbarHost(snackbarHostState) },
            topBar = {
                when (selectedTab) {
                    AppTab.HOME -> DashboardTopBar(
                        mode = loaded?.mode,
                        editing = editing,
                        onToggleEditing = { editing = !editing },
                        onModeChange = entitiesViewModel::setMode,
                        onAddSection = { addingSection = true },
                        onReorderSections = { reorderingSections = true },
                        onSignOut = { lifecycleScope.launch { gateway.auth.signOut() } },
                    )

                    AppTab.WORKFLOWS -> if (runsOpen) {
                        TitleTopBar("Runs", onBack = { runsOpen = false })
                    } else {
                        TitleTopBar("Workflows") {
                            IconButton(onClick = { openRuns(null, null) }) {
                                Icon(painterResource(R.drawable.ic_history), contentDescription = "Runs")
                            }
                        }
                    }

                    AppTab.MODES -> TitleTopBar("Modes")

                    AppTab.NOTIFICATIONS -> Unit

                    AppTab.MORE -> when (val page = morePage) {
                        null -> TitleTopBar("More")
                        else -> TitleTopBar(page.title, onBack = { morePage = null })
                    }
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
            tabStates.SaveableStateProvider(tabKey) {
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
                            onRenameSection = entitiesViewModel::renameSection,
                            onDeleteSection = entitiesViewModel::deleteSection,
                            modifier = Modifier.padding(innerPadding),
                        )

                        if (reorderingSections && loaded != null) {
                            SectionOrderSheet(
                                sections = loaded.sections.filterNot {
                                    loaded.mode == DashboardMode.CUSTOM && it.key == DashboardLayout.UNSORTED_KEY
                                },
                                onMove = entitiesViewModel::moveSection,
                                onDismiss = { reorderingSections = false },
                            )
                        }

                        if (addingSection) {
                            SectionNameDialog(
                                title = "Add section",
                                initial = "",
                                confirmLabel = "Add",
                                onConfirm = { name ->
                                    entitiesViewModel.addSection(name)
                                    addingSection = false
                                },
                                onDismiss = { addingSection = false },
                            )
                        }
                    }

                    AppTab.WORKFLOWS -> if (runsOpen) {
                        val state by runsViewModel.state.collectAsStateWithLifecycle()
                        val traces by runsViewModel.traces.collectAsStateWithLifecycle()
                        val lifecycleOwner = LocalLifecycleOwner.current
                        LaunchedEffect(lifecycleOwner, pendingRun) {
                            lifecycleOwner.repeatOnLifecycle(Lifecycle.State.STARTED) {
                                runsViewModel.load(pendingRun)
                            }
                        }

                        RunsScreen(
                            state = state,
                            traces = traces,
                            filter = runsFilter,
                            pendingEventId = pendingRun,
                            onFilter = { runsFilter = it },
                            onExpand = runsViewModel::loadTrace,
                            modifier = Modifier.padding(innerPadding),
                        )
                    } else {
                        val state by workflowsViewModel.state.collectAsStateWithLifecycle()
                        val dryRunning by workflowsViewModel.dryRunning.collectAsStateWithLifecycle()
                        val dryRunError by workflowsViewModel.dryRunError.collectAsStateWithLifecycle()
                        val lifecycleOwner = LocalLifecycleOwner.current
                        LaunchedEffect(lifecycleOwner) {
                            lifecycleOwner.repeatOnLifecycle(Lifecycle.State.STARTED) {
                                workflowsViewModel.refresh()
                            }
                        }

                        WorkflowsScreen(
                            state = state,
                            dryRunning = dryRunning,
                            dryRunError = dryRunError,
                            onToggle = { workflow, enabled -> workflowsViewModel.setEnabled(workflow.slug, enabled) },
                            onDryRun = { workflow ->
                                workflowsViewModel.dryRun(workflow) { eventId -> openRuns(null, eventId) }
                            },
                            onHistory = { workflow -> openRuns(workflow.slug, null) },
                            modifier = Modifier.padding(innerPadding),
                        )
                    }

                    AppTab.MODES -> {
                        val state by modesViewModel.state.collectAsStateWithLifecycle()
                        val lifecycleOwner = LocalLifecycleOwner.current
                        LaunchedEffect(lifecycleOwner) {
                            lifecycleOwner.repeatOnLifecycle(Lifecycle.State.STARTED) {
                                modesViewModel.refresh()
                            }
                        }

                        ModesScreen(
                            state = state,
                            onSelect = modesViewModel::select,
                            modifier = Modifier.padding(innerPadding),
                        )
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

                    AppTab.MORE -> MoreContent(
                        page = morePage,
                        onOpen = { morePage = it },
                        modifier = Modifier.padding(innerPadding),
                    )
                }
            }
        }
    }

    @Composable
    private fun MoreContent(page: MorePage?, onOpen: (MorePage) -> Unit, modifier: Modifier) {
        val lifecycleOwner = LocalLifecycleOwner.current

        when (page) {
            null -> MoreScreen(onOpen = onOpen, modifier = modifier)

            MorePage.TRANSPERTH -> {
                val state by transperthViewModel.state.collectAsStateWithLifecycle()
                RunWhileStarted(transperthViewModel::run)

                TransperthScreen(state, modifier)
            }

            MorePage.FUEL -> {
                val state by fuelViewModel.state.collectAsStateWithLifecycle()
                RunWhileStarted(fuelViewModel::run)

                FuelScreen(state, modifier)
            }

            MorePage.WOOLWORTHS -> {
                val state by woolworthsViewModel.state.collectAsStateWithLifecycle()
                val histories by woolworthsViewModel.histories.collectAsStateWithLifecycle()
                RunWhileStarted(woolworthsViewModel::run)

                WoolworthsScreen(
                    state = state,
                    histories = histories,
                    range = woolworthsViewModel.range,
                    onExpand = woolworthsViewModel::loadHistory,
                    modifier = modifier,
                )
            }

            MorePage.JELLYFIN -> {
                val state by jellyfinViewModel.state.collectAsStateWithLifecycle()
                RunWhileStarted(jellyfinViewModel::run)

                JellyfinScreen(state, modifier)
            }

            MorePage.BATTERIES -> HistoryContent(batteriesViewModel, BATTERY_HEALTHY, modifier)

            MorePage.PLANTS -> HistoryContent(plantsViewModel, PLANT_HEALTHY, modifier)

            MorePage.ADHOC_TASKS -> {
                val state by adhocTasksViewModel.state.collectAsStateWithLifecycle()
                val running by adhocTasksViewModel.running.collectAsStateWithLifecycle()
                val runError by adhocTasksViewModel.runError.collectAsStateWithLifecycle()
                LaunchedEffect(lifecycleOwner) {
                    lifecycleOwner.repeatOnLifecycle(Lifecycle.State.STARTED) {
                        adhocTasksViewModel.refresh()
                    }
                }

                AdhocTasksScreen(
                    state = state,
                    running = running,
                    runError = runError,
                    onRunCron = { adhocTasksViewModel.runCron(it.name) },
                    onRunPending = adhocTasksViewModel::runPending,
                    modifier = modifier,
                )
            }

            MorePage.HOME_ASSISTANT -> {
                val state by homeAssistantViewModel.state.collectAsStateWithLifecycle()
                RunWhileStarted(homeAssistantViewModel::run)

                HomeAssistantScreen(state, modifier)
            }
        }
    }

    @Composable
    private fun RunWhileStarted(block: suspend () -> Unit) {
        val lifecycleOwner = LocalLifecycleOwner.current
        LaunchedEffect(lifecycleOwner) {
            lifecycleOwner.repeatOnLifecycle(Lifecycle.State.STARTED) {
                block()
            }
        }
    }

    @Composable
    private fun HistoryContent(
        viewModel: HistoryViewModel,
        healthy: ClosedFloatingPointRange<Double>,
        modifier: Modifier,
    ) {
        val state by viewModel.state.collectAsStateWithLifecycle()
        val range by viewModel.range.collectAsStateWithLifecycle()
        val lifecycleOwner = LocalLifecycleOwner.current
        LaunchedEffect(lifecycleOwner, viewModel) {
            lifecycleOwner.repeatOnLifecycle(Lifecycle.State.STARTED) {
                viewModel.refresh()
            }
        }

        HistoryScreen(
            state = state,
            range = range,
            healthy = healthy,
            onRange = viewModel::select,
            modifier = modifier,
        )
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

    private companion object {
        val BATTERY_HEALTHY = 20.0..100.0
        val PLANT_HEALTHY = 20.0..80.0
    }
}
