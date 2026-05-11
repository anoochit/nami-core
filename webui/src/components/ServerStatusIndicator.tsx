import React, { useState, useEffect } from 'react';
import { api } from '../lib/api';
import { cn } from '../lib/utils';
import { Wifi, WifiOff } from 'lucide-react';

export const ServerStatusIndicator = () => {
  const [isOnline, setIsOnline] = useState<boolean | null>(null);

  useEffect(() => {
    const checkStatus = async () => {
      const status = await api.checkHealth();
      setIsOnline(status);
    };

    checkStatus();
    const interval = setInterval(checkStatus, 10000); // Check every 10 seconds
    return () => clearInterval(interval);
  }, []);

  if (isOnline === null) return null;

  return (
    <div className={cn(
      "flex items-center gap-2 text-xs px-2 py-1 rounded-full border",
      isOnline ? "bg-green-50 border-green-200 text-green-700" : "bg-red-50 border-red-200 text-red-700"
    )}>
      {isOnline ? <Wifi size={12} /> : <WifiOff size={12} />}
      <span>{isOnline ? 'Online' : 'Offline'}</span>
    </div>
  );
};
