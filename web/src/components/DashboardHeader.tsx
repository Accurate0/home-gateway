import { Button } from "@/components/ui/button";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import type { TabType } from "../types";

interface DashboardHeaderProps {
  title: string;
  activeTab: TabType;
  setActiveTab: (tab: TabType) => void;
  selectedHours: number;
  setSelectedHours: (hours: number) => void;
}

export const DashboardHeader = ({ 
  title, 
  activeTab, 
  setActiveTab, 
  selectedHours, 
  setSelectedHours 
}: DashboardHeaderProps) => {
  return (
    <div className="flex justify-between items-center">
      <div className="text-center space-y-2">
        <h1 className="text-3xl font-bold">{title}</h1>
      </div>
      <div className="flex items-center gap-2">
        {/* Tab Selector */}
        <div className="flex gap-2">
          <Button
            variant={activeTab === 'overview' ? 'default' : 'outline'}
            onClick={() => setActiveTab('overview')}
          >
            Overview
          </Button>
          <Button
            variant={activeTab === 'solar' ? 'default' : 'outline'}
            onClick={() => setActiveTab('solar')}
          >
            Solar & Energy
          </Button>
        </div>
        {/* Time Range Selector */}
        <Popover>
          <PopoverTrigger asChild>
            <Button variant="outline" className="w-24">
              {selectedHours <= 24 ? `${selectedHours}h` : selectedHours === 72 ? '3d' : selectedHours === 168 ? '7d' : '14d'}
            </Button>
          </PopoverTrigger>
          <PopoverContent className="w-24">
            <div className="space-y-1">
              {[
                { value: 1, label: '1h' },
                { value: 3, label: '3h' },
                { value: 6, label: '6h' },
                { value: 12, label: '12h' },
                { value: 24, label: '1d' },
                { value: 72, label: '3d' },
                { value: 168, label: '7d' },
                { value: 336, label: '14d' }
              ].map(({ value, label }) => (
                <Button
                  key={value}
                  variant={selectedHours === value ? "default" : "ghost"}
                  className="w-full justify-start"
                  onClick={() => setSelectedHours(value)}
                >
                  {label}
                </Button>
              ))}
            </div>
          </PopoverContent>
        </Popover>
      </div>
    </div>
  );
}; 