import React from "react";
import { getRingStrokeColor, getRingTrackColor, getPercentTextColor } from "../utils/quotaFormat";

export interface QuotaRingProps {
  percent: number | string | null | undefined;
  size?: number;
  strokeWidth?: number;
  isPrimary?: boolean;
  isMuted?: boolean;
  title?: string;
  showText?: boolean;
  className?: string;
}

export const QuotaRing: React.FC<QuotaRingProps> = ({
  percent,
  size = 48,
  strokeWidth = 3.8,
  isPrimary = false,
  isMuted = false,
  title,
  showText = true,
  className = "",
}) => {
  const pNum = percent !== null && percent !== undefined ? Number(percent) : null;
  const hasPercent = pNum !== null && !isNaN(pNum);
  const clamped = hasPercent ? Math.min(100, Math.max(0, pNum)) : 100;

  const radius = (size - strokeWidth) / 2;
  const circumference = 2 * Math.PI * radius;
  const strokeDashoffset = circumference - (clamped / 100) * circumference;

  const strokeColorClass = getRingStrokeColor(percent, isMuted);
  const trackColorClass = getRingTrackColor(isMuted, isPrimary);

  // Center text styling: bold, clear, status-colored
  const textColorClass = `${getPercentTextColor(pNum)} font-bold`;

  const textSizeClass = size >= 46
    ? "text-xs"
    : size >= 40
    ? "text-[11px]"
    : "text-[10px]";

  return (
    <div
      className={`relative inline-flex items-center justify-center shrink-0 ${className}`}
      style={{ width: size, height: size }}
      title={title}
    >
      <svg
        width={size}
        height={size}
        viewBox={`0 0 ${size} ${size}`}
        className="-rotate-90 transform"
      >
        {/* Background Track Circle */}
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          strokeWidth={strokeWidth}
          stroke="currentColor"
          fill="transparent"
          className={`${trackColorClass} transition-colors`}
        />
        {/* Animated Progress Circle */}
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          strokeWidth={strokeWidth}
          strokeDasharray={circumference}
          strokeDashoffset={strokeDashoffset}
          strokeLinecap="round"
          stroke="currentColor"
          fill="transparent"
          className={`${strokeColorClass} transition-all duration-500 ease-out`}
        />
      </svg>
      {showText && (
        <div className="absolute inset-0 flex items-center justify-center pointer-events-none select-none">
          <span className={`font-mono leading-none tracking-tighter ${textSizeClass} ${textColorClass}`}>
            {hasPercent ? `${Math.round(clamped)}%` : "100%"}
          </span>
        </div>
      )}
    </div>
  );
};
