import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";

import {
  DoorOpen,
  Thermometer,
  Activity,
  Zap,
  Wifi,
  Filter,
  X,
  CheckCircle,
  RefreshCcw,
} from "lucide-react";
import { useState, useMemo, useEffect, useRef } from "react";
import ReactECharts from "echarts-for-react";
import { useFragment } from "react-relay";
import { OVERVIEW_TAB_FRAGMENT } from "../fragments/OverviewTabFragment";
import type { OverviewTabFragment$key } from "../fragments/__generated__/OverviewTabFragment.graphql";
import type {
  Event,
  ChartEvent,
  DeviceInfo,
  DevicesByType,
  VisibleEventTypes,
  TabType,
} from "../types";
import { DeviceFilterSection } from "./DeviceFilterSection";
import { DashboardHeader } from "./DashboardHeader";
import { useXAxisLabelDisplay } from "../hooks/useXAxisLabelDisplay";

// Helper functions
const toMs = (iso: string | number) => new Date(iso).getTime();
const fmtTime = (iso: string | number) =>
  new Date(iso).toLocaleTimeString([], {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });



interface OverviewTabProps {
  fragmentKey: OverviewTabFragment$key;
  selectedHours: number;
  setSelectedHours: (hours: number) => void;
  activeTab: TabType;
  setActiveTab: (tab: TabType) => void;
}

export const OverviewTab = ({ fragmentKey, selectedHours, setSelectedHours, activeTab, setActiveTab }: OverviewTabProps) => {
  const fragmentData = useFragment(OVERVIEW_TAB_FRAGMENT, fragmentKey);
  const xAxisLabelShow = useXAxisLabelDisplay();
  const echartsRef = useRef<any>(null);

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
    
    fragmentData.events.doors.forEach(door => {
      deviceMap.set(door.name, { name: door.name, type: 'door', icon: DoorOpen });
    });
    
    fragmentData.events.appliances.forEach(appliance => {
      deviceMap.set(appliance.name, { name: appliance.name, type: 'appliance', icon: Zap });
    });
    
    fragmentData.events.wifi.forEach(wifi => {
      deviceMap.set(wifi.name, { name: wifi.name, type: 'wifi', icon: Wifi });
    });
    
    return Array.from(deviceMap.values()).sort((a, b) => a.name.localeCompare(b.name));
  }, [fragmentData.events]);

  // Device filter state - initialize with all devices selected
  const [selectedDevices, setSelectedDevices] = useState<string[]>([]);

  // Update selected devices when allDevices changes (only on initial load)
  useEffect(() => {
    if (allDevices.length > 0 && selectedDevices.length === 0) {
      setSelectedDevices(allDevices.map(d => d.name));
    }
  }, [allDevices]);

  // Reset device filter when timescale changes
  useEffect(() => {
    if (allDevices.length > 0) {
      setSelectedDevices(allDevices.map(d => d.name));
    }
  }, [selectedHours, allDevices]);

  // Group devices by type
  const devicesByType = useMemo(() => {
    const grouped: DevicesByType = {
      doors: allDevices.filter(d => d.type === 'door'),
      appliances: allDevices.filter(d => d.type === 'appliance'),
      wifi: allDevices.filter(d => d.type === 'wifi')
    };
    return grouped;
  }, [allDevices]);

  // Process all events for scatterplot
  const allEvents = useMemo(() => {
    const events: ChartEvent[] = [];
    
    // Door events on level 3.0
    fragmentData.events.doors.forEach((e: Event) => {
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
            color: e.state === "OPEN" ? '#cc6666' : '#22c55e',
            type: "door",
            shape: "circle",
          });
        }
      }
    });
    
    // Appliance events on level 2.0
    fragmentData.events.appliances.forEach((e: Event) => {
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
            color: e.state === "ON" ? '#3b82f6' : '#6b7280',
            type: "appliance",
            shape: "triangle",
          });
        }
      }
    });
    
    // WiFi events on level 1.0
    fragmentData.events.wifi.forEach((e: Event) => {
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
            color: e.state === "CONNECTED" ? '#8b5cf6' : '#06b6d4',
            type: "wifi",
            shape: "square",
          });
        }
      }
    });
    
    return events.sort((a, b) => a.x - b.x);
  }, [fragmentData.events, visibleEventTypes, selectedDevices]);

  // Environment data
  const env = useMemo(() => {
    const environment = fragmentData.environment;
    return [
      {
        room: 'Outdoor',
        temperature: environment.outdoor?.temperature ?? 0,
        humidity: environment.outdoor?.humidity ?? 0,
      },
      {
        room: 'Laundry',
        temperature: environment.laundry?.temperature ?? 0,
        humidity: environment.laundry?.humidity ?? 0,
      },
      {
        room: 'Living Room',
        temperature: environment.livingRoom?.temperature ?? 0,
        humidity: environment.livingRoom?.humidity ?? 0,
      },
      {
        room: 'Bedroom',
        temperature: environment.bedroom?.temperature ?? 0,
        humidity: environment.bedroom?.humidity ?? 0,
      },
    ];
  }, [fragmentData.environment]);

  const handleResetMinimap = () => {
    if (echartsRef.current) {
      const chart = echartsRef.current.getEchartsInstance();
      chart.dispatchAction({
        type: 'dataZoom',
        start: 0,
        end: 100,
      });
    }
  };

  return (
    <>
      <DashboardHeader
        title="Timeline"
        activeTab={activeTab}
        setActiveTab={setActiveTab}
        selectedHours={selectedHours}
        setSelectedHours={setSelectedHours}
      />

      {/* Event Timeline Scatterplot - TOP */}
      <Card className="w-full">
        <CardHeader className="flex justify-between items-center pb-0">
          <div>
            <div className="flex items-center gap-2">
              <Activity className="h-5 w-5" />
              <span className="text-lg font-semibold">Event Timeline</span>
            </div>
            <div className="text-sm text-muted-foreground mt-1">
              All events from the last {selectedHours} hours ({allEvents.length} total events)
            </div>
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="sm"
              className="ml-2 flex items-center gap-1"
              title="Reset zoom"
              onClick={handleResetMinimap}
            >
              <RefreshCcw className="w-5 h-5" />
              <span className="hidden sm:inline">Reset zoom</span>
            </Button>
            <Popover>
              <PopoverTrigger asChild>
                <Button variant="outline" className="w-[44px] sm:w-[200px] justify-center sm:justify-start px-0 sm:px-4">
                  <div className="flex items-center gap-2">
                    <Filter className="h-4 w-4" />
                    <span className="hidden sm:inline">{selectedDevices.length === allDevices.length ? "All Devices" : `${selectedDevices.length} devices`}</span>
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
                        title="Select all"
                      >
                        <CheckCircle className="h-4 w-4" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => setSelectedDevices([])}
                        title="Clear all"
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
        <CardContent className="pt-0">
          {/* ECharts Timeline Scatterplot */}
          <div className="w-full h-[400px] flex items-center justify-center px-0 md:px-2">
            <ReactECharts
              ref={echartsRef}
              option={{
                tooltip: {
                  trigger: 'item',
                  backgroundColor: '#fff',
                  borderColor: '#e5e7eb',
                  textStyle: { color: '#111' },
                  formatter: (params: any) => {
                    const [time, y, name, state] = params.value;
                    const color = params.color || (params.data && params.data.itemStyle && params.data.itemStyle.color) || '#888';
                    let stateColor = color;
                    if (state === 'OPEN') stateColor = '#cc6666';
                    if (state === 'CLOSED') stateColor = '#22c55e';
                    if (state === 'ON' || state === 'CONNECTED') stateColor = '#22c55e';
                    if (state === 'OFF') stateColor = '#6b7280';
                    if (state === 'DISCONNECTED') stateColor = '#06b6d4';
                    if (state === 'CONNECTED') stateColor = '#8b5cf6';

                    // Icon SVGs
                    let iconSvg = '';
                    if (y === 3) {
                      iconSvg = `<span style='position:relative;top:-2px;display:inline-block;'><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="${color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 20H2"/><path d="M11 4.562v16.157a1 1 0 0 0 1.242.97L19 20V5.562a2 2 0 0 0-1.515-1.94l-4-1A2 2 0 0 0 11 4.561z"/><path d="M11 4H8a2 2 0 0 0-2 2v14"/><path d="M14 12h.01"/><path d="M22 20h-3"/></svg></span>`;
                    } else if (y === 2) {
                      iconSvg = `<span style='position:relative;top:-2px;display:inline-block;'><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="${color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg></span>`;
                    } else if (y === 1) {
                      iconSvg = `<span style='position:relative;top:-2px;display:inline-block;'><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="${color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 13a10 10 0 0 1 14 0"/><path d="M8.5 16.5a5 5 0 0 1 7 0"/><path d="M12 20h.01"/></svg></span>`;
                    }

                    return `
                      <div style="border-radius:8px;padding:8px;min-width:120px;">
                        <div style="font-size:13px;color:#666;margin-bottom:4px;">Time: <span style="color:#222">${new Date(time).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })}</span></div>
                        <div style="display:flex; align-items:center; gap:8px; margin-bottom:2px;">
                          ${iconSvg}
                          <span style="font-weight:500; color:#222;">${name}</span>
                        </div>
                        <div style="font-size:13px;margin-top:2px;font-weight:500;color:${stateColor}">State: ${state}</div>
                      </div>
                    `;
                  },
                  extraCssText: 'box-shadow: 0 2px 8px rgba(0,0,0,0.08); border-radius: 8px; padding: 8px;'
                },
                legend: { show: false },
                xAxis: {
                  type: 'time',
                  axisLabel: {
                    show: xAxisLabelShow,
                    formatter: (value: number) => new Date(value).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
                    color: '#222',
                    showMaxLabel: true,
                    showMinLabel: true,
                  },
                  splitLine: {
                    show: (params: any) => {
                      const ticks = params?.axis?.scale?.getTicks?.() || [];
                      return params.dataIndex !== 0 && params.dataIndex !== ticks.length - 1;
                    },
                    lineStyle: { color: '#e5e7eb', type: 'dotted' }
                  },
                  axisLine: { show: true, lineStyle: { color: '#e5e7eb' } },
                  axisTick: { show: false },
                  min: allEvents.length > 0 ? Math.min(...allEvents.map(e => e.x)) : undefined,
                  max: allEvents.length > 0 ? Math.max(...allEvents.map(e => e.x)) : undefined,
                },
                yAxis: {
                  type: 'value',
                  min: 0.5,
                  max: 3.5,
                  interval: 1,
                  axisLabel: {
                    formatter: (value: number) => {
                      if (value === 3) return 'Doors';
                      if (value === 2) return 'Appliances';
                      if (value === 1) return 'WiFi';
                      return '';
                    },
                    color: '#222',
                  },
                  splitLine: { show: false },
                  axisLine: { show: false },
                  axisTick: { show: false },
                },
                dataZoom: [
                  {
                    type: 'inside',
                    xAxisIndex: 0,
                  },
                ],
                series: [
                  {
                    name: 'Doors',
                    type: 'scatter',
                    data: allEvents.filter(e => e.type === 'door').map(e => ({
                      value: [e.x, e.y, e.name, e.state],
                      itemStyle: { color: e.color }
                    })),
                    symbolSize: 10,
                  },
                  {
                    name: 'Appliances',
                    type: 'scatter',
                    data: allEvents.filter(e => e.type === 'appliance').map(e => ({
                      value: [e.x, e.y, e.name, e.state],
                      itemStyle: { color: e.color }
                    })),
                    symbolSize: 10,
                  },
                  {
                    name: 'WiFi',
                    type: 'scatter',
                    data: allEvents.filter(e => e.type === 'wifi').map(e => ({
                      value: [e.x, e.y, e.name, e.state],
                      itemStyle: { color: e.color }
                    })),
                    symbolSize: 10,
                  },
                ],
                grid: { left: 24, right: 24, top: 24, bottom: 32 },
              }}
              style={{ width: '100%', height: 380 }}
              opts={{ renderer: 'svg' }}
            />
          </div>
          {/* Legend - Bottom Centered */}
          <div className="flex justify-center mt-0">
            <div className="flex flex-wrap gap-2 p-3 bg-muted/50 rounded-lg">
              {/* Doors */}
              <Button
                variant={visibleEventTypes.doorsOpen ? "default" : "outline"}
                size="sm"
                onClick={() => setVisibleEventTypes(prev => ({ ...prev, doorsOpen: !prev.doorsOpen }))}
                className="text-xs flex items-center gap-1"
                style={{ backgroundColor: visibleEventTypes.doorsOpen ? '#cc6666' : undefined, borderColor: '#cc6666' }}
              >
                <DoorOpen size={20} color={visibleEventTypes.doorsOpen ? '#fff' : '#cc6666'} />
                Doors Open ({fragmentData.events.doors.filter(d => d.state === "OPEN").length})
              </Button>
              <Button
                variant={visibleEventTypes.doorsClosed ? "default" : "outline"}
                size="sm"
                onClick={() => setVisibleEventTypes(prev => ({ ...prev, doorsClosed: !prev.doorsClosed }))}
                className="text-xs flex items-center gap-1"
                style={{ backgroundColor: visibleEventTypes.doorsClosed ? '#22c55e' : undefined, borderColor: '#22c55e' }}
              >
                <DoorOpen size={20} color={visibleEventTypes.doorsClosed ? '#fff' : '#22c55e'} />
                Doors Closed ({fragmentData.events.doors.filter(d => d.state === "CLOSED").length})
              </Button>
              {/* Appliances */}
              <Button
                variant={visibleEventTypes.appliancesOn ? "default" : "outline"}
                size="sm"
                onClick={() => setVisibleEventTypes(prev => ({ ...prev, appliancesOn: !prev.appliancesOn }))}
                className="text-xs flex items-center gap-1"
                style={{ backgroundColor: visibleEventTypes.appliancesOn ? '#3b82f6' : undefined, borderColor: '#3b82f6' }}
              >
                <Zap size={20} color={visibleEventTypes.appliancesOn ? '#fff' : '#3b82f6'} />
                Appliances On ({fragmentData.events.appliances.filter(a => a.state === "ON").length})
              </Button>
              <Button
                variant={visibleEventTypes.appliancesOff ? "default" : "outline"}
                size="sm"
                onClick={() => setVisibleEventTypes(prev => ({ ...prev, appliancesOff: !prev.appliancesOff }))}
                className="text-xs flex items-center gap-1"
                style={{ backgroundColor: visibleEventTypes.appliancesOff ? '#6b7280' : undefined, borderColor: '#6b7280' }}
              >
                <Zap size={20} color={visibleEventTypes.appliancesOff ? '#fff' : '#6b7280'} />
                Appliances Off ({fragmentData.events.appliances.filter(a => a.state === "OFF").length})
              </Button>
              {/* WiFi */}
              <Button
                variant={visibleEventTypes.wifiConnected ? "default" : "outline"}
                size="sm"
                onClick={() => setVisibleEventTypes(prev => ({ ...prev, wifiConnected: !prev.wifiConnected }))}
                className="text-xs flex items-center gap-1"
                style={{ backgroundColor: visibleEventTypes.wifiConnected ? '#8b5cf6' : undefined, borderColor: '#8b5cf6' }}
              >
                <Wifi size={20} color={visibleEventTypes.wifiConnected ? '#fff' : '#8b5cf6'} />
                WiFi Connected ({fragmentData.events.wifi.filter(w => w.state === "CONNECTED").length})
              </Button>
              <Button
                variant={visibleEventTypes.wifiDisconnected ? "default" : "outline"}
                size="sm"
                onClick={() => setVisibleEventTypes(prev => ({ ...prev, wifiDisconnected: !prev.wifiDisconnected }))}
                className="text-xs flex items-center gap-1"
                style={{ backgroundColor: visibleEventTypes.wifiDisconnected ? '#06b6d4' : undefined, borderColor: '#06b6d4' }}
              >
                <Wifi size={20} color={visibleEventTypes.wifiDisconnected ? '#fff' : '#06b6d4'} />
                WiFi Disconnected ({fragmentData.events.wifi.filter(w => w.state === "DISCONNECTED").length})
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Current Temperatures */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Thermometer className="h-5 w-5 text-red-500" />
            Current temperatures
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            {env.map((room) => {
              const tempColor = room.temperature < 20 ? 'text-blue-600' : room.temperature > 25 ? 'text-orange-600' : 'text-green-600';
              return (
                <div key={room.room} className="flex items-center gap-3 p-3 bg-muted/50 rounded-lg">
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
    </>
  );
}; 