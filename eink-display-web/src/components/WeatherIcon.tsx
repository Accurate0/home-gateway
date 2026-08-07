import { BLUE, INK, PAPER, YELLOW } from "../theme";

type CloudShape = { cx: number; cy: number; r: number };

const CLOUD_LOBES: CloudShape[] = [
  { cx: 36, cy: 56, r: 17 },
  { cx: 57, cy: 45, r: 23 },
  { cx: 76, cy: 58, r: 14 },
];

const CLOUD_BODY = { x: 34, y: 56, width: 44, height: 17 };

function Cloud({ fill = PAPER }: { fill?: string }) {
  const lobes = (props: React.SVGProps<SVGCircleElement>) =>
    CLOUD_LOBES.map((c) => <circle key={c.cx} cx={c.cx} cy={c.cy} r={c.r} {...props} />);

  return (
    <>
      <g fill={INK} stroke={INK} strokeWidth={9} strokeLinejoin="round">
        {lobes({})}
        <rect {...CLOUD_BODY} />
      </g>

      <g fill={fill}>
        {lobes({})}
        <rect {...CLOUD_BODY} />
      </g>
    </>
  );
}

function Sun({ cx = 50, cy = 48, r = 24 }: { cx?: number; cy?: number; r?: number }) {
  const rays = Array.from({ length: 8 }, (_, i) => {
    const angle = (i * Math.PI) / 4;
    const inner = r + 6;
    const outer = r + 17;

    return (
      <line
        key={i}
        x1={cx + Math.cos(angle) * inner}
        y1={cy + Math.sin(angle) * inner}
        x2={cx + Math.cos(angle) * outer}
        y2={cy + Math.sin(angle) * outer}
        stroke={INK}
        strokeWidth={6}
        strokeLinecap="round"
      />
    );
  });

  return (
    <>
      {rays}
      <circle cx={cx} cy={cy} r={r} fill={YELLOW} stroke={INK} strokeWidth={6} />
    </>
  );
}

function Drops({ color = BLUE }: { color?: string }) {
  return (
    <g stroke={color} strokeWidth={7} strokeLinecap="round">
      <line x1={40} y1={80} x2={34} y2={94} />
      <line x1={57} y1={80} x2={51} y2={94} />
      <line x1={74} y1={80} x2={68} y2={94} />
    </g>
  );
}

function icon(code: string) {
  const c = code.toLowerCase();

  if (c.includes("storm") || c.includes("thunder"))
    return (
      <>
        <Cloud />
        <path
          d="M56 76 L40 76 L52 96 L44 96 L62 76 L50 76 Z"
          fill={YELLOW}
          stroke={INK}
          strokeWidth={5}
          strokeLinejoin="round"
        />
      </>
    );

  if (c.includes("rain") || c.includes("shower") || c.includes("drizzle"))
    return (
      <>
        <Cloud />
        <Drops />
      </>
    );

  if (c.includes("snow"))
    return (
      <>
        <Cloud />
        <Drops color={INK} />
      </>
    );

  if (c.includes("fog") || c.includes("mist") || c.includes("haze"))
    return (
      <g stroke={INK} strokeWidth={8} strokeLinecap="round">
        <line x1={16} y1={38} x2={84} y2={38} />
        <line x1={24} y1={58} x2={92} y2={58} />
        <line x1={16} y1={78} x2={84} y2={78} />
      </g>
    );

  if (c.includes("partly") || c.includes("mostly sunny"))
    return (
      <>
        <Sun cx={68} cy={32} r={17} />
        <Cloud />
      </>
    );

  if (c.includes("cloud") || c.includes("overcast")) return <Cloud />;

  return <Sun />;
}

export default function WeatherIcon({ code, size }: { code: string; size: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 100 100">
      {icon(code)}
    </svg>
  );
}
