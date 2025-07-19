import { usePreloadedQuery } from "react-relay";
import { APP_QUERY } from "../AppQuery";
import type { AppQuery } from "../__generated__/AppQuery.graphql";
import type { DashboardContentInnerProps } from "../types";
import { useTabState } from "../hooks/useTabState";
import { OverviewTab } from "./OverviewTab";
import { SolarEnergyTab } from "./SolarEnergyTab";

export const DashboardContentInner = ({ 
  queryRef, 
  selectedHours,
  setSelectedHours}: DashboardContentInnerProps) => {
  // Use the queryRef to get data
  const data = usePreloadedQuery<AppQuery>(APP_QUERY, queryRef);

  // Add tab state at the top of the component
  const [activeTab, setActiveTab] = useTabState();

  return (
    <div className="min-h-screen bg-background p-6">
      <div className="mx-auto max-w-7xl space-y-6">
        {activeTab === 'overview' ? (
          <OverviewTab 
            fragmentKey={data} 
            selectedHours={selectedHours} 
            setSelectedHours={setSelectedHours}
            activeTab={activeTab}
            setActiveTab={setActiveTab}
          />
        ) : (
          <SolarEnergyTab 
            fragmentKey={data} 
            selectedHours={selectedHours} 
            setSelectedHours={setSelectedHours}
            activeTab={activeTab}
            setActiveTab={setActiveTab}
          />
        )}
      </div>
    </div>
  );
}; 