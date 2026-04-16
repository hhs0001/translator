'use client';

import { motion } from 'framer-motion';
import { cn } from '@/lib/utils';

interface LoadingTextProps {
  text: string;
  className?: string;
  variant?: 'shimmer' | 'pulse' | 'dots';
}

/**
 * Premium loading text with animated shimmer effect.
 * The shimmer creates a "light sweep" across the text.
 */
export function LoadingText({
  text,
  className,
  variant = 'shimmer',
}: LoadingTextProps) {
  if (variant === 'pulse') {
    return (
      <span className={cn('inline-flex items-center', className)}>
        <motion.span
          animate={{ opacity: [0.4, 1, 0.4] }}
          transition={{ duration: 1.5, repeat: Infinity, ease: 'easeInOut' }}
        >
          {text}
        </motion.span>
      </span>
    );
  }

  if (variant === 'dots') {
    return (
      <span className={cn('inline-flex items-center', className)}>
        <span>{text}</span>
        <AnimatedDots />
      </span>
    );
  }

  // Shimmer variant - CSS gradient animation
  return (
    <span className={cn('relative inline-flex items-center overflow-hidden', className)}>
      {/* Base text */}
      <span className="text-muted-foreground/60">{text}</span>

      {/* Shimmer overlay using mask */}
      <motion.span
        className="absolute inset-0 text-foreground"
        style={{
          maskImage: 'linear-gradient(90deg, transparent, white 40%, white 60%, transparent)',
          WebkitMaskImage: 'linear-gradient(90deg, transparent, white 40%, white 60%, transparent)',
        }}
        initial={{ x: '-100%' }}
        animate={{ x: '100%' }}
        transition={{
          duration: 1.8,
          repeat: Infinity,
          ease: 'linear',
          repeatDelay: 0.5,
        }}
      >
        {text}
      </motion.span>
    </span>
  );
}

/**
 * Animated ellipsis dots
 */
function AnimatedDots() {
  return (
    <span className="ml-0.5 inline-flex">
      {[0, 1, 2].map((i) => (
        <motion.span
          key={i}
          className="mx-[1px] inline-block h-[3px] w-[3px] rounded-full bg-current"
          animate={{ opacity: [0.2, 1, 0.2] }}
          transition={{
            duration: 0.8,
            repeat: Infinity,
            ease: 'easeInOut',
            delay: i * 0.15,
          }}
        />
      ))}
    </span>
  );
}

/**
 * Compact status badge with breathing pulse indicator.
 */
export function LoadingBadge({
  text,
  className,
}: {
  text: string;
  className?: string;
}) {
  return (
    <span
      className={cn(
        'inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full',
        'bg-primary/10 text-primary text-[10px] font-medium'
      )}
    >
      {/* Pulse dot */}
      <span className="relative flex h-1.5 w-1.5">
        <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-current opacity-75" />
        <span className="relative inline-flex h-1.5 w-1.5 rounded-full bg-current" />
      </span>

      {/* Breathing text */}
      <motion.span
        animate={{ opacity: [0.6, 1, 0.6] }}
        transition={{
          duration: 2,
          repeat: Infinity,
          ease: 'easeInOut',
        }}
        className={className}
      >
        {text}
      </motion.span>
    </span>
  );
}

/**
 * Skeleton shimmer placeholder for loading states.
 */
export function TextSkeleton({
  className,
  lines = 1,
}: {
  className?: string;
  lines?: number;
}) {
  return (
    <div className={cn('space-y-2', className)}>
      {Array.from({ length: lines }).map((_, i) => (
        <div
          key={i}
          className="h-4 w-full rounded bg-muted animate-pulse"
          style={{ width: i === lines - 1 ? '75%' : '100%' }}
        />
      ))}
    </div>
  );
}
