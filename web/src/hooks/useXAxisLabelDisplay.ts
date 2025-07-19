import { useState, useEffect } from "react";

// Responsive x-axis label interval hook
export function useXAxisLabelDisplay() {
  const [show, setShow] = useState(true);
  
  useEffect(() => {
    function handleResize() {
      if (window.innerWidth < 640) {
        setShow(false); // Hide labels on phones
      } else {
        setShow(true);
      }
    }
    
    handleResize();
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);
  
  return show;
} 