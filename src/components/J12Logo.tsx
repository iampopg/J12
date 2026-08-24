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
      <img 
        src="/logo.png" 
        alt="J12 Logo" 
        style={{ 
          width: size, 
          height: size, 
          objectFit: "contain",
          flexShrink: 0 
        }} 
      />

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
