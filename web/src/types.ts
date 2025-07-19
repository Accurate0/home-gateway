import type React from "react";

// Type definitions
export interface Event {
  name: string;
  time: string;
  state: string;
  id: string;
}



export interface ChartEvent {
  x: number;
  y: number;
  time: string;
  name: string;
  state: string;
  id: string;
  color: string;
  type: string;
  shape: string;
}

export interface DeviceInfo {
  name: string;
  type: 'door' | 'appliance' | 'wifi';
  icon: React.ComponentType<{ className?: string }>;
}

export interface DevicesByType {
  doors: DeviceInfo[];
  appliances: DeviceInfo[];
  wifi: DeviceInfo[];
}

export interface VisibleEventTypes {
  doorsOpen: boolean;
  doorsClosed: boolean;
  appliancesOn: boolean;
  appliancesOff: boolean;
  wifiConnected: boolean;
  wifiDisconnected: boolean;
}

import type { AppQuery } from "./__generated__/AppQuery.graphql";
import type { PreloadedQuery } from "react-relay";

export interface DashboardContentProps {
  queryRef: PreloadedQuery<AppQuery> | null | undefined;
  loadQuery: (variables: { since: string }) => void;
}

export type TabType = 'overview' | 'solar';

export interface DashboardContentInnerProps {
  queryRef: PreloadedQuery<AppQuery>;
  selectedHours: number;
  setSelectedHours: (hours: number) => void;
}
