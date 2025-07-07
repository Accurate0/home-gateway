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
  Sun,
} from "lucide-react";
import { usePreloadedQuery } from "react-relay";
import { useState, useMemo, useEffect, useRef } from "react";
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
} from "../types";
import { DeviceFilterSection } from "./DeviceFilterSection";
import ReactECharts from "echarts-for-react";
  // Restore helpers needed for event processing
  const toMs = (iso: string | number) => new Date(iso).getTime();
  const fmtTime = (iso: string | number) =>
    new Date(iso).toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
    
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
            color: e.state === "OPEN" ? '#cc6666' : '#22c55e',
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
            color: e.state === "ON" ? '#3b82f6' : '#6b7280',
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
            color: e.state === "CONNECTED" ? '#8b5cf6' : '#06b6d4',
            type: "wifi",
            shape: "square",
          });
        }
      }
    });
    
    return events.sort((a, b) => a.x - b.x);
  }, [data.events, visibleEventTypes, selectedDevices]);

  // Environment data
  const env = useMemo(() => {
    return Object.entries(data.environment).map(([key, val]: [string, EnvironmentData]) => ({
      room: key.replace(/([A-Z])/g, " $1").replace(/^./, s => s.toUpperCase()),
      temperature: val?.temperature ?? 0,
      humidity: val?.humidity ?? 0,
      pressure: val?.pressure ?? 0,
    }));
  }, [data.environment]);

  // Ref for ECharts instance
  const echartsRef = useRef<any>(null);

  // Handler to reset minimap/dataZoom
  const handleResetMinimap = () => {
    const echartsInstance = echartsRef.current?.getEchartsInstance?.();
    if (echartsInstance) {
      echartsInstance.dispatchAction({
        type: 'dataZoom',
        // Reset both dataZooms (slider and inside)
        start: 0,
        end: 100,
      });
    }
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
                <span>Reset zoom</span>
              </Button>
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
            <div className="w-full h-[500px] flex items-center justify-center px-0 md:px-2">
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
                        // Provided SVG for Door Open icon (with color and -2px offset)
                        iconSvg = `<span style='position:relative;top:-2px;display:inline-block;'><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="${color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M11 20H2"/><path d="M11 4.562v16.157a1 1 0 0 0 1.242.97L19 20V5.562a2 2 0 0 0-1.515-1.94l-4-1A2 2 0 0 0 11 4.561z"/><path d="M11 4H8a2 2 0 0 0-2 2v14"/><path d="M14 12h.01"/><path d="M22 20h-3"/></svg></span>`;
                      } else if (y === 2) {
                        // Zap icon
                        iconSvg = `<span style='position:relative;top:-2px;display:inline-block;'><svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="${color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg></span>`;
                      } else if (y === 1) {
                        // Wifi icon
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
                    // name: 'Time', // Remove label
                    axisLabel: {
                      formatter: (value: number) => new Date(value).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
                      color: '#222',
                      interval: 'auto',
                    },
                    splitLine: {
                      show: (params: any) => {
                        // params: { dataIndex, axis }
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
                style={{ width: '100%', height: 480 }}
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
                  Doors Open ({data.events.doors.filter(d => d.state === "OPEN").length})
                </Button>
                <Button
                  variant={visibleEventTypes.doorsClosed ? "default" : "outline"}
                  size="sm"
                  onClick={() => setVisibleEventTypes(prev => ({ ...prev, doorsClosed: !prev.doorsClosed }))}
                  className="text-xs flex items-center gap-1"
                  style={{ backgroundColor: visibleEventTypes.doorsClosed ? '#22c55e' : undefined, borderColor: '#22c55e' }}
                >
                  <DoorOpen size={20} color={visibleEventTypes.doorsClosed ? '#fff' : '#22c55e'} />
                  Doors Closed ({data.events.doors.filter(d => d.state === "CLOSED").length})
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
                  Appliances On ({data.events.appliances.filter(a => a.state === "ON").length})
                </Button>
                <Button
                  variant={visibleEventTypes.appliancesOff ? "default" : "outline"}
                  size="sm"
                  onClick={() => setVisibleEventTypes(prev => ({ ...prev, appliancesOff: !prev.appliancesOff }))}
                  className="text-xs flex items-center gap-1"
                  style={{ backgroundColor: visibleEventTypes.appliancesOff ? '#6b7280' : undefined, borderColor: '#6b7280' }}
                >
                  <Zap size={20} color={visibleEventTypes.appliancesOff ? '#fff' : '#6b7280'} />
                  Appliances Off ({data.events.appliances.filter(a => a.state === "OFF").length})
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
                  WiFi Connected ({data.events.wifi.filter(w => w.state === "CONNECTED").length})
                </Button>
                <Button
                  variant={visibleEventTypes.wifiDisconnected ? "default" : "outline"}
                  size="sm"
                  onClick={() => setVisibleEventTypes(prev => ({ ...prev, wifiDisconnected: !prev.wifiDisconnected }))}
                  className="text-xs flex items-center gap-1"
                  style={{ backgroundColor: visibleEventTypes.wifiDisconnected ? '#06b6d4' : undefined, borderColor: '#06b6d4' }}
                >
                  <Wifi size={20} color={visibleEventTypes.wifiDisconnected ? '#fff' : '#06b6d4'} />
                  WiFi Disconnected ({data.events.wifi.filter(w => w.state === "DISCONNECTED").length})
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* Current Temperatures */}
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Sun className="h-5 w-5 text-yellow-500" />
              Solar
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 md:grid-cols-[220px_1fr] gap-4 h-full min-h-[180px]">
              {/* Left: Info, styled with bg-muted/50 */}
              <div className="flex flex-col justify-center p-4 rounded-lg bg-muted/50 h-full">
                {data.solar?.history && data.solar.history.length > 0 ? (() => {
                  const history = data.solar.history;
                  const latest = history[history.length - 1];
                  const todayKwh = data.solar.current?.todayProductionKwh ?? 0;
                  const yesterdayKwh = data.solar.current?.yesterdayProductionKwh ?? 0;
                  return (
                    <div className="space-y-1">
                      <div>
                        <span className="text-xs text-muted-foreground">Current</span>
                        <p className="text-2xl font-bold text-yellow-600">
                          {(latest.wh / 1000).toFixed(2)} kWh
                        </p>
                      </div>
                      <div>
                        <span className="text-xs text-muted-foreground">Today</span>
                        <p className="text-lg font-semibold text-yellow-700">
                          {todayKwh.toFixed(2)} kWh
                        </p>
                      </div>
                      <div>
                        <span className="text-xs text-muted-foreground">Yesterday</span>
                        <p className="text-lg font-semibold text-yellow-700">
                          {yesterdayKwh.toFixed(2)} kWh
                        </p>
                      </div>
                    </div>
                  );
                })() : (
                  <div className="text-muted-foreground">No data</div>
                )}
              </div>
              {/* Right: Solar Chart (same as before) */}
              <div className="flex items-center h-full min-h-[188px]">
                <ReactECharts
                  option={{
                    tooltip: {
                      trigger: 'axis',
                      backgroundColor: '#fff',
                      borderColor: '#e5e7eb',
                      textStyle: { color: '#111' },
                      formatter: (params: any) => {
                        if (!params || params.length === 0) return '';
                        let kwh: number | null = null, uv: number | null = null, time: number | null = null;
                        params.forEach((p: any) => {
                          if (p.seriesName === 'Solar') {
                            kwh = p.value[1];
                            time = p.value[0];
                          }
                          if (p.seriesName === 'UV Level') {
                            uv = p.value[1];
                            time = p.value[0];
                          }
                        });
                        return `<div style='padding:6px 10px;'>${time ? `<span style="color:#888;">${new Date(time).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span><br/>` : ''}${kwh !== null ? `<b>${Number(kwh).toFixed(2)} kWh</b><br/>` : ''}${uv !== null ? `<span style='color:#a21caf;'>UV: ${Number(uv).toFixed(1)}</span>` : ''}</div>`;
                      },
                      extraCssText: 'box-shadow: 0 2px 8px rgba(0,0,0,0.08); border-radius: 8px; padding: 8px;'
                    },
                    grid: { left: 16, right: 16, top: 4, bottom: 32 },
                    xAxis: {
                      type: 'time',
                      axisLabel: {
                        formatter: (value: number) => new Date(value).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
                        color: '#222',
                      },
                      splitLine: { show: true, lineStyle: { color: '#e5e7eb', type: 'dotted' } },
                      axisLine: { show: true, lineStyle: { color: '#e5e7eb' } },
                      axisTick: { show: false },
                    },
                    yAxis: [
                      {
                        type: 'value',
                        min: 0,
                        axisLabel: { show: false },
                        splitLine: { show: false },
                        axisLine: { show: false },
                        axisTick: { show: false },
                      },
                      {
                        type: 'value',
                        min: 0,
                        max: 12,
                        position: 'right',
                        axisLabel: { show: false },
                        splitLine: { show: false },
                        axisLine: { show: false },
                        axisTick: { show: false },
                      },
                    ],
                    series: [
                      {
                        name: 'Solar',
                        type: 'line',
                        data: (data.solar?.history ?? [])
                          .filter(h => new Date(h.timestamp).getTime() >= Date.now() - selectedHours * 60 * 60 * 1000)
                          .map(h => [new Date(h.timestamp).getTime(), h.wh / 1000]),
                        symbol: 'none',
                        lineStyle: { color: '#f59e42', width: 2 },
                        itemStyle: { color: '#f59e42' },
                        areaStyle: { color: 'rgba(245,158,66,0.08)' },
                        emphasis: { focus: 'series' },
                        z: 1,
                        yAxisIndex: 0,
                      },
                      {
                        name: 'UV Level',
                        type: 'line',
                        data: (data.solar?.history ?? [])
                          .filter(h => new Date(h.timestamp).getTime() >= Date.now() - selectedHours * 60 * 60 * 1000 && h.uvLevel != null)
                          .map(h => [new Date(h.timestamp).getTime(), h.uvLevel]),
                        symbol: 'none',
                        lineStyle: { color: '#a21caf', width: 2, type: 'dashed' },
                        itemStyle: { color: '#a21caf' },
                        emphasis: { focus: 'series' },
                        z: 2,
                        yAxisIndex: 1,
                      },
                    ],
                  }}
                  style={{ width: '100%', height: '100%' }}
                  opts={{ renderer: 'svg' }}
                />
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
                    {/* Optionally add a static icon or leave blank for now */}
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