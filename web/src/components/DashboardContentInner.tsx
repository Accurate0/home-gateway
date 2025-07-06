import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { ChartContainer, ChartTooltip } from "@/components/ui/chart";
import { Button } from "@/components/ui/button";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";


import {
  ResponsiveContainer,
  Scatter,
  CartesianGrid,
  XAxis,
  YAxis,
  Cell,
  ScatterChart,
} from "recharts";
import {
  DoorOpen,
  Thermometer,
  Activity,
  Power,
  Zap,
  Wifi,
  Monitor,
  Wind,
  Filter,
  X,
  Sun,
  CheckCircle,
} from "lucide-react";
import { usePreloadedQuery } from "react-relay";
import { useState, useMemo, useEffect } from "react";
import type { AppQuery } from "../__generated__/AppQuery.graphql";
import { APP_QUERY } from "../AppQuery";
import type {
  Event,
  EnvironmentData,
  ChartEvent,
  DeviceInfo,
  DevicesByType,
  VisibleEventTypes,
  DashboardContentInnerProps,
  EventTooltipProps,
} from "../types";
import { Badge } from "./ui/badge";
import { DeviceFilterSection } from "./DeviceFilterSection";

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
    connected: "#8b5cf6", // violet-500 (purple)
    disconnected: "#06b6d4", // cyan-500 (teal - distinct from red and blue)
  },
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

