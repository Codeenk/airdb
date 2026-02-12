import React from 'react';

interface LogoProps {
  size?: 'sm' | 'md' | 'lg' | number;
  showText?: boolean;
}

export const Logo: React.FC<LogoProps> = ({ size = 'md', showText = true }) => {
  const sizeMap = {
    sm: 24,
    md: 32,
    lg: 48,
  };
  
  const iconSize = typeof size === 'number' ? size : sizeMap[size];
  
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
      <svg
        width={iconSize}
        height={iconSize}
        viewBox="0 0 32 32"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        <defs>
          <linearGradient id="airdb-gradient" x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor="#3b82f6" />
            <stop offset="100%" stopColor="#22d3ee" />
          </linearGradient>
        </defs>
        
        {/* Database cylinder */}
        <ellipse cx="16" cy="10" rx="12" ry="4" fill="url(#airdb-gradient)" opacity="0.3" />
        <rect x="4" y="10" width="24" height="12" fill="url(#airdb-gradient)" opacity="0.2" />
        <ellipse cx="16" cy="22" rx="12" ry="4" fill="url(#airdb-gradient)" />
        
        {/* Cloud/Air icon */}
        <path
          d="M 20 6 C 18.5 6 17.2 6.8 16.5 8 C 16.3 8 16.2 8 16 8 C 14 8 12.5 9.5 12.5 11.5 C 12.5 11.7 12.5 11.8 12.6 12 C 11.1 12.3 10 13.5 10 15 C 10 16.7 11.3 18 13 18 L 23 18 C 24.7 18 26 16.7 26 15 C 26 13.5 24.9 12.3 23.4 12 C 23.5 11.8 23.5 11.7 23.5 11.5 C 23.5 8.5 21.5 6 20 6 Z"
          fill="white"
          opacity="0.9"
          transform="scale(0.8) translate(4, -2)"
        />
      </svg>
      {showText && (
        <span style={{ 
          fontSize: iconSize * 0.75, 
          fontWeight: 600, 
          color: 'var(--text-primary)',
          letterSpacing: '0.02em'
        }}>
          AirDB
        </span>
      )}
    </div>
  );
};
