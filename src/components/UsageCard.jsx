import React, { useState } from 'react'
import { Pin, Settings, LogOut } from 'lucide-react'

const UsageCard = ({ title, percentage, resetTime, isHighUsage = false, onNext, onPrev, isPinned, onTogglePin, isRefreshing, isSuccess, onRefresh, onOpenSettings, onLogout }) => {
  const numVal = parseInt(percentage) || 0
  const barColor = numVal > 80 ? '#ef4444' : numVal > 60 ? '#f59e0b' : '#3b82f6'
  const [hoverLogout, setHoverLogout] = useState(false)

  return (
    <div className="drag-region w-64 h-52 rounded-2xl overflow-hidden shadow-2xl flex flex-col"
         style={{ backgroundColor: 'rgba(30, 30, 30, 0.75)' }}>
      <div className="flex items-center justify-center pt-4 pb-1 relative">
        <button onClick={onTogglePin} className="no-drag absolute left-3 top-3 w-6 h-6 rounded-full flex items-center justify-center hover:bg-white/10 transition-colors no-select" style={{ backgroundColor: 'rgba(255,255,255,0.06)' }}>
          <Pin size={14} style={{ color: isPinned ? '#3b82f6' : '#6b7280', transform: isPinned ? 'rotate(45deg)' : 'rotate(0deg)', transition: 'all 0.2s' }} fill={isPinned ? '#3b82f6' : 'none'} />
        </button>
        <span className="text-white text-xs font-bold tracking-wide">{title}</span>
        <button onClick={onRefresh} disabled={isRefreshing} className="no-drag absolute right-3 top-3 w-6 h-6 rounded-full flex items-center justify-center transition-all duration-300 no-select disabled:opacity-40 hover:bg-white/10" style={{ backgroundColor: isSuccess ? 'rgba(59, 130, 246, 0.3)' : 'rgba(255,255,255,0.06)' }}>
          {isRefreshing ? (
            <div className="w-3 h-3 border-[1.5px] border-gray-500 border-t-blue-400 rounded-full animate-spin" />
          ) : isSuccess ? (
            <svg className="w-3 h-3 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M5 13l4 4L19 7" />
            </svg>
          ) : (
            <svg className="w-3 h-3 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          )}
        </button>
        <button onClick={onOpenSettings} className="no-drag absolute right-3 top-9 w-6 h-6 rounded-full flex items-center justify-center hover:bg-white/10 transition-colors no-select" style={{ backgroundColor: 'rgba(255,255,255,0.06)' }}>
          <Settings size={12} className="text-gray-400" />
        </button>
        <div className="no-drag absolute right-3 top-[60px]">
          <button onClick={onLogout}
                  onMouseEnter={() => setHoverLogout(true)}
                  onMouseLeave={() => setHoverLogout(false)}
                  className="w-6 h-6 rounded-full flex items-center justify-center hover:bg-red-500/20 transition-colors no-select relative"
                  style={{ backgroundColor: hoverLogout ? 'rgba(239,68,68,0.2)' : 'rgba(255,255,255,0.06)' }}>
            <LogOut size={11} style={{ color: hoverLogout ? '#ef4444' : '#9ca3af' }} />
          </button>
          {hoverLogout && (
            <span className="absolute right-8 top-0.5 text-[9px] text-red-400 whitespace-nowrap no-select pointer-events-none">注销</span>
          )}
        </div>
      </div>

      <div className="flex-1 flex flex-col items-center justify-center -mt-2">
        <span className="text-white text-5xl font-bold tracking-tight">{percentage}</span>
      </div>

      <div className="px-5 mb-3">
        <div className="w-full h-1.5 rounded-full" style={{ backgroundColor: 'rgba(255,255,255,0.1)' }}>
          <div className="h-full rounded-full transition-all duration-700 ease-out"
            style={{ width: `${numVal}%`, backgroundColor: barColor }} />
        </div>
      </div>

      <div className="flex items-center justify-between px-5 pb-3">
        <span className="text-white text-[10px] font-bold">重置: {resetTime}</span>
        <div className="flex gap-1 items-center">
          <button onClick={onPrev} className="no-drag w-5 h-5 rounded-full flex items-center justify-center hover:bg-white/10 transition-colors no-select" style={{ backgroundColor: 'rgba(255,255,255,0.05)' }}>
            <svg className="w-3 h-3 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M15 19l-7-7 7-7" /></svg>
          </button>
          <button onClick={onNext} className="no-drag w-5 h-5 rounded-full flex items-center justify-center hover:bg-white/10 transition-colors no-select" style={{ backgroundColor: 'rgba(255,255,255,0.05)' }}>
            <svg className="w-3 h-3 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M9 5l7 7-7 7" /></svg>
          </button>
        </div>
      </div>
      <div className="flex items-center justify-center pb-1.5">
        <a href="https://gitee.com/genmers/zhipu-usage-webview" target="_blank" rel="noopener noreferrer"
           className="no-drag text-gray-500 hover:text-gray-300 transition-colors"
           onClick={e => e.stopPropagation()}>
          <svg width="10" height="10" viewBox="0 0 24 24" fill="currentColor">
            <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12"/>
          </svg>
        </a>
      </div>
    </div>
  )
}

export default UsageCard
