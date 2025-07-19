import { useState, useEffect } from "react";
import type { TabType } from "../types";

// Custom hook to manage tab state with URL query parameters
export function useTabState() {
  const [activeTab, setActiveTabState] = useState<TabType>(() => {
    // Initialize from URL query parameter
    const urlParams = new URLSearchParams(window.location.search);
    const tabParam = urlParams.get('tab');
    return (tabParam === 'overview' || tabParam === 'solar') ? tabParam : 'overview';
  });

  const setActiveTab = (tab: TabType) => {
    setActiveTabState(tab);
    
    // Update URL query parameter
    const url = new URL(window.location.href);
    url.searchParams.set('tab', tab);
    window.history.replaceState({}, '', url.toString());
  };

  // Listen for browser navigation (back/forward buttons)
  useEffect(() => {
    const handlePopState = () => {
      const urlParams = new URLSearchParams(window.location.search);
      const tabParam = urlParams.get('tab');
      const newTab = (tabParam === 'overview' || tabParam === 'solar') ? tabParam : 'overview';
      setActiveTabState(newTab);
    };

    window.addEventListener('popstate', handlePopState);
    return () => window.removeEventListener('popstate', handlePopState);
  }, []);

  return [activeTab, setActiveTab] as const;
} 