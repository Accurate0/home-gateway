import Frame from "./Frame";
import Header from "./Header";
import { PAPER, RED, TYPE } from "../theme";

export default function StatusPanel({
  message,
  updatedAt,
}: {
  message: string;
  updatedAt: string;
}) {
  return (
    <Frame>
      <Header updatedAt={updatedAt} stale />

      <div
        style={{
          height: "60%",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        <div
          style={{
            ...TYPE.display,
            backgroundColor: RED,
            color: PAPER,
            padding: "24px 40px",
            textTransform: "uppercase",
          }}
        >
          {message}
        </div>
      </div>
    </Frame>
  );
}
