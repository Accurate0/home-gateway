export interface GenerationHistory {
  wh: number;
  uvLevel: number | null;
  temperature: number | null;
  at: string;
  timestamp: number;
}

export interface SolarCurrentResponse {
  currentProductionWh: number;
  monthProductionKwh: number;
  todayProductionKwh: number;
  yesterdayProductionKwh: number;
  allTimeProductionKwh: number;
  uvLevel: number | null;
  temperature: number | null;
  statistics: {
    averages: {
      last15Mins: number | null;
      last1Hour: number | null;
      last3Hours: number | null;
    };
  };
}

export interface SolarHistoryResponse {
  today: GenerationHistory[];
  yesterday: GenerationHistory[];
}
