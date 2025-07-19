import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { DashboardHeader } from "./DashboardHeader";

import {
  Sun,
  Zap,
} from "lucide-react";
import { useState, useMemo } from "react";
import ReactECharts from "echarts-for-react";
import { useFragment } from "react-relay";
import { SOLAR_ENERGY_TAB_FRAGMENT } from "../fragments/SolarEnergyTabFragment";
import { useXAxisLabelDisplay } from "../hooks/useXAxisLabelDisplay";
import type { SolarEnergyTabFragment$key } from "../fragments/__generated__/SolarEnergyTabFragment.graphql";
import type { TabType } from "../types";



interface SolarEnergyTabProps {
  fragmentKey: SolarEnergyTabFragment$key;
  selectedHours: number;
  setSelectedHours: (hours: number) => void;
  activeTab: TabType;
  setActiveTab: (tab: TabType) => void;
}

export const SolarEnergyTab = ({ 
  fragmentKey, 
  selectedHours, 
  setSelectedHours, 
  activeTab,
  setActiveTab
}: SolarEnergyTabProps) => {
  const fragmentData = useFragment(SOLAR_ENERGY_TAB_FRAGMENT, fragmentKey);
  const xAxisLabelShow = useXAxisLabelDisplay();

  // Energy chart type state
  const [energyChartType, setEnergyChartType] = useState<'line' | 'bar'>('bar');

  // Energy history data
  const energyHistory = useMemo(() => {
    return fragmentData.energy?.history ?? [];
  }, [fragmentData.energy?.history]);

  // Bar chart data (daily aggregation)
  const barData = useMemo(() => {
    const dailyMap = new Map<string, { used: number; solarExported: number }>();
    
    energyHistory.forEach(h => {
      const date = new Date(h.time).toLocaleDateString();
      const existing = dailyMap.get(date) || { used: 0, solarExported: 0 };
      dailyMap.set(date, {
        used: existing.used + h.used,
        solarExported: existing.solarExported + h.solarExported
      });
    });
    
    return Array.from(dailyMap.entries()).map(([day, data]) => ({
      day,
      used: data.used,
      solarExported: data.solarExported
    }));
  }, [energyHistory]);

  // Calculate totals for display
  const totalCost = useMemo(() => {
    const costPerKwh = 0.323719;
    return energyHistory.reduce((sum, h) => sum + h.used * costPerKwh, 0);
  }, [energyHistory]);

  const totalSolarValue = useMemo(() => {
    function getSolarValue(h: { time: string; solarExported: number }) {
      const perthTime = new Date(h.time);
      const perthHour = perthTime.getUTCHours() + 8;
      const adjustedHour = perthHour >= 24 ? perthHour - 24 : perthHour;
      const isPeak = adjustedHour >= 15 && adjustedHour < 21;
      const solarRate = isPeak ? 0.10 : 0.02;
      return h.solarExported * solarRate;
    }
    return energyHistory.reduce((sum, h) => sum + getSolarValue(h), 0);
  }, [energyHistory]);

  return (
    <>
      <DashboardHeader
        title="Solar & Energy"
        activeTab={activeTab}
        setActiveTab={setActiveTab}
        selectedHours={selectedHours}
        setSelectedHours={setSelectedHours}
      />

      {/* Solar Card */}
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
              {fragmentData.solar?.history && fragmentData.solar.history.length > 0 ? (() => {
                const history = fragmentData.solar.history;
                const latest = history[history.length - 1];
                const todayKwh = fragmentData.solar.current?.todayProductionKwh ?? 0;
                const yesterdayKwh = fragmentData.solar.current?.yesterdayProductionKwh ?? 0;
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
            {/* Right: Solar Chart */}
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
                      return `<div style='padding:6px 10px;'>${time ? `<span style="color:#888;">${new Date(time).toLocaleDateString() + ' ' + new Date(time).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span><br/>` : ''}${kwh !== null ? `<b>${Number(kwh).toFixed(2)} kWh</b><br/>` : ''}${uv !== null ? `<span style='color:#a21caf;'>UV: ${Number(uv).toFixed(1)}</span>` : ''}</div>`;
                    },
                    extraCssText: 'box-shadow: 0 2px 8px rgba(0,0,0,0.08); border-radius: 8px; padding: 8px;'
                  },
                  grid: { left: 16, right: 16, top: 4, bottom: 32 },
                  xAxis: {
                    type: 'time',
                    axisLabel: {
                      show: xAxisLabelShow,
                      formatter: (value: number) => new Date(value).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }),
                      color: '#222',
                      showMaxLabel: true,
                      showMinLabel: true,
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
                      data: (fragmentData.solar?.history ?? [])
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
                      data: (fragmentData.solar?.history ?? [])
                        .filter(h => h.uvLevel != null)
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

      {/* Energy Usage Card */}
      <Card>
        <CardHeader>
          <div className="flex justify-between items-center">
            <CardTitle className="flex items-center gap-2">
              <Zap className="h-5 w-5 text-blue-500" />
              Energy Usage
            </CardTitle>
            <div className="flex items-center gap-2">
              <div className="flex gap-2">
                <Button
                  variant={energyChartType === 'line' ? 'default' : 'outline'}
                  size="sm"
                  onClick={() => setEnergyChartType('line')}
                  className="text-xs"
                >
                  Line
                </Button>
                <Button
                  variant={energyChartType === 'bar' ? 'default' : 'outline'}
                  size="sm"
                  onClick={() => setEnergyChartType('bar')}
                  className="text-xs"
                >
                  Bar (daily)
                </Button>
              </div>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 md:grid-cols-[220px_1fr] gap-4 h-full min-h-[180px]">
            {/* Left: Info, styled with bg-muted/50 */}
            <div className="flex flex-col justify-center p-4 rounded-lg bg-muted/50 h-full">
              <div className="space-y-1">
                <div>
                  <span className="text-xs text-muted-foreground">Total Energy Cost</span>
                  <p className="text-2xl font-bold text-red-600">
                    ${totalCost.toFixed(2)}
                  </p>
                </div>
                <div>
                  <span className="text-xs text-muted-foreground">Total Solar Value</span>
                  <p className="text-lg font-semibold text-green-700">
                    ${totalSolarValue.toFixed(2)}
                  </p>
                </div>
              </div>
            </div>
            {/* Right: Chart */}
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
                      let used: number | null = null, solar: number | null = null, time: string | null = null;
                      params.forEach((p: any) => {
                        if (p.seriesName === 'Used') {
                          used = p.value[1];
                          time = p.value[0];
                        }
                        if (p.seriesName === 'Solar Exported') {
                          solar = p.value[1];
                          time = p.value[0];
                        }
                      });
                      const costPerKwh = 0.323719;
                      const totalCost = (used || 0) * costPerKwh;
                      // Calculate solar value using Perth time-of-use rates
                      let solarValue = 0;
                      if (solar && time) {
                        const perthTime = new Date(time);
                        const perthHour = perthTime.getUTCHours() + 8;
                        const adjustedHour = perthHour >= 24 ? perthHour - 24 : perthHour;
                        const isPeak = adjustedHour >= 15 && adjustedHour < 21;
                        const solarRate = isPeak ? 0.10 : 0.02;
                        solarValue = solar * solarRate;
                      }
                      return `<div style='padding:6px 10px;'>${time ? `<span style="color:#888;">${energyChartType === 'bar' ? time : new Date(time).toLocaleDateString() + ' ' + new Date(time).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}</span><br/>` : ''}${used !== null ? `<b>Used: ${(Number(used)).toFixed(2)} kWh</b><br/>` : ''}${solar !== null ? `<span style='color:#22c55e;'>Solar Exported: ${(Number(solar)).toFixed(2)} kWh</span><br/>` : ''}<span style='color:#dc2626;font-weight:500;'>Energy Cost: $${totalCost.toFixed(2)}</span>${solar !== null ? `<br/><span style='color:#16a34a;font-weight:500;'>Solar Value: $${solarValue.toFixed(2)}</span>` : ''}</div>`;
                    },
                    extraCssText: 'box-shadow: 0 2px 8px rgba(0,0,0,0.08); border-radius: 8px; padding: 8px;'
                  },
                  grid: { left: 16, right: 16, top: 8, bottom: 32 },
                  xAxis: {
                    type: energyChartType === 'bar' ? 'category' : 'time',
                    axisLabel: {
                      show: true,
                      formatter: (value: number | string) => energyChartType === 'bar' ? value : new Date(value).toLocaleDateString(),
                      color: '#222',
                      showMaxLabel: true,
                      showMinLabel: true,
                    },
                    splitLine: { show: true, lineStyle: { color: '#e5e7eb', type: 'dotted' } },
                    axisLine: { show: true, lineStyle: { color: '#e5e7eb' } },
                    axisTick: { show: false },
                  },
                  yAxis: {
                    type: 'value',
                    min: 0,
                    minInterval: 1,
                    axisLabel: { show: false },
                    splitLine: { show: true, lineStyle: { color: '#e5e7eb', type: 'dotted' } },
                    axisLine: { show: false },
                    axisTick: { show: false },
                  },
                  series: energyChartType === 'bar'
                    ? [
                        {
                          name: 'Used',
                          type: 'bar',
                          data: barData.map((d: { day: string; used: number; solarExported: number }) => [d.day, d.used]),
                          itemStyle: { color: '#3b82f6' },
                          emphasis: { focus: 'series' },
                          z: 1,
                          barCategoryGap: '10%',
                        },
                        {
                          name: 'Solar Exported',
                          type: 'bar',
                          data: barData.map((d: { day: string; used: number; solarExported: number }) => [d.day, d.solarExported]),
                          itemStyle: { color: '#22c55e' },
                          emphasis: { focus: 'series' },
                          z: 2,
                          barCategoryGap: '10%',
                        },
                      ]
                    : [
                        {
                          name: 'Used',
                          type: 'line',
                          data: energyHistory.map((h: { time: string; used: number; solarExported: number }) => [new Date(h.time).getTime(), h.used]),
                          symbol: 'none',
                          lineStyle: { color: '#3b82f6', width: 2 },
                          itemStyle: { color: '#3b82f6' },
                          emphasis: { focus: 'series' },
                          z: 1,
                        },
                        {
                          name: 'Solar Exported',
                          type: 'line',
                          data: energyHistory.map((h: { time: string; used: number; solarExported: number }) => [new Date(h.time).getTime(), h.solarExported]),
                          symbol: 'none',
                          lineStyle: { color: '#22c55e', width: 2, type: 'dashed' },
                          itemStyle: { color: '#22c55e' },
                          emphasis: { focus: 'series' },
                          z: 2,
                        },
                      ],
                }}
                style={{ width: '100%', height: 220 }}
                opts={{ renderer: 'svg' }}
              />
            </div>
          </div>
        </CardContent>
      </Card>
    </>
  );
}; 