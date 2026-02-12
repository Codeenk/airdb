import React from 'react';

interface SkeletonProps {
  width?: string | number;
  height?: string | number;
  borderRadius?: string;
  style?: React.CSSProperties;
  className?: string;
}

/** Animated skeleton placeholder */
export function Skeleton({ width = '100%', height = 16, borderRadius = '4px', style, className }: SkeletonProps) {
  return (
    <div
      className={`skeleton-pulse ${className ?? ''}`}
      style={{
        width: typeof width === 'number' ? `${width}px` : width,
        height: typeof height === 'number' ? `${height}px` : height,
        borderRadius,
        ...style,
      }}
    />
  );
}

/** Multiple skeleton lines for text blocks */
export function SkeletonLines({ lines = 3, gap = 8 }: { lines?: number; gap?: number }) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap }}>
      {Array.from({ length: lines }).map((_, i) => (
        <Skeleton key={i} width={i === lines - 1 ? '60%' : '100%'} height={14} />
      ))}
    </div>
  );
}

/** Skeleton for a table/grid */
export function SkeletonTable({ rows = 5, cols = 4 }: { rows?: number; cols?: number }) {
  return (
    <div className="skeleton-table">
      <div className="skeleton-table-header">
        {Array.from({ length: cols }).map((_, i) => (
          <Skeleton key={i} height={32} borderRadius="4px" />
        ))}
      </div>
      {Array.from({ length: rows }).map((_, r) => (
        <div key={r} className="skeleton-table-row">
          {Array.from({ length: cols }).map((_, c) => (
            <Skeleton key={c} height={24} borderRadius="3px" />
          ))}
        </div>
      ))}
    </div>
  );
}

/** Skeleton for stat cards */
export function SkeletonStats({ count = 4 }: { count?: number }) {
  return (
    <div className="skeleton-stats">
      {Array.from({ length: count }).map((_, i) => (
        <div key={i} className="skeleton-stat-card">
          <Skeleton width={40} height={40} borderRadius="8px" />
          <div style={{ flex: 1 }}>
            <Skeleton width="50%" height={12} style={{ marginBottom: 6 }} />
            <Skeleton width="30%" height={20} />
          </div>
        </div>
      ))}
    </div>
  );
}
