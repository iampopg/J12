import React from "react";

interface J12LogoProps {
  size?: number;
  showText?: boolean;
  className?: string;
}

export function J12Logo({ size = 32, showText = false, className = "" }: J12LogoProps) {
  return (
    <div 
      className={`j12-logo-container ${className}`} 
      style={{ display: "inline-flex", alignItems: "center", gap: 10 }}
    >
      {/* Emblem SVG */}
      <svg 
        width={size} 
        height={size} 
        viewBox="0 0 48 48" 
        fill="none" 
        xmlns="http://www.w3.org/2000/svg"
        style={{ flexShrink: 0 }}
      >
        <defs>
          <linearGradient id="j12Bg" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#1e293b" />
            <stop offset="100%" stopColor="#0f172a" />
          </linearGradient>
          <linearGradient id="j12Glow" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#22c55e" stopOpacity="0.3" />
            <stop offset="100%" stopColor="#10b981" stopOpacity="0" />
          </linearGradient>
          <filter id="j12Neon" x="-20%" y="-20%" width="140%" height="140%">
            <feGaussianBlur stdDeviation="1" result="blur" />
            <feComposite in="SourceGraphic" in2="blur" operator="over" />
          </filter>
        </defs>

        {/* Outer Shield Box */}
        <rect width="48" height="48" rx="10" fill="url(#j12Bg)" stroke="#334155" strokeWidth="1.2" />
        
        {/* Subtle Cyber Grid Lines */}
        <line x1="12" y1="4" x2="12" y2="44" stroke="#334155" strokeWidth="0.5" strokeOpacity="0.3" />
        <line x1="36" y1="4" x2="36" y2="44" stroke="#334155" strokeWidth="0.5" strokeOpacity="0.3" />
        
        {/* Green Forensic Scanner Accent */}
        <circle cx="24" cy="24" r="18" stroke="#22c55e" strokeWidth="1.2" strokeDasharray="6 3" strokeOpacity="0.4" />
        
        {/* J12 Typography */}
        <text 
          x="24" 
          y="31" 
          textAnchor="middle" 
          fontFamily="-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif" 
          fontWeight="900" 
          fontSize="20" 
          letterSpacing="-0.5"
        >
          <tspan fill="#ffffff">J</tspan>
          <tspan fill="#22c55e" filter="url(#j12Neon)">12</tspan>
        </text>

        {/* Micro Forensic Dot */}
        <circle cx="39" cy="11" r="2.5" fill="#22c55e" />
      </svg>

      {/* Optional Side Title */}
      {showText && (
        <div style={{ display: "flex", flexDirection: "column" }}>
          <div style={{ fontSize: 15, fontWeight: 800, letterSpacing: "-0.3px", lineHeight: 1.2 }}>
            <span style={{ color: "#ffffff" }}>J</span>
            <span style={{ color: "#22c55e" }}>12</span>
            <span style={{ color: "var(--text-0)", marginLeft: 4 }}>Investigations</span>
          </div>
          <div style={{ fontSize: 10, color: "var(--text-3)", letterSpacing: "0.5px", fontWeight: 600 }}>
            EMAIL FORENSIC SUITE
          </div>
        </div>
      )}
    </div>
  );
}
