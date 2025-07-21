import { useQueryLoader } from "react-relay";
import { useState, useMemo, useEffect, useRef, Suspense } from "react";
import { APP_QUERY } from "./AppQuery";
import { DashboardContentInner } from "./components/DashboardContentInner";
import { LoadingDashboard } from "./components/LoadingSkeletons";
import type { DashboardContentProps } from "./types";
import type { AppQuery } from "./__generated__/AppQuery.graphql";
import { useTimeState } from "./hooks/useTimeState";

const DashboardContent = ({ queryRef, loadQuery }: DashboardContentProps) => {
  // Add time state at the top of the component
  const [selectedHours, setSelectedHours] = useTimeState();
  const [appliedTimestamp, setAppliedTimestamp] = useState(() => Date.now());

  // Calculate since based on applied timestamp and hours
  const since = useMemo(() => {
    return new Date(appliedTimestamp - selectedHours * 60 * 60 * 1000).toISOString();
  }, [appliedTimestamp, selectedHours]);

  // Only update appliedTimestamp when selectedHours changes (but not on mount)
  const prevSelectedHoursRef = useRef(selectedHours);
  useEffect(() => {
    if (prevSelectedHoursRef.current !== selectedHours) {
      setAppliedTimestamp(Date.now());
      prevSelectedHoursRef.current = selectedHours;
    }
  }, [selectedHours]);

  // Only loadQuery when since changes
  useEffect(() => {
    loadQuery({
      since,
    });
  }, [since, loadQuery]);

  return (
    <Suspense fallback={<LoadingDashboard selectedHours={selectedHours} />}>
      {queryRef && (
        <DashboardContentInner
          queryRef={queryRef}
          selectedHours={selectedHours}
          setSelectedHours={setSelectedHours}
        />
      )}
    </Suspense>
  );
};



const Dashboard = () => {
  const [queryRef, loadQuery] = useQueryLoader<AppQuery>(APP_QUERY);
  return <DashboardContent queryRef={queryRef} loadQuery={loadQuery} />;
};

export default Dashboard;
