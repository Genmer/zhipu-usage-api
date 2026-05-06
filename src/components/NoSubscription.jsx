import React, { useState } from 'react'
import { Pin, LogOut, Github } from 'lucide-react'
import { open } from '@tauri-apps/plugin-shell'

const NoSubscription = ({ isPinned, onTogglePin, isRefreshing, isSuccess, onRefresh, onLogout }) => {
  const [hoverLogout, setHoverLogout] = useState(false)

  return (
    <div className="drag-region w-64 h-52 rounded-2xl overflow-hidden shadow-2xl flex flex-col"
         style={{ backgroundColor: 'rgba(30, 30, 30, 0.75)' }}>
      <div className="flex items-center justify-center pt-4 pb-1 relative">
        <button onClick={onTogglePin} className="no-drag absolute left-3 top-3 w-6 h-6 rounded-full flex items-center justify-center hover:bg-white/10 transition-colors no-select" style={{ backgroundColor: 'rgba(255,255,255,0.06)' }}>
          <Pin size={14} style={{ color: isPinned ? '#3b82f6' : '#6b7280', transform: isPinned ? 'rotate(45deg)' : 'rotate(0deg)', transition: 'all 0.2s' }} fill={isPinned ? '#3b82f6' : 'none'} />
        </button>
        <span className="text-white text-xs font-bold tracking-wide">额度监控</span>
        <button onClick={onRefresh} disabled={isRefreshing} className="no-drag absolute right-3 top-3 w-6 h-6 rounded-full flex items-center justify-center transition-all duration-300 no-select disabled:opacity-40 hover:bg-white/10" style={{ backgroundColor: isSuccess ? 'rgba(59, 130, 246, 0.3)' : 'rgba(255,255,255,0.06)' }}>
          {isRefreshing ? (
            <div className="w-3 h-3 border-[1.5px] border-gray-500 border-t-blue-400 rounded-full animate-spin" />
          ) : (
            <svg className="w-3 h-3 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2.5} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          )}
        </button>
        <div className="no-drag absolute right-3 top-[36px]">
          <button onClick={onLogout}
                  onMouseEnter={() => setHoverLogout(true)}
                  onMouseLeave={() => setHoverLogout(false)}
                  className="w-6 h-6 rounded-full flex items-center justify-center transition-colors no-select relative"
                  style={{ backgroundColor: hoverLogout ? 'rgba(239,68,68,0.2)' : 'rgba(255,255,255,0.06)' }}>
            <LogOut size={11} style={{ color: hoverLogout ? '#ef4444' : '#9ca3af' }} />
          </button>
          {hoverLogout && (
            <span className="absolute right-8 top-0.5 text-[9px] text-red-400 whitespace-nowrap no-select pointer-events-none">注销</span>
          )}
        </div>
      </div>

      <div className="flex-1 flex flex-col items-center justify-center px-6">
        <div className="w-14 h-14 rounded-2xl flex items-center justify-center mb-3"
             style={{ backgroundColor: 'rgba(239, 68, 68, 0.15)' }}>
          <svg className="w-7 h-7 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z" />
          </svg>
        </div>
        <p className="text-red-400 text-sm text-center font-bold mb-1">未订阅 Coding Plan</p>
        <p className="text-gray-500 text-[10px] text-center">请前往官网订阅后刷新</p>
      </div>

      <div className="flex items-center justify-center pb-3 gap-1.5">
        <span className="text-gray-600 text-[9px]">订阅地址: open.bigmodel.cn</span>
        <span className="text-gray-600 text-[9px]">·</span>
        <button onClick={() => open('https://gitee.com/genmers/zhipu-usage-api')}
                className="no-drag text-gray-500 hover:text-gray-300 transition-colors">
          <Github size={11} />
        </button>
      </div>
    </div>
  )
}

export default NoSubscription
