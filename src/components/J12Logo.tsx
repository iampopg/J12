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
      <svg 
        viewBox="0 0 64 64" 
        width={size} 
        height={size} 
        style={{ flexShrink: 0 }}
      >
        <defs>
          <linearGradient id={`favBg-${size}`} x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#1e293b"/>
            <stop offset="100%" stopColor="#0f172a"/>
          </linearGradient>
          <linearGradient id={`thumbArc-${size}`} x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#22c55e" stopOpacity="0.45"/>
            <stop offset="100%" stopColor="#10b981" stopOpacity="0.15"/>
          </linearGradient>
        </defs>
        <rect width="64" height="64" rx="14" fill={`url(#favBg-${size})`} stroke="#334155" strokeWidth="1.5"/>
        
        {/* Biometric Thumbprint / Fingerprint Background Arcs */}
        <g stroke={`url(#thumbArc-${size})`} strokeWidth="1.6" fill="none" strokeLinecap="round">
          <path d="M 22 18 C 27 13, 37 13, 42 18 C 47 23, 47 33, 43 39"/>
          <path d="M 18 22 C 25 15, 39 15, 46 22 C 51 28, 51 38, 46 45"/>
          <path d="M 14 28 C 22 17, 42 17, 50 28 C 54 34, 53 44, 48 51"/>
          <path d="M 26 23 C 29 20, 34 20, 37 23 C 40 26, 40 31, 37 35"/>
        </g>

        <circle cx="32" cy="32" r="24" stroke="#22c55e" strokeWidth="1.5" strokeDasharray="8 4" strokeOpacity="0.4"/>
        <text x="32" y="42" textAnchor="middle" fontFamily="-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif" fontSize="26" fontWeight="900" letterSpacing="-1">
          <tspan fill="#ffffff">J</tspan><tspan fill="#22c55e">12</tspan>
        </text>
        <circle cx="52" cy="14" r="3" fill="#22c55e"/>
      </svg>

      {showText && (
        <div style={{ display: "flex", flexDirection: "column" }}>
          <div style={{ fontSize: 15, fontWeight: 800, letterSpacing: "-0.3px", lineHeight: 1.2 }}>
            <span style={{ color: "#22c55e" }}>J</span>
            <span style={{ color: "#ffffff" }}>1</span>
            <span style={{ color: "#22c55e" }}>2</span>
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
