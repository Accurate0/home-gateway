import { Checkbox } from "@/components/ui/checkbox";
import type { DeviceInfo } from "@/types";

interface DeviceFilterSectionProps {
  devices: DeviceInfo[];
  selectedDevices: string[];
  setSelectedDevices: React.Dispatch<React.SetStateAction<string[]>>;
  title: string;
  icon: React.ComponentType<{ className?: string }>;
  iconColor: string;
  showBorder?: boolean;
}

export const DeviceFilterSection = ({
  devices,
  selectedDevices,
  setSelectedDevices,
  title,
  icon: Icon,
  iconColor,
  showBorder = false
}: DeviceFilterSectionProps) => {
  if (devices.length === 0) return null;

  return (
    <div className={`space-y-3 ${showBorder ? 'pb-4 border-b border-border/50' : ''}`}>
      <div className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
        <Icon className={`h-4 w-4 ${iconColor}`} />
        {title}
      </div>
      <div className="space-y-2 pl-1">
        {devices.map(device => (
          <div key={device.name} className="flex items-center space-x-2">
            <Checkbox
              id={device.name}
              checked={selectedDevices.includes(device.name)}
                              onCheckedChange={(checked) => {
                  if (checked) {
                    setSelectedDevices(prev => [...prev, device.name]);
                  } else {
                    setSelectedDevices(prev => prev.filter(d => d !== device.name));
                  }
                }}
            />
            <label
              htmlFor={device.name}
              className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
            >
              {device.name}
            </label>
          </div>
        ))}
      </div>
    </div>
  );
}; 