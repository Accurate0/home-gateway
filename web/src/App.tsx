import {
  Card,
  CardContent,
  CardHeader,
} from "@/components/ui/card";
import { useQueryLoader } from "react-relay";
import { useState, useMemo, useEffect } from "react";
import { APP_QUERY } from "./AppQuery";
import { DashboardContentInner } from "./components/DashboardContentInner";
import type { DashboardContentProps } from "./types";
import type { AppQuery } from "./__generated__/AppQuery.graphql";

// Place this at the top-level, outside of DashboardContent
const INITIAL_TIMESTAMP = Date.now();

const DashboardContent = ({ queryRef, loadQuery }: DashboardContentProps) => {
  // Time range selection state
  const [selectedHours, setSelectedHours] = useState(12);
  const [appliedTimestamp, setAppliedTimestamp] = useState(INITIAL_TIMESTAMP);

  // Calculate since based on applied timestamp and hours
  const since = useMemo(() => {
    return new Date(appliedTimestamp - selectedHours * 60 * 60 * 1000).toISOString();
  }, [appliedTimestamp, selectedHours]);

  // Load query on mount and when hours change
  useEffect(() => {
    loadQuery({ since });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Auto-apply when selectedHours changes
  useEffect(() => {
    setAppliedTimestamp(Date.now());
    const newSince = new Date(Date.now() - selectedHours * 60 * 60 * 1000).toISOString();
    loadQuery({ since: newSince });
  }, [selectedHours, loadQuery]);

  return (
    queryRef != null ? (
      <DashboardContentInner
        queryRef={queryRef}
        selectedHours={selectedHours}
        setSelectedHours={setSelectedHours}
      />
    ) : (
      <LoadingDashboard />
    )
  );
};

const LoadingDashboard = () => {
  return (
    <div className="min-h-screen bg-background p-6">
      <div className="mx-auto max-w-7xl space-y-6">
        {/* Header */}
        <div className="flex justify-between items-center">
          <div className="text-center space-y-2">
            <h1 className="text-3xl font-bold">Timeline</h1>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-24 h-9 bg-muted rounded-md animate-pulse"></div>
          </div>
        </div>

        {/* Event Timeline Card */}
        <Card className="w-full">
          <CardHeader className="flex justify-between items-center">
            <div className="flex items-center gap-2">
              <div className="w-5 h-5 bg-muted rounded animate-pulse"></div>
              <div className="w-32 h-6 bg-muted rounded animate-pulse"></div>
            </div>
            <div className="w-[200px] h-9 bg-muted rounded-md animate-pulse"></div>
          </CardHeader>
          <CardContent>
            <div className="mb-2">
              <div className="w-64 h-4 bg-muted rounded animate-pulse"></div>
            </div>
            <div className="w-full h-[500px] bg-muted rounded animate-pulse"></div>
            <div className="flex justify-center mt-2">
              <div className="flex flex-wrap gap-2 p-3 bg-muted/50 rounded-lg">
                {[1, 2, 3, 4, 5, 6].map((i) => (
                  <div key={i} className="w-24 h-8 bg-muted rounded animate-pulse"></div>
                ))}
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Summary Cards */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          {[1, 2, 3, 4].map((i) => (
            <Card key={i}>
              <CardHeader className="pb-3">
                <div className="flex items-center gap-2">
                  <div className="w-5 h-5 bg-muted rounded animate-pulse"></div>
                  <div className="w-16 h-6 bg-muted rounded animate-pulse"></div>
                </div>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  <div className="flex justify-between items-center">
                    <div className="w-12 h-4 bg-muted rounded animate-pulse"></div>
                    <div className="w-8 h-5 bg-muted rounded animate-pulse"></div>
                  </div>
                  <div className="flex justify-between items-center">
                    <div className="w-16 h-4 bg-muted rounded animate-pulse"></div>
                    <div className="w-8 h-5 bg-muted rounded animate-pulse"></div>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>

        {/* Temperature Card */}
        <Card>
          <CardHeader>
            <div className="flex items-center gap-2">
              <div className="w-5 h-5 bg-muted rounded animate-pulse"></div>
              <div className="w-40 h-6 bg-muted rounded animate-pulse"></div>
            </div>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
              {[1, 2, 3, 4].map((i) => (
                <div key={i} className="flex items-center gap-3 p-3 bg-muted/50 rounded-lg">
                  <div className="w-4 h-4 bg-muted rounded animate-pulse"></div>
                  <div className="flex-1">
                    <div className="w-20 h-4 bg-muted rounded animate-pulse mb-2"></div>
                    <div className="w-16 h-6 bg-muted rounded animate-pulse mb-1"></div>
                    <div className="w-24 h-3 bg-muted rounded animate-pulse"></div>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};

const Dashboard = () => {
  const [queryRef, loadQuery] = useQueryLoader<AppQuery>(APP_QUERY);
  return <DashboardContent queryRef={queryRef} loadQuery={loadQuery} />;
};

export default Dashboard;
