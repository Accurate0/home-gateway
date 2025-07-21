import {
  Card,
  CardContent,
  CardHeader,
} from "@/components/ui/card";
import { DashboardHeader } from "./DashboardHeader";

export const LoadingDashboard = ({ selectedHours }: { selectedHours: number }) => {
  // Get tab state from URL query parameters
  const urlParams = new URLSearchParams(window.location.search);
  const tabParam = urlParams.get('tab');
  const activeTab = (tabParam === 'overview' || tabParam === 'solar') ? tabParam : 'overview';

  return (
    <div className="min-h-screen bg-background p-6">
      <div className="mx-auto max-w-7xl space-y-6">
        {/* Header Skeleton - matches DashboardHeader */}
        <DashboardHeader 
        title="Loading..." 
        activeTab={activeTab} 
        setActiveTab={() => {}} 
        selectedHours={selectedHours} 
        setSelectedHours={() => {}} />

        {activeTab === 'overview' ? (
          <LoadingOverviewTab />
        ) : (
          <LoadingSolarEnergyTab />
        )}
      </div>
    </div>
  );
};

const LoadingOverviewTab = () => {
  return (
    <>
      {/* Event Timeline Card Skeleton - matches OverviewTab */}
      <Card className="w-full">
        <CardHeader className="flex justify-between items-center pb-0">
          <div>
            <div className="flex items-center gap-2">
              <div className="w-5 h-5 bg-muted rounded animate-pulse"></div>
              <div className="w-32 h-6 bg-muted rounded animate-pulse"></div>
            </div>
            <div className="w-64 h-4 bg-muted rounded animate-pulse mt-1"></div>
          </div>
          <div className="flex items-center gap-2">
            <div className="w-9 h-9 bg-muted rounded-md animate-pulse"></div>
            <div className="w-[200px] h-9 bg-muted rounded-md animate-pulse"></div>
          </div>
        </CardHeader>
        <CardContent className="pt-0">
          <div className="w-full h-[400px] bg-muted rounded animate-pulse"></div>
          <div className="flex justify-center mt-2">
            <div className="flex flex-wrap gap-2 p-3 bg-muted/50 rounded-lg">
              {[1, 2, 3, 4, 5, 6].map((i) => (
                <div key={i} className="w-36 h-8 bg-muted rounded animate-pulse"></div>
              ))}
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Temperature Card Skeleton - matches OverviewTab */}
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
    </>
  );
};

const LoadingSolarEnergyTab = () => {
  return (
    <>
      {/* Solar Card Skeleton - matches SolarEnergyTab */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-2">
            <div className="w-5 h-5 bg-muted rounded animate-pulse"></div>
            <div className="w-24 h-6 bg-muted rounded animate-pulse"></div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-[220px_1fr] gap-4 h-full min-h-[180px]">
            {/* Left: Info skeleton */}
            <div className="flex flex-col justify-center p-4 rounded-lg bg-muted/50 h-full">
              <div className="space-y-1">
                <div>
                  <div className="w-20 h-4 bg-muted rounded animate-pulse mb-2"></div>
                  <div className="w-24 h-8 bg-muted rounded animate-pulse mb-2"></div>
                </div>
                <div>
                  <div className="w-20 h-4 bg-muted rounded animate-pulse mb-2"></div>
                  <div className="w-24 h-8 bg-muted rounded animate-pulse mb-2"></div>
                </div>
                <div>
                  <div className="w-20 h-4 bg-muted rounded animate-pulse mb-2"></div>
                  <div className="w-24 h-8 bg-muted rounded animate-pulse"></div>
                </div>
              </div>
            </div>
            {/* Right: Chart skeleton */}
            <div className="flex items-center h-full min-h-[188px]">
              <div className="w-full h-[188px] bg-muted rounded animate-pulse"></div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Energy Usage Card Skeleton - matches SolarEnergyTab */}
      <Card>
        <CardHeader>
          <div className="flex justify-between items-center">
            <div className="flex items-center gap-2">
              <div className="w-5 h-5 bg-muted rounded animate-pulse"></div>
              <div className="w-32 h-6 bg-muted rounded animate-pulse"></div>
            </div>
            <div className="flex items-center gap-2">
              <div className="flex flex-wrap gap-2 items-center">
                <div className="w-16 h-4 bg-muted rounded animate-pulse"></div>
                <div className="w-24 h-6 bg-muted rounded animate-pulse"></div>
                <div className="w-4 h-4 bg-muted rounded animate-pulse"></div>
                <div className="w-24 h-6 bg-muted rounded animate-pulse"></div>
              </div>
              <div className="flex gap-2">
                <div className="w-12 h-6 bg-muted rounded animate-pulse"></div>
                <div className="w-16 h-6 bg-muted rounded animate-pulse"></div>
              </div>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-[220px_1fr] gap-4 h-full min-h-[180px]">
            {/* Left: Info skeleton */}
            <div className="flex flex-col justify-center p-4 rounded-lg bg-muted/50 h-full">
              <div className="space-y-1">
                <div>
                  <div className="w-24 h-4 bg-muted rounded animate-pulse mb-2"></div>
                  <div className="w-28 h-8 bg-muted rounded animate-pulse mb-2"></div>
                </div>
                <div>
                  <div className="w-28 h-4 bg-muted rounded animate-pulse mb-2"></div>
                  <div className="w-24 h-6 bg-muted rounded animate-pulse"></div>
                </div>
              </div>
            </div>
            {/* Right: Chart skeleton */}
            <div className="flex items-center h-full min-h-[188px]">
              <div className="w-full h-[220px] bg-muted rounded animate-pulse"></div>
            </div>
          </div>
        </CardContent>
      </Card>
    </>
  );
}; 