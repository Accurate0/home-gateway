import "./App.css";
import SolarChart from "./components/SolarChart";
import ForecastCard from "./components/ForecastCard";

function App() {
  return (
    <>
      <div
        style={{
          display: "flex",
          gap: 24,
          alignItems: "flex-start",
          justifyContent: "center",
          paddingTop: 16,
        }}
      >
        <SolarChart />
        <ForecastCard />
      </div>
    </>
  );
}

export default App;
