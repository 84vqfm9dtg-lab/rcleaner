import type { RiskLevel } from "./types";

const dateTimeFormatter = new Intl.DateTimeFormat("zh-CN", {
  month: "2-digit",
  day: "2-digit",
  hour: "2-digit",
  minute: "2-digit",
});

export function formatTimestamp(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  return dateTimeFormatter.format(date);
}

export function formatPercent(value: number): string {
  return `${Math.round(value * 100)}%`;
}

export function riskLabel(value: RiskLevel): string {
  switch (value) {
    case "low":
      return "低风险";
    case "medium":
      return "中风险";
    case "high":
      return "高风险";
    default:
      return value;
  }
}

export function riskPalette(value: RiskLevel): {
  bg: string;
  color: string;
  border: string;
} {
  switch (value) {
    case "low":
      return {
        bg: "rgba(128, 145, 162, 0.12)",
        color: "#dfe6ef",
        border: "rgba(128, 145, 162, 0.18)",
      };
    case "medium":
      return {
        bg: "rgba(193, 149, 79, 0.14)",
        color: "#f2ddba",
        border: "rgba(193, 149, 79, 0.2)",
      };
    case "high":
      return {
        bg: "rgba(180, 92, 92, 0.14)",
        color: "#f4cbcb",
        border: "rgba(180, 92, 92, 0.18)",
      };
    default:
      return {
        bg: "rgba(128, 145, 162, 0.12)",
        color: "#dfe6ef",
        border: "rgba(128, 145, 162, 0.18)",
      };
  }
}
