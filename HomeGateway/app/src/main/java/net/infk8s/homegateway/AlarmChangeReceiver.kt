package net.infk8s.homegateway

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import androidx.work.ExistingWorkPolicy
import androidx.work.OneTimeWorkRequest
import androidx.work.WorkManager

class AlarmChangeReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context?, intent: Intent?) {
        if(intent?.action != "android.app.action.NEXT_ALARM_CLOCK_CHANGED") {
            return;
        }

        context?.let {
            WorkManager.getInstance(it)
                .beginUniqueWork(
                    "alarm-state-update",
                    ExistingWorkPolicy.REPLACE,
                    OneTimeWorkRequest.from(AlarmWorker::class.java)
                ).enqueue()
        }
    }
}