export const DashboardContentInner = ({ 
  queryRef, 
  selectedHours, 
  setSelectedHours
}: DashboardContentInnerProps) => {
  // Use the queryRef to get data
  const data = usePreloadedQuery<AppQuery>(APP_QUERY, queryRef);

  // Event type visibility state
  const [visibleEventTypes, setVisibleEventTypes] = useState<VisibleEventTypes>({
    doorsOpen: true,
    doorsClosed: true,
    appliancesOn: true,
    appliancesOff: true,
    wifiConnected: true,
    wifiDisconnected: true,
  });

  // Create deduplicated device list with icons
  const allDevices = useMemo(() => {
    const deviceMap = new Map<string, DeviceInfo>();
    
    data.events.doors.forEach(door => {
      deviceMap.set(door.name, { name: door.name, type: 'door', icon: DoorOpen });
    });
    
    data.events.appliances.forEach(appliance => {
      deviceMap.set(appliance.name, { name: appliance.name, type: 'appliance', icon: Zap });
    });
    
    data.events.wifi.forEach(wifi => {
      deviceMap.set(wifi.name, { name: wifi.name, type: 'wifi', icon: Wifi });
    });
    
    return Array.from(deviceMap.values()).sort((a, b) => a.name.localeCompare(b.name));
  }, [data.events]);

  // Device filter state - initialize with all devices selected
  const [selectedDevices, setSelectedDevices] = useState<string[]>([]);

  // Update selected devices when allDevices changes (only on initial load)
  useEffect(() => {
    if (allDevices.length > 0 && selectedDevices.length === 0) {
      setSelectedDevices(allDevices.map(d => d.name));
    }
  }, [allDevices]); // Remove selectedDevices.length from dependencies

  // Group devices by type
  const devicesByType = useMemo(() => {
    const grouped: DevicesByType = {
      doors: allDevices.filter(d => d.type === 'door'),
      appliances: allDevices.filter(d => d.type === 'appliance'),
      wifi: allDevices.filter(d => d.type === 'wifi')
    };
    return grouped;
  }, [allDevices]);

  // ---------- helpers ----------
  const toMs = (iso: string | number) => new Date(iso).getTime();
  const fmtTime = (iso: string | number) =>
    new Date(iso).toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });



  // Process all events for scatterplot
  const allEvents = useMemo(() => {
    const events: ChartEvent[] = [];
    
    // Door events on level 3.0
    data.events.doors.forEach((e: Event) => {
      if ((e.state === "OPEN" && visibleEventTypes.doorsOpen) || 
          (e.state === "CLOSED" && visibleEventTypes.doorsClosed)) {
        if (selectedDevices.includes(e.name)) {
          events.push({
            x: toMs(e.time),
            y: 3.0,
            time: fmtTime(e.time),
            name: e.name,
            state: e.state,
            id: e.id,
            color: e.state === "OPEN" ? chartConfig.doors.open : chartConfig.doors.closed,
            type: "door",
            shape: "circle",
          });
        }
      }
    });
    
    // Appliance events on level 2.0
    data.events.appliances.forEach((e: Event) => {
      if ((e.state === "ON" && visibleEventTypes.appliancesOn) || 
          (e.state === "OFF" && visibleEventTypes.appliancesOff)) {
        if (selectedDevices.includes(e.name)) {
          events.push({
            x: toMs(e.time),
            y: 2.0,
            time: fmtTime(e.time),
            name: e.name,
            state: e.state,
            id: e.id,
            color: e.state === "ON" ? chartConfig.appliances.on : chartConfig.appliances.off,
            type: "appliance",
            shape: "triangle",
          });
        }
      }
    });
    
    // WiFi events on level 1.0
    data.events.wifi.forEach((e: Event) => {
      if ((e.state === "CONNECTED" && visibleEventTypes.wifiConnected) || 
          (e.state === "DISCONNECTED" && visibleEventTypes.wifiDisconnected)) {
        if (selectedDevices.includes(e.name)) {
          events.push({
            x: toMs(e.time),
            y: 1.0,
            time: fmtTime(e.time),
            name: e.name,
            state: e.state,
            id: e.id,
            color: e.state === "CONNECTED" ? chartConfig.wifi.connected : chartConfig.wifi.disconnected,
            type: "wifi",
            shape: "square",
          });
        }
      }
    });
    
    return events.sort((a, b) => a.x - b.x);
  }, [data.events, visibleEventTypes, selectedDevices]);

  // Calculate explicit domain for X-axis
  const xDomain = useMemo(() => {
    if (allEvents.length === 0) {
      const now = Date.now();
      return [now - selectedHours * 60 * 60 * 1000, now];
    }
    const minTime = Math.min(...allEvents.map(e => e.x));
    const maxTime = Math.max(...allEvents.map(e => e.x));
    const padding = Math.max((maxTime - minTime) * 0.1, 60 * 60 * 1000); // 10% or 1 hour
    return [minTime - padding, maxTime + padding];
  }, [allEvents, selectedHours]);

  // Environment data
  const env = useMemo(() => {
    return Object.entries(data.environment).map(([key, val]: [string, EnvironmentData]) => ({
      room: key.replace(/([A-Z])/g, " $1").replace(/^./, s => s.toUpperCase()),
      temperature: val?.temperature ?? 0,
      humidity: val?.humidity ?? 0,
      pressure: val?.pressure ?? 0,
    }));
  }, [data.environment]);

  // Custom tooltip for events
  const EventTooltip = ({ active, payload }: EventTooltipProps) => {
    if (!active || !payload?.length) return null;

    // Filter out duplicate entries (x and y coordinates for same event)
    const uniqueEvents = payload
      .filter((entry) => entry.payload && entry.payload.id)
      .reduce((acc: ChartEvent[], entry) => {
        const event = entry.payload;
        const existing = acc.find(e => e.id === event.id);
        if (!existing) {
          acc.push(event);
        }
        return acc;
      }, []);

    // Use the first event's time for the tooltip header
    const timeLabel = uniqueEvents.length > 0 
      ? uniqueEvents[0].time 
      : new Date().toLocaleTimeString([], {
          hour: "2-digit",
          minute: "2-digit",
          second: "2-digit",
        });

    return (
      <div className="rounded-md border bg-background p-3 shadow-lg">
        <p className="text-sm text-muted-foreground mb-2">Time: {timeLabel}</p>
        {uniqueEvents.map((event, idx) => (
          <div key={`event-${event.id}-${idx}`} className="flex items-center gap-2 mb-1">
            <div
              className="w-3 h-3 rounded-full"
              style={{ backgroundColor: event.color }}
            />
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
        ))}
      </div>
    );
  };

  return (
    <div className="min-h-screen bg-background p-6">
      <div className="mx-auto max-w-7xl space-y-6">
        <div className="flex justify-between items-center">
          <div className="text-center space-y-2">
            <h1 className="text-3xl font-bold">Timeline</h1>
          </div>
          <div className="flex items-center gap-2">
            <Popover>
              <PopoverTrigger asChild>
                <Button variant="outline" className="w-24">
                  {selectedHours}h
                </Button>
              </PopoverTrigger>
              <PopoverContent className="w-24">
                <div className="space-y-1">
                  {[1, 3, 6, 12, 24].map((hours) => (
                    <Button
                      key={hours}
                      variant={selectedHours === hours ? "default" : "ghost"}
                      className="w-full justify-start"
                      onClick={() => setSelectedHours(hours)}
                    >
                      {hours}h
                    </Button>
                  ))}
                </div>
              </PopoverContent>
            </Popover>
          </div>
        </div>

        {/* Event Timeline Scatterplot - TOP */}
        <Card className="w-full">
          <CardHeader className="flex justify-between items-center">
            <CardTitle className="flex items-center gap-2">
              <Activity className="h-5 w-5" />
              Event Timeline
            </CardTitle>
            <div className="flex items-center gap-2">
              <Popover>
                <PopoverTrigger asChild>
                  <Button variant="outline" className="w-[200px] justify-start">
                    <div className="flex items-center gap-2">
                      <Filter className="h-4 w-4" />
                      {selectedDevices.length === allDevices.length ? "All Devices" : `${selectedDevices.length} devices`}
                    </div>
                  </Button>
                </PopoverTrigger>
                <PopoverContent className="w-[300px]">
                  <div className="space-y-6">
                    <div className="flex items-center justify-between">
                      <h4 className="font-medium">Filter by Device</h4>
                      <div className="flex gap-1">
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => setSelectedDevices(allDevices.map(d => d.name))}
                          title="Select All"
                        >
                          <CheckCircle className="h-4 w-4" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => setSelectedDevices([])}
                          title="Clear All"
                        >
                          <X className="h-4 w-4" />
                        </Button>
                      </div>
                    </div>
                    
                    <DeviceFilterSection
                      devices={devicesByType.doors}
                      selectedDevices={selectedDevices}
                      setSelectedDevices={setSelectedDevices}
                      title="Doors"
                      icon={DoorOpen}
                      iconColor="text-green-500"
                      showBorder={true}
                    />

                    <DeviceFilterSection
                      devices={devicesByType.appliances}
                      selectedDevices={selectedDevices}
                      setSelectedDevices={setSelectedDevices}
                      title="Appliances"
                      icon={Zap}
                      iconColor="text-blue-500"
                      showBorder={true}
                    />

                    <DeviceFilterSection
                      devices={devicesByType.wifi}
                      selectedDevices={selectedDevices}
                      setSelectedDevices={setSelectedDevices}
                      title="WiFi"
                      icon={Wifi}
                      iconColor="text-violet-500"
                      showBorder={false}
                    />
                  </div>
                </PopoverContent>
              </Popover>
            </div>
          </CardHeader>
          <CardContent>
            <div className="mb-2 text-sm text-muted-foreground">
              All events from the last {selectedHours} hours ({allEvents.length} total events)
            </div>
            
            <ChartContainer
              config={chartConfig}
              className="w-full h-[500px]"
            >
              <ResponsiveContainer width="100%" height="100%">
                <ScatterChart
                  width={100}
                  height={100}
                  margin={{ top: 20, right: 30, left: 10, bottom: 20 }}
                >
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis
                    dataKey="x"
                    type="number"
                    scale="time"
                    domain={xDomain}
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
                  <YAxis yAxisId="left" domain={[0, 3.5]} hide={true} />
                  <ChartTooltip content={<EventTooltip />} />

                  {/* All events */}
                  <Scatter
                    yAxisId="left"
                    name="All Events"
                    data={allEvents}
                    shape="circle"
                    dataKey="y"
                  >
                    {allEvents.map((event, i) => (
                      <Cell key={i} fill={event.color} />
                    ))}
                  </Scatter>
                </ScatterChart>
              </ResponsiveContainer>
            </ChartContainer>
            
            {/* Legend - Bottom Centered */}
            <div className="flex justify-center mt-2">
              <div className="flex flex-wrap gap-2 p-3 bg-muted/50 rounded-lg">
                {/* Doors */}
                <Button
                  variant={visibleEventTypes.doorsOpen ? "default" : "outline"}
                  size="sm"
                  onClick={() => setVisibleEventTypes(prev => ({ ...prev, doorsOpen: !prev.doorsOpen }))}
                  className="text-xs"
                  style={{ backgroundColor: visibleEventTypes.doorsOpen ? '#22c55e' : undefined, borderColor: '#22c55e' }}
                >
                  Doors Open ({data.events.doors.filter(d => d.state === "OPEN").length})
                </Button>
                
                <Button
                  variant={visibleEventTypes.doorsClosed ? "default" : "outline"}
                  size="sm"
                  onClick={() => setVisibleEventTypes(prev => ({ ...prev, doorsClosed: !prev.doorsClosed }))}
                  className="text-xs"
                  style={{ backgroundColor: visibleEventTypes.doorsClosed ? '#ef4444' : undefined, borderColor: '#ef4444' }}
                >
                  Doors Closed ({data.events.doors.filter(d => d.state === "CLOSED").length})
                </Button>
                
                {/* Appliances */}
                <Button
                  variant={visibleEventTypes.appliancesOn ? "default" : "outline"}
                  size="sm"
                  onClick={() => setVisibleEventTypes(prev => ({ ...prev, appliancesOn: !prev.appliancesOn }))}
                  className="text-xs"
                  style={{ backgroundColor: visibleEventTypes.appliancesOn ? '#3b82f6' : undefined, borderColor: '#3b82f6' }}
                >
                  Appliances On ({data.events.appliances.filter(a => a.state === "ON").length})
                </Button>
                
                <Button
                  variant={visibleEventTypes.appliancesOff ? "default" : "outline"}
                  size="sm"
                  onClick={() => setVisibleEventTypes(prev => ({ ...prev, appliancesOff: !prev.appliancesOff }))}
                  className="text-xs"
                  style={{ backgroundColor: visibleEventTypes.appliancesOff ? '#6b7280' : undefined, borderColor: '#6b7280' }}
                >
                  Appliances Off ({data.events.appliances.filter(a => a.state === "OFF").length})
                </Button>
                
                {/* WiFi */}
                <Button
                  variant={visibleEventTypes.wifiConnected ? "default" : "outline"}
                  size="sm"
                  onClick={() => setVisibleEventTypes(prev => ({ ...prev, wifiConnected: !prev.wifiConnected }))}
                  className="text-xs"
                  style={{ backgroundColor: visibleEventTypes.wifiConnected ? '#8b5cf6' : undefined, borderColor: '#8b5cf6' }}
                >
                  WiFi Connected ({data.events.wifi.filter(w => w.state === "CONNECTED").length})
                </Button>
                
                <Button
                  variant={visibleEventTypes.wifiDisconnected ? "default" : "outline"}
                  size="sm"
                  onClick={() => setVisibleEventTypes(prev => ({ ...prev, wifiDisconnected: !prev.wifiDisconnected }))}
                  className="text-xs"
                  style={{ backgroundColor: visibleEventTypes.wifiDisconnected ? '#06b6d4' : undefined, borderColor: '#06b6d4' }}
                >
                  WiFi Disconnected ({data.events.wifi.filter(w => w.state === "DISCONNECTED").length})
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Event Summary Cards */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          {/* Doors Summary */}
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="flex items-center gap-2 text-lg">
                <DoorOpen className="h-5 w-5 text-green-500" />
                Doors
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                <div className="flex justify-between items-center">
                  <span className="text-sm text-muted-foreground">Open</span>
                  <Badge variant="secondary" className="bg-green-100 text-green-800">
                    {data.events.doors.filter(d => d.state === "OPEN").length}
                  </Badge>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm text-muted-foreground">Closed</span>
                  <Badge variant="secondary" className="bg-red-100 text-red-800">
                    {data.events.doors.filter(d => d.state === "CLOSED").length}
                  </Badge>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Appliances Summary */}
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="flex items-center gap-2 text-lg">
                <Zap className="h-5 w-5 text-blue-500" />
                Appliances
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                <div className="flex justify-between items-center">
                  <span className="text-sm text-muted-foreground">On</span>
                  <Badge variant="secondary" className="bg-blue-100 text-blue-800">
                    {data.events.appliances.filter(a => a.state === "ON").length}
                  </Badge>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm text-muted-foreground">Off</span>
                  <Badge variant="secondary" className="bg-gray-100 text-gray-800">
                    {data.events.appliances.filter(a => a.state === "OFF").length}
                  </Badge>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* WiFi Summary */}
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="flex items-center gap-2 text-lg">
                <Wifi className="h-5 w-5 text-violet-500" />
                WiFi
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                <div className="flex justify-between items-center">
                  <span className="text-sm text-muted-foreground">Connected</span>
                  <Badge variant="secondary" className="bg-violet-100 text-violet-800">
                    {data.events.wifi.filter(w => w.state === "CONNECTED").length}
                  </Badge>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm text-muted-foreground">Disconnected</span>
                  <Badge variant="secondary" className="bg-cyan-100 text-cyan-800">
                    {data.events.wifi.filter(w => w.state === "DISCONNECTED").length}
                  </Badge>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Solar Summary */}
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="flex items-center gap-2 text-lg">
                <Sun className="h-5 w-5 text-amber-500" />
                Solar
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                <div className="flex justify-between items-center">
                  <span className="text-sm text-muted-foreground">Today</span>
                  <Badge variant="secondary" className="bg-amber-100 text-amber-800">
                    {data.solar?.current?.todayProductionKwh?.toFixed(1) ?? "0.0"} kWh
                  </Badge>
                </div>
                <div className="flex justify-between items-center">
                  <span className="text-sm text-muted-foreground">Yesterday</span>
                  <Badge variant="secondary" className="bg-amber-100 text-amber-800">
                    {data.solar?.current?.yesterdayProductionKwh?.toFixed(1) ?? "0.0"} kWh
                  </Badge>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Current Temperatures */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Thermometer className="h-5 w-5 text-red-500" />
              Current Temperatures
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
              {env.map((room) => {
                const tempColor = room.temperature < 20 ? 'text-blue-600' : room.temperature > 25 ? 'text-red-600' : 'text-green-600';
                return (
                  <div key={room.room} className="flex items-center gap-3 p-3 bg-muted/50 rounded-lg">
                    {getRoomIcon(room.room)}
                    <div>
                      <p className="font-medium">{room.room}</p>
                      <p className={`text-2xl font-bold ${tempColor}`}>
                        {room.temperature.toFixed(1)}°C
                      </p>
                      <p className="text-sm text-muted-foreground">
                        {room.humidity.toFixed(0)}% humidity
                      </p>
                    </div>
                  </div>
                );
              })}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}; 