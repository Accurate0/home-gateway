import type React from "react";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { ChartContainer, ChartTooltip } from "@/components/ui/chart";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Checkbox } from "@/components/ui/checkbox";
import {
  ResponsiveContainer,
  Scatter,
  CartesianGrid,
  XAxis,
  YAxis,
  Legend,
  Cell,
  Line,
  ComposedChart,
  LineChart,
} from "recharts";
import {
  DoorOpen,
  DoorClosed,
  Thermometer,
  Droplets,
  Wifi,
  WifiOff,
  Activity,
  Gauge,
  Power,
  PowerOff,
  Zap,
  Router,
  Monitor,
  Wind,
  ZoomIn,
  ZoomOut,
  RotateCcw,
  Filter,
  X,
  ChevronDown,
  Sun,
  Eye,
} from "lucide-react";
import { useLazyLoadQuery, graphql } from "react-relay";
import { Suspense, useState, useCallback, useMemo, useRef } from "react";
import type { AppQuery } from "./__generated__/AppQuery.graphql";

// Calculate 48 hours ago outside of React
const since = new Date(Date.now() - 48 * 60 * 60 * 1000).toISOString();

// Enhanced chart configuration with state-based colors
const chartConfig = {
  doors: {
    label: "Doors",
    open: "#22c55e", // green-500
    closed: "#ef4444", // red-500
  },
  appliances: {
    label: "Appliances",
    on: "#3b82f6", // blue-500
    off: "#6b7280", // gray-500
  },
  wifi: {
    label: "WiFi",
    connected: "#8b5cf6", // violet-500
    disconnected: "#f97316", // orange-500
  },
  solar: {
    label: "Solar",
    color: "#fbbf24", // amber-400
  },
  uv: {
    label: "UV Level",
    color: "#f59e0b", // amber-500
  },
};

// Updated GraphQL query with solar data
const APP_QUERY = graphql`
  query AppQuery($since: DateTime!) {
    solar {
      current {
        todayProductionKwh
      }
      history {
        at
        uvLevel
        wh
        timestamp
      }
    }
    events(input: { since: $since }) {
      doors {
        name
        time
        state
        id
      }
      appliances {
        name
        time
        id
        state
      }
      wifi {
        name
        time
        id
        state
      }
    }
    environment {
      outdoor {
        temperature
        humidity
        pressure
      }
      laundry {
        temperature
        humidity
        pressure
      }
      livingRoom {
        temperature
        humidity
        pressure
      }
      bedroom {
        temperature
        humidity
        pressure
      }
    }
  }
`;

// Helper function to get device icon based on name
const getDeviceIcon = (name: string, state: string) => {
  const lowerName = name.toLowerCase();

  // Door icons
  if (lowerName.includes("door") || lowerName.includes("garage")) {
    return state === "OPEN" ? (
      <DoorOpen className="h-4 w-4 text-green-500" />
    ) : (
      <DoorClosed className="h-4 w-4 text-red-500" />
    );
  }

  // Appliance icons
  if (lowerName.includes("washing") || lowerName.includes("machine")) {
    return state === "ON" ? (
      <Power className="h-4 w-4 text-blue-500" />
    ) : (
      <PowerOff className="h-4 w-4 text-gray-500" />
    );
  }

  // Default WiFi icon for all other devices
  if (state === "CONNECTED" || state === "DISCONNECTED") {
    return state === "CONNECTED" ? (
      <Wifi className="h-4 w-4 text-violet-500" />
    ) : (
      <WifiOff className="h-4 w-4 text-orange-500" />
    );
  }

  // Default icons
  return <Activity className="h-4 w-4" />;
};

// Helper function to get room icon
const getRoomIcon = (room: string) => {
  const lowerRoom = room.toLowerCase();

  if (lowerRoom.includes("outdoor")) {
    return <Wind className="h-4 w-4 text-blue-500" />;
  }
  if (lowerRoom.includes("bedroom")) {
    return <Monitor className="h-4 w-4 text-purple-500" />;
  }
  if (lowerRoom.includes("living")) {
    return <Activity className="h-4 w-4 text-green-500" />;
  }
  if (lowerRoom.includes("laundry")) {
    return <Power className="h-4 w-4 text-blue-600" />;
  }

  return <Thermometer className="h-4 w-4 text-muted-foreground" />;
};

