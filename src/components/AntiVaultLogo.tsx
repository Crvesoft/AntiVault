import React from "react";

interface AntiVaultLogoProps {
  size?: number;
  className?: string;
  showText?: boolean;
  textClassName?: string;
}

export const AntiVaultLogo: React.FC<AntiVaultLogoProps> = ({
  size = 26,
  className = "",
  showText = false,
  textClassName = "",
}) => {
  return (
    <div className={`flex items-center space-x-2.5 select-none ${className}`}>
      {/* Precision High-Tech Vector Logo Icon */}
      <div 
        style={{ width: size, height: size }}
        className="relative shrink-0 flex items-center justify-center filter drop-shadow-xs"
      >
        <svg
          viewBox="0 0 64 64"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
          className="w-full h-full relative z-10"
        >
          <defs>
            {/* Gradients */}
            <linearGradient id="avBgGrad" x1="8" y1="4" x2="56" y2="60" gradientUnits="userSpaceOnUse">
              <stop offset="0%" stopColor="#0F172A" />
              <stop offset="50%" stopColor="#0B132B" />
              <stop offset="100%" stopColor="#030712" />
            </linearGradient>

            <linearGradient id="avStrokeGrad" x1="10" y1="6" x2="54" y2="58" gradientUnits="userSpaceOnUse">
              <stop offset="0%" stopColor="#38BDF8" stopOpacity="0.8" />
              <stop offset="50%" stopColor="#10B981" stopOpacity="0.5" />
              <stop offset="100%" stopColor="#064E3B" stopOpacity="0.3" />
            </linearGradient>

            {/* Anti Ribbon Gradient (Upper 'A' Ascension) */}
            <linearGradient id="antiRibbon" x1="14" y1="12" x2="48" y2="28" gradientUnits="userSpaceOnUse">
              <stop offset="0%" stopColor="#38BDF8" />
              <stop offset="50%" stopColor="#34D399" />
              <stop offset="100%" stopColor="#10B981" />
            </linearGradient>

            {/* Vault Ribbon Gradient (Lower 'V' Lock) */}
            <linearGradient id="vaultRibbon" x1="16" y1="36" x2="50" y2="50" gradientUnits="userSpaceOnUse">
              <stop offset="0%" stopColor="#059669" />
              <stop offset="50%" stopColor="#10B981" />
              <stop offset="100%" stopColor="#06B6D4" />
            </linearGradient>

            {/* Core Levitating Diamond Glow */}
            <linearGradient id="coreDiamond" x1="28" y1="27" x2="36" y2="35" gradientUnits="userSpaceOnUse">
              <stop offset="0%" stopColor="#FFFFFF" />
              <stop offset="100%" stopColor="#67E8F9" />
            </linearGradient>

            {/* Specular Highlight on Ribbon Top Edge */}
            <linearGradient id="specularEdge" x1="16" y1="12" x2="32" y2="12" gradientUnits="userSpaceOnUse">
              <stop offset="0%" stopColor="#FFFFFF" stopOpacity="0.9" />
              <stop offset="100%" stopColor="#38BDF8" stopOpacity="0" />
            </linearGradient>
          </defs>

          {/* Premium Rounded Squircle Badge Base */}
          <rect
            x="3.5"
            y="3.5"
            width="57"
            height="57"
            rx="16"
            fill="url(#avBgGrad)"
            stroke="url(#avStrokeGrad)"
            strokeWidth="1.5"
          />

          {/* Ambient Inner Subtle Grid / Glow Ring */}
          <circle
            cx="32"
            cy="32"
            r="19"
            stroke="#10B981"
            strokeWidth="0.75"
            strokeOpacity="0.18"
            strokeDasharray="3 3"
          />

          {/* Isometric Monogram 'A' Ribbon (Anti-Gravity Ascension Delta) */}
          <path
            d="M 15 23 L 32 13 L 48 22 L 39 27 L 32 23 L 23 28 L 23 37 L 15 41 Z"
            fill="url(#antiRibbon)"
            stroke="#6EE7B7"
            strokeWidth="0.75"
            strokeLinejoin="round"
          />

          {/* Specular Edge Highlight on Peak */}
          <path
            d="M 17 22 L 32 13.5 L 46 21.5"
            stroke="url(#specularEdge)"
            strokeWidth="1.2"
            strokeLinecap="round"
          />

          {/* Isometric Monogram 'V' Ribbon (Secure Vault Cradle) */}
          <path
            d="M 49 39 L 32 49 L 16 40 L 25 35 L 32 39 L 41 34 L 41 25 L 49 21 Z"
            fill="url(#vaultRibbon)"
            stroke="#34D399"
            strokeWidth="0.75"
            strokeLinejoin="round"
          />

          {/* Center Levitating Anti-Gravity Crystal Core */}
          <polygon
            points="32,27 36.5,31 32,35 27.5,31"
            fill="url(#coreDiamond)"
          />

          {/* Core Specular Dot */}
          <circle cx="32" cy="31" r="1.5" fill="#FFFFFF" />

          {/* Ambient Micro-Rays */}
          <line x1="32" y1="24.5" x2="32" y2="23" stroke="#67E8F9" strokeWidth="1" strokeLinecap="round" />
          <line x1="32" y1="37.5" x2="32" y2="39" stroke="#34D399" strokeWidth="1" strokeLinecap="round" />
          <line x1="25" y1="31" x2="23.5" y2="31" stroke="#38BDF8" strokeWidth="1" strokeLinecap="round" />
          <line x1="39" y1="31" x2="40.5" y2="31" stroke="#10B981" strokeWidth="1" strokeLinecap="round" />
        </svg>
      </div>

      {/* Brand Typography */}
      {showText && (
        <div className={`flex items-center space-x-1.5 leading-none ${textClassName}`}>
          <span className="text-[14px] font-black tracking-tight text-slate-900 dark:text-white font-sans transition-colors">
            Anti<span className="text-emerald-600 dark:text-emerald-400">Vault</span>
          </span>
        </div>
      )}
    </div>
  );
};
