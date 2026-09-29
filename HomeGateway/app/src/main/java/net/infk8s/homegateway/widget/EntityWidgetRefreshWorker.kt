package net.infk8s.homegateway.widget

import android.content.Context
import androidx.work.CoroutineWorker
import androidx.work.WorkerParameters

class EntityWidgetRefreshWorker(context: Context, params: WorkerParameters) : CoroutineWorker(context, params) {
    override suspend fun doWork(): Result {
        EntityWidgets.refreshAll(applicationContext)

        return Result.success()
    }
}