function DashboardContent() {
  const data = useLazyLoadQuery<AppQuery>(APP_QUERY, { since });

  // Zoom and pan state
  const [zoomDomain, setZoomDomain] = useState<[number, number] | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const dragStartRef = useRef<{ x: number; domain: [number, number] } | null>(
    null,
  );

  // Event state visibility
  const [visibleStates, setVisibleStates] = useState({
    doorsOpen: true,
    doorsClosed: true,
    appliancesOn: true,
    appliancesOff: true,
    wifiConnected: true,
    wifiDisconnected: true,
    solar: true,
    uv: true,
  });

  // Device filter state
  const [selectedDevices, setSelectedDevices] = useState<Set<string>>(
    new Set(),
  );
  const [deviceFilterOpen, setDeviceFilterOpen] = useState(false);

  // ---------- helpers ----------
  const toMs = (iso: string | number) => new Date(iso).getTime();
  const fmtTime = (iso: string | number) =>
    new Date(iso).toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });

  // Group events by type with color coding
  const doors = data.events.doors.map((e) => ({
    x: toMs(e.time),
    y: 1.5,
    time: fmtTime(e.time),
    name: e.name,
    state: e.state,
    id: e.id,
    color:
      e.state === "OPEN" ? chartConfig.doors.open : chartConfig.doors.closed,
    type: "door",
  }));

  const apps = data.events.appliances.map((e) => ({
    x: toMs(e.time),
    y: 1.0,
    time: fmtTime(e.time),
    name: e.name,
    state: e.state,
    id: e.id,
    color:
      e.state === "ON" ? chartConfig.appliances.on : chartConfig.appliances.off,
    type: "appliance",
  }));

  const wifis = data.events.wifi.map((e) => ({
    x: toMs(e.time),
    y: 0.5,
    time: fmtTime(e.time),
    name: e.name,
    state: e.state,
    id: e.id,
    color:
      e.state === "CONNECTED"
        ? chartConfig.wifi.connected
        : chartConfig.wifi.disconnected,
    type: "wifi",
  }));

  // Process solar data and combine with events for the chart
  const solarData = useMemo(() => {
    if (!data.solar?.history) return [];
    return data.solar.history.map((point) => ({
      x: toMs(point.timestamp),
      wh: point.wh,
      uvLevel: point.uvLevel,
      time: fmtTime(point.timestamp),
    }));
  }, [data.solar]);

  // Combine all events
  const allEvents = useMemo(
    () => [...doors, ...apps, ...wifis].sort((a, b) => a.x - b.x),
    [doors, apps, wifis],
  );

  // Create combined data for the chart that includes both events and solar data
  const combinedChartData = useMemo(() => {
    const allTimePoints = new Set<number>();
    solarData.forEach((point) => allTimePoints.add(point.x));
    allEvents.forEach((event) => allTimePoints.add(event.x));
    // Create combined data points
    return Array.from(allTimePoints)
      .sort()
      .map((timePoint) => {
        const solarPoint = solarData.find((s) => s.x === timePoint);
        return {
          x: timePoint,
          time: fmtTime(new Date(timePoint).toISOString()),
          wh: solarPoint?.wh ?? null,
          uvLevel: solarPoint?.uvLevel ?? null,
          doors: doors.filter((e) => e.x === timePoint),
          appliances: apps.filter((e) => e.x === timePoint),
          wifi: wifis.filter((e) => e.x === timePoint),
        };
      });
  }, [solarData, allEvents, doors, apps, wifis]);

  // Get all unique device names for filtering
  const allDeviceNames = useMemo(() => {
    const names = new Set<string>();
    allEvents.forEach((event) => names.add(event.name));
    return Array.from(names).sort();
  }, [allEvents]);

  // Calculate time domain including solar data
  const timeDomain = useMemo(() => {
    const eventTimes = allEvents.map((e) => e.x);
    const solarTimes = solarData.map((s) => s.x);
    const allTimes = [...eventTimes, ...solarTimes];

    if (allTimes.length === 0)
      return [Date.now() - 24 * 60 * 60 * 1000, Date.now()];

    const minTime = Math.min(...allTimes);
    const maxTime = Math.max(...allTimes);
    const padding = (maxTime - minTime) * 0.05; // 5% padding
    return [minTime - padding, maxTime + padding];
  }, [allEvents, solarData]);

  // Get current domain for display
  const currentDomain = zoomDomain || timeDomain;

  // Cards for temperature / humidity / pressure
  const env = Object.entries(data.environment).map(
    ([key, val]: [string, any]) => ({
      room: key
        .replace(/([A-Z])/g, " $1")
        .replace(/^./, (s) => s.toUpperCase()),
      temperature: val?.temperature ?? 0,
      humidity: val?.humidity ?? 0,
      pressure: val?.pressure ?? 0,
    }),
  );

  // --- Optimize device filters for performance ---
  // Memoize filtered events to avoid re-computation
  const filteredEvents = useMemo(() => {
    return allEvents.filter((event) => {
      const inTimeRange =
        event.x >= currentDomain[0] && event.x <= currentDomain[1];

      let stateVisible = false;
      if (event.type === "door") {
        stateVisible =
          (event.state === "OPEN" && visibleStates.doorsOpen) ||
          (event.state === "CLOSED" && visibleStates.doorsClosed);
      } else if (event.type === "appliance") {
        stateVisible =
          (event.state === "ON" && visibleStates.appliancesOn) ||
          (event.state === "OFF" && visibleStates.appliancesOff);
      } else if (event.type === "wifi") {
        stateVisible =
          (event.state === "CONNECTED" && visibleStates.wifiConnected) ||
          (event.state === "DISCONNECTED" && visibleStates.wifiDisconnected);
      }

      const deviceVisible =
        selectedDevices.size === 0 || selectedDevices.has(event.name);
      
      return inTimeRange && stateVisible && deviceVisible;
    });
  }, [allEvents, currentDomain, visibleStates, selectedDevices]);

  // Memoize filtered event arrays by type
  const visibleDoors = useMemo(() => {
    return filteredEvents.filter((event) => event.type === "door");
  }, [filteredEvents]);

  const visibleApps = useMemo(() => {
    return filteredEvents.filter((event) => event.type === "appliance");
  }, [filteredEvents]);

  const visibleWifis = useMemo(() => {
    return filteredEvents.filter((event) => event.type === "wifi");
  }, [filteredEvents]);

  // Memoize device counts for legend
  const eventCounts = useMemo(() => ({
    doorsOpen: doors.filter((d) => d.state === "OPEN").length,
    doorsClosed: doors.filter((d) => d.state === "CLOSED").length,
    appliancesOn: apps.filter((a) => a.state === "ON").length,
    appliancesOff: apps.filter((a) => a.state === "OFF").length,
    wifiConnected: wifis.filter((w) => w.state === "CONNECTED").length,
    wifiDisconnected: wifis.filter((w) => w.state === "DISCONNECTED").length,
  }), [doors, apps, wifis]);

  // Group devices by type for better organization
  const devicesByType = useMemo(() => {
    const doorDevices = [...new Set(doors.map((d) => d.name))].sort();
    const applianceDevices = [...new Set(apps.map((a) => a.name))].sort();
    const wifiDevices = [...new Set(wifis.map((w) => w.name))].sort();

    return {
      doors: doorDevices,
      appliances: applianceDevices,
      wifi: wifiDevices,
    };
  }, [doors, apps, wifis]);

  // Optimize device filter functions
  const toggleDevice = useCallback((deviceName: string) => {
    setSelectedDevices((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(deviceName)) {
        newSet.delete(deviceName);
      } else {
        newSet.add(deviceName);
      }
      return newSet;
    });
  }, []);

  const selectAllDevices = useCallback(() => {
    setSelectedDevices(new Set(allDeviceNames));
  }, [allDeviceNames]);

  const clearAllDevices = useCallback(() => {
    setSelectedDevices(new Set());
  }, []);

  const selectDevicesByType = useCallback(
    (type: "doors" | "appliances" | "wifi") => {
      setSelectedDevices((prev) => {
        const newSet = new Set(prev);
        devicesByType[type].forEach((device) => newSet.add(device));
        return newSet;
      });
    },
    [devicesByType],
  );

  const clearDevicesByType = useCallback(
    (type: "doors" | "appliances" | "wifi") => {
      setSelectedDevices((prev) => {
        const newSet = new Set(prev);
        devicesByType[type].forEach((device) => newSet.delete(device));
        return newSet;
      });
    },
    [devicesByType],
  );

  // Memoize state toggle functions
  const toggleState = useCallback((stateKey: keyof typeof visibleStates) => {
    setVisibleStates((prev) => ({
      ...prev,
      [stateKey]: !prev[stateKey],
    }));
  }, []);

  // Zoom controls
  const handleZoomIn = useCallback(() => {
    const center = (currentDomain[0] + currentDomain[1]) / 2;
    const range = (currentDomain[1] - currentDomain[0]) * 0.5; // Zoom in by 50%
    const newDomain: [number, number] = [
      center - range / 2,
      center + range / 2,
    ];
    setZoomDomain(newDomain);
  }, [currentDomain]);

  const handleZoomOut = useCallback(() => {
    const center = (currentDomain[0] + currentDomain[1]) / 2;
    const range = (currentDomain[1] - currentDomain[0]) * 2; // Zoom out by 200%
    const newRange = Math.min(range, timeDomain[1] - timeDomain[0]); // Don't zoom out beyond full range
    const newDomain: [number, number] = [
      Math.max(timeDomain[0], center - newRange / 2),
      Math.min(timeDomain[1], center + newRange / 2),
    ];

    // If we're at full range, reset zoom
    if (newRange >= timeDomain[1] - timeDomain[0]) {
      setZoomDomain(null);
    } else {
      setZoomDomain(newDomain);
    }
  }, [currentDomain, timeDomain]);

  const handleResetZoom = useCallback(() => {
    setZoomDomain(null);
  }, []);

  const handleWheelZoom = useCallback(
    (event: React.WheelEvent) => {
      event.preventDefault();

      // Get mouse position relative to chart
      const rect = event.currentTarget.getBoundingClientRect();
      const mouseX = event.clientX - rect.left;
      const chartWidth = rect.width;

      // Calculate mouse position as percentage of chart width
      const mousePercent = mouseX / chartWidth;

      const currentRange = currentDomain[1] - currentDomain[0];

      // Calculate zoom center based on mouse position
      const zoomCenter = currentDomain[0] + currentRange * mousePercent;

      // Zoom factor based on wheel direction
      const zoomFactor = event.deltaY > 0 ? 1.2 : 0.8; // Zoom out or in
      const newRange = currentRange * zoomFactor;

      // Don't zoom out beyond full range
      if (newRange >= timeDomain[1] - timeDomain[0]) {
        setZoomDomain(null);
        return;
      }

      // Calculate new domain centered on mouse position
      const newStart = Math.max(
        timeDomain[0],
        zoomCenter - newRange * mousePercent,
      );
      const newEnd = Math.min(
        timeDomain[1],
        zoomCenter + newRange * (1 - mousePercent),
      );

      // Adjust if we hit boundaries
      const adjustedStart = Math.max(
        timeDomain[0],
        Math.min(newStart, timeDomain[1] - newRange),
      );
      const adjustedEnd = Math.min(
        timeDomain[1],
        Math.max(newEnd, timeDomain[0] + newRange),
      );

      setZoomDomain([adjustedStart, adjustedEnd]);
    },
    [currentDomain, timeDomain],
  );

  // Drag and pan functionality
  const handleMouseDown = useCallback(
    (event: React.MouseEvent) => {
      if (!zoomDomain) return; // Only allow dragging when zoomed

      event.preventDefault();
      const rect = event.currentTarget.getBoundingClientRect();
      const startX = event.clientX - rect.left;

      setIsDragging(true);
      dragStartRef.current = {
        x: startX,
        domain: [...zoomDomain],
      };
    },
    [zoomDomain],
  );

  const handleMouseMove = useCallback(
    (event: React.MouseEvent) => {
      if (!isDragging || !dragStartRef.current || !zoomDomain) return;

      event.preventDefault();
      const rect = event.currentTarget.getBoundingClientRect();
      const currentX = event.clientX - rect.left;
      const deltaX = currentX - dragStartRef.current.x;
      const chartWidth = rect.width;

      // Calculate time delta based on pixel movement
      const currentRange = zoomDomain[1] - zoomDomain[0];
      const timeDelta = (deltaX / chartWidth) * currentRange;

      // Calculate new domain
      const newStart = dragStartRef.current.domain[0] - timeDelta;
      const newEnd = dragStartRef.current.domain[1] - timeDelta;

      // Clamp to boundaries
      const totalRange = timeDomain[1] - timeDomain[0];
      const clampedStart = Math.max(
        timeDomain[0],
        Math.min(newStart, timeDomain[1] - currentRange),
      );
      const clampedEnd = Math.min(
        timeDomain[1],
        Math.max(newEnd, timeDomain[0] + currentRange),
      );

      setZoomDomain([clampedStart, clampedEnd]);
    },
    [isDragging, zoomDomain, timeDomain],
  );

  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
    dragStartRef.current = null;
  }, []);

  const handleMouseLeave = useCallback(() => {
    setIsDragging(false);
    dragStartRef.current = null;
  }, []);

  // Filter solar data based on time range and visibility
  const visibleSolarData = useMemo(() => {
    if (!visibleStates.solar && !visibleStates.uv) return [];
    return solarData.filter(
      (point) => point.x >= currentDomain[0] && point.x <= currentDomain[1],
    );
  }, [solarData, currentDomain, visibleStates]);

  // Generate evenly spaced time ticks
  const generateTimeTicks = useCallback((domain: [number, number]) => {
    const [start, end] = domain;
    const range = end - start;
    const targetTicks = 8; // Aim for about 8 ticks

    // Calculate appropriate interval
    const rawInterval = range / targetTicks;

    // Round to nice intervals (1min, 5min, 15min, 30min, 1hr, 2hr, 6hr, 12hr, 24hr)
    const intervals = [
      60 * 1000, // 1 minute
      5 * 60 * 1000, // 5 minutes
      15 * 60 * 1000, // 15 minutes
      30 * 60 * 1000, // 30 minutes
      60 * 60 * 1000, // 1 hour
      2 * 60 * 60 * 1000, // 2 hours
      6 * 60 * 60 * 1000, // 6 hours
      12 * 60 * 60 * 1000, // 12 hours
      24 * 60 * 60 * 1000, // 24 hours
    ];

    const interval =
      intervals.find((i) => i >= rawInterval) ||
      intervals[intervals.length - 1];

    // Generate ticks
    const ticks = [];
    const startTick = Math.ceil(start / interval) * interval;

    for (let tick = startTick; tick <= end; tick += interval) {
      ticks.push(tick);
    }

    return ticks;
  }, []);

  const timeTicks = useMemo(
    () => generateTimeTicks(currentDomain),
    [currentDomain, generateTimeTicks],
  );

  // Custom legend component
  const CustomLegend = () => {
    return (
      <div className="flex flex-wrap justify-center gap-3 mt-4">
        <button
          onClick={() => toggleState("doorsOpen")}
          className={`flex items-center gap-2 px-3 py-1 rounded-md transition-all ${
            visibleStates.doorsOpen
              ? "bg-green-100 text-green-800 hover:bg-green-200"
              : "bg-gray-100 text-gray-500 hover:bg-gray-200"
          }`}
        >
          <DoorOpen className="w-4 h-4 text-green-500" />
          <span className="text-sm font-medium">Open ({eventCounts.doorsOpen})</span>
        </button>

        <button
          onClick={() => toggleState("doorsClosed")}
          className={`flex items-center gap-2 px-3 py-1 rounded-md transition-all ${
            visibleStates.doorsClosed
              ? "bg-red-100 text-red-800 hover:bg-red-200"
              : "bg-gray-100 text-gray-500 hover:bg-gray-200"
          }`}
        >
          <DoorClosed className="w-4 h-4 text-red-500" />
          <span className="text-sm font-medium">
            Closed ({eventCounts.doorsClosed})
          </span>
        </button>

        <button
          onClick={() => toggleState("appliancesOn")}
          className={`flex items-center gap-2 px-3 py-1 rounded-md transition-all ${
            visibleStates.appliancesOn
              ? "bg-blue-100 text-blue-800 hover:bg-blue-200"
              : "bg-gray-100 text-gray-500 hover:bg-gray-200"
          }`}
        >
          <Power className="w-4 h-4 text-blue-500" />
          <span className="text-sm font-medium">
            On ({eventCounts.appliancesOn})
          </span>
        </button>

        <button
          onClick={() => toggleState("appliancesOff")}
          className={`flex items-center gap-2 px-3 py-1 rounded-md transition-all ${
            visibleStates.appliancesOff
              ? "bg-gray-100 text-gray-800 hover:bg-gray-200"
              : "bg-gray-100 text-gray-500 hover:bg-gray-200"
          }`}
        >
          <PowerOff className="w-4 h-4 text-gray-500" />
          <span className="text-sm font-medium">
            Off ({eventCounts.appliancesOff})
          </span>
        </button>

        <button
          onClick={() => toggleState("wifiConnected")}
          className={`flex items-center gap-2 px-3 py-1 rounded-md transition-all ${
            visibleStates.wifiConnected
              ? "bg-violet-100 text-violet-800 hover:bg-violet-200"
              : "bg-gray-100 text-gray-500 hover:bg-gray-200"
          }`}
        >
          <Wifi className="w-4 h-4 text-violet-500" />
          <span className="text-sm font-medium">
            Connected ({eventCounts.wifiConnected})
          </span>
        </button>

        <button
          onClick={() => toggleState("wifiDisconnected")}
          className={`flex items-center gap-2 px-3 py-1 rounded-md transition-all ${
            visibleStates.wifiDisconnected
              ? "bg-orange-100 text-orange-800 hover:bg-orange-200"
              : "bg-gray-100 text-gray-500 hover:bg-gray-200"
          }`}
        >
          <WifiOff className="w-4 h-4 text-orange-500" />
          <span className="text-sm font-medium">
            Disconnected ({eventCounts.wifiDisconnected})
          </span>
        </button>
      </div>
    );
  };

  // --- Remove debugging and fix tooltip ---
  // Remove debug logging

  // --- Fix tooltip to properly extract solar and UV values ---
  const Tooltip = ({ active, payload, label }: any) => {
    if (!active || !payload?.length) return null;
    
    const timeLabel = new Date(label).toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });

    // Find events at this time
    const eventsAtTime = allEvents.filter((event) => event.x === label);
    
    return (
      <div className="rounded-md border bg-background p-3 shadow-lg">
        <p className="text-sm text-muted-foreground mb-2">Time: {timeLabel}</p>
        
        {eventsAtTime.map((event: any, idx: number) => {
          // Check if this event should be visible based on current filters
          let stateVisible = false;
          if (event.type === "door") {
            stateVisible =
              (event.state === "OPEN" && visibleStates.doorsOpen) ||
              (event.state === "CLOSED" && visibleStates.doorsClosed);
          } else if (event.type === "appliance") {
            stateVisible =
              (event.state === "ON" && visibleStates.appliancesOn) ||
              (event.state === "OFF" && visibleStates.appliancesOff);
          } else if (event.type === "wifi") {
            stateVisible =
              (event.state === "CONNECTED" && visibleStates.wifiConnected) ||
              (event.state === "DISCONNECTED" &&
                visibleStates.wifiDisconnected);
          }
          
          const deviceVisible =
            selectedDevices.size === 0 || selectedDevices.has(event.name);
          
          if (!stateVisible || !deviceVisible) return null;
          
          return (
            <div key={`event-${idx}`} className="flex items-center gap-2 mb-1">
              {getDeviceIcon(event.name, event.state)}
              <div>
                <p className="font-medium">{event.name}</p>
                <p
                  className={`text-sm font-medium ${
                    event.state === "OPEN" ||
                    event.state === "ON" ||
                    event.state === "CONNECTED"
                      ? "text-green-600"
                      : "text-red-600"
                  }`}
                >
                  State: {event.state}
                </p>
              </div>
            </div>
          );
        })}
      </div>
    );
  };

  // --- Hide Y axes completely ---
  return (
    <div className="min-h-screen bg-background p-6">
      <div className="mx-auto max-w-7xl space-y-6">
        <div className="text-center space-y-2">
          <h1 className="text-3xl font-bold">Timeline</h1>
        </div>

        {/* Timeline with Zoom Controls and Device Filters - Full Width */}
        <Card className="w-full">
          <CardHeader>
            <div className="flex items-center justify-between">
              <div>
                <CardTitle className="flex items-center gap-2">
                  <Activity className="h-5 w-5" />
                  Event Flow Timeline
                </CardTitle>
                {zoomDomain && (
                  <p className="text-xs text-muted-foreground mt-1">
                    Viewing: {new Date(currentDomain[0]).toLocaleTimeString()} -{" "}
                    {new Date(currentDomain[1]).toLocaleTimeString()}
                  </p>
                )}
              </div>
              <div className="flex items-center gap-2">
                <Button variant="outline" size="sm" onClick={handleZoomIn}>
                  <ZoomIn className="h-4 w-4" />
                </Button>
                <Button variant="outline" size="sm" onClick={handleZoomOut}>
                  <ZoomOut className="h-4 w-4" />
                </Button>
                <Button variant="outline" size="sm" onClick={handleResetZoom}>
                  <RotateCcw className="h-4 w-4" />
                </Button>
              </div>
            </div>
          </CardHeader>
          <CardContent className="w-full space-y-4">
            {/* Device Filter Section - Integrated */}
            <div className="border-b pb-4">
              <div className="flex items-center justify-between mb-3">
                <h3 className="text-sm font-medium flex items-center gap-2">
                  <Filter className="h-4 w-4" />
                  Device Filters
                </h3>
                <div className="flex items-center gap-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={selectAllDevices}
                  >
                    Select All
                  </Button>
                  <Button variant="outline" size="sm" onClick={clearAllDevices}>
                    Clear All
                  </Button>
                  <Popover
                    open={deviceFilterOpen}
                    onOpenChange={setDeviceFilterOpen}
                  >
                    <PopoverTrigger asChild>
                      <Button variant="outline" size="sm">
                        <Filter className="h-4 w-4 mr-2" />
                        Devices ({selectedDevices.size})
                        <ChevronDown className="h-4 w-4 ml-2" />
                      </Button>
                    </PopoverTrigger>
                    <PopoverContent className="w-80 p-0" align="end">
                      <div className="p-4 space-y-4 max-h-96 overflow-y-auto">
                        {/* Doors Section */}
                        <div className="space-y-2">
                          <div className="flex items-center justify-between">
                            <h4 className="font-medium flex items-center gap-2">
                              <DoorOpen className="h-4 w-4 text-green-500" />
                              Doors ({devicesByType.doors.length})
                            </h4>
                            <div className="flex gap-1">
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => selectDevicesByType("doors")}
                                className="h-6 px-2 text-xs"
                              >
                                All
                              </Button>
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => clearDevicesByType("doors")}
                                className="h-6 px-2 text-xs"
                              >
                                None
                              </Button>
                            </div>
                          </div>
                          <div className="space-y-2 pl-6">
                            {devicesByType.doors.map((device) => (
                              <div
                                key={device}
                                className="flex items-center space-x-2"
                              >
                                <Checkbox
                                  id={`door-${device}`}
                                  checked={selectedDevices.has(device)}
                                  onCheckedChange={() => toggleDevice(device)}
                                />
                                <label
                                  htmlFor={`door-${device}`}
                                  className="text-sm cursor-pointer"
                                >
                                  {device}
                                </label>
                              </div>
                            ))}
                          </div>
                        </div>

                        {/* Appliances Section */}
                        <div className="space-y-2">
                          <div className="flex items-center justify-between">
                            <h4 className="font-medium flex items-center gap-2">
                              <Power className="h-4 w-4 text-blue-500" />
                              Appliances ({devicesByType.appliances.length})
                            </h4>
                            <div className="flex gap-1">
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() =>
                                  selectDevicesByType("appliances")
                                }
                                className="h-6 px-2 text-xs"
                              >
                                All
                              </Button>
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => clearDevicesByType("appliances")}
                                className="h-6 px-2 text-xs"
                              >
                                None
                              </Button>
                            </div>
                          </div>
                          <div className="space-y-2 pl-6">
                            {devicesByType.appliances.map((device) => (
                              <div
                                key={device}
                                className="flex items-center space-x-2"
                              >
                                <Checkbox
                                  id={`appliance-${device}`}
                                  checked={selectedDevices.has(device)}
                                  onCheckedChange={() => toggleDevice(device)}
                                />
                                <label
                                  htmlFor={`appliance-${device}`}
                                  className="text-sm cursor-pointer"
                                >
                                  {device}
                                </label>
                              </div>
                            ))}
                          </div>
                        </div>

                        {/* WiFi Section */}
                        <div className="space-y-2">
                          <div className="flex items-center justify-between">
                            <h4 className="font-medium flex items-center gap-2">
                              <Wifi className="h-4 w-4 text-violet-500" />
                              WiFi Devices ({devicesByType.wifi.length})
                            </h4>
                            <div className="flex gap-1">
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => selectDevicesByType("wifi")}
                                className="h-6 px-2 text-xs"
                              >
                                All
                              </Button>
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => clearDevicesByType("wifi")}
                                className="h-6 px-2 text-xs"
                              >
                                None
                              </Button>
                            </div>
                          </div>
                          <div className="space-y-2 pl-6">
                            {devicesByType.wifi.map((device) => (
                              <div
                                key={device}
                                className="flex items-center space-x-2"
                              >
                                <Checkbox
                                  id={`wifi-${device}`}
                                  checked={selectedDevices.has(device)}
                                  onCheckedChange={() => toggleDevice(device)}
                                />
                                <label
                                  htmlFor={`wifi-${device}`}
                                  className="text-sm cursor-pointer"
                                >
                                  {device}
                                </label>
                              </div>
                            ))}
                          </div>
                        </div>
                      </div>
                    </PopoverContent>
                  </Popover>
                </div>
              </div>

              {/* Selected Device Badges */}
              {selectedDevices.size > 0 && (
                <div className="flex flex-wrap gap-2">
                  {Array.from(selectedDevices).map((device) => (
                    <Badge
                      key={device}
                      variant="secondary"
                      className="flex items-center gap-1"
                    >
                      {device}
                      <button
                        onClick={() => toggleDevice(device)}
                        className="ml-1 hover:bg-muted-foreground/20 rounded-full p-0.5"
                      >
                        <X className="h-3 w-3" />
                      </button>
                    </Badge>
                  ))}
                </div>
              )}
              {selectedDevices.size === 0 && (
                <p className="text-sm text-muted-foreground">
                  All devices are visible. Use the filter menu to select
                  specific devices.
                </p>
              )}
            </div>

            {/* Main Timeline Chart */}
            <ChartContainer
              config={chartConfig}
              className={`w-full h-[500px] ${zoomDomain ? "cursor-grab" : ""} ${isDragging ? "cursor-grabbing" : ""}`}
              onWheel={handleWheelZoom}
              onMouseDown={handleMouseDown}
              onMouseMove={handleMouseMove}
              onMouseUp={handleMouseUp}
              onMouseLeave={handleMouseLeave}
            >
              <ResponsiveContainer width="100%" height="100%">
                <ComposedChart
                  width={100}
                  height={100}
                  margin={{ top: 20, right: 30, left: 10, bottom: 40 }}
                  data={combinedChartData.filter(
                    (point) =>
                      point.x >= currentDomain[0] &&
                      point.x <= currentDomain[1],
                  )}
                >
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis
                    dataKey="x"
                    type="number"
                    domain={currentDomain}
                    scale="time"
                    ticks={timeTicks}
                    tickFormatter={(t) =>
                      new Date(t).toLocaleTimeString([], {
                        hour: "2-digit",
                        minute: "2-digit",
                      })
                    }
                    angle={-45}
                    textAnchor="end"
                    height={60}
                  />
                  <YAxis yAxisId="left" domain={[0, 2]} hide={true} />
                  <ChartTooltip content={<Tooltip />} />
                  <Legend content={<CustomLegend />} />

                  {/* Scatter for doors */}
                  {(visibleStates.doorsOpen || visibleStates.doorsClosed) && (
                    <Scatter
                      yAxisId="left"
                      name="Doors"
                      data={visibleDoors}
                      shape="circle"
                      dataKey="y"
                    >
                      {visibleDoors.map((door, i) => (
                        <Cell key={i} fill={door.color} />
                      ))}
                    </Scatter>
                  )}

                  {/* Scatter for appliances */}
                  {(visibleStates.appliancesOn ||
                    visibleStates.appliancesOff) && (
                    <Scatter
                      yAxisId="left"
                      name="Appliances"
                      data={visibleApps}
                      shape="triangle"
                      dataKey="y"
                    >
                      {visibleApps.map((app, i) => (
                        <Cell key={i} fill={app.color} />
                      ))}
                    </Scatter>
                  )}

                  {/* Scatter for wifi */}
                  {(visibleStates.wifiConnected ||
                    visibleStates.wifiDisconnected) && (
                    <Scatter
                      yAxisId="left"
                      name="WiFi"
                      data={visibleWifis}
                      shape="square"
                      dataKey="y"
                    >
                      {visibleWifis.map((wifi, i) => (
                        <Cell key={i} fill={wifi.color} />
                      ))}
                    </Scatter>
                  )}
                </ComposedChart>
              </ResponsiveContainer>
            </ChartContainer>
          </CardContent>
        </Card>

        {/* Solar Chart Section */}
        <Card className="w-full">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Sun className="h-5 w-5 text-amber-500" />
              Solar Generation (24hr)
            </CardTitle>
            <CardDescription>
              Real-time solar production and UV levels
            </CardDescription>
          </CardHeader>
          <CardContent>
            {/* Solar Chart */}
            <ChartContainer 
              config={chartConfig}
              className="aspect-auto h-[300px] w-full"
            >
              <LineChart
                data={solarData.filter(
                  (point) =>
                    point.x >= currentDomain[0] && point.x <= currentDomain[1],
                )}
                margin={{
                  left: 12,
                  right: 12,
                  top: 12,
                  bottom: 12,
                }}
              >
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis
                  dataKey="x"
                  type="number"
                  domain={currentDomain}
                  scale="time"
                  ticks={timeTicks}
                  tickFormatter={(t) =>
                    new Date(t).toLocaleTimeString([], {
                      hour: "2-digit",
                      minute: "2-digit",
                    })
                  }
                  angle={-45}
                  textAnchor="end"
                  height={60}
                />
                {/* --- Hide Y-axis markers for solar chart --- */}
                <YAxis
                  yAxisId="solar"
                  domain={[0, 5000]}
                  tick={{ fill: chartConfig.solar.color }}
                  axisLine={false}
                  tickLine={false}
                  hide={true}
                />
                <YAxis
                  yAxisId="uv"
                  orientation="right"
                  domain={[0, 13]}
                  tick={{ fill: chartConfig.uv.color }}
                  axisLine={false}
                  tickLine={false}
                  hide={true}
                />
                <ChartTooltip
                  content={({ active, payload, label }) => {
                    if (!active || !payload?.length) return null;
                    
                    const timeLabel = label ? new Date(label).toLocaleTimeString([], {
                      hour: "2-digit",
                      minute: "2-digit",
                      second: "2-digit",
                    }) : "";
                    
                    return (
                      <div className="rounded-md border bg-background p-3 shadow-lg">
                        <p className="text-sm text-muted-foreground mb-2">
                          Time: {timeLabel}
                        </p>
                        {payload.map((entry: any, index: number) => {
                          if (entry.dataKey === "wh" && entry.value !== null) {
                            return (
                              <div key={`solar-${index}`} className="flex items-center gap-2 mb-1">
                                <Sun className="h-4 w-4 text-amber-500" />
                                <span className="text-sm font-medium">
                                  Solar: {entry.value.toFixed(1)} Wh
                                </span>
                              </div>
                            );
                          }
                          if (entry.dataKey === "uvLevel" && entry.value !== null) {
                            return (
                              <div key={`uv-${index}`} className="flex items-center gap-2 mb-1">
                                <Eye className="h-4 w-4 text-amber-600" />
                                <span className="text-sm font-medium">
                                  UV Level: {entry.value.toFixed(1)}
                                </span>
                              </div>
                            );
                          }
                          return null;
                        })}
                      </div>
                    );
                  }}
                />

                {/* Solar line */}
                <Line
                  yAxisId="solar"
                  type="monotone"
                  dataKey="wh"
                  stroke={chartConfig.solar.color}
                  strokeWidth={2}
                  dot={false}
                  name="Solar (Wh)"
                  connectNulls={false}
                />

                {/* UV level line */}
                <Line
                  yAxisId="uv"
                  type="monotone"
                  dataKey="uvLevel"
                  stroke={chartConfig.uv.color}
                  strokeWidth={2}
                  strokeDasharray="5 5"
                  dot={false}
                  name="UV Level"
                  connectNulls={false}
                />
              </LineChart>
            </ChartContainer>
          </CardContent>
        </Card>

        {/* Enhanced Temperature / Humidity / Pressure cards */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {env.map((r) => (
            <Card key={r.room} className="hover:shadow-md transition-shadow">
              <CardHeader className="flex flex-row items-center justify-between pb-2">
                <CardTitle className="text-sm flex items-center gap-2">
                  {getRoomIcon(r.room)}
                  {r.room}
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div className="flex items-center gap-2">
                    <Thermometer className="h-5 w-5 text-red-500" />
                    <div className="text-2xl font-bold">{r.temperature}°C</div>
                  </div>
                  <div className="space-y-2">
                    {r.humidity > 0 && (
                      <div className="flex items-center gap-2 text-sm">
                        <Droplets className="h-4 w-4 text-blue-500" />
                        <span className="font-medium">{r.humidity}%</span>
                        <span className="text-muted-foreground">humidity</span>
                      </div>
                    )}
                    {r.pressure > 0 && (
                      <div className="flex items-center gap-2 text-sm">
                        <Gauge className="h-4 w-4 text-purple-500" />
                        <span className="font-medium">{r.pressure}</span>
                        <span className="text-muted-foreground">hPa</span>
                      </div>
                    )}
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>

        {/* Enhanced Summary stats */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          {/* ... rest of code here ... */}
          <Card className="hover:shadow-md transition-shadow">
            <CardHeader className="flex flex-row items-center justify-between pb-2">
              <CardTitle className="text-sm flex items-center gap-2">
                <DoorOpen className="h-4 w-4 text-green-500" />
                Door Events
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {data.events.doors.length}
              </div>
              <div className="flex gap-4 text-xs mt-2">
                <span className="flex items-center gap-1">
                  <div className="w-2 h-2 rounded-full bg-green-500"></div>
                  {
                    data.events.doors.filter((d) => d.state === "OPEN").length
                  }{" "}
                  open
                </span>
                <span className="flex items-center gap-1">
                  <div className="w-2 h-2 rounded-full bg-red-500"></div>
                  {
                    data.events.doors.filter((d) => d.state === "CLOSED").length
                  }{" "}
                  closed
                </span>
              </div>
            </CardContent>
          </Card>

          <Card className="hover:shadow-md transition-shadow">
            <CardHeader className="flex flex-row items-center justify-between pb-2">
              <CardTitle className="text-sm flex items-center gap-2">
                <Zap className="h-4 w-4 text-blue-500" />
                Appliance Events
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {data.events.appliances.length}
              </div>
              <div className="flex gap-4 text-xs mt-2">
                <span className="flex items-center gap-1">
                  <div className="w-2 h-2 rounded-full bg-blue-500"></div>
                  {
                    data.events.appliances.filter((a) => a.state === "ON")
                      .length
                  }{" "}
                  on
                </span>
                <span className="flex items-center gap-1">
                  <div className="w-2 h-2 rounded-full bg-gray-500"></div>
                  {
                    data.events.appliances.filter((a) => a.state === "OFF")
                      .length
                  }{" "}
                  off
                </span>
              </div>
            </CardContent>
          </Card>

          <Card className="hover:shadow-md transition-shadow">
            <CardHeader className="flex flex-row items-center justify-between pb-2">
              <CardTitle className="text-sm flex items-center gap-2">
                <Router className="h-4 w-4 text-violet-500" />
                WiFi Events
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {data.events.wifi.length}
              </div>
              <div className="flex gap-4 text-xs mt-2">
                <span className="flex items-center gap-1">
                  <div className="w-2 h-2 rounded-full bg-violet-500"></div>
                  {
                    data.events.wifi.filter((w) => w.state === "CONNECTED")
                      .length
                  }{" "}
                  connected
                </span>
                <span className="flex items-center gap-1">
                  <div className="w-2 h-2 rounded-full bg-orange-500"></div>
                  {
                    data.events.wifi.filter((w) => w.state === "DISCONNECTED")
                      .length
                  }{" "}
                  disconnected
                </span>
              </div>
            </CardContent>
          </Card>

          {/* --- Update solar data card to show today's production --- */}
          <Card className="hover:shadow-md transition-shadow">
            <CardHeader className="flex flex-row items-center justify-between pb-2">
              <CardTitle className="text-sm flex items-center gap-2">
                <Sun className="h-4 w-4 text-amber-500" />
                Solar Data
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {data.solar?.current?.todayProductionKwh
                  ? `${data.solar.current.todayProductionKwh.toFixed(1)} kWh`
                  : `${solarData.length} points`}
              </div>
              <div className="flex gap-4 text-xs mt-2">
                <span className="flex items-center gap-1">
                  <div className="w-2 h-2 bg-amber-400"></div>
                  Solar (Wh)
                </span>
                <span className="flex items-center gap-1">
                  <div
                    className="w-2 h-2 bg-amber-500"
                    style={{ borderStyle: "dashed" }}
                  ></div>
                  UV Level
                </span>
              </div>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}

// Loading component
function LoadingDashboard() {
  return (
    <div className="min-h-screen bg-background p-6">
      <div className="mx-auto max-w-7xl space-y-6">
        <div className="text-center space-y-2">
          <h1 className="text-3xl font-bold">Timelin</h1>
        </div>

        <Card className="w-full">
          <CardHeader>
            <CardTitle>Event Flow Timeline</CardTitle>
            <CardDescription>Loading timeline data...</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="h-[500px] flex items-center justify-center">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
            </div>
          </CardContent>
        </Card>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          {[1, 2, 3, 4].map((i) => (
            <Card key={i}>
              <CardHeader>
                <div className="h-4 bg-muted rounded animate-pulse"></div>
              </CardHeader>
              <CardContent>
                <div className="h-8 bg-muted rounded animate-pulse"></div>
              </CardContent>
            </Card>
          ))}
        </div>
      </div>
    </div>
  );
}

export default function Dashboard() {
  return (
    <Suspense fallback={<LoadingDashboard />}>
      <DashboardContent />
    </Suspense>
  );
}
