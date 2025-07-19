import { useState, useEffect } from "react";

// Custom hook to manage time state with URL query parameters
export function useTimeState() {
  const [selectedHours, setSelectedHoursState] = useState<number>(() => {
    // Initialize from URL query parameter
    const urlParams = new URLSearchParams(window.location.search);
    const hoursParam = urlParams.get('hours');
    const validHours = [1, 3, 6, 12, 24, 72, 168, 336]; // 1h, 3h, 6h, 12h, 24h, 3d, 7d, 14d
    return hoursParam ? (validHours.includes(Number(hoursParam)) ? Number(hoursParam) : 12) : 12;
  });

  const setSelectedHours = (hours: number) => {
    setSelectedHoursState(hours);
    
    // Update URL query parameter
    const url = new URL(window.location.href);
    url.searchParams.set('hours', hours.toString());
    window.history.replaceState({}, '', url.toString());
  };

  // Listen for browser navigation (back/forward buttons)
  useEffect(() => {
    const handlePopState = () => {
      const urlParams = new URLSearchParams(window.location.search);
      const hoursParam = urlParams.get('hours');
      const validHours = [1, 3, 6, 12, 24, 72, 168, 336]; // 1h, 3h, 6h, 12h, 24h, 3d, 7d, 14d
      const newHours = hoursParam ? (validHours.includes(Number(hoursParam)) ? Number(hoursParam) : 12) : 12;
      setSelectedHoursState(newHours);
    };

    window.addEventListener('popstate', handlePopState);
    return () => window.removeEventListener('popstate', handlePopState);
  }, []);

  return [selectedHours, setSelectedHours] as const;
} 